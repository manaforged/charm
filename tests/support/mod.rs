use charm::{Launch, Session, Timing, resolve};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn launch(wait: Duration) -> Session {
    let exe = resolve().expect("an installed Chrome");
    let timing = Timing {
        wait,
        ..Timing::default()
    };
    let mut launch = Launch::new(exe).timing(timing);
    if std::env::var("CHARM_CHROME_NO_SANDBOX").is_ok_and(|value| !value.is_empty()) {
        launch = launch.arg("--no-sandbox");
    }
    Session::launch(launch).await.expect("launch")
}

fn page(port: u16) -> String {
    format!(
        "<title>charm live</title><h1>Welcome 42</h1><div id=hidden hidden>secret</div>\
         <iframe id=same srcdoc=\"<button id=sbtn onclick=&quot;this.textContent='done'&quot;>same</button>\"></iframe>\
         <iframe id=cross src=\"http://localhost:{port}/frame\"></iframe>"
    )
}

const FRAME: &str = "<button id=inner onclick=\"document.getElementById('fout').textContent='clicked'\">inner</button><div id=fout></div>";

fn respond(path: &str, port: u16) -> (&'static str, String) {
    match path {
        "/" => ("text/html", page(port)),
        "/frame" => ("text/html", FRAME.to_owned()),
        "/api" => ("application/json", "{\"ok\":true}".to_owned()),
        "/data" => ("text/plain", "payload-123".to_owned()),
        _ => ("text/plain", String::new()),
    }
}

pub async fn serve() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("address").port();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let Ok(read) = stream.read(&mut buf).await else {
                    return;
                };
                let head = String::from_utf8_lossy(&buf[..read]).into_owned();
                let path = head.split_whitespace().nth(1).unwrap_or("/").to_owned();
                let (kind, body) = respond(&path, port);
                let reply = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: {kind}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                drop(stream.write_all(reply.as_bytes()).await);
            });
        }
    });
    port
}
