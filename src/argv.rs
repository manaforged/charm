use crate::Timing;
use crate::events::{DialogPolicy, Events};
use crate::storage::StorageState;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Child;
use tokio::task::JoinHandle;

pub(crate) const BLANK: &str = "about:blank";
const STDERR_TAIL: usize = 4096;
const STDERR_CHUNK: usize = 1024;

#[derive(Clone)]
pub struct Launch {
    exe: PathBuf,
    user_data_dir: Option<PathBuf>,
    headless: bool,
    start_url: String,
    offscreen: bool,
    proxy: Option<String>,
    timing: Timing,
    events: Events,
    dialogs: DialogPolicy,
    extra: Vec<String>,
    storage: Option<StorageState>,
}

impl Launch {
    pub fn installed() -> Result<Self, crate::Error> {
        crate::resolve()
            .map(Self::new)
            .ok_or(crate::Error::MissingExe)
    }

    #[must_use]
    pub fn new(exe: impl Into<PathBuf>) -> Self {
        Self {
            exe: exe.into(),
            user_data_dir: None,
            headless: true,
            start_url: BLANK.into(),
            offscreen: false,
            proxy: None,
            timing: Timing::default(),
            events: Events::default(),
            dialogs: DialogPolicy::default(),
            extra: Vec::new(),
            storage: None,
        }
    }

    #[must_use]
    pub fn profile(mut self, dir: impl Into<PathBuf>) -> Self {
        self.user_data_dir = Some(dir.into());
        self
    }

    #[must_use]
    pub fn headed(mut self) -> Self {
        self.headless = false;
        self
    }

    #[must_use]
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.start_url = url.into();
        self
    }

    #[must_use]
    pub fn offscreen(mut self) -> Self {
        self.offscreen = true;
        self
    }

    #[must_use]
    pub fn proxy(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        if !url.is_empty() {
            self.proxy = Some(url);
        }
        self
    }

    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.extra.push(arg.into());
        self
    }

    #[must_use]
    pub fn timing(mut self, timing: Timing) -> Self {
        self.timing = timing;
        self
    }

    #[must_use]
    pub fn events(mut self, events: Events) -> Self {
        self.events = events;
        self
    }

    #[must_use]
    pub fn dialogs(mut self, policy: DialogPolicy) -> Self {
        self.dialogs = policy;
        self
    }

    #[must_use]
    pub fn storage_state(mut self, state: StorageState) -> Self {
        self.storage = Some(state);
        self
    }

    pub(crate) fn storage(&self) -> Option<&StorageState> {
        self.storage.as_ref()
    }

    pub(crate) fn event_opt_in(&self) -> (Events, DialogPolicy) {
        (self.events, self.dialogs.clone())
    }

    pub(crate) fn timings(&self) -> Timing {
        self.timing
    }

    pub(crate) fn login(&self) -> Option<(String, String)> {
        proxy_login(self.proxy.as_deref()?)
    }

    pub(crate) fn exe(&self) -> &Path {
        &self.exe
    }

    pub(crate) fn user_data_dir(&self) -> Option<&Path> {
        self.user_data_dir.as_deref()
    }

    pub(crate) fn start_url(&self) -> &str {
        &self.start_url
    }

    #[must_use]
    pub fn args(&self, user_data_dir: &Path) -> Vec<String> {
        let mut args = Vec::new();
        if self.headless {
            args.push("--headless=new".into());
        }
        args.push(format!("--user-data-dir={}", user_data_dir.display()));
        args.push("--remote-debugging-port=0".into());
        args.push("--no-first-run".into());
        args.push("--no-default-browser-check".into());
        args.push("--window-size=1536,864".into());
        args.push("--remote-allow-origins=*".into());
        args.extend(KEYSTORE.iter().map(|arg| (*arg).to_string()));
        if let Some(proxy) = &self.proxy {
            args.push(format!("--proxy-server={}", proxy_server(proxy)));
        }
        if self.offscreen && cfg!(windows) {
            args.push("--window-position=-32000,-32000".into());
        }
        args.extend(self.extra.iter().cloned());
        args.push(BLANK.into());
        args
    }
}

#[cfg(target_os = "macos")]
const KEYSTORE: &[&str] = &["--use-mock-keychain"];
#[cfg(target_os = "linux")]
const KEYSTORE: &[&str] = &["--password-store=basic"];
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
const KEYSTORE: &[&str] = &[];

impl fmt::Debug for Launch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Launch")
            .field("exe", &self.exe)
            .field("user_data_dir", &self.user_data_dir)
            .field("headless", &self.headless)
            .field("start_url", &without_login(&self.start_url))
            .field("offscreen", &self.offscreen)
            .field("proxy", &self.proxy.as_deref().map(proxy_server))
            .field("timing", &self.timing)
            .field("events", &self.events)
            .field("dialogs", &self.dialogs)
            .field("extra_args", &self.extra.len())
            .field("storage_state", &self.storage.is_some())
            .finish()
    }
}

pub(crate) async fn clear_stale_port(user_data_dir: &Path) -> std::io::Result<()> {
    match tokio::fs::remove_file(user_data_dir.join("DevToolsActivePort")).await {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

const LOCK: &str = if cfg!(windows) {
    "lockfile"
} else {
    "SingletonLock"
};

pub(crate) async fn check_profile_lock(user_data_dir: &Path) -> Result<(), crate::Error> {
    let lock = user_data_dir.join(LOCK);
    if lock_held(&lock).await? {
        return Err(crate::Error::ProfileInUse {
            path: user_data_dir.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(windows)]
async fn lock_held(lock: &Path) -> std::io::Result<bool> {
    match tokio::fs::remove_file(lock).await {
        Ok(()) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(true),
        Err(error) if error.raw_os_error() == Some(SHARING_VIOLATION) => Ok(true),
        Err(error) => Err(error),
    }
}

#[cfg(windows)]
const SHARING_VIOLATION: i32 = 32;

#[cfg(not(windows))]
async fn lock_held(lock: &Path) -> std::io::Result<bool> {
    let target = match tokio::fs::read_link(lock).await {
        Ok(target) => target,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if lock_pid(&target.to_string_lossy()).is_some_and(pid_alive) {
        return Ok(true);
    }
    match tokio::fs::remove_file(lock).await {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
        _ => Ok(false),
    }
}

pub(crate) fn lock_pid(target: &str) -> Option<u32> {
    target
        .rsplit_once('-')?
        .1
        .parse()
        .ok()
        .filter(|pid| *pid > 0)
}

#[cfg(not(windows))]
fn pid_alive(pid: u32) -> bool {
    std::process::Command::new("ps")
        .arg("-p")
        .arg(pid.to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub(crate) struct StderrTail {
    tail: Arc<Mutex<Vec<u8>>>,
    task: Option<JoinHandle<()>>,
}

impl StderrTail {
    pub(crate) fn start(child: &mut Child) -> Self {
        let tail = Arc::new(Mutex::new(Vec::new()));
        let task = child.stderr.take().map(|mut stderr| {
            let sink = Arc::clone(&tail);
            tokio::spawn(async move {
                let mut chunk = [0u8; STDERR_CHUNK];
                while let Ok(read) = stderr.read(&mut chunk).await {
                    if read == 0 {
                        break;
                    }
                    if let Ok(mut bytes) = sink.lock() {
                        bytes.extend_from_slice(&chunk[..read]);
                        let excess = bytes.len().saturating_sub(STDERR_TAIL);
                        bytes.drain(..excess);
                    }
                }
            })
        });
        Self { tail, task }
    }

    pub(crate) async fn finish(mut self, budget: Duration) -> String {
        if let Some(task) = self.task.take() {
            drop(tokio::time::timeout(budget, task).await);
        }
        self.tail
            .lock()
            .map(|bytes| String::from_utf8_lossy(&bytes).trim().to_string())
            .unwrap_or_default()
    }
}

#[cfg(test)]
#[path = "argv_tests.rs"]
mod tests;

pub(crate) fn proxy_server(url: &str) -> String {
    let (scheme, rest) = url.split_once("://").unwrap_or(("http", url));
    let rest = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host = rest.rsplit_once('@').map_or(rest, |(_, host)| host);
    format!("{scheme}://{host}")
}

fn without_login(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = rest[..end]
        .rsplit_once('@')
        .map_or(&rest[..end], |(_, host)| host);
    format!("{scheme}://{host}{}", &rest[end..])
}

pub(crate) fn proxy_login(url: &str) -> Option<(String, String)> {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let (login, _) = rest.rsplit_once('@')?;
    let (user, pass) = login.split_once(':').unwrap_or((login, ""));
    if user.is_empty() {
        return None;
    }
    Some((unescape(user), unescape(pass)))
}

fn unescape(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = (bytes[i] == b'%' && i + 2 < bytes.len())
            .then(|| std::str::from_utf8(&bytes[i + 1..i + 3]).ok())
            .flatten()
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match hex {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
