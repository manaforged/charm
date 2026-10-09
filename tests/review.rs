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

const HOSTILE: &str = "Element.prototype.getBoundingClientRect=()=>({x:-9999,y:-9999,width:0,height:0,top:-9999,left:-9999,bottom:-9999,right:-9999});window.requestAnimationFrame=()=>0;JSON.stringify=()=>'{}';window.Promise=function(){};";

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn a_hostile_page_cannot_steer_clicks() {
    let mut session = support::launch(Duration::from_secs(5)).await;
    let page = format!(
        "data:text/html,<button id=b onclick=\"document.title='hit'\">go</button><iframe id=f srcdoc=\"<button id=i onclick=&quot;parent.document.title='frame hit'&quot;>in</button><script>{HOSTILE}</script>\"></iframe><script>{HOSTILE}</script>"
    );
    session.goto(&page).await.expect("goto");
    session.click("#b").await.expect("click on a hostile page");
    assert_eq!(session.title().await.expect("title"), "hit");
    let bounds = session.locator("#b").bounding_box().await.expect("box");
    assert!(
        bounds.is_some_and(|b| b.x >= 0.0 && b.width > 0.0),
        "{bounds:?}"
    );
    let framed = session.frame_locator("#f").locator("#i").click().await;
    framed.expect("click in a hostile same-origin frame");
    assert_eq!(session.title().await.expect("title"), "frame hit");
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn bad_selectors_fail_fast_and_protocol_errors_keep_their_code() {
    let mut session = support::launch(Duration::from_secs(20)).await;
    session.goto("data:text/html,<p>x</p>").await.expect("goto");
    let started = std::time::Instant::now();
    let css = session.click("div[").await;
    assert!(
        matches!(
            css,
            Err(Error::Script {
                kind: charm::ScriptError::Syntax,
                ..
            })
        ),
        "{css:?}"
    );
    let regex = session
        .locator(charm::Selector::Text(charm::TextMatch::regex("(", "")))
        .count()
        .await;
    assert!(
        matches!(
            regex,
            Err(Error::Script {
                kind: charm::ScriptError::Syntax,
                ..
            })
        ),
        "{regex:?}"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
    let protocol = session.send("Page.navigate", serde_json::json!({})).await;
    assert!(
        matches!(protocol, Err(Error::Protocol { code: Some(_), .. })),
        "{protocol:?}"
    );
    session
        .goto("data:text/html,<p id=two>two</p>")
        .await
        .expect("second goto");
    assert_eq!(
        session
            .locator("#two")
            .count()
            .await
            .expect("count after navigation"),
        1
    );
    assert!(session.sent().contains("Page.createIsolatedWorld"));
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn proxy_auth_covers_the_page_and_its_cross_origin_frames() {
    let site = support::serve().await;
    let proxy = support::proxy(site).await;
    let mut session = support::launch_through(proxy).await;
    session
        .goto(&format!("http://127.0.0.1:{site}/"))
        .await
        .expect("goto through proxy");
    assert_eq!(session.title().await.expect("title"), "charm live");
    let mut out = session.frame_locator("#cross").locator("#inner");
    expect(&mut out)
        .to_be_visible()
        .await
        .expect("cross-origin frame loaded through proxy");
    assert!(support::PROXIED.load(Ordering::SeqCst) >= 2);
    session.close().await.expect("close");
}
