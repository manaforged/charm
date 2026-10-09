use charm::{FORBIDDEN_METHODS, Launch, ScreenshotOptions, Session, resolve};

const PAGE: &str = "data:text/html,<title>charm live</title><input id=name aria-label=Name><button id=go onclick=\"document.getElementById('out').textContent='hi '+document.getElementById('name').value\">Go</button><div id=out></div><input type=checkbox id=c checked>";

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn drives_a_page_without_forbidden_methods() {
    let exe = resolve().expect("an installed Chrome");
    let mut session = Session::launch(Launch::new(exe)).await.expect("launch");
    session.goto(PAGE).await.expect("goto");
    assert_eq!(session.title().await.expect("title"), "charm live");

    session.fill("#name", "Ada").await.expect("fill");
    session.click("#go").await.expect("click");
    session
        .wait_for_function("document.getElementById('out').textContent === 'hi Ada'")
        .await
        .expect("the click ran the handler");
    let typed = session
        .get_by_label("Name", true)
        .input_value()
        .await
        .expect("read");
    assert_eq!(typed.as_deref(), Some("Ada"));
    session.fill("#name", "Grace").await.expect("refill");
    session.press("Control+a").await.expect("chord");
    let typed = session
        .get_by_label("Name", true)
        .input_value()
        .await
        .expect("read");
    assert_eq!(typed.as_deref(), Some("Grace"));
    assert!(session.locator("#c").is_checked().await.expect("checked"));
    assert_eq!(session.locator("input").count().await.expect("count"), 2);

    let png = session
        .screenshot(ScreenshotOptions::default())
        .await
        .expect("screenshot");
    assert!(png.starts_with(b"\x89PNG"));

    let sent = session.sent().clone();
    assert!(sent.contains("Input.dispatchMouseEvent"));
    for method in FORBIDDEN_METHODS {
        assert!(!sent.contains(method), "{method} was sent");
    }
    session.close().await.expect("close");
}
