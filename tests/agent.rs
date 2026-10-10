mod support;

use charm::{
    Device, DialogKind, DialogPolicy, Endpoint, Events, FORBIDDEN_METHODS, Launch, LoadState,
    PdfOptions, SelectBy, Selector, Session, SnapshotOptions, TextFormat, Timing,
};
use std::time::Duration;

const LOGIN: &str = "data:text/html,<title>login</title><h1>Sign in</h1><form aria-label=Login><label>Email <input id=email></label><label>Password <input id=pw type=password></label><label><input id=keep type=checkbox> Remember me</label><select id=plan><option value=free>Free</option><option value=pro>Pro</option></select><input id=file type=file><button type=button onclick=\"document.title='signed '+document.getElementById('email').value\">Sign in</button></form><a href='/x'>Forgot password?</a>";

fn reference(snapshot: &str, line_has: &str) -> String {
    let line = snapshot
        .lines()
        .find(|line| line.contains(line_has))
        .unwrap_or_else(|| panic!("no line with {line_has} in\n{snapshot}"));
    let start = line.find("[ref=").expect("a ref") + 5;
    let end = line[start..].find(']').expect("ref end") + start;
    format!("ref={}", &line[start..end])
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn snapshot_refs_text_and_forms() {
    let mut session = support::launch(Duration::from_secs(5)).await;
    session.goto(LOGIN).await.expect("goto");
    let snapshot = session
        .snapshot(SnapshotOptions::default())
        .await
        .expect("snapshot")
        .text();
    assert!(snapshot.contains("heading \"Sign in\""), "{snapshot}");
    let email = reference(&snapshot, "textbox \"Email\"");
    let button = reference(&snapshot, "button \"Sign in\"");
    session
        .fill(Selector::parse(&email), "ada@example.com")
        .await
        .expect("fill by ref");
    session
        .click(Selector::parse(&button))
        .await
        .expect("click by ref");
    assert_eq!(
        session.title().await.expect("title"),
        "signed ada@example.com"
    );

    let markdown = session
        .text(TextFormat::Markdown, 10_000)
        .await
        .expect("markdown");
    assert!(markdown.contains("# Sign in"), "{markdown}");

    session.check("#keep").await.expect("check");
    assert!(
        session
            .locator("#keep")
            .is_checked()
            .await
            .expect("checked")
    );
    let chosen = session
        .select_option("#plan", SelectBy::Value(vec!["pro".into()]))
        .await
        .expect("select");
    assert_eq!(chosen, vec!["pro".to_owned()]);
    let file = std::env::temp_dir().join("charm-upload.txt");
    std::fs::write(&file, "hello").expect("write file");
    session
        .set_input_files("#file", vec![file])
        .await
        .expect("upload");
    let count = session
        .evaluate("document.getElementById('file').files.length")
        .await
        .expect("files");
    assert_eq!(count.as_u64(), Some(1));
    for &method in FORBIDDEN_METHODS {
        assert!(!session.sent().contains(method), "{method} was sent");
    }
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn tabs_popups_emulation_and_pdf() {
    let port = support::serve().await;
    let mut session = support::launch(Duration::from_secs(10)).await;
    let first = session.active_page().clone();
    session
        .goto(&format!(
            "data:text/html,<a id=pop target=_blank href='http://127.0.0.1:{port}/start'>open</a>"
        ))
        .await
        .expect("goto");
    session.click("#pop").await.expect("open popup");
    let popup = session.wait_for_page().await.expect("popup");
    assert_ne!(popup, first);
    session
        .wait_for_load_state(LoadState::Load)
        .await
        .expect("popup load");
    assert_eq!(session.title().await.expect("popup title"), "start");
    assert_eq!(session.pages().await.expect("pages").len(), 2);
    session.select_page(&first).await.expect("back to first");
    session.close_page(&popup).await.expect("close popup");

    let tab = session
        .new_page(Some(&format!("http://127.0.0.1:{port}/")))
        .await
        .expect("new tab");
    assert_eq!(session.active_page(), &tab);
    session
        .wait_for_load_state(LoadState::NetworkIdle)
        .await
        .expect("network idle");
    session.emulate(Device::Iphone15).await.expect("emulate");
    session
        .goto("data:text/html,<meta name=viewport content='width=device-width'><p>m</p>")
        .await
        .expect("mobile page");
    let width = session.evaluate("innerWidth").await.expect("width");
    assert!(width.as_u64().is_some_and(|w| w < 500), "{width}");
    let pdf = session.pdf(PdfOptions::default()).await.expect("pdf");
    assert!(pdf.starts_with(b"%PDF"));
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn opt_in_events_answer_dialogs_and_capture_console() {
    let mut launch = Launch::installed()
        .expect("chrome")
        .events(Events::all())
        .dialogs(DialogPolicy::Accept);
    if std::env::var("CHARM_CHROME_NO_SANDBOX").is_ok_and(|v| !v.is_empty()) {
        launch = launch.arg("--no-sandbox");
    }
    let mut session = Session::launch(launch).await.expect("launch");
    session
        .goto("data:text/html,<button id=b onclick=\"console.log('before');alert('hi');document.title='after'\">go</button>")
        .await
        .expect("goto");
    session.click("#b").await.expect("click");
    session
        .wait_for_function("document.title === 'after'")
        .await
        .expect("the alert was answered");
    let dialogs = session.dialogs();
    assert_eq!(dialogs.len(), 1);
    assert_eq!(dialogs[0].kind, DialogKind::Alert);
    assert_eq!(dialogs[0].message, "hi");
    assert!(
        session
            .console()
            .iter()
            .any(|message| message.text.contains("before"))
    );
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn connect_attaches_without_owning_the_browser() {
    let exe = charm::resolve().expect("chrome");
    let profile = tempfile::tempdir().expect("profile");
    let mut args = vec![
        "--headless=new".to_owned(),
        format!("--user-data-dir={}", profile.path().display()),
        "--remote-debugging-port=0".to_owned(),
        "about:blank".to_owned(),
    ];
    if std::env::var("CHARM_CHROME_NO_SANDBOX").is_ok_and(|v| !v.is_empty()) {
        args.insert(0, "--no-sandbox".to_owned());
    }
    let mut chrome = std::process::Command::new(exe)
        .args(&args)
        .spawn()
        .expect("spawn");
    let port_file = profile.path().join("DevToolsActivePort");
    let mut waited = 0;
    while !port_file.exists() && waited < 100 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        waited += 1;
    }
    let body = std::fs::read_to_string(&port_file).expect("port file");
    let port: u16 = body
        .lines()
        .next()
        .expect("port line")
        .parse()
        .expect("port");
    let mut session = Session::connect(Endpoint::Port(port), Timing::default())
        .await
        .expect("connect");
    session
        .goto("data:text/html,<title>attached</title>")
        .await
        .expect("goto");
    assert_eq!(session.title().await.expect("title"), "attached");
    session.close().await.expect("detach");
    assert!(
        chrome.try_wait().expect("status").is_none(),
        "close must not kill an attached browser"
    );
    drop(chrome.kill());
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn refs_inside_a_same_origin_frame_act_on_the_right_element() {
    let mut session = support::launch(Duration::from_secs(5)).await;
    session
        .goto("data:text/html,<div style='height:40px'></div><iframe id=f srcdoc=\"<button onclick=&quot;parent.document.title='inner'&quot;>Inner button</button>\"></iframe>")
        .await
        .expect("goto");
    let snapshot = session
        .snapshot(SnapshotOptions::default())
        .await
        .expect("snapshot")
        .text();
    let button = reference(&snapshot, "button \"Inner button\"");
    session
        .click(Selector::parse(&button))
        .await
        .expect("click a ref inside a same-origin frame");
    assert_eq!(session.title().await.expect("title"), "inner");
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn clicking_a_ref_that_navigates_reports_the_new_page() {
    let port = support::serve().await;
    let mut session = support::launch(Duration::from_secs(5)).await;
    session
        .goto(&format!(
            "data:text/html,<a href='http://127.0.0.1:{port}/start'>Go on</a>"
        ))
        .await
        .expect("goto");
    let snapshot = session
        .snapshot(SnapshotOptions::default())
        .await
        .expect("snapshot")
        .text();
    let link = reference(&snapshot, "link \"Go on\"");
    let action: charm::Action = serde_json::from_str(&format!(
        "{{\"action\":\"click\",\"target\":\"{}\"}}",
        &link[4..]
    ))
    .expect("parse");
    let outcome = session
        .act(action, charm::ActReport::None)
        .await
        .expect("a click that navigates succeeds");
    assert!(
        outcome.url_after.ends_with("/start"),
        "{}",
        outcome.url_after
    );
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn act_follows_a_form_submit_by_click_and_by_enter() {
    let port = support::serve().await;
    let mut session = support::launch(Duration::from_secs(30)).await;
    for (step, wire) in [
        ("click", "{\"action\":\"click\",\"target\":\"#go\"}"),
        (
            "enter",
            "{\"action\":\"press\",\"key\":\"Enter\",\"target\":\"#q\"}",
        ),
    ] {
        session
            .goto(&format!("http://127.0.0.1:{port}/form"))
            .await
            .expect("goto");
        session.fill("#q", "rust").await.expect("fill");
        let action: charm::Action = serde_json::from_str(wire).expect("parse");
        let outcome = session
            .act(action, charm::ActReport::None)
            .await
            .expect(step);
        assert!(
            outcome.url_after.contains("/plain?q=rust"),
            "{step}: {}",
            outcome.url_after
        );
        assert_eq!(outcome.title, "plain", "{step}");
    }
    session.close().await.expect("close");
}

#[tokio::test]
#[ignore = "launches the installed Chrome"]
async fn the_opener_tab_still_takes_clicks_after_a_popup() {
    let port = support::serve().await;
    let mut session = support::launch(Duration::from_secs(30)).await;
    session
        .goto(&format!(
            "data:text/html,<button id=pop onclick=\"window.open('http://127.0.0.1:{port}/plain')\">Pop</button><button id=after onclick=\"document.title='after'\">After</button>"
        ))
        .await
        .expect("goto");
    session.click("#pop").await.expect("open the popup");
    for _ in 0..50 {
        if session.pages().await.expect("pages").len() > 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(session.pages().await.expect("pages").len() > 1, "no popup");
    let started = std::time::Instant::now();
    session.click("#after").await.expect("click in the opener");
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(session.title().await.expect("title"), "after");
    session.close().await.expect("close");
}
