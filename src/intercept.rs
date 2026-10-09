use crate::Error;
use crate::route::{Route, route_reply};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::VecDeque;

const MAX_ATTEMPTS: usize = 1024;

#[derive(Deserialize)]
pub(crate) struct Pattern {
    #[serde(rename = "urlPattern")]
    url: Option<String>,
    #[serde(rename = "resourceType")]
    resource: Option<String>,
    #[serde(rename = "requestStage")]
    stage: Option<String>,
}

impl Pattern {
    pub(crate) fn matches(&self, params: &Value) -> bool {
        if self.stage.as_deref().unwrap_or("Request") != "Request" {
            return false;
        }
        let resource = params.get("resourceType").and_then(Value::as_str);
        let selected = match self.resource.as_deref() {
            None | Some("") => true,
            Some("Fetch") => matches!(resource, Some("XHR" | "Fetch")),
            Some("XHR" | "EventSource") => resource == Some("XHR"),
            Some(filter) => resource == Some(filter),
        };
        selected
            && params
                .pointer("/request/url")
                .and_then(Value::as_str)
                .is_some_and(|url| {
                    let url = url.split('#').next().unwrap_or(url);
                    matches_url(self.url.as_deref().unwrap_or("*"), url)
                })
    }
}

pub(crate) fn matches_url(pattern: &str, mut url: &str) -> bool {
    let mut chars = pattern.chars().peekable();
    loop {
        let mut distance = Some(0_usize);
        while let Some(&wildcard @ ('*' | '?')) = chars.peek() {
            chars.next();
            distance = match wildcard {
                '*' => None,
                _ => distance.map(|limit| limit + 1),
            };
        }
        let mut literal = String::new();
        while let Some(&character) = chars.peek() {
            if matches!(character, '*' | '?') {
                break;
            }
            chars.next();
            if character == '\\' {
                if let Some(escaped) = chars.next() {
                    literal.push(escaped);
                }
            } else {
                literal.push(character);
            }
        }
        let last = chars.peek().is_none();
        loop {
            if let Some(rest) = url.strip_prefix(&literal)
                && (!last || rest.is_empty())
            {
                url = rest;
                break;
            }
            if distance == Some(0) {
                return false;
            }
            let Some(character) = url.chars().next() else {
                return false;
            };
            url = &url[character.len_utf8()..];
            distance = distance.map(|limit| limit - 1);
        }
        if last {
            return true;
        }
    }
}

pub(crate) struct Reply {
    pub(crate) method: &'static str,
    pub(crate) params: Value,
    pub(crate) kind: ReplyKind,
}

pub(crate) enum ReplyKind {
    Auth { proxy: bool },
    Continue,
    Route,
}

pub(crate) fn continue_reply(id: &str) -> Reply {
    Reply {
        method: "Fetch.continueRequest",
        params: json!({ "requestId": id }),
        kind: ReplyKind::Continue,
    }
}

pub(crate) fn auth_reply(msg: &Value, user: &str, pass: &str) -> Option<Reply> {
    let method = msg.get("method").and_then(Value::as_str)?;
    let params = msg.get("params")?;
    let id = params.get("requestId").and_then(Value::as_str)?;
    match method {
        "Fetch.authRequired" => {
            let proxy = params
                .pointer("/authChallenge/source")
                .and_then(Value::as_str)
                == Some("Proxy");
            let response = if proxy {
                json!({ "response": "ProvideCredentials", "username": user, "password": pass })
            } else {
                json!({ "response": "Default" })
            };
            Some(Reply {
                method: "Fetch.continueWithAuth",
                params: json!({ "requestId": id, "authChallengeResponse": response }),
                kind: ReplyKind::Auth { proxy },
            })
        }
        "Fetch.requestPaused" => Some(continue_reply(id)),
        _ => None,
    }
}

pub(crate) enum Verdict {
    Pass,
    Hold,
    Reply(Reply),
}

pub(crate) struct Intercept {
    pub(crate) session: Option<String>,
    login: Option<(String, String)>,
    attempted: VecDeque<String>,
    patterns: Vec<Pattern>,
    pub(crate) routes: Vec<Route>,
}

impl Intercept {
    pub(crate) fn new(session: Option<&str>) -> Self {
        Self {
            session: session.map(str::to_owned),
            login: None,
            attempted: VecDeque::new(),
            patterns: Vec::new(),
            routes: Vec::new(),
        }
    }

    pub(crate) fn login(&mut self, user: String, pass: String) {
        self.login = Some((user, pass));
    }

    pub(crate) fn active(&self) -> bool {
        self.login.is_some() || !self.routes.is_empty()
    }

    pub(crate) fn accept(&mut self, patterns: Value) -> Result<(), Error> {
        self.patterns = serde_json::from_value(patterns)?;
        Ok(())
    }

    pub(crate) fn configure(&self, method: &str, mut params: Value) -> (Value, Option<Value>) {
        if method == "Fetch.disable" {
            params = json!({ "patterns": [] });
        }
        let mut caller = None;
        if let Some(object) = params.as_object_mut() {
            let list = object.entry("patterns").or_insert_with(|| json!([{}]));
            caller = Some(list.clone());
            if let Some(list) = list.as_array_mut() {
                list.push(json!({ "urlPattern": "*", "requestStage": "Request" }));
            }
            if self.login.is_some() {
                let auth = object.entry("handleAuthRequests").or_insert(json!(true));
                if auth.is_boolean() {
                    *auth = json!(true);
                }
            }
        }
        (params, caller)
    }

    pub(crate) fn decide(&mut self, msg: &Value, configuring: bool) -> Result<Verdict, Error> {
        if let Some(session) = self.session.as_deref()
            && msg.get("sessionId").and_then(Value::as_str) != Some(session)
        {
            return Ok(Verdict::Pass);
        }
        let params = &msg["params"];
        if msg.get("method").and_then(Value::as_str) == Some("Fetch.requestPaused") {
            if params.get("responseStatusCode").is_some()
                || params.get("responseErrorReason").is_some()
            {
                if let Some(id) = params.get("requestId").and_then(Value::as_str) {
                    self.forget(id);
                }
                return Ok(Verdict::Pass);
            }
            if configuring {
                return Ok(Verdict::Hold);
            }
            if self.patterns.iter().any(|pattern| pattern.matches(params)) {
                return Ok(Verdict::Pass);
            }
            if let Some(reply) = route_reply(&self.routes, params) {
                return Ok(Verdict::Reply(reply));
            }
            if self.login.is_none() {
                return Ok(params
                    .get("requestId")
                    .and_then(Value::as_str)
                    .map_or(Verdict::Pass, |id| Verdict::Reply(continue_reply(id))));
            }
        }
        let Some((user, pass)) = self.login.as_ref() else {
            return Ok(Verdict::Pass);
        };
        let Some(mut reply) = auth_reply(msg, user, pass) else {
            return Ok(Verdict::Pass);
        };
        if let ReplyKind::Auth { proxy: true } = reply.kind {
            let id = params
                .get("requestId")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::Protocol {
                    method: "Fetch.authRequired".into(),
                    code: None,
                    message: "auth request has no requestId".into(),
                })?;
            if self.attempted.iter().any(|seen| seen == id) {
                reply.params["authChallengeResponse"] = json!({ "response": "CancelAuth" });
            } else {
                if self.attempted.len() >= MAX_ATTEMPTS {
                    drop(self.attempted.pop_front());
                }
                self.attempted.push_back(id.to_string());
            }
        }
        Ok(Verdict::Reply(reply))
    }

    fn forget(&mut self, id: &str) {
        self.attempted.retain(|seen| seen != id);
    }
}
