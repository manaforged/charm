use crate::argv::{BLANK, Launch, StderrTail, clear_stale_port};
use crate::error::Error;
use crate::pages::{Endpoint, Tabs, initial_tabs, websocket_for};
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
    tabs: Tabs,
    owner: Owner,
    timing: Timing,
}

enum Owner {
    Owned(Box<Owned>),
    Detached(Box<Owned>),
    Attached,
}

struct Owned {
    child: Child,
    _job: Option<crate::job::Job>,
    _profile: Option<TempDir>,
    kept: Option<PathBuf>,
}

pub(crate) use crate::world::World;

struct Spawned {
    child: Child,
    job: Option<crate::job::Job>,
    stderr: StderrTail,
    user_data_dir: PathBuf,
    profile: Option<TempDir>,
    timing: Timing,
}

impl Session {
    pub async fn launch(launch: Launch) -> Result<Self, Error> {
        let (profile, user_data_dir) = profile_dir(&launch)?;
        fs::create_dir_all(&user_data_dir).await?;
        if launch.user_data_dir().is_some() {
            crate::argv::check_profile_lock(&user_data_dir, launch.timings().close).await?;
        }
        clear_stale_port(&user_data_dir).await?;
        let mut child = spawn(&launch, &user_data_dir)?;
        let job = (!launch.is_detached())
            .then(|| crate::job::contain(&child))
            .transpose()?;
        let stderr = StderrTail::start(&mut child);
        let mut session = attach(Spawned {
            child,
            job,
            stderr,
            user_data_dir,
            profile,
            timing: launch.timings(),
        })
        .await?;
        session.start(&launch).await?;
        Ok(session)
    }

    pub async fn connect(endpoint: Endpoint, timing: Timing) -> Result<Self, Error> {
        let url = websocket_for(endpoint, timing).await?;
        let (ws, _) = connect_async(&url).await.map_err(Error::from)?;
        let mut transport = Transport::new(ws, timing);
        let tabs = initial_tabs(&mut transport, true).await?;
        Ok(Self {
            transport,
            tabs,
            owner: Owner::Attached,
            timing,
        })
    }

    async fn start(&mut self, launch: &Launch) -> Result<(), Error> {
        let (events, policy) = launch.event_opt_in();
        self.set_events(events, policy).await?;
        if let Some((user, pass)) = launch.login() {
            self.transport.arm_auth(None, user, pass).await?;
        }
        if let Some(dir) = launch.user_data_dir() {
            let kept = dir.join(crate::storage::KEPT_COOKIES);
            if kept.exists() {
                self.apply_storage_state(&crate::StorageState::load(&kept)?)
                    .await?;
            }
        }
        if let Some(state) = launch.storage() {
            self.apply_storage_state(state).await?;
        }
        if launch.start_url() != BLANK {
            self.goto(launch.start_url()).await?;
        }
        Ok(())
    }

    pub(crate) fn tabs(&self) -> &Tabs {
        &self.tabs
    }

    pub(crate) fn tabs_mut(&mut self) -> &mut Tabs {
        &mut self.tabs
    }

    pub(crate) fn refs(&mut self) -> &mut crate::snapshot::RefMap {
        &mut self.tabs.active.refs
    }

    #[must_use]
    pub fn reply_failures(&self) -> u64 {
        self.transport.reply_failures()
    }

    #[must_use]
    pub fn dropped_events(&self) -> u64 {
        self.transport.dropped_events()
    }

    pub(crate) fn frame_sessions(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.tabs.active.frame_sessions
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
            .send(method, params, Some(&self.tabs.active.session_id))
            .await
    }

    pub(crate) fn session_id(&self) -> &str {
        &self.tabs.active.session_id
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
        match &mut self.owner {
            Owner::Owned(owned) | Owner::Detached(owned) => {
                matches!(owned.child.try_wait(), Ok(Some(_)))
            }
            Owner::Attached => false,
        }
    }

    pub async fn evaluate(&mut self, expression: &str) -> Result<Value, Error> {
        self.evaluate_in(None, World::Main, expression).await
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

    pub async fn keep_cookies(&mut self) -> Result<(), Error> {
        let kept = match &self.owner {
            Owner::Owned(owned) | Owner::Detached(owned) => owned.kept.clone(),
            Owner::Attached => None,
        };
        let Some(dir) = kept else {
            return Ok(());
        };
        let cookies = self.cookies().await?;
        crate::StorageState {
            cookies,
            origins: Vec::new(),
        }
        .save(dir.join(crate::storage::KEPT_COOKIES))
    }

    pub async fn close(mut self) -> Result<(), Error> {
        self.keep_cookies().await?;
        match &mut self.owner {
            Owner::Owned(owned) => {
                drop(self.transport.send("Browser.close", json!({}), None).await);
                self.transport.close().await;
                drop(owned.child.stdin.take());
                drop(tokio::time::timeout(self.timing.close, owned.child.wait()).await);
                drop(owned.child.kill().await);
                drop(owned.child.wait().await);
            }
            Owner::Detached(owned) => {
                drop(self.transport.send("Browser.close", json!({}), None).await);
                self.transport.close().await;
                drop(tokio::time::timeout(self.timing.close, owned.child.wait()).await);
            }
            Owner::Attached => self.transport.close().await,
        }
        Ok(())
    }
}

fn profile_dir(launch: &Launch) -> Result<(Option<TempDir>, PathBuf), Error> {
    match launch.user_data_dir() {
        Some(dir) => Ok((None, dir.to_path_buf())),
        None => {
            let temp = tempfile::tempdir()?;
            let path = temp.path().to_path_buf();
            Ok((Some(temp), path))
        }
    }
}

#[cfg(unix)]
const GUARD: &str = r#"exe="$1"; shift; exec 3<&0; "$exe" "$@" </dev/null 3<&- & child=$!; (read -r _ <&3; kill -9 "$child" 2>/dev/null) & exec 3<&-; wait "$child""#;

#[cfg(unix)]
fn command(launch: &Launch, user_data_dir: &Path) -> Command {
    if launch.is_detached() {
        let mut command = Command::new(launch.exe());
        command
            .args(launch.args(user_data_dir))
            .stdin(Stdio::null());
        return command;
    }
    let mut command = Command::new("/bin/sh");
    command
        .arg("-c")
        .arg(GUARD)
        .arg("charm")
        .arg(launch.exe())
        .args(launch.args(user_data_dir))
        .stdin(Stdio::piped());
    command
}

#[cfg(not(unix))]
fn command(launch: &Launch, user_data_dir: &Path) -> Command {
    let mut command = Command::new(launch.exe());
    command
        .args(launch.args(user_data_dir))
        .stdin(Stdio::null());
    if !launch.is_detached() {
        command.kill_on_drop(true);
    }
    command
}

fn spawn(launch: &Launch, user_data_dir: &Path) -> Result<Child, Error> {
    let exe = launch.exe();
    if exe.components().count() > 1 && !exe.is_file() {
        return Err(Error::Spawn {
            path: exe.to_path_buf(),
            source: std::io::Error::from(std::io::ErrorKind::NotFound),
        });
    }
    command(launch, user_data_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| Error::Spawn {
            path: launch.exe().to_path_buf(),
            source,
        })
}

fn release(child: &mut Child) {
    drop(child.stdin.take());
    #[cfg(not(unix))]
    drop(child.start_kill());
}

async fn attach(spawned: Spawned) -> Result<Session, Error> {
    let Spawned {
        mut child,
        job,
        stderr,
        user_data_dir,
        profile,
        timing,
    } = spawned;
    match handshake(&user_data_dir, timing, &mut child, stderr).await {
        Ok((transport, tabs)) => {
            let detached = job.is_none();
            let kept = profile.is_none().then(|| user_data_dir.clone());
            let owned = Box::new(Owned {
                child,
                _job: job,
                kept,
                _profile: profile,
            });
            let owner = if detached {
                Owner::Detached(owned)
            } else {
                Owner::Owned(owned)
            };
            Ok(Session {
                transport,
                tabs,
                owner,
                timing,
            })
        }
        Err(error) => {
            release(&mut child);
            drop(tokio::time::timeout(timing.close, child.wait()).await);
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
) -> Result<(Transport, Tabs), Error> {
    let mut transport = connect_transport(user_data_dir, timing, child, stderr).await?;
    let tabs = initial_tabs(&mut transport, true).await?;
    Ok((transport, tabs))
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

impl Drop for Session {
    fn drop(&mut self) {
        if let Owner::Owned(owned) = &mut self.owner {
            release(&mut owned.child);
        }
    }
}
#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
