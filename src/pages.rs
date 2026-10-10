use crate::argv::BLANK;
use crate::error::Error;
use crate::session::Session;
use crate::snapshot::RefMap;
use crate::timing::Timing;
use crate::transport::Transport;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Instant, timeout};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageId(String);

impl PageId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct PageInfo {
    pub id: PageId,
    pub url: String,
    pub title: String,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    Port(u16),
    WebSocket(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TargetKind {
    Page,
    Iframe,
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TargetInfo {
    pub(crate) target_id: String,
    #[serde(rename = "type")]
    pub(crate) kind: TargetKind,
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) title: String,
}

#[derive(Debug)]
pub(crate) struct PageState {
    pub(crate) id: PageId,
    pub(crate) session_id: String,
    pub(crate) frame_sessions: BTreeMap<String, String>,
    pub(crate) worlds: BTreeMap<String, i64>,
    pub(crate) refs: RefMap,
}

impl PageState {
    fn new(id: PageId, session_id: String) -> Self {
        Self {
            id,
            session_id,
            frame_sessions: BTreeMap::new(),
            worlds: BTreeMap::new(),
            refs: RefMap::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Tabs {
    pub(crate) active: PageState,
    parked: BTreeMap<PageId, PageState>,
    seen: BTreeSet<String>,
    pub(crate) human: Option<crate::world::Human>,
}

impl Tabs {
    pub(crate) fn session_ids(&self) -> Vec<String> {
        std::iter::once(&self.active)
            .chain(self.parked.values())
            .map(|page| page.session_id.clone())
            .collect()
    }

    pub(crate) fn new(active: PageState, seen: BTreeSet<String>) -> Self {
        Self {
            active,
            parked: BTreeMap::new(),
            seen,
            human: None,
        }
    }

    fn push_active(&mut self, state: PageState) {
        self.seen.insert(state.id.0.clone());
        let previous = std::mem::replace(&mut self.active, state);
        self.parked.insert(previous.id.clone(), previous);
    }

    fn activate(&mut self, id: &PageId) -> bool {
        match self.parked.remove(id) {
            Some(state) => {
                self.push_active(state);
                true
            }
            None => self.active.id == *id,
        }
    }
}

pub(crate) async fn list_targets(transport: &mut Transport) -> Result<Vec<TargetInfo>, Error> {
    let listed = transport.send("Target.getTargets", json!({}), None).await?;
    Ok(serde_json::from_value(
        listed
            .get("targetInfos")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new())),
    )?)
}

pub(crate) async fn attach_target(
    transport: &mut Transport,
    target_id: &str,
) -> Result<String, Error> {
    let attached = transport
        .send(
            "Target.attachToTarget",
            json!({ "targetId": target_id, "flatten": true }),
            None,
        )
        .await?;
    let session_id = attached
        .get("sessionId")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| missing("Target.attachToTarget", "sessionId"))?;
    let events = transport.observer.events;
    transport.enable_events(&session_id, events).await?;
    transport.track(&session_id).await?;
    Ok(session_id)
}

async fn adopt(transport: &mut Transport, target_id: String) -> Result<PageState, Error> {
    let session_id = attach_target(transport, &target_id).await?;
    Ok(PageState::new(PageId(target_id), session_id))
}

async fn open_target(transport: &mut Transport) -> Result<PageState, Error> {
    let created = transport
        .send("Target.createTarget", json!({ "url": BLANK }), None)
        .await?;
    let target_id = created
        .get("targetId")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| missing("Target.createTarget", "targetId"))?;
    adopt(transport, target_id).await
}

pub(crate) async fn initial_tabs(transport: &mut Transport, reuse: bool) -> Result<Tabs, Error> {
    let pages: Vec<TargetInfo> = list_targets(transport)
        .await?
        .into_iter()
        .filter(|info| info.kind == TargetKind::Page)
        .collect();
    let existing = pages.first().filter(|_| reuse).map(|t| t.target_id.clone());
    let active = match existing {
        Some(target_id) => adopt(transport, target_id).await?,
        None => open_target(transport).await?,
    };
    let mut seen: BTreeSet<String> = pages.into_iter().map(|t| t.target_id).collect();
    seen.insert(active.id.0.clone());
    Ok(Tabs::new(active, seen))
}

pub(crate) async fn websocket_for(endpoint: Endpoint, timing: Timing) -> Result<String, Error> {
    match endpoint {
        Endpoint::WebSocket(url) => Ok(url),
        Endpoint::Port(port) => timeout(timing.connect, version_url(port))
            .await
            .map_err(|_elapsed| Error::Timeout(format!("127.0.0.1:{port}/json/version")))?,
    }
}

async fn version_url(port: u16) -> Result<String, Error> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
    let request = format!(
        "GET /json/version HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await?;
    let mut raw = Vec::new();
    let mut chunk = [0u8; 4096];
    let (head_end, length) = loop {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            return Err(Error::Connect("/json/version reply ended early".into()));
        }
        raw.extend_from_slice(&chunk[..read]);
        if let Some(end) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
            break (end + 4, content_length(&raw[..end]));
        }
    };
    while raw.len() < head_end + length {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        raw.extend_from_slice(&chunk[..read]);
    }
    let version: Value = serde_json::from_slice(&raw[head_end..])?;
    version
        .get("webSocketDebuggerUrl")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| Error::Connect("/json/version missing webSocketDebuggerUrl".into()))
}

fn content_length(head: &[u8]) -> usize {
    String::from_utf8_lossy(head)
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse().ok())
        .unwrap_or(0)
}

fn missing(method: &str, field: &str) -> Error {
    Error::Protocol {
        method: method.into(),
        code: None,
        message: format!("missing {field}"),
    }
}

fn unknown(id: &PageId) -> Error {
    Error::Protocol {
        method: "Target.getTargets".into(),
        code: None,
        message: format!("no page {id}"),
    }
}

impl Session {
    async fn page_targets(&mut self) -> Result<Vec<TargetInfo>, Error> {
        let targets = list_targets(self.transport_mut()).await?;
        Ok(targets
            .into_iter()
            .filter(|info| info.kind == TargetKind::Page)
            .collect())
    }

    pub async fn pages(&mut self) -> Result<Vec<PageInfo>, Error> {
        let targets = self.page_targets().await?;
        let tabs = self.tabs_mut();
        tabs.seen
            .extend(targets.iter().map(|t| t.target_id.clone()));
        let active = tabs.active.id.0.clone();
        Ok(targets
            .into_iter()
            .map(|t| PageInfo {
                active: t.target_id == active,
                id: PageId(t.target_id),
                url: t.url,
                title: t.title,
            })
            .collect())
    }

    #[must_use]
    pub fn active_page(&self) -> &PageId {
        &self.tabs().active.id
    }

    pub async fn new_page(&mut self, url: Option<&str>) -> Result<PageId, Error> {
        let state = open_target(self.transport_mut()).await?;
        let id = state.id.clone();
        self.tabs_mut().push_active(state);
        if let Some(url) = url {
            self.goto(url).await?;
        }
        Ok(id)
    }

    pub async fn select_page(&mut self, id: &PageId) -> Result<(), Error> {
        if !self.tabs_mut().activate(id) {
            let known = self.page_targets().await?;
            if !known.iter().any(|t| t.target_id == id.0) {
                return Err(unknown(id));
            }
            let state = adopt(self.transport_mut(), id.0.clone()).await?;
            self.tabs_mut().push_active(state);
        }
        drop(
            self.browser_send("Target.activateTarget", json!({ "targetId": id.0 }))
                .await?,
        );
        Ok(())
    }

    pub async fn close_page(&mut self, id: &PageId) -> Result<(), Error> {
        if self.tabs().active.id == *id {
            let next = self.tabs().parked.keys().next().cloned();
            match next {
                Some(next) => self.select_page(&next).await?,
                None => drop(self.new_page(None).await?),
            }
        }
        if let Some(state) = self.tabs_mut().parked.remove(id) {
            self.transport_mut().untrack(&state.session_id);
        }
        drop(
            self.browser_send("Target.closeTarget", json!({ "targetId": id.0 }))
                .await?,
        );
        Ok(())
    }

    pub async fn wait_for_page(&mut self) -> Result<PageId, Error> {
        let deadline = Instant::now() + self.timing().wait;
        loop {
            let fresh = self
                .page_targets()
                .await?
                .into_iter()
                .find(|t| !self.tabs().seen.contains(&t.target_id));
            if let Some(target) = fresh {
                match adopt(self.transport_mut(), target.target_id.clone()).await {
                    Ok(state) => {
                        let id = state.id.clone();
                        self.tabs_mut().push_active(state);
                        return Ok(id);
                    }
                    Err(Error::Protocol { .. }) => {
                        self.tabs_mut().seen.insert(target.target_id);
                        continue;
                    }
                    Err(error) => return Err(error),
                }
            }
            if Instant::now() >= deadline {
                return Err(Error::Timeout("wait_for_page".into()));
            }
            self.idle(self.timing().poll).await?;
        }
    }

    pub async fn page_info(&mut self, id: &PageId) -> Result<PageInfo, Error> {
        self.pages()
            .await?
            .into_iter()
            .find(|page| page.id == *id)
            .ok_or_else(|| unknown(id))
    }
}
