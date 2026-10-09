use super::{ReplyKind, Transport, auth_reply, blocked, matches_url};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{WebSocketStream, accept_async, connect_async};

#[test]
fn subscription_methods_are_blocked() {
    assert_eq!(blocked("Runtime.enable"), Some("Runtime.enable"));
    assert_eq!(blocked("Page.enable"), Some("Page.enable"));
    assert_eq!(
        blocked("Target.setAutoAttach"),
        Some("Target.setAutoAttach")
    );
    assert_eq!(blocked("Runtime.evaluate"), None);
}

#[test]
fn auth_reply_covers_proxy_server_and_paused_requests() {
    let proxy = json!({
        "method": "Fetch.authRequired",
        "params": { "requestId": "req-1", "authChallenge": { "source": "Proxy" } }
    });
    let reply = auth_reply(&proxy, "user", "pass").expect("proxy challenge answered");
    assert_eq!(reply.method, "Fetch.continueWithAuth");
    assert!(matches!(reply.kind, ReplyKind::Auth { proxy: true }));
    let auth = &reply.params["authChallengeResponse"];
    assert_eq!(auth["response"], "ProvideCredentials");
    assert_eq!(auth["username"], "user");
    assert_eq!(auth["password"], "pass");

    let server = json!({
        "method": "Fetch.authRequired",
        "params": { "requestId": "req-2", "authChallenge": { "source": "Server" } }
    });
    let reply = auth_reply(&server, "user", "pass").expect("server challenge answered");
    assert!(matches!(reply.kind, ReplyKind::Auth { proxy: false }));
    assert_eq!(reply.params["authChallengeResponse"]["response"], "Default");

    let paused = json!({
        "method": "Fetch.requestPaused",
        "params": { "requestId": "req-3", "request": { "url": "https://x" } }
    });
    let reply = auth_reply(&paused, "user", "pass").expect("paused request handled");
    assert_eq!(reply.method, "Fetch.continueRequest");

    assert!(
        auth_reply(
            &json!({ "method": "Network.responseReceived", "params": {} }),
            "user",
            "pass"
        )
        .is_none()
    );
}

#[test]
fn url_pattern_matching_follows_cdp_wildcards() {
    for (pattern, url, expected) in [
        ("*", "https://example.test/cart", true),
        ("*/cart", "https://example.test/cart", true),
        ("*/Cart", "https://example.test/cart", false),
        ("*/car?", "https://example.test/cart", true),
        ("*/car?", "https://example.test/carts", false),
        ("*/cart\\?q=*", "https://example.test/cart?q=1", true),
    ] {
        assert_eq!(matches_url(pattern, url), expected);
    }
}

async fn connection() -> (Transport, WebSocketStream<TcpStream>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind CDP");
    let address = listener.local_addr().expect("CDP address");
    let (client, server) = tokio::join!(connect_async(format!("ws://{address}")), async {
        let (stream, _) = listener.accept().await.expect("accept CDP");
        accept_async(stream).await.expect("accept websocket")
    });
    (Transport::new(client.expect("connect CDP").0), server)
}

async fn receive(server: &mut WebSocketStream<TcpStream>) -> Value {
    let frame = timeout(Duration::from_secs(2), server.next())
        .await
        .expect("CDP command deadline")
        .expect("CDP remains open")
        .expect("read CDP");
    let Message::Text(text) = frame else {
        panic!("expected CDP text, got {frame:?}");
    };
    serde_json::from_str(&text).expect("parse command")
}

async fn emit(server: &mut WebSocketStream<TcpStream>, message: Value) {
    server
        .send(Message::Text(message.to_string().into()))
        .await
        .expect("write CDP");
}

#[tokio::test]
async fn fetch_ack_assigns_pauses_to_the_accepted_filter() {
    let (mut transport, mut server) = connection().await;
    let (armed, ()) = tokio::join!(
        transport.arm_auth("page", "user".into(), "pass".into()),
        async {
            let command = receive(&mut server).await;
            assert_eq!(command["method"], "Fetch.enable");
            assert_eq!(command["params"]["handleAuthRequests"], true);
            emit(&mut server, json!({"id": command["id"], "result": {}})).await;
        }
    );
    armed.expect("enable proxy authentication");

    let cart = json!({
        "sessionId": "page",
        "method": "Fetch.requestPaused",
        "params": {
            "requestId": "cart",
            "resourceType": "XHR",
            "request": {"url": "https://example.test/cart", "method": "POST"}
        }
    });
    let (configured, ()) = tokio::join!(
        transport.send(
            "Fetch.enable",
            json!({"patterns": [{"urlPattern": "*/cart", "resourceType": "Fetch"}]}),
            Some("page")
        ),
        async {
            let command = receive(&mut server).await;
            assert_eq!(
                command["params"]["patterns"],
                json!([
                    {"urlPattern": "*/cart", "resourceType": "Fetch"},
                    {"urlPattern": "*", "requestStage": "Request"}
                ])
            );
            emit(&mut server, cart.clone()).await;
            emit(&mut server, json!({"id": command["id"], "result": {}})).await;
        }
    );
    configured.expect("accept Fetch filter");
    assert_eq!(transport.take_events(), vec![cart]);

    let (pumped, ()) = tokio::join!(transport.pump(), async {
        emit(
            &mut server,
            json!({
                "sessionId": "page",
                "method": "Fetch.requestPaused",
                "params": {
                    "requestId": "ordinary",
                    "resourceType": "XHR",
                    "request": {"url": "https://example.test/ordinary", "method": "POST"}
                }
            }),
        )
        .await;
        let continued = receive(&mut server).await;
        assert_eq!(continued["method"], "Fetch.continueRequest");
        assert_eq!(continued["params"]["requestId"], "ordinary");
        emit(&mut server, json!({"id": continued["id"], "result": {}})).await;
    });
    pumped.expect("continue auth-only pause");
    assert!(transport.take_events().is_empty());
}
