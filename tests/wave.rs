mod support;

use charm::{
    ActReport, Action, Cookie, Error, Events, Filter, RouteAction, Selector, Session,
    SnapshotOptions, StorageState, TextMatch, UrlMatch, expect,
};
use std::collections::BTreeMap;
use std::time::Duration;

fn reference(snapshot: &str, line_has: &str) -> String {
    let line = snapshot
        .lines()
        .find(|line| line.contains(line_has))
        .unwrap_or_else(|| panic!("no line with {line_has} in\n{snapshot}"));
    let start = line.find("[ref=").expect("a ref") + 5;
    let end = line[start..].find(']').expect("ref end") + start;
    line[start..end].to_owned()
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn logins_carry_over_and_a_held_profile_is_reported() {
    let port = support::serve().await;
    let origin = format!("http://127.0.0.1:{port}");
    let mut first = Session::launch(support::base(Duration::from_secs(5)))
        .await
        .expect("launch");
    first.goto(&format!("{origin}/plain")).await.expect("goto");
    first
        .add_cookies(vec![Cookie {
            name: "sid".into(),
            value: "abc".into(),
            url: Some(format!("{origin}/")),
            ..Cookie::default()
        }])
        .await
        .expect("cookie");
    drop(
        first
            .evaluate("localStorage.setItem('token','t1')")
            .await
            .expect("storage"),
    );
    let state = first.storage_state().await.expect("export");
    let file = tempfile::tempdir().expect("dir");
    let path = file.path().join("state.json");
    state.save(&path).expect("save");
    first.close().await.expect("close");

    let loaded = StorageState::load(&path).expect("load");
    let mut second = Session::launch(support::base(Duration::from_secs(5)).storage_state(loaded))
        .await
        .expect("launch with state");
    second.goto(&format!("{origin}/plain")).await.expect("goto");
    let token = second
        .evaluate("localStorage.getItem('token')")
        .await
        .expect("token");
    assert_eq!(token.as_str(), Some("t1"));
    let sid = second.cookie_named("sid").await.expect("cookie");
    assert_eq!(sid.map(|cookie| cookie.value).as_deref(), Some("abc"));
    second.close().await.expect("close");

    let profile = tempfile::tempdir().expect("profile");
    let mut holder = Session::launch(support::base(Duration::from_secs(5)).profile(profile.path()))
        .await
        .expect("holder");
    let blocked =
        Session::launch(support::base(Duration::from_secs(5)).profile(profile.path())).await;
    assert!(
        matches!(blocked, Err(Error::ProfileInUse { .. })),
        "{:?}",
        blocked.err()
    );
    holder
        .goto("data:text/html,ok")
        .await
        .expect("holder still works");
    holder.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn downloads_and_file_choosers() {
    let port = support::serve().await;
    let launch = support::base(Duration::from_secs(10)).events(Events {
        page: true,
        ..Events::default()
    });
    let mut session = Session::launch(launch).await.expect("launch");
    let dir = tempfile::tempdir().expect("dir");
    session
        .set_downloads(dir.path().to_path_buf())
        .await
        .expect("downloads");
    session
        .goto(&format!("data:text/html,<a id=d href='http://127.0.0.1:{port}/file'>get</a><input id=f type=file>"))
        .await
        .expect("goto");
    session.click("#d").await.expect("start download");
    let download = session.wait_for_download().await.expect("download");
    let saved = download.path.expect("saved path");
    assert_eq!(
        saved.file_name().and_then(|n| n.to_str()),
        Some("report.txt")
    );
    assert_eq!(
        std::fs::read_to_string(&saved).expect("read"),
        "report body"
    );

    let upload = dir.path().join("upload.txt");
    std::fs::write(&upload, "x").expect("write");
    session
        .on_file_chooser(vec![upload])
        .await
        .expect("arm chooser");
    session.click("#f").await.expect("open picker");
    session
        .wait_for_function("document.getElementById('f').files.length === 1")
        .await
        .expect("picker answered");
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn act_takes_model_json_and_reports_what_changed() {
    let mut session = support::launch(Duration::from_secs(5)).await;
    session
        .goto("data:text/html,<title>t</title><button onclick=\"document.body.insertAdjacentHTML('beforeend','<p>done</p>')\">Go</button>")
        .await
        .expect("goto");
    let snapshot = session
        .snapshot(SnapshotOptions::default())
        .await
        .expect("snapshot")
        .text();
    let button = reference(&snapshot, "button \"Go\"");
    let call = format!("{{\"action\":\"click\",\"target\":\"{button}\"}}");
    let action: Action = serde_json::from_str(&call).expect("parse tool call");
    let outcome = session.act(action, ActReport::Delta).await.expect("act");
    let delta = outcome.snapshot.expect("delta");
    assert!(delta.contains("done"), "{delta}");
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn locators_filter_chain_and_check_editability() {
    let mut session = support::launch(Duration::from_secs(2)).await;
    session
        .goto("data:text/html,<ul><li>apple <button>Buy</button></li><li>pear <button>Buy</button></li></ul><input id=ro readonly value=x><h1>Fruit</h1>")
        .await
        .expect("goto");
    let filter = Filter {
        has_text: Some(TextMatch::Contains("pear".into())),
        ..Filter::default()
    };
    let count = session
        .locator("li")
        .filter(filter)
        .locator("button")
        .count()
        .await
        .expect("count");
    assert_eq!(count, 1);
    assert_eq!(session.locator("h1").or("li").count().await.expect("or"), 3);
    let readonly = session.fill("#ro", "y").await;
    assert!(
        matches!(readonly, Err(Error::NotActionable { .. })),
        "{readonly:?}"
    );
    expect(&mut session.locator("ul"))
        .to_match_aria_snapshot("- list\n  - listitem\n    - button \"Buy\"")
        .await
        .expect("aria snapshot");
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn routes_cover_new_tabs_and_cross_origin_refs_click() {
    let port = support::serve().await;
    let mut session = support::launch(Duration::from_secs(5)).await;
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
    session
        .new_page(Some(&format!("http://127.0.0.1:{port}/plain")))
        .await
        .expect("tab");
    let routed = session
        .evaluate("fetch('/api').then(r=>r.text())")
        .await
        .expect("fetch");
    assert_eq!(routed.as_str(), Some("routed"));

    session
        .goto(&format!("http://127.0.0.1:{port}/"))
        .await
        .expect("goto");
    let mut inner = session.frame_locator("#cross").locator("#inner");
    assert_eq!(inner.count().await.expect("attach frame"), 1);
    let snapshot = session
        .snapshot(SnapshotOptions::default())
        .await
        .expect("snapshot")
        .text();
    let button = reference(&snapshot, "button \"inner\"");
    session
        .click(Selector::Ref(button))
        .await
        .expect("click ref in cross-origin frame");
    let mut out = session.frame_locator("#cross").locator("#fout");
    expect(&mut out)
        .to_have_text("clicked")
        .await
        .expect("clicked");
    session.close().await.expect("close");
}
