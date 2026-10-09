use crate::error::Error;
use crate::intercept::{Reply, ReplyKind};
use crate::network::{UrlMatch, encode_base64};
use crate::session::Session;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum ErrorReason {
    Failed,
    Aborted,
    TimedOut,
    AccessDenied,
    ConnectionClosed,
    ConnectionReset,
    ConnectionRefused,
    ConnectionAborted,
    ConnectionFailed,
    NameNotResolved,
    InternetDisconnected,
    AddressUnreachable,
    BlockedByClient,
    BlockedByResponse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RouteAction {
    Abort(ErrorReason),
    Fulfill {
        status: u16,
        headers: BTreeMap<String, String>,
        body: Vec<u8>,
    },
    Continue {
        url: Option<String>,
        method: Option<String>,
        headers: Option<BTreeMap<String, String>>,
        post_data: Option<Vec<u8>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Route {
    pub(crate) url: UrlMatch,
    pub(crate) action: RouteAction,
}

fn header_entries(headers: &BTreeMap<String, String>) -> Value {
    headers
        .iter()
        .map(|(name, value)| json!({ "name": name, "value": value }))
        .collect()
}

pub(crate) fn route_reply(routes: &[Route], params: &Value) -> Option<Reply> {
    let id = params.get("requestId").and_then(Value::as_str)?;
    let url = params.pointer("/request/url").and_then(Value::as_str)?;
    let route = routes.iter().find(|route| route.url.matches(url))?;
    let (method, params) = match &route.action {
        RouteAction::Abort(reason) => (
            "Fetch.failRequest",
            json!({ "requestId": id, "errorReason": reason }),
        ),
        RouteAction::Fulfill {
            status,
            headers,
            body,
        } => (
            "Fetch.fulfillRequest",
            json!({
                "requestId": id,
                "responseCode": status,
                "responseHeaders": header_entries(headers),
                "body": encode_base64(body),
            }),
        ),
        RouteAction::Continue {
            url,
            method,
            headers,
            post_data,
        } => {
            let mut params = json!({ "requestId": id });
            if let Some(url) = url {
                params["url"] = json!(url);
            }
            if let Some(method) = method {
                params["method"] = json!(method);
            }
            if let Some(headers) = headers {
                params["headers"] = header_entries(headers);
            }
            if let Some(data) = post_data {
                params["postData"] = json!(encode_base64(data));
            }
            ("Fetch.continueRequest", params)
        }
    };
    Some(Reply {
        method,
        params,
        kind: ReplyKind::Route,
    })
}

impl Session {
    pub async fn route(&mut self, url: UrlMatch, action: RouteAction) -> Result<(), Error> {
        let session = self.session_id().to_string();
        self.transport_mut()
            .reroute(&session, |routes| routes.push(Route { url, action }))
            .await
    }

    pub async fn unroute(&mut self, url: UrlMatch) -> Result<(), Error> {
        let session = self.session_id().to_string();
        self.transport_mut()
            .reroute(&session, |routes| routes.retain(|route| route.url != url))
            .await
    }
}
