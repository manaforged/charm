use charm::{Cookie, Launch, Session, StorageState};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn serve() -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                if stream.read(&mut buf).await.is_err() {
                    return;
                }
                let body = "<title>app</title><p>app</p>";
                let reply = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: text/html\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                drop(stream.write_all(reply.as_bytes()).await);
            });
        }
    });
    Ok(port)
}

fn launch() -> Result<Launch, charm::Error> {
    let mut launch = Launch::installed()?;
    if std::env::var("CHARM_CHROME_NO_SANDBOX").is_ok_and(|value| !value.is_empty()) {
        launch = launch.arg("--no-sandbox");
    }
    Ok(launch)
}

#[tokio::main]
async fn main() -> Result<(), charm::Error> {
    let port = serve().await?;
    let origin = format!("http://127.0.0.1:{port}");
    let path = std::env::temp_dir().join("charm-keep-login.json");

    let mut first = Session::launch(launch()?).await?;
    first.goto(&format!("{origin}/")).await?;
    first
        .add_cookies(vec![Cookie {
            name: "sid".into(),
            value: "signed-in".into(),
            url: Some(format!("{origin}/")),
            ..Cookie::default()
        }])
        .await?;
    drop(first.evaluate("localStorage.setItem('token','t1')").await?);
    first.storage_state().await?.save(&path)?;
    first.close().await?;

    let state = StorageState::load(&path)?;
    let mut second = Session::launch(launch()?.storage_state(state)).await?;
    second.goto(&format!("{origin}/")).await?;
    let token = second.evaluate("localStorage.getItem('token')").await?;
    let sid = second.cookie_named("sid").await?.map(|cookie| cookie.value);
    println!("token={token} sid={sid:?}");
    second.close().await?;

    std::fs::remove_file(&path)?;
    Ok(())
}
