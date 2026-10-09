use super::websocket_url;

#[test]
fn devtools_active_port_parses_or_rejects() {
    assert_eq!(
        websocket_url("9333\n/devtools/browser/abc").as_deref(),
        Some("ws://127.0.0.1:9333/devtools/browser/abc")
    );
    assert_eq!(websocket_url("9333"), None);
    assert_eq!(websocket_url("9333\n\n"), None);
}
