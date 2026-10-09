mod support;

use charm::{Error, RouteAction, UrlMatch, expect};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::time::Duration;

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn the_start_url_loads_once_and_navigation_waits_for_the_new_document() {
    let port = support::serve().await;
    let mut session = support::launch_with(
        Duration::from_secs(5),
        Some(format!("http://127.0.0.1:{port}/start")),
    )
    .await;
    assert_eq!(support::STARTS.load(Ordering::SeqCst), 1);
    assert_eq!(session.title().await.expect("title"), "start");
    session
        .goto(&format!("http://127.0.0.1:{port}/frame"))
        .await
        .expect("goto");
    let path = session.evaluate("location.pathname").await.expect("path");
    assert_eq!(path.as_str(), Some("/frame"));
    drop(session.evaluate("window.marker = 1").await.expect("mark"));
    session.reload().await.expect("reload");
    let marker = session
        .evaluate("typeof window.marker")
        .await
        .expect("marker");
    assert_eq!(marker.as_str(), Some("undefined"));
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn locators_and_assertions_agree_and_waits_answer_routes() {
    let port = support::serve().await;
    let mut session = support::launch(Duration::from_secs(3)).await;
    session
        .goto(&format!("http://127.0.0.1:{port}/"))
        .await
        .expect("goto");
    assert_eq!(
        session
            .get_by_text("welcome", false)
            .count()
            .await
            .expect("count"),
        1
    );
    expect(&mut session.locator("h1"))
        .to_contain_text("welcome")
        .await
        .expect("contains ignores case in both");
    let hidden = session.click("#hidden").await;
    assert!(
        matches!(hidden, Err(Error::NotActionable { .. })),
        "{hidden:?}"
    );

    session
        .route(
            UrlMatch::Contains("/api".into()),
            RouteAction::Fulfill {
                status: 200,
                headers: BTreeMap::new(),
                body: b"routed".to_vec(),
            },
        )
        .await
        .expect("route");
    drop(
        session
            .evaluate("void fetch('/api').then(r=>r.text()).then(t=>{window.got=t})")
            .await
            .expect("start fetch"),
    );
    session.wait_for_timeout(Duration::from_millis(500)).await;
    let got = session.evaluate("String(window.got)").await.expect("got");
    assert_eq!(got.as_str(), Some("routed"));
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn a_frame_below_the_fold_is_clicked_after_scrolling() {
    let port = support::serve().await;
    let mut session = support::launch(Duration::from_secs(5)).await;
    session
        .goto(&format!("http://127.0.0.1:{port}/tall"))
        .await
        .expect("goto");
    session
        .frame_locator("#cross")
        .locator("#inner")
        .click()
        .await
        .expect("click below the fold");
    let mut out = session.frame_locator("#cross").locator("#fout");
    expect(&mut out)
        .to_have_text("clicked")
        .await
        .expect("clicked");
    session.close().await.expect("close");
}
