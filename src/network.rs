use crate::error::Error;
use crate::session::Session;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
    let (mut at, mut to) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    let mut globstar: Option<(usize, usize)> = None;
    while let Some(&current) = text.get(to) {
        match pattern.get(at) {
            Some('*') if pattern.get(at + 1) == Some(&'*') => {
                at += 2;
                globstar = Some((at, to));
                star = None;
                continue;
            }
            Some('*') => {
                at += 1;
                star = Some((at, to));
                continue;
            }
            Some('?') => {
                at += 1;
                to += 1;
                continue;
            }
            Some(literal) if *literal == current => {
                at += 1;
                to += 1;
                continue;
            }
            _ => {}
        }
        match (star, globstar) {
            (Some((resume, from)), _) if text.get(from) != Some(&'/') => {
                star = Some((resume, from + 1));
                (at, to) = (resume, from + 1);
            }
            (_, Some((resume, from))) => {
                star = None;
                globstar = Some((resume, from + 1));
                (at, to) = (resume, from + 1);
            }
            _ => return false,
        }
    }
    pattern
        .get(at..)
        .is_some_and(|rest| rest.iter().all(|c| *c == '*'))
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ObservedRequest {
    pub request_id: String,
    pub url: String,
    pub method: String,
    pub headers: BTreeMap<String, String>,
    pub resource_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ObservedResponse {
    pub request_id: String,
    pub url: String,
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub resource_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
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

pub(crate) fn decode_base64(data: &str, method: &str) -> Result<Vec<u8>, Error> {
    let mut out = Vec::with_capacity(data.len() / 4 * 3);
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for byte in data.bytes() {
        let sextet = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\n' | b'\r' => continue,
            _ => {
                return Err(Error::Protocol {
                    method: method.into(),
                    code: None,
                    message: "invalid base64 data".into(),
                });
            }
        };
        buffer = (buffer << 6) | u32::from(sextet);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.extend_from_slice(&((buffer >> bits) & 0xff).to_be_bytes()[3..]);
        }
    }
    Ok(out)
}

pub(crate) fn encode_base64(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let bytes = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let group = u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
        for slot in 0..4 {
            if slot <= chunk.len() {
                let index = (group >> (18 - 6 * slot)) & 0x3f;
                out.push(char::from(ALPHABET[index as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

impl Session {
    async fn wait_event(
        &mut self,
        method: &str,
        wanted: impl Fn(&Value) -> bool,
    ) -> Result<Value, Error> {
        self.network_on().await?;
        let deadline = Instant::now() + self.timing().wait;
        loop {
            let found = self
                .take_event(|event| {
                    event.get("method").and_then(Value::as_str) == Some(method)
                        && event.get("params").is_some_and(&wanted)
                })
                .await?;
            if let Some(event) = found {
                return Ok(event.get("params").cloned().unwrap_or(Value::Null));
            }
            if Instant::now() >= deadline {
                return Err(Error::Timeout(method.into()));
            }
        }
    }

    async fn wait_network(
        &mut self,
        kind: NetworkEvent,
        url: &UrlMatch,
    ) -> Result<(Value, Value), Error> {
        let params = self
            .wait_event(kind.method(), |params| {
                params
                    .get(kind.payload())
                    .is_some_and(|payload| url.matches(&text(payload, "url")))
            })
            .await?;
        let payload = params.get(kind.payload()).cloned().unwrap_or(Value::Null);
        Ok((params, payload))
    }

    pub async fn response_body(&mut self, response: &ObservedResponse) -> Result<Vec<u8>, Error> {
        let id = response.request_id.as_str();
        drop(
            self.wait_event("Network.loadingFinished", |params| {
                params.get("requestId").and_then(Value::as_str) == Some(id)
            })
            .await?,
        );
        let body = self
            .send("Network.getResponseBody", json!({ "requestId": id }))
            .await?;
        let data = body.get("body").and_then(Value::as_str).unwrap_or_default();
        if body.get("base64Encoded").and_then(Value::as_bool) == Some(true) {
            decode_base64(data, "Network.getResponseBody")
        } else {
            Ok(data.as_bytes().to_vec())
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
        drop(
            self.send("Network.setCookies", json!({ "cookies": cookies }))
                .await?,
        );
        Ok(())
    }

    pub async fn cookies(&mut self) -> Result<Vec<Cookie>, Error> {
        let jar = self.browser_send("Storage.getCookies", json!({})).await?;
        Ok(serde_json::from_value(
            jar.get("cookies")
                .cloned()
                .unwrap_or(Value::Array(Vec::new())),
        )?)
    }

    pub async fn cookie_named(&mut self, name: &str) -> Result<Option<Cookie>, Error> {
        Ok(self
            .cookies()
            .await?
            .into_iter()
            .find(|cookie| cookie.name == name))
    }
}

#[cfg(test)]
#[path = "network_tests.rs"]
mod tests;

const GLOB_SPECIAL: &str = r"\^$.|+()[]{}";

impl UrlMatch {
    pub(crate) fn text_match(self) -> crate::selector::TextMatch {
        use crate::selector::TextMatch;
        match self {
            Self::Exact(want) => TextMatch::Exact(want),
            Self::Contains(part) => TextMatch::Contains(part),
            Self::Glob(glob) => TextMatch::regex(&glob_regex(&glob), ""),
        }
    }
}

fn glob_regex(glob: &str) -> String {
    let mut out = String::from("^");
    let mut chars = glob.chars().peekable();
    while let Some(current) = chars.next() {
        match current {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                out.push_str(".*");
            }
            '*' => out.push_str("[^/]*"),
            '?' => out.push('.'),
            special if GLOB_SPECIAL.contains(special) => {
                out.push('\\');
                out.push(special);
            }
            literal => out.push(literal),
        }
    }
    out.push('$');
    out
}
