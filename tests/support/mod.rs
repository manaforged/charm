#![allow(
    dead_code,
    reason = "each test binary uses a subset of the shared helpers"
)]

use charm::{Launch, Session, Timing};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn launch(wait: Duration) -> Session {
    launch_with(wait, None).await
}

pub fn base(wait: Duration) -> Launch {
    let timing = Timing {
        wait,
        ..Timing::default()
    };
    let launch = Launch::installed()
        .expect("an installed Chrome")
        .timing(timing);
    if std::env::var("CHARM_CHROME_NO_SANDBOX").is_ok_and(|value| !value.is_empty()) {
        launch.arg("--no-sandbox")
    } else {
        launch
    }
}

pub async fn launch_with(wait: Duration, url: Option<String>) -> Session {
    let timing = Timing {
        wait,
        ..Timing::default()
    };
    let mut launch = Launch::installed()
        .expect("an installed Chrome")
        .timing(timing);
    if std::env::var("CHARM_CHROME_NO_SANDBOX").is_ok_and(|value| !value.is_empty()) {
        launch = launch.arg("--no-sandbox");
    }
    if let Some(url) = url {
        launch = launch.url(url);
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

pub static STARTS: AtomicUsize = AtomicUsize::new(0);

const FRAME: &str = "<button id=inner onclick=\"document.getElementById('fout').textContent='clicked'\">inner</button><div id=fout></div>";

fn respond(path: &str, port: u16) -> (&'static str, String) {
    match path {
        "/" => ("text/html", page(port)),
        "/start" => {
            STARTS.fetch_add(1, Ordering::SeqCst);
            ("text/html", "<title>start</title>".to_owned())
        }
        "/tall" => (
            "text/html",
            format!(
                "<div style=\"height:3000px\"></div><iframe id=cross src=\"http://localhost:{port}/frame\"></iframe>"
            ),
        ),
        "/frame" => ("text/html", FRAME.to_owned()),
        "/api" => ("application/json", "{\"ok\":true}".to_owned()),
        "/data" => ("text/plain", "payload-123".to_owned()),
        "/file" => (
            "text/plain\r\ncontent-disposition: attachment; filename=report.txt",
            "report body".to_owned(),
        ),
        "/plain" => ("text/html", "<title>plain</title><p>plain</p>".to_owned()),
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

pub static PROXIED: AtomicUsize = AtomicUsize::new(0);

pub async fn proxy(site_port: u16) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind proxy");
    let port = listener.local_addr().expect("proxy address").port();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = vec![0u8; 16384];
                let Ok(read) = stream.read(&mut buf).await else {
                    return;
                };
                let head = String::from_utf8_lossy(&buf[..read]).into_owned();
                let authorized = head.lines().any(|line| {
                    line.to_ascii_lowercase()
                        .starts_with("proxy-authorization: basic dxnlcjpwyxnz")
                });
                if !authorized {
                    let reply = "HTTP/1.1 407 Proxy Authentication Required\r\nproxy-authenticate: Basic realm=\"t\"\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";
                    drop(stream.write_all(reply.as_bytes()).await);
                    return;
                }
                PROXIED.fetch_add(1, Ordering::SeqCst);
                let target = head.split_whitespace().nth(1).unwrap_or("/").to_owned();
                let path = target
                    .split_once("://")
                    .and_then(|(_, rest)| rest.find('/').map(|at| rest[at..].to_owned()))
                    .unwrap_or(target);
                let (kind, body) = respond(&path, site_port);
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

pub async fn launch_through(proxy_port: u16) -> Session {
    let mut launch = Launch::installed()
        .expect("an installed Chrome")
        .proxy(format!("http://user:pass@127.0.0.1:{proxy_port}"))
        .arg("--proxy-bypass-list=<-loopback>");
    if std::env::var("CHARM_CHROME_NO_SANDBOX").is_ok_and(|value| !value.is_empty()) {
        launch = launch.arg("--no-sandbox");
    }
    Session::launch(launch).await.expect("launch")
}
