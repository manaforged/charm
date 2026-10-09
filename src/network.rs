use crate::error::Error;
use crate::session::Session;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlMatch {
    Exact(String),
    Contains(String),
    Glob(String),
}

impl UrlMatch {
    #[must_use]
    pub fn matches(&self, url: &str) -> bool {
        match self {
            Self::Exact(want) => url == want,
            Self::Contains(part) => url.contains(part.as_str()),
            Self::Glob(glob) => {
                let pattern: Vec<char> = glob.chars().collect();
                let text: Vec<char> = url.chars().collect();
                glob_match(&pattern, &text)
            }
        }
    }
}

fn glob_match(pattern: &[char], text: &[char]) -> bool {
    match pattern.split_first() {
        None => text.is_empty(),
        Some(('*', rest)) if rest.first() == Some(&'*') => {
            let tail = &rest[1..];
            (0..=text.len()).any(|skip| glob_match(tail, &text[skip..]))
        }
        Some(('*', rest)) => {
            let span = text.iter().take_while(|c| **c != '/').count();
            (0..=span).any(|skip| glob_match(rest, &text[skip..]))
        }
        Some(('?', rest)) => !text.is_empty() && glob_match(rest, &text[1..]),
        Some((c, rest)) => text.first() == Some(c) && glob_match(rest, &text[1..]),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedRequest {
    pub request_id: String,
    pub url: String,
    pub method: String,
    pub headers: BTreeMap<String, String>,
    pub resource_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedResponse {
    pub request_id: String,
    pub url: String,
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub resource_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Cookie {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<f64>,
    #[serde(default)]
    pub http_only: bool,
    #[serde(default)]
    pub secure: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same_site: Option<SameSite>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NetworkEvent {
    RequestWillBeSent,
    ResponseReceived,
}

impl NetworkEvent {
    fn method(self) -> &'static str {
        match self {
            Self::RequestWillBeSent => "Network.requestWillBeSent",
            Self::ResponseReceived => "Network.responseReceived",
        }
    }

    fn payload(self) -> &'static str {
        match self {
            Self::RequestWillBeSent => "request",
            Self::ResponseReceived => "response",
        }
    }
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn lower_headers(value: Option<&Value>) -> BTreeMap<String, String> {
    value
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .map(|(key, val)| {
                    let shown = val.as_str().map_or_else(|| val.to_string(), str::to_string);
                    (key.to_lowercase(), shown)
                })
                .collect()
        })
        .unwrap_or_default()
}

impl Session {
    async fn wait_network(
        &mut self,
        kind: NetworkEvent,
        url: &UrlMatch,
    ) -> Result<(Value, Value), Error> {
        self.network_on().await?;
        let deadline = Instant::now() + self.timing().wait;
        loop {
            let found = self
                .take_event(|event| {
                    event.get("method").and_then(Value::as_str) == Some(kind.method())
                        && event
                            .get("params")
                            .and_then(|params| params.get(kind.payload()))
                            .is_some_and(|payload| url.matches(&text(payload, "url")))
                })
                .await?;
            if let Some(event) = found {
                let params = event.get("params").cloned().unwrap_or(Value::Null);
                let payload = params.get(kind.payload()).cloned().unwrap_or(Value::Null);
                return Ok((params, payload));
            }
            if Instant::now() >= deadline {
                return Err(Error::Timeout(kind.method().into()));
            }
        }
    }

    pub async fn wait_for_request(&mut self, url: UrlMatch) -> Result<ObservedRequest, Error> {
        let (params, request) = self
            .wait_network(NetworkEvent::RequestWillBeSent, &url)
            .await?;
        Ok(ObservedRequest {
            request_id: text(&params, "requestId"),
            url: text(&request, "url"),
            method: text(&request, "method"),
            headers: lower_headers(request.get("headers")),
            resource_type: text(&params, "type"),
        })
    }

    pub async fn wait_for_response(&mut self, url: UrlMatch) -> Result<ObservedResponse, Error> {
        let (params, response) = self
            .wait_network(NetworkEvent::ResponseReceived, &url)
            .await?;
        let status = response
            .get("status")
            .and_then(Value::as_u64)
            .and_then(|code| u16::try_from(code).ok())
            .unwrap_or_default();
        Ok(ObservedResponse {
            request_id: text(&params, "requestId"),
            url: text(&response, "url"),
            status,
            headers: lower_headers(response.get("headers")),
            resource_type: text(&params, "type"),
        })
    }

    pub async fn clear_cookies(&mut self) -> Result<(), Error> {
        drop(self.send("Network.clearBrowserCookies", json!({})).await?);
        Ok(())
    }

    pub async fn add_cookies(&mut self, cookies: Vec<Cookie>) -> Result<(), Error> {
        self.set_cookies(serde_json::to_value(cookies)?).await
    }
}

#[cfg(test)]
#[path = "network_tests.rs"]
mod tests;
