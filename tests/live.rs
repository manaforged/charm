mod support;

use charm::{
    Cookie, Error, ErrorReason, FORBIDDEN_METHODS, RouteAction, ScreenshotOptions, Selector,
    Session, TextMatch, UrlMatch, expect,
};
use std::collections::BTreeMap;
use std::time::Duration;

const FORM: &str = "data:text/html,<title>charm live</title><input id=name aria-label=Name><button id=go onclick=\"document.getElementById('out').textContent='hi '+document.getElementById('name').value\">Go</button><div id=out></div><input type=checkbox id=c checked>";

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn drives_a_page_without_forbidden_methods() {
    let mut session = support::launch(Duration::from_secs(30)).await;
    session.goto(FORM).await.expect("goto");
    assert_eq!(session.title().await.expect("title"), "charm live");
    session.fill("#name", "Ada").await.expect("fill");
    session.click("#go").await.expect("click");
    session
        .wait_for_function("document.getElementById('out').textContent === 'hi Ada'")
        .await
        .expect("the click ran the handler");
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
    for &method in FORBIDDEN_METHODS {
        assert!(!sent.contains(method), "{method} was sent");
    }
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn frames_regex_and_assertions() {
    let port = support::serve().await;
    let mut session = support::launch(Duration::from_secs(5)).await;
    session
        .goto(&format!("http://127.0.0.1:{port}/"))
        .await
        .expect("goto");
    let regex = Selector::Text(TextMatch::regex("^Welcome \\d+$", ""));
    assert_eq!(session.locator(regex).count().await.expect("regex"), 1);
    expect(&mut session.locator("#hidden"))
        .to_be_hidden()
        .await
        .expect("hidden");
    let failed = expect(&mut session.locator("h1"))
        .to_have_text("Goodbye")
        .await;
    assert!(matches!(failed, Err(Error::Assertion { .. })), "{failed:?}");

    let mut same = session.frame_locator("#same").locator("#sbtn");
    same.click().await.expect("same-origin click");
    expect(&mut same)
        .to_have_text("done")
        .await
        .expect("same-origin text");

    session
        .frame_locator("#cross")
        .locator("#inner")
        .click()
        .await
        .expect("cross-origin click");
    let mut out = session.frame_locator("#cross").locator("#fout");
    expect(&mut out)
        .to_have_text("clicked")
        .await
        .expect("cross-origin text");
    for &method in FORBIDDEN_METHODS {
        assert!(!session.sent().contains(method), "{method} was sent");
    }
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn routes_and_response_bodies() {
    let port = support::serve().await;
    let mut session = support::launch(Duration::from_secs(10)).await;
    session
        .goto(&format!("http://127.0.0.1:{port}/"))
        .await
        .expect("goto");
    let fetch = |path: &str| format!("fetch('{path}').then(r=>r.text(),()=>'failed')");
    session
        .route(
            UrlMatch::Contains("/api".into()),
            RouteAction::Fulfill {
                status: 200,
                headers: BTreeMap::from([("content-type".into(), "application/json".into())]),
                body: b"{\"routed\":true}".to_vec(),
            },
        )
        .await
        .expect("route api");
    session
        .route(
            UrlMatch::Contains("/blocked".into()),
            RouteAction::Abort(ErrorReason::Failed),
        )
        .await
        .expect("route blocked");
    let routed = session.evaluate(&fetch("/api")).await.expect("routed");
    assert_eq!(routed.as_str(), Some("{\"routed\":true}"));
    let blocked = session.evaluate(&fetch("/blocked")).await.expect("aborted");
    assert_eq!(blocked.as_str(), Some("failed"));
    session
        .unroute(UrlMatch::Contains("/api".into()))
        .await
        .expect("unroute");
    let real = session.evaluate(&fetch("/api")).await.expect("unrouted");
    assert_eq!(real.as_str(), Some("{\"ok\":true}"));

    session
        .add_cookies(vec![Cookie {
            name: "k".into(),
            value: "v".into(),
            url: Some(format!("http://127.0.0.1:{port}/")),
            ..Cookie::default()
        }])
        .await
        .expect("add cookie");
    let cookie = session.cookie_named("k").await.expect("read cookie");
    assert_eq!(cookie.map(|cookie| cookie.value).as_deref(), Some("v"));
    session.network_on().await.expect("network");
    let fetched = session.evaluate(&fetch("/data")).await.expect("data");
    assert_eq!(fetched.as_str(), Some("payload-123"));
    let response = session
        .wait_for_response(UrlMatch::Contains("/data".into()))
        .await
        .expect("response event");
    assert_eq!(response.status, 200);
    let body = session.response_body(&response).await.expect("body");
    assert_eq!(body, b"payload-123");
    for &method in FORBIDDEN_METHODS {
        assert!(!session.sent().contains(method), "{method} was sent");
    }
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn a_profile_keeps_its_cookies_across_launches() {
    let port = support::serve().await;
    let url = format!("http://127.0.0.1:{port}/start");
    let profile = tempfile::tempdir().expect("profile");
    let open = || support::base(Duration::from_secs(30)).profile(profile.path());
    let mut first = Session::launch(open()).await.expect("launch");
    first.goto(&url).await.expect("goto");
    first
        .evaluate("document.cookie = 'kept=yes; max-age=86400; path=/'")
        .await
        .expect("set the cookie");
    first.close().await.expect("close");
    let mut second = Session::launch(open()).await.expect("relaunch");
    second.goto(&url).await.expect("goto again");
    let cookie = second.evaluate("document.cookie").await.expect("read");
    second.close().await.expect("close again");
    assert!(
        cookie
            .as_str()
            .is_some_and(|text| text.contains("kept=yes")),
        "{cookie}"
    );
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn kept_cookies_survive_a_killed_browser() {
    let port = support::serve().await;
    let url = format!("http://127.0.0.1:{port}/start");
    let profile = tempfile::tempdir().expect("profile");
    let open = || support::base(Duration::from_secs(30)).profile(profile.path());
    let mut first = Session::launch(open()).await.expect("launch");
    first.goto(&url).await.expect("goto");
    first
        .evaluate("document.cookie = 'kept=yes; max-age=86400; path=/'")
        .await
        .expect("set the cookie");
    first.keep_cookies().await.expect("keep");
    drop(first);
    tokio::time::sleep(Duration::from_secs(1)).await;
    let mut second = Session::launch(open()).await.expect("relaunch");
    second.goto(&url).await.expect("goto again");
    let cookie = second.evaluate("document.cookie").await.expect("read");
    second.close().await.expect("close");
    assert!(
        cookie
            .as_str()
            .is_some_and(|text| text.contains("kept=yes")),
        "{cookie}"
    );
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn chrome_ends_when_the_host_process_exits() {
    if let Ok(dir) = std::env::var("CHARM_EXIT_PROFILE") {
        let launch = support::base(Duration::from_secs(30)).profile(&dir);
        let session = Session::launch(launch).await.expect("launch");
        std::mem::forget(session);
        std::process::exit(0);
    }
    let profile = tempfile::tempdir().expect("profile");
    let status = std::process::Command::new(std::env::current_exe().expect("exe"))
        .args([
            "chrome_ends_when_the_host_process_exits",
            "--exact",
            "--ignored",
        ])
        .env("CHARM_EXIT_PROFILE", profile.path())
        .status()
        .expect("host process");
    assert!(status.success());
    let marker = format!("--user-data-dir={}", profile.path().display());
    let mut left = true;
    for _ in 0..30 {
        let ps = std::process::Command::new("ps")
            .args(["-A", "-o", "args="])
            .output()
            .expect("ps");
        left = String::from_utf8_lossy(&ps.stdout)
            .lines()
            .any(|line| line.contains(&marker) && !line.starts_with("/bin/sh"));
        if !left {
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    assert!(!left, "Chrome outlived its host process");
}
