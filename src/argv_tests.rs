use super::Launch;
use std::path::Path;

#[test]
fn default_launch_is_headless_port_zero_and_url_last() {
    let args = Launch::new("chrome").args(Path::new("/tmp/p"));
    assert_eq!(args[0], "--headless=new");
    assert!(args.iter().any(|a| a == "--remote-debugging-port=0"));
    assert_eq!(args.last().map(String::as_str), Some("about:blank"));
}

#[test]
fn flags_cover_headed_proxy_and_url() {
    let args = Launch::new("chrome")
        .headed()
        .proxy("http://127.0.0.1:9")
        .url("https://example.com")
        .args(Path::new("/data"));
    assert!(!args.iter().any(|a| a == "--headless=new"));
    assert!(args.iter().any(|a| a == "--remote-allow-origins=*"));
    assert!(
        args.iter()
            .any(|a| a == "--proxy-server=http://127.0.0.1:9")
    );
    assert_eq!(args.last().map(String::as_str), Some("https://example.com"));
}
