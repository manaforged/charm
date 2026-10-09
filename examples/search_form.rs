use charm::{FORBIDDEN_METHODS, Launch, Selector, Session, TextMatch, expect, expect_page};

const PAGE: &str = "data:text/html,<title>search</title><input id=q aria-label=Search><button id=go onclick=\"document.getElementById('result').textContent='Results for '+document.getElementById('q').value\">Search</button><p id=result></p>";

#[tokio::main]
async fn main() -> Result<(), charm::Error> {
    let mut launch = Launch::installed()?;
    if std::env::var("CHARM_CHROME_NO_SANDBOX").is_ok_and(|value| !value.is_empty()) {
        launch = launch.arg("--no-sandbox");
    }
    let mut session = Session::launch(launch).await?;

    session.goto(PAGE).await?;
    expect_page(&mut session).to_have_title("search").await?;

    session.get_by_label("Search", true).fill("rust").await?;
    session.click("#go").await?;

    let done = Selector::Text(TextMatch::regex("^Results for \\w+$", ""));
    expect(&mut session.locator(done)).to_have_count(1).await?;
    expect(&mut session.locator("#result"))
        .to_have_text("Results for rust")
        .await?;
    println!("{:?}", session.locator("#result").inner_text().await?);

    for &method in FORBIDDEN_METHODS {
        assert!(!session.sent().contains(method), "{method} was sent");
    }
    session.close().await
}
