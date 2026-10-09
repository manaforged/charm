use crate::argv::Launch;
use crate::error::Error;
use crate::port::websocket_url;
use crate::timing::Timing;
use crate::transport::Transport;
use crate::wait::LoadState;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::time::sleep;
use tokio_tungstenite::connect_async;

const STDERR_TAIL: usize = 4096;

pub struct Session {
    transport: Transport,
    session_id: String,
    child: Child,
    user_data_dir: PathBuf,
    ephemeral: bool,
    timing: Timing,
}

impl Session {
    pub async fn launch(launch: Launch) -> Result<Self, Error> {
        let ephemeral = launch.user_data_dir().is_none();
        let user_data_dir = match launch.user_data_dir() {
            Some(dir) => dir.to_path_buf(),
            None => tempfile::tempdir()?.keep(),
        };
        fs::create_dir_all(&user_data_dir).await?;
        let mut command = Command::new(launch.exe());
        command
            .args(launch.args(&user_data_dir))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let child = command.spawn().map_err(|source| Error::Spawn {
            path: launch.exe().to_path_buf(),
            source,
        })?;
        let mut session = attach(
            child,
            user_data_dir,
            ephemeral,
            launch.start_url(),
            launch.timings(),
        )
        .await?;
        if let Some((user, pass)) = launch.login() {
            let id = session.session_id.clone();
            session.transport.arm_auth(&id, user, pass).await?;
        }
        Ok(session)
    }

    pub(crate) fn timing(&self) -> Timing {
        self.timing
    }

    #[must_use]
    pub fn sent(&self) -> &BTreeSet<String> {
        self.transport.sent()
    }

    pub async fn send(&mut self, method: &str, params: Value) -> Result<Value, Error> {
        self.transport
            .send(method, params, Some(&self.session_id))
            .await
    }

    pub(crate) fn session_id(&self) -> &str {
        &self.session_id
    }

    pub(crate) async fn send_on(
        &mut self,
        session_id: &str,
        method: &str,
        params: Value,
    ) -> Result<Value, Error> {
        self.transport.send(method, params, Some(session_id)).await
    }

    pub async fn browser_send(&mut self, method: &str, params: Value) -> Result<Value, Error> {
        self.transport.send(method, params, None).await
    }

    pub async fn network_on(&mut self) -> Result<(), Error> {
        drop(self.send("Network.enable", json!({})).await?);
        Ok(())
    }

    pub async fn block_urls(&mut self, urls: &[String]) -> Result<(), Error> {
        drop(
            self.send("Network.setBlockedURLs", json!({ "urls": urls }))
                .await?,
        );
        Ok(())
    }

    pub async fn take_events(&mut self) -> Result<Vec<Value>, Error> {
        self.transport.pump().await?;
        Ok(self.transport.take_events())
    }

    pub fn exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    pub async fn evaluate(&mut self, expression: &str) -> Result<Value, Error> {
        self.evaluate_in(None, expression).await
    }

    pub(crate) async fn evaluate_in(
        &mut self,
        session_id: Option<&str>,
        expression: &str,
    ) -> Result<Value, Error> {
        let params = json!({
            "expression": expression,
            "returnByValue": true,
            "awaitPromise": true,
        });
        let result = match session_id {
            Some(id) => self.send_on(id, "Runtime.evaluate", params).await?,
            None => self.send("Runtime.evaluate", params).await?,
        };
        if let Some(exception) = result.get("exceptionDetails") {
            return Err(Error::Protocol {
                method: "Runtime.evaluate".into(),
                message: exception.to_string(),
            });
        }
        Ok(remote_value(
            result.get("result").cloned().unwrap_or(Value::Null),
        ))
    }

    pub(crate) fn transport_mut(&mut self) -> &mut Transport {
        &mut self.transport
    }

    pub async fn open(&mut self, url: &str) -> Result<(), Error> {
        let nav = self.send("Page.navigate", json!({ "url": url })).await?;
        if let Some(error) = nav.get("errorText").and_then(Value::as_str)
            && error != "net::ERR_ABORTED"
        {
            return Err(Error::Protocol {
                method: "Page.navigate".into(),
                message: error.to_string(),
            });
        }
        Ok(())
    }

    pub async fn goto(&mut self, url: &str) -> Result<(), Error> {
        let deadline = Instant::now() + self.timing.navigate;
        loop {
            let nav = self.send("Page.navigate", json!({ "url": url })).await?;
            if let Some(error) = nav.get("errorText").and_then(Value::as_str)
                && error != "net::ERR_ABORTED"
            {
                return Err(Error::Protocol {
                    method: "Page.navigate".into(),
                    message: error.to_string(),
                });
            }
            if self.url_left_blank(self.timing.settle).await {
                break;
            }
            if Instant::now() >= deadline {
                break;
            }
        }
        self.wait_ready(deadline).await
    }

    async fn url_left_blank(&mut self, budget: Duration) -> bool {
        self.poll_expression("location.href !== 'about:blank'", budget)
            .await
            .is_ok()
    }

    async fn wait_ready(&mut self, deadline: Instant) -> Result<(), Error> {
        let left = deadline.saturating_duration_since(Instant::now());
        self.load_state_within(LoadState::DomContentLoaded, left)
            .await
            .map_err(|_timeout| Error::Timeout("Page.navigate".into()))
    }

    pub(crate) async fn take_event(
        &mut self,
        wanted: impl Fn(&Value) -> bool,
    ) -> Result<Option<Value>, Error> {
        self.transport.pump().await?;
        Ok(self.transport.take_event(wanted))
    }

    pub async fn close(mut self) -> Result<(), Error> {
        drop(self.transport.send("Browser.close", json!({}), None).await);
        self.transport.close().await;
        drop(tokio::time::timeout(self.timing.close, self.child.wait()).await);
        drop(self.child.kill().await);
        sleep(self.timing.poll).await;
        if self.ephemeral {
            drop(fs::remove_dir_all(&self.user_data_dir).await);
        }
        Ok(())
    }
}

async fn attach(
    mut child: Child,
    user_data_dir: PathBuf,
    ephemeral: bool,
    start_url: &str,
    timing: Timing,
) -> Result<Session, Error> {
    match handshake(&user_data_dir, start_url, timing, &mut child).await {
        Ok((transport, session_id)) => {
            drain_stderr(&mut child);
            Ok(Session {
                transport,
                session_id,
                child,
                user_data_dir,
                ephemeral,
                timing,
            })
        }
        Err(error) => {
            drop(child.kill().await);
            if ephemeral {
                drop(fs::remove_dir_all(&user_data_dir).await);
            }
            Err(error)
        }
    }
}

async fn handshake(
    user_data_dir: &Path,
    start_url: &str,
    timing: Timing,
    child: &mut Child,
) -> Result<(Transport, String), Error> {
    let mut transport = connect_transport(user_data_dir, timing, child).await?;
    let target_id = create_target(&mut transport, start_url).await?;
    let attached = transport
        .send(
            "Target.attachToTarget",
            json!({ "targetId": target_id, "flatten": true }),
            None,
        )
        .await?;
    let session_id = attached
        .get("sessionId")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Connect("Target.attachToTarget missing sessionId".into()))?
        .to_string();
    Ok((transport, session_id))
}

async fn connect_transport(
    user_data_dir: &Path,
    timing: Timing,
    child: &mut Child,
) -> Result<Transport, Error> {
    let deadline = Instant::now() + timing.connect;
    let mut last = Error::Connect("DevTools endpoint missing".into());
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            return Err(Error::Exited {
                status: status.to_string(),
                stderr: stderr_tail(child).await,
            });
        }
        match read_ws_url(user_data_dir).await {
            Ok(url) => match connect_async(&url).await {
                Ok((ws, _)) => return Ok(Transport::new(ws, timing)),
                Err(error) => last = Error::from(error),
            },
            Err(error) => last = error,
        }
        sleep(timing.connect_poll).await;
    }
    Err(last)
}

async fn stderr_tail(child: &mut Child) -> String {
    let mut bytes = Vec::new();
    if let Some(mut stderr) = child.stderr.take() {
        drop(stderr.read_to_end(&mut bytes).await);
    }
    let start = bytes.len().saturating_sub(STDERR_TAIL);
    String::from_utf8_lossy(&bytes[start..]).trim().to_string()
}

fn drain_stderr(child: &mut Child) {
    if let Some(mut stderr) = child.stderr.take() {
        tokio::spawn(async move {
            drop(tokio::io::copy(&mut stderr, &mut tokio::io::sink()).await);
        });
    }
}

async fn read_ws_url(user_data_dir: &Path) -> Result<String, Error> {
    let path = user_data_dir.join("DevToolsActivePort");
    let body = fs::read_to_string(&path).await?;
    websocket_url(&body).ok_or_else(|| Error::Connect("DevToolsActivePort malformed".into()))
}

async fn create_target(transport: &mut Transport, url: &str) -> Result<String, Error> {
    let created = transport
        .send("Target.createTarget", json!({ "url": url }), None)
        .await?;
    created
        .get("targetId")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| Error::Connect("Target.createTarget missing targetId".into()))
}

fn remote_value(remote: Value) -> Value {
    remote.get("value").cloned().unwrap_or(remote)
}

impl Drop for Session {
    fn drop(&mut self) {
        drop(self.child.start_kill());
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
