use crate::Error;
use crate::downloads::Transfers;
use crate::transport::{Transport, blocked};
use serde_json::{Value, json};
use std::collections::{BTreeSet, VecDeque};
use std::time::Duration;
use tokio::time::Instant;

const MAX_RECORDS: usize = 1024;
const MAX_INFLIGHT: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Events {
    pub page: bool,
    pub runtime: bool,
}

impl Events {
    #[must_use]
    pub fn all() -> Self {
        Self {
            page: true,
            runtime: true,
        }
    }

    pub(crate) fn methods(self) -> impl Iterator<Item = &'static str> {
        [(self.page, "Page.enable"), (self.runtime, "Runtime.enable")]
            .into_iter()
            .filter_map(|(on, method)| on.then_some(method))
    }

    pub(crate) fn refused(self, method: &str) -> Option<&'static str> {
        blocked(method).filter(|name| !self.methods().any(|allowed| allowed == *name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DialogPolicy {
    Accept,
    #[default]
    Dismiss,
    AcceptWith(String),
}

impl DialogPolicy {
    fn params(&self) -> Value {
        match self {
            Self::Accept => json!({ "accept": true }),
            Self::Dismiss => json!({ "accept": false }),
            Self::AcceptWith(text) => json!({ "accept": true, "promptText": text }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    Alert,
    Confirm,
    Prompt,
    BeforeUnload,
}

impl DialogKind {
    fn parse(name: &str) -> Self {
        match name {
            "confirm" => Self::Confirm,
            "prompt" => Self::Prompt,
            "beforeunload" => Self::BeforeUnload,
            _ => Self::Alert,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dialog {
    pub kind: DialogKind,
    pub message: String,
    pub url: String,
    pub default_prompt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleLevel {
    Debug,
    Info,
    Log,
    Warning,
    Error,
    Exception,
}

impl ConsoleLevel {
    fn parse(name: &str) -> Self {
        match name {
            "debug" | "trace" => Self::Debug,
            "info" => Self::Info,
            "warning" => Self::Warning,
            "error" | "assert" => Self::Error,
            _ => Self::Log,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleMessage {
    pub level: ConsoleLevel,
    pub text: String,
    pub url: String,
    pub line: u64,
}

fn text(value: &Value, pointer: &str) -> String {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn arg_text(arg: &Value) -> String {
    match arg.get("value") {
        Some(Value::String(text)) => text.clone(),
        Some(value) if !value.is_null() => value.to_string(),
        _ => text(arg, "/description"),
    }
}

fn console_called(params: &Value) -> ConsoleMessage {
    let args = params.get("args").and_then(Value::as_array);
    let text = args
        .map(|args| args.iter().map(arg_text).collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    ConsoleMessage {
        level: ConsoleLevel::parse(params.get("type").and_then(Value::as_str).unwrap_or("log")),
        text,
        url: self::text(params, "/stackTrace/callFrames/0/url"),
        line: params
            .pointer("/stackTrace/callFrames/0/lineNumber")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
    }
}

fn exception_thrown(params: &Value) -> ConsoleMessage {
    let description = text(params, "/exceptionDetails/exception/description");
    ConsoleMessage {
        level: ConsoleLevel::Exception,
        text: if description.is_empty() {
            text(params, "/exceptionDetails/text")
        } else {
            description
        },
        url: text(params, "/exceptionDetails/url"),
        line: params
            .pointer("/exceptionDetails/lineNumber")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
    }
}

pub(crate) fn bounded<T>(ring: &mut VecDeque<T>, item: T) -> u64 {
    let mut dropped = 0;
    if ring.len() >= MAX_RECORDS {
        drop(ring.pop_front());
        dropped = 1;
    }
    ring.push_back(item);
    dropped
}

pub(crate) struct Observer {
    pub(crate) events: Events,
    policy: DialogPolicy,
    dialogs: VecDeque<Dialog>,
    console: VecDeque<ConsoleMessage>,
    pub(crate) dropped: u64,
    inflight: BTreeSet<(String, String)>,
    pub(crate) transfers: Transfers,
    quiet_since: Instant,
}

impl Observer {
    pub(crate) fn new() -> Self {
        Self {
            events: Events::default(),
            policy: DialogPolicy::default(),
            dialogs: VecDeque::new(),
            console: VecDeque::new(),
            dropped: 0,
            inflight: BTreeSet::new(),
            transfers: Transfers::default(),
            quiet_since: Instant::now(),
        }
    }

    pub(crate) fn opt_in(&mut self, events: Events, policy: DialogPolicy) {
        self.events = events;
        self.policy = policy;
    }

    pub(crate) fn observe(&mut self, msg: &Value) -> Option<(&'static str, Value)> {
        let params = &msg["params"];
        match msg.get("method").and_then(Value::as_str)? {
            "Page.javascriptDialogOpening" => {
                return Some(("Page.handleJavaScriptDialog", self.dialog(params)));
            }
            "Page.fileChooserOpened" => return self.transfers.file_chooser(params),
            "Browser.downloadWillBegin" => self.dropped += self.transfers.begin(params),
            "Browser.downloadProgress" => self.transfers.progress(params),
            "Runtime.consoleAPICalled" => {
                self.dropped += bounded(&mut self.console, console_called(params));
            }
            "Runtime.exceptionThrown" => {
                self.dropped += bounded(&mut self.console, exception_thrown(params));
            }
            "Network.requestWillBeSent" => self.request(msg, true),
            "Network.loadingFinished" | "Network.loadingFailed" => self.request(msg, false),
            _ => {}
        }
        None
    }

    fn dialog(&mut self, params: &Value) -> Value {
        let dialog = Dialog {
            kind: DialogKind::parse(params.get("type").and_then(Value::as_str).unwrap_or("")),
            message: text(params, "/message"),
            url: text(params, "/url"),
            default_prompt: text(params, "/defaultPrompt"),
        };
        self.dropped += bounded(&mut self.dialogs, dialog);
        self.policy.params()
    }

    fn request(&mut self, msg: &Value, started: bool) {
        let Some(id) = msg.pointer("/params/requestId").and_then(Value::as_str) else {
            return;
        };
        let session = msg.get("sessionId").and_then(Value::as_str).unwrap_or("");
        let key = (session.to_string(), id.to_string());
        let changed = if started {
            self.inflight.len() < MAX_INFLIGHT && self.inflight.insert(key)
        } else {
            self.inflight.remove(&key)
        };
        if changed {
            self.quiet_since = Instant::now();
        }
    }

    pub(crate) fn quiet_at(&self, quiet: Duration) -> Option<Instant> {
        self.inflight.is_empty().then_some(self.quiet_since + quiet)
    }

    pub(crate) fn take_dialogs(&mut self) -> Vec<Dialog> {
        self.dialogs.drain(..).collect()
    }

    pub(crate) fn take_console(&mut self) -> Vec<ConsoleMessage> {
        self.console.drain(..).collect()
    }
}

impl Transport {
    pub(crate) fn opt_in(&mut self, events: Events, policy: DialogPolicy) {
        self.observer.opt_in(events, policy);
    }

    pub(crate) async fn enable_events(
        &mut self,
        session_id: &str,
        events: Events,
    ) -> Result<(), Error> {
        for method in events.methods() {
            drop(self.send(method, json!({}), Some(session_id)).await?);
        }
        Ok(())
    }

    pub(crate) fn take_dialogs(&mut self) -> Vec<Dialog> {
        self.observer.take_dialogs()
    }

    pub(crate) fn take_console(&mut self) -> Vec<ConsoleMessage> {
        self.observer.take_console()
    }

    pub(crate) fn dropped_records(&self) -> u64 {
        self.observer.dropped
    }

    pub(crate) async fn network_quiet(
        &mut self,
        quiet: Duration,
        poll: Duration,
        timeout: Duration,
    ) -> Result<(), Error> {
        let deadline = Instant::now() + timeout;
        loop {
            let now = Instant::now();
            let ready = self.observer.quiet_at(quiet);
            if ready.is_some_and(|at| at <= now) {
                return Ok(());
            }
            if now >= deadline {
                return Err(Error::Timeout("network idle".into()));
            }
            let next = ready.unwrap_or(now + poll).min(now + poll).min(deadline);
            self.idle(next).await?;
        }
    }
}

impl crate::session::Session {
    pub async fn set_events(&mut self, events: Events, policy: DialogPolicy) -> Result<(), Error> {
        self.transport_mut().opt_in(events, policy);
        let sessions: Vec<String> = self.tabs().session_ids();
        for session_id in sessions {
            self.transport_mut()
                .enable_events(&session_id, events)
                .await?;
        }
        Ok(())
    }

    pub fn dialogs(&mut self) -> Vec<Dialog> {
        self.transport_mut().take_dialogs()
    }

    pub fn console(&mut self) -> Vec<ConsoleMessage> {
        self.transport_mut().take_console()
    }

    #[must_use]
    pub fn dropped_records(&mut self) -> u64 {
        self.transport_mut().dropped_records()
    }
}
