#[cfg(test)]
use crate::intercept::matches_url;
use crate::intercept::{Intercept, Verdict};
use crate::route::Route;
use crate::{Error, Timing};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use tokio::net::TcpStream;
use tokio::time::{Instant, timeout_at};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const MAX_PENDING: usize = 1024;
const MAX_EVENTS: usize = 4096;
const GONE_REQUEST: i64 = -32602;

pub const FORBIDDEN_METHODS: &[&str] = &[
    "Runtime.enable",
    "Page.enable",
    "Target.setAutoAttach",
    "Target.sendMessageToTarget",
    "Target.exposeDevToolsProtocol",
    "Target.autoAttachRelated",
];

const ATTACH_TO_TARGET: &str = "Target.attachToTarget";

pub(crate) fn blocked(method: &str) -> Option<&'static str> {
    FORBIDDEN_METHODS
        .iter()
        .copied()
        .find(|forbidden| *forbidden == method)
}

fn unflattened(method: &str, params: &Value) -> Option<&'static str> {
    (method == ATTACH_TO_TARGET && params.get("flatten") != Some(&Value::Bool(true)))
        .then_some(ATTACH_TO_TARGET)
}

pub(crate) struct Transport {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
    timing: Timing,
    sent: BTreeSet<String>,
    events: VecDeque<Value>,
    dropped: u64,
    reply_failures: u64,
    intercept: Option<Intercept>,
    pending: BTreeMap<u64, &'static str>,
    fetch: Option<(u64, Value)>,
    held: VecDeque<Value>,
}

impl Transport {
    pub(crate) fn new(ws: WebSocketStream<MaybeTlsStream<TcpStream>>, timing: Timing) -> Self {
        Self {
            ws,
            next_id: 1,
            timing,
            sent: BTreeSet::new(),
            events: VecDeque::new(),
            dropped: 0,
            reply_failures: 0,
            intercept: None,
            pending: BTreeMap::new(),
            fetch: None,
            held: VecDeque::new(),
        }
    }

    async fn intercept_for(&mut self, session: Option<&str>) -> Result<Option<String>, Error> {
        let narrowed = match self.intercept.take() {
            Some(current) if current.session.is_none() || current.session.as_deref() == session => {
                let scope = current.session.clone();
                self.intercept = Some(current);
                return Ok(scope);
            }
            Some(mut current) if session.is_none() => {
                let previous = current.session.take();
                self.intercept = Some(current);
                previous
            }
            _ => {
                self.intercept = Some(Intercept::new(session));
                None
            }
        };
        if let Some(previous) = narrowed {
            drop(
                self.send("Fetch.disable", json!({}), Some(&previous))
                    .await?,
            );
        }
        Ok(session.map(str::to_owned))
    }

    pub(crate) async fn arm_auth(
        &mut self,
        session: Option<&str>,
        user: String,
        pass: String,
    ) -> Result<(), Error> {
        let scope = self.intercept_for(session).await?;
        if let Some(intercept) = self.intercept.as_mut() {
            intercept.login(user, pass);
        }
        drop(
            self.send("Fetch.disable", json!({}), scope.as_deref())
                .await?,
        );
        Ok(())
    }

    pub(crate) async fn reroute(
        &mut self,
        session: &str,
        edit: impl FnOnce(&mut Vec<Route>),
    ) -> Result<(), Error> {
        let scope = self.intercept_for(Some(session)).await?;
        if let Some(intercept) = self.intercept.as_mut() {
            edit(&mut intercept.routes);
            if !intercept.active() {
                self.intercept = None;
            }
        }
        drop(
            self.send("Fetch.disable", json!({}), scope.as_deref())
                .await?,
        );
        Ok(())
    }

    async fn write(
        &mut self,
        method: &str,
        params: Value,
        session: Option<&str>,
    ) -> Result<u64, Error> {
        let id = self.next_id;
        self.next_id += 1;
        let mut frame = json!({ "id": id, "method": method, "params": params });
        if let Some(session) = session {
            frame["sessionId"] = json!(session);
        }
        self.ws
            .send(Message::Text(frame.to_string().into()))
            .await?;
        self.sent.insert(method.to_string());
        Ok(id)
    }

    async fn post(
        &mut self,
        method: &'static str,
        params: Value,
        session: Option<&str>,
    ) -> Result<(), Error> {
        if self.pending.len() >= MAX_PENDING {
            return Err(Error::Protocol {
                method: method.into(),
                code: None,
                message: "too many unacknowledged CDP commands".into(),
            });
        }
        let id = self.write(method, params, session).await?;
        self.pending.insert(id, method);
        Ok(())
    }

    async fn answer(&mut self, msg: &Value) -> Result<bool, Error> {
        let configuring = self.fetch.is_some();
        let Some(intercept) = self.intercept.as_mut() else {
            return Ok(false);
        };
        match intercept.decide(msg, configuring)? {
            Verdict::Pass => Ok(false),
            Verdict::Hold => {
                if self.held.len() >= MAX_EVENTS {
                    return Err(Error::Protocol {
                        method: "Fetch.enable".into(),
                        code: None,
                        message: "too many requests waiting for Fetch configuration".into(),
                    });
                }
                self.held.push_back(msg.clone());
                Ok(true)
            }
            Verdict::Reply(reply) => {
                let session = msg.get("sessionId").and_then(Value::as_str);
                self.post(reply.method, reply.params, session).await?;
                Ok(true)
            }
        }
    }

    fn queued(&mut self, msg: Value) {
        if self.events.len() >= MAX_EVENTS {
            drop(self.events.pop_front());
            self.dropped += 1;
        }
        self.events.push_back(msg);
    }

    pub(crate) fn dropped_events(&self) -> u64 {
        self.dropped
    }

    pub(crate) fn reply_failures(&self) -> u64 {
        self.reply_failures
    }

    async fn event(&mut self, msg: Value) -> Result<(), Error> {
        if !self.answer(&msg).await? {
            self.queued(msg);
        }
        Ok(())
    }

    async fn acknowledged(&mut self, msg: &Value) -> Result<(), Error> {
        let Some(id) = msg.get("id").and_then(Value::as_u64) else {
            return Ok(());
        };
        if let Some((_, patterns)) = self.fetch.take_if(|(pending, _)| *pending == id) {
            if msg.get("error").is_none()
                && let Some(intercept) = self.intercept.as_mut()
            {
                intercept.accept(patterns)?;
            }
            while let Some(event) = self.held.pop_front() {
                self.event(event).await?;
            }
        }
        if self.pending.remove(&id).is_some()
            && msg.get("error").is_some()
            && msg.pointer("/error/code").and_then(Value::as_i64) != Some(GONE_REQUEST)
        {
            self.reply_failures += 1;
        }
        Ok(())
    }

    pub(crate) fn sent(&self) -> &BTreeSet<String> {
        &self.sent
    }

    pub(crate) async fn send(
        &mut self,
        method: &str,
        mut params: Value,
        session_id: Option<&str>,
    ) -> Result<Value, Error> {
        if let Some(name) = blocked(method).or_else(|| unflattened(method, &params)) {
            return Err(Error::Blocked(name));
        }
        let configuring = matches!(method, "Fetch.enable" | "Fetch.disable")
            && self.intercept.as_ref().is_some_and(|intercept| {
                intercept.active() && intercept.session.as_deref() == session_id
            });
        let mut patterns = None;
        let wire_method = match self.intercept.as_ref() {
            Some(intercept) if configuring => {
                if self.fetch.is_some() {
                    return Err(Error::Protocol {
                        method: method.into(),
                        code: None,
                        message: "previous Fetch configuration is not acknowledged".into(),
                    });
                }
                (params, patterns) = intercept.configure(method, params);
                "Fetch.enable"
            }
            _ => method,
        };
        let deadline = Instant::now() + self.timing.command;
        timeout_at(
            deadline,
            self.command(wire_method, params, session_id, patterns),
        )
        .await
        .map_err(|_elapsed| Error::Timeout(wire_method.to_string()))?
    }

    async fn command(
        &mut self,
        method: &str,
        params: Value,
        session_id: Option<&str>,
        patterns: Option<Value>,
    ) -> Result<Value, Error> {
        let id = self.write(method, params, session_id).await?;
        if let Some(patterns) = patterns {
            self.fetch = Some((id, patterns));
            self.pending.insert(id, "Fetch.enable");
        }
        self.wait_result(id, method).await
    }

    async fn wait_result(&mut self, id: u64, method: &str) -> Result<Value, Error> {
        loop {
            let frame = self.ws.next().await.ok_or_else(closed)??;
            match frame {
                Message::Text(text) => {
                    let msg: Value = serde_json::from_str(&text)?;
                    if let Some(got) = msg.get("id").and_then(Value::as_u64) {
                        self.acknowledged(&msg).await?;
                        if got == id {
                            return result(&msg, method);
                        }
                    } else {
                        self.event(msg).await?;
                    }
                }
                Message::Close(_) => return Err(closed()),
                _ => {}
            }
        }
    }

    pub(crate) async fn pump(&mut self) -> Result<(), Error> {
        let deadline = Instant::now() + self.timing.command;
        let quiet = Instant::now() + self.timing.drain;
        timeout_at(deadline, self.idle(quiet))
            .await
            .map_err(|_elapsed| Error::Timeout("CDP pump".into()))?
    }

    pub(crate) async fn pump_then_take(&mut self) -> Result<Vec<Value>, Error> {
        let mut taken = self.take_events();
        let pumped = self.pump().await;
        taken.extend(self.take_events());
        match pumped {
            Err(error) if taken.is_empty() => Err(error),
            _ => Ok(taken),
        }
    }

    pub(crate) async fn idle(&mut self, deadline: Instant) -> Result<(), Error> {
        loop {
            let frame = match timeout_at(deadline, self.ws.next()).await {
                Err(_elapsed) => return Ok(()),
                Ok(frame) => frame.ok_or_else(closed)??,
            };
            match frame {
                Message::Text(text) => {
                    let msg: Value = serde_json::from_str(&text)?;
                    if msg.get("id").and_then(Value::as_u64).is_some() {
                        self.acknowledged(&msg).await?;
                    } else {
                        self.event(msg).await?;
                    }
                }
                Message::Close(_) => return Err(closed()),
                _ => {}
            }
        }
    }

    pub(crate) fn take_event(&mut self, wanted: impl Fn(&Value) -> bool) -> Option<Value> {
        let at = self.events.iter().position(wanted)?;
        self.events.remove(at)
    }

    pub(crate) fn take_events(&mut self) -> Vec<Value> {
        self.events.drain(..).collect()
    }

    pub(crate) async fn close(&mut self) {
        drop(self.ws.close(None).await);
    }
}

fn result(msg: &Value, method: &str) -> Result<Value, Error> {
    if let Some(error) = msg.get("error") {
        return Err(Error::protocol(method, error));
    }
    Ok(msg.get("result").cloned().unwrap_or(Value::Null))
}

fn closed() -> Error {
    Error::Closed
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;
