use charm::{ErrorReason, FORBIDDEN_METHODS, Launch, RouteAction, Session, UrlMatch};
use std::collections::BTreeMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn respond(path: &str) -> (&'static str, &'static str) {
    match path {
        "/" => ("text/html", "<title>intercept</title><p>ready</p>"),
        "/api" => ("application/json", "{\"ok\":true}"),
        "/data" => ("text/plain", "payload-123"),
        _ => ("text/plain", ""),
    }
}

async fn serve() -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let Ok(read) = stream.read(&mut buf).await else {
                    return;
                };
                let head = String::from_utf8_lossy(&buf[..read]).into_owned();
                let path = head.split_whitespace().nth(1).unwrap_or("/").to_owned();
                let (kind, body) = respond(&path);
                let reply = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: {kind}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                drop(stream.write_all(reply.as_bytes()).await);
            });
        }
    });
    Ok(port)
}

fn fetch(path: &str) -> String {
    format!("fetch('{path}').then(r=>r.text(),()=>'failed')")
}

#[tokio::main]
async fn main() -> Result<(), charm::Error> {
    let port = serve().await?;
    let mut launch = Launch::installed()?;
    if std::env::var("CHARM_CHROME_NO_SANDBOX").is_ok_and(|value| !value.is_empty()) {
        launch = launch.arg("--no-sandbox");
    }
    let mut session = Session::launch(launch).await?;
    session.goto(&format!("http://127.0.0.1:{port}/")).await?;

    session
        .route(
            UrlMatch::Contains("/api".into()),
            RouteAction::Fulfill {
                status: 200,
                headers: BTreeMap::from([("content-type".into(), "application/json".into())]),
                body: b"{\"routed\":true}".to_vec(),
            },
        )
        .await?;
    session
        .route(
            UrlMatch::Contains("/blocked".into()),
            RouteAction::Abort(ErrorReason::Failed),
        )
        .await?;
    println!("routed: {}", session.evaluate(&fetch("/api")).await?);
    println!("blocked: {}", session.evaluate(&fetch("/blocked")).await?);

    session.unroute(UrlMatch::Contains("/api".into())).await?;
    println!("unrouted: {}", session.evaluate(&fetch("/api")).await?);

    session.network_on().await?;
    drop(session.evaluate(&fetch("/data")).await?);
    let response = session
        .wait_for_response(UrlMatch::Contains("/data".into()))
        .await?;
    let body = session.response_body(&response).await?;
    println!(
        "{} {} {}",
        response.status,
        response.url,
        String::from_utf8_lossy(&body)
    );

    for method in FORBIDDEN_METHODS {
        assert!(!session.sent().contains(method), "{method} was sent");
    }
    session.close().await
}
