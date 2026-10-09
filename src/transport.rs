use crate::Error;
#[cfg(test)]
use crate::intercept::matches_url;
use crate::intercept::{Pattern, ReplyKind, auth_reply};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::{Instant, timeout_at};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const COMMAND_DEADLINE: Duration = Duration::from_secs(30);
const PUMP: Duration = Duration::from_millis(25);
const MAX_PENDING: usize = 1024;
const MAX_EVENTS: usize = 4096;

pub(crate) fn blocked(method: &str) -> Option<&'static str> {
    match method {
        "Runtime.enable" => Some("Runtime.enable"),
        "Page.enable" => Some("Page.enable"),
        "Target.setAutoAttach" => Some("Target.setAutoAttach"),
        _ => None,
    }
}

pub(crate) struct Transport {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
    #[cfg(test)]
    sent: BTreeSet<String>,
    events: Vec<Value>,
    auth: Option<Auth>,
    pending: BTreeMap<u64, &'static str>,
    fetch: Option<(u64, Value)>,
    held: VecDeque<Value>,
}

struct Auth {
    user: String,
    pass: String,
    session: String,
    attempted: BTreeSet<String>,
    patterns: Vec<Pattern>,
}

impl Transport {
    pub(crate) fn new(ws: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self {
            ws,
            next_id: 1,
            #[cfg(test)]
            sent: BTreeSet::new(),
            events: Vec::new(),
            auth: None,
            pending: BTreeMap::new(),
            fetch: None,
            held: VecDeque::new(),
        }
    }

    pub(crate) async fn arm_auth(
        &mut self,
        session: &str,
        user: String,
        pass: String,
    ) -> Result<(), Error> {
        self.auth = Some(Auth {
            user,
            pass,
            session: session.to_string(),
            attempted: BTreeSet::new(),
            patterns: Vec::new(),
        });
        drop(self.send("Fetch.disable", json!({}), Some(session)).await?);
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
        #[cfg(test)]
        self.sent.insert(method.to_string());
        Ok(id)
    }

    async fn post(
        &mut self,
        method: &'static str,
        params: Value,
        session: &str,
    ) -> Result<(), Error> {
        if self.pending.len() >= MAX_PENDING {
            return Err(Error::Protocol {
                method: method.into(),
                message: "too many unacknowledged CDP commands".into(),
            });
        }
        let id = self.write(method, params, Some(session)).await?;
        self.pending.insert(id, method);
        Ok(())
    }

    async fn answer(&mut self, msg: &Value) -> Result<bool, Error> {
        let Some(auth) = self.auth.as_mut() else {
            return Ok(false);
        };
        if msg.get("sessionId").and_then(Value::as_str) != Some(auth.session.as_str()) {
            return Ok(false);
        }
        if msg.get("method").and_then(Value::as_str) == Some("Fetch.requestPaused") {
            let params = &msg["params"];
            if params.get("responseStatusCode").is_some()
                || params.get("responseErrorReason").is_some()
            {
                if let Some(id) = params.get("requestId").and_then(Value::as_str) {
                    auth.attempted.remove(id);
                }
                return Ok(false);
            }
            if self.fetch.is_some() {
                if self.held.len() >= MAX_EVENTS {
                    return Err(Error::Protocol {
                        method: "Fetch.enable".into(),
                        message: "too many requests waiting for Fetch configuration".into(),
                    });
                }
                self.held.push_back(msg.clone());
                return Ok(true);
            }
            if auth.patterns.iter().any(|pattern| pattern.matches(params)) {
                return Ok(false);
            }
        }
        let Some(mut reply) = auth_reply(msg, &auth.user, &auth.pass) else {
            return Ok(false);
        };
        if let ReplyKind::Auth { proxy: true } = reply.kind {
            let id = msg["params"]["requestId"]
                .as_str()
                .ok_or_else(|| Error::Protocol {
                    method: "Fetch.authRequired".into(),
                    message: "auth request has no requestId".into(),
                })?;
            if auth.attempted.contains(id) {
                reply.params["authChallengeResponse"] = json!({ "response": "CancelAuth" });
            } else {
                if auth.attempted.len() >= MAX_PENDING {
                    return Err(Error::Protocol {
                        method: "Fetch.continueWithAuth".into(),
                        message: "proxy authentication request limit exceeded".into(),
                    });
                }
                auth.attempted.insert(id.to_string());
            }
        }
        let session = auth.session.clone();
        self.post(reply.method, reply.params, &session).await?;
        Ok(true)
    }

    fn queued(&mut self, msg: Value) -> Result<(), Error> {
        if self.events.len() >= MAX_EVENTS {
            return Err(Error::Protocol {
                method: "CDP events".into(),
                message: "browser event queue is full".into(),
            });
        }
        self.events.push(msg);
        Ok(())
    }

    async fn event(&mut self, msg: Value) -> Result<(), Error> {
        if !self.answer(&msg).await? {
            self.queued(msg)?;
        }
        Ok(())
    }

    async fn acknowledged(&mut self, msg: &Value) -> Result<(), Error> {
        let Some(id) = msg.get("id").and_then(Value::as_u64) else {
            return Ok(());
        };
        if self
            .fetch
            .as_ref()
            .is_some_and(|(pending, _)| *pending == id)
        {
            let (_, patterns) = self.fetch.take().expect("invariant: Fetch ID matched");
            if msg.get("error").is_none() {
                let patterns = serde_json::from_value(patterns)?;
                if let Some(auth) = self.auth.as_mut() {
                    auth.patterns = patterns;
                }
            }
            while let Some(event) = self.held.pop_front() {
                self.event(event).await?;
            }
        }
        if let Some(method) = self.pending.remove(&id) {
            if method == "Fetch.continueRequest"
                && msg.pointer("/error/code").and_then(Value::as_i64) == Some(-32602)
                && msg.pointer("/error/message").and_then(Value::as_str)
                    == Some("Invalid InterceptionId.")
            {
                return Ok(());
            }
            drop(result(msg, method)?);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn sent(&self) -> &BTreeSet<String> {
        &self.sent
    }

    pub(crate) async fn send(
        &mut self,
        method: &str,
        mut params: Value,
        session_id: Option<&str>,
    ) -> Result<Value, Error> {
        if let Some(name) = blocked(method) {
            return Err(Error::Blocked(name));
        }
        let configuring = matches!(method, "Fetch.enable" | "Fetch.disable")
            && self
                .auth
                .as_ref()
                .is_some_and(|auth| Some(auth.session.as_str()) == session_id);
        let mut patterns = None;
        let wire_method = if configuring {
            if self.fetch.is_some() {
                return Err(Error::Protocol {
                    method: method.into(),
                    message: "previous Fetch configuration is not acknowledged".into(),
                });
            }
            if method == "Fetch.disable" {
                params = json!({ "patterns": [] });
            }
            if let Some(object) = params.as_object_mut() {
                let caller = object.entry("patterns").or_insert_with(|| json!([{}]));
                patterns = Some(caller.clone());
                if let Some(list) = caller.as_array_mut() {
                    list.push(json!({ "urlPattern": "*", "requestStage": "Request" }));
                }
                let auth = object.entry("handleAuthRequests").or_insert(json!(true));
                if auth.is_boolean() {
                    *auth = json!(true);
                }
            }
            "Fetch.enable"
        } else {
            method
        };
        let deadline = Instant::now() + COMMAND_DEADLINE;
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
        let deadline = Instant::now() + COMMAND_DEADLINE;
        timeout_at(deadline, self.drain())
            .await
            .map_err(|_elapsed| Error::Timeout("CDP pump".into()))?
    }

    async fn drain(&mut self) -> Result<(), Error> {
        let deadline = Instant::now() + PUMP;
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

    pub(crate) fn take_events(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.events)
    }

    pub(crate) async fn close(&mut self) {
        drop(self.ws.close(None).await);
    }
}

fn result(msg: &Value, method: &str) -> Result<Value, Error> {
    if let Some(error) = msg.get("error") {
        return Err(Error::Protocol {
            method: method.to_string(),
            message: error.to_string(),
        });
    }
    Ok(msg.get("result").cloned().unwrap_or(Value::Null))
}

fn closed() -> Error {
    Error::Connect("CDP websocket closed".into())
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;
