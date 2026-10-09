use serde::Deserialize;
use serde_json::{Value, json};

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
        "Fetch.requestPaused" => Some(Reply {
            method: "Fetch.continueRequest",
            params: json!({ "requestId": id }),
            kind: ReplyKind::Continue,
        }),
        _ => None,
    }
}
