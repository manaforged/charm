use crate::Error;
use crate::events::bounded;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use tokio::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadState {
    InProgress,
    Completed,
    Canceled,
}

impl DownloadState {
    fn parse(name: &str) -> Self {
        match name {
            "completed" => Self::Completed,
            "canceled" => Self::Canceled,
            _ => Self::InProgress,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Download {
    pub guid: String,
    pub url: String,
    pub suggested_filename: String,
    pub state: DownloadState,
    pub path: Option<PathBuf>,
}

#[derive(Default)]
pub(crate) struct Transfers {
    dir: Option<PathBuf>,
    list: VecDeque<Download>,
    chooser: Option<Vec<PathBuf>>,
}

fn text(params: &Value, key: &str) -> String {
    params
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

impl Transfers {
    pub(crate) fn begin(&mut self, params: &Value) -> u64 {
        let guid = text(params, "guid");
        let path = self.dir.as_ref().map(|dir| dir.join(&guid));
        bounded(
            &mut self.list,
            Download {
                guid,
                url: text(params, "url"),
                suggested_filename: text(params, "suggestedFilename"),
                state: DownloadState::InProgress,
                path,
            },
        )
    }

    pub(crate) fn progress(&mut self, params: &Value) {
        let guid = text(params, "guid");
        let Some(download) = self.list.iter_mut().find(|d| d.guid == guid) else {
            return;
        };
        download.state = DownloadState::parse(&text(params, "state"));
        if let Some(path) = params.get("filePath").and_then(Value::as_str) {
            download.path = Some(PathBuf::from(path));
        }
    }

    pub(crate) fn file_chooser(&self, params: &Value) -> Option<(&'static str, Value)> {
        let files = self.chooser.as_ref()?;
        let node = params.get("backendNodeId").and_then(Value::as_i64)?;
        Some((
            "DOM.setFileInputFiles",
            json!({ "files": files, "backendNodeId": node }),
        ))
    }

    fn take_completed(&mut self) -> Option<Download> {
        let at = self
            .list
            .iter()
            .position(|d| d.state == DownloadState::Completed)?;
        self.list.remove(at)
    }
}

async fn free_name(dir: &Path, suggested: &str) -> Result<PathBuf, Error> {
    let name = Path::new(suggested)
        .file_name()
        .map_or_else(|| "download".into(), |n| n.to_string_lossy().into_owned());
    let base = Path::new(&name);
    let stem = base
        .file_stem()
        .map_or_else(|| name.clone(), |s| s.to_string_lossy().into_owned());
    let ext = base
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let mut candidate = dir.join(&name);
    let mut n = 1u32;
    while tokio::fs::try_exists(&candidate).await? {
        candidate = dir.join(format!("{stem} ({n}){ext}"));
        n += 1;
    }
    Ok(candidate)
}

async fn settle(mut download: Download, dir: Option<&Path>) -> Result<Download, Error> {
    let (Some(dir), Some(from)) = (dir, download.path.clone()) else {
        return Ok(download);
    };
    if !tokio::fs::try_exists(&from).await? {
        return Ok(download);
    }
    let to = free_name(dir, &download.suggested_filename).await?;
    tokio::fs::rename(&from, &to).await?;
    download.path = Some(to);
    Ok(download)
}

impl crate::session::Session {
    pub async fn set_downloads(&mut self, dir: PathBuf) -> Result<(), Error> {
        tokio::fs::create_dir_all(&dir).await?;
        let dir = tokio::fs::canonicalize(&dir).await?;
        drop(
            self.browser_send(
                "Browser.setDownloadBehavior",
                json!({
                    "behavior": "allowAndName",
                    "downloadPath": dir,
                    "eventsEnabled": true,
                }),
            )
            .await?,
        );
        self.transport_mut().observer.transfers.dir = Some(dir);
        Ok(())
    }

    pub fn downloads(&mut self) -> Vec<Download> {
        self.transport_mut()
            .observer
            .transfers
            .list
            .drain(..)
            .collect()
    }

    pub async fn wait_for_download(&mut self) -> Result<Download, Error> {
        let deadline = Instant::now() + self.timing().wait;
        loop {
            let transfers = &mut self.transport_mut().observer.transfers;
            if let Some(download) = transfers.take_completed() {
                let dir = transfers.dir.clone();
                return settle(download, dir.as_deref()).await;
            }
            if Instant::now() >= deadline {
                return Err(Error::Timeout("wait_for_download".into()));
            }
            self.idle(self.timing().poll).await?;
        }
    }

    pub async fn on_file_chooser(&mut self, files: Vec<PathBuf>) -> Result<(), Error> {
        if !self.transport_mut().observer.events.page {
            return Err(Error::EventsRequired("page"));
        }
        let enabled = !files.is_empty();
        self.transport_mut().observer.transfers.chooser = enabled.then_some(files);
        let sessions: Vec<String> = self.tabs().session_ids();
        for session_id in sessions {
            drop(
                self.send_on(
                    &session_id,
                    "Page.setInterceptFileChooserDialog",
                    json!({ "enabled": enabled }),
                )
                .await?,
            );
        }
        Ok(())
    }
}
