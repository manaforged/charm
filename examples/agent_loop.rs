use charm::{ActReport, Action, Launch, Session, SnapshotOptions};

const PAGE: &str = "data:text/html,<title>agent</title><h1>Orders</h1><button onclick=\"document.body.insertAdjacentHTML('beforeend','<p>Order placed</p>')\">Place order</button>";

fn reference(snapshot: &str, line_has: &str) -> Option<String> {
    let line = snapshot.lines().find(|line| line.contains(line_has))?;
    let start = line.find("[ref=")? + 5;
    let end = line[start..].find(']')? + start;
    Some(line[start..end].to_owned())
}

#[tokio::main]
async fn main() -> Result<(), charm::Error> {
    let mut launch = Launch::installed()?;
    if std::env::var("CHARM_CHROME_NO_SANDBOX").is_ok_and(|value| !value.is_empty()) {
        launch = launch.arg("--no-sandbox");
    }
    let mut session = Session::launch(launch).await?;
    session.goto(PAGE).await?;

    let snapshot = session.snapshot(SnapshotOptions::default()).await?.text();
    println!("{snapshot}");

    let Some(button) = reference(&snapshot, "button \"Place order\"") else {
        session.close().await?;
        return Ok(());
    };
    let call = format!("{{\"action\":\"click\",\"target\":\"{button}\"}}");
    let action: Action = serde_json::from_str(&call)?;
    let outcome = session.act(action, ActReport::Delta).await?;

    println!("{} -> {}", outcome.url_before, outcome.url_after);
    if let Some(delta) = outcome.snapshot {
        println!("{delta}");
    }
    session.close().await
}
