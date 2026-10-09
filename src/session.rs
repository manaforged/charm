use crate::argv::{BLANK, Launch, StderrTail, clear_stale_port};
use crate::error::{Error, Fault};
use crate::port::websocket_url;
use crate::timing::Timing;
use crate::transport::Transport;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Instant;
use tempfile::TempDir;
use tokio::fs;
use tokio::process::{Child, Command};
use tokio::time::sleep;
use tokio_tungstenite::connect_async;

pub struct Session {
    transport: Transport,
    session_id: String,
    child: Child,
    timing: Timing,
    _profile: Option<TempDir>,
    frame_sessions: BTreeMap<String, String>,
    worlds: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum World {
    Main,
    Isolated,
}

const WORLD_NAME: &str = "charm";

struct Spawned {
    child: Child,
    stderr: StderrTail,
    user_data_dir: PathBuf,
    profile: Option<TempDir>,
    timing: Timing,
}

impl Session {
    pub async fn launch(launch: Launch) -> Result<Self, Error> {
        let (profile, user_data_dir) = match launch.user_data_dir() {
            Some(dir) => (None, dir.to_path_buf()),
            None => {
                let temp = tempfile::tempdir()?;
                let path = temp.path().to_path_buf();
                (Some(temp), path)
            }
        };
        fs::create_dir_all(&user_data_dir).await?;
        clear_stale_port(&user_data_dir).await?;
        let mut child = Command::new(launch.exe())
            .args(launch.args(&user_data_dir))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|source| Error::Spawn {
                path: launch.exe().to_path_buf(),
                source,
            })?;
        let stderr = StderrTail::start(&mut child);
        let mut session = attach(Spawned {
            child,
            stderr,
            user_data_dir,
            profile,
            timing: launch.timings(),
        })
        .await?;
        if let Some((user, pass)) = launch.login() {
            session.transport.arm_auth(None, user, pass).await?;
        }
        if launch.start_url() != BLANK {
            session.goto(launch.start_url()).await?;
        }
        Ok(session)
    }

    #[must_use]
    pub fn dropped_events(&self) -> u64 {
        self.transport.dropped_events()
    }

    pub(crate) fn frame_sessions(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.frame_sessions
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
        self.transport.pump_then_take().await
    }

    pub(crate) async fn idle(&mut self, duration: std::time::Duration) -> Result<(), Error> {
        self.transport
            .idle(tokio::time::Instant::now() + duration)
            .await
    }

    pub fn exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    pub async fn evaluate(&mut self, expression: &str) -> Result<Value, Error> {
        self.evaluate_in(None, World::Main, expression).await
    }

    pub(crate) async fn evaluate_in(
        &mut self,
        session_id: Option<&str>,
        world: World,
        expression: &str,
    ) -> Result<Value, Error> {
        let target = session_id.map_or_else(|| self.session_id.clone(), str::to_owned);
        let params = json!({
            "expression": expression,
            "returnByValue": true,
            "awaitPromise": true,
        });
        let result = match world {
            World::Main => self.send_on(&target, "Runtime.evaluate", params).await?,
            World::Isolated => self.evaluate_isolated(&target, params).await?,
        };
        if let Some(exception) = result.get("exceptionDetails") {
            return Err(Error::script(exception));
        }
        Ok(remote_value(
            result.get("result").cloned().unwrap_or(Value::Null),
        ))
    }

    async fn evaluate_isolated(&mut self, target: &str, params: Value) -> Result<Value, Error> {
        let cached = self.worlds.get(target).copied();
        let context = match cached {
            Some(id) => id,
            None => self.create_world(target).await?,
        };
        match self.evaluate_on(target, context, params.clone()).await {
            Err(error) if cached.is_some() && error.fault() == Fault::ContextGone => {
                self.worlds.remove(target);
                let context = self.create_world(target).await?;
                self.evaluate_on(target, context, params).await
            }
            other => other,
        }
    }

    async fn evaluate_on(
        &mut self,
        target: &str,
        context: i64,
        mut params: Value,
    ) -> Result<Value, Error> {
        params["contextId"] = json!(context);
        self.send_on(target, "Runtime.evaluate", params).await
    }

    async fn create_world(&mut self, target: &str) -> Result<i64, Error> {
        let tree = self.send_on(target, "Page.getFrameTree", json!({})).await?;
        let frame_id = tree
            .pointer("/frameTree/frame/id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| Error::Frame("Page.getFrameTree missing frame id".into()))?;
        let created = self
            .send_on(
                target,
                "Page.createIsolatedWorld",
                json!({
                    "frameId": frame_id,
                    "worldName": WORLD_NAME,
                    "grantUniveralAccess": false,
                }),
            )
            .await?;
        let context = created
            .get("executionContextId")
            .and_then(Value::as_i64)
            .ok_or_else(|| Error::Frame("Page.createIsolatedWorld missing context id".into()))?;
        self.worlds.insert(target.to_owned(), context);
        Ok(context)
    }

    pub(crate) fn transport_mut(&mut self) -> &mut Transport {
        &mut self.transport
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
        drop(self.child.wait().await);
        Ok(())
    }
}

async fn attach(spawned: Spawned) -> Result<Session, Error> {
    let Spawned {
        mut child,
        stderr,
        user_data_dir,
        profile,
        timing,
    } = spawned;
    match handshake(&user_data_dir, timing, &mut child, stderr).await {
        Ok((transport, session_id)) => Ok(Session {
            transport,
            session_id,
            child,
            timing,
            _profile: profile,
            frame_sessions: BTreeMap::new(),
            worlds: BTreeMap::new(),
        }),
        Err(error) => {
            drop(child.kill().await);
            drop(profile);
            Err(error)
        }
    }
}

async fn handshake(
    user_data_dir: &Path,
    timing: Timing,
    child: &mut Child,
    stderr: StderrTail,
) -> Result<(Transport, String), Error> {
    let mut transport = connect_transport(user_data_dir, timing, child, stderr).await?;
    let target_id = create_target(&mut transport).await?;
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
    stderr: StderrTail,
) -> Result<Transport, Error> {
    let deadline = Instant::now() + timing.connect;
    let mut last = Error::Connect("DevTools endpoint missing".into());
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            return Err(Error::Exited {
                code: status.code(),
                stderr: stderr.finish(timing.close).await,
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

async fn read_ws_url(user_data_dir: &Path) -> Result<String, Error> {
    let path = user_data_dir.join("DevToolsActivePort");
    let body = fs::read_to_string(&path).await?;
    websocket_url(&body).ok_or_else(|| Error::Connect("DevToolsActivePort malformed".into()))
}

async fn create_target(transport: &mut Transport) -> Result<String, Error> {
    let created = transport
        .send("Target.createTarget", json!({ "url": BLANK }), None)
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
