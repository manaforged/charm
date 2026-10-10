use crate::error::Error;
use crate::network::{Cookie, UrlMatch};
use crate::route::RouteAction;
use crate::session::{Session, World};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;

const READ_STORAGE: &str =
    "JSON.stringify(Object.keys(localStorage).map(k=>({name:k,value:localStorage.getItem(k)})))";
const HTTP: &str = "http://";
const HTTPS: &str = "https://";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StorageState {
    #[serde(default)]
    pub cookies: Vec<Cookie>,
    #[serde(default)]
    pub origins: Vec<OriginStorage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginStorage {
    pub origin: String,
    #[serde(default)]
    pub local_storage: Vec<StorageEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StorageEntry {
    pub name: String,
    pub value: String,
}

impl StorageState {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        let body = serde_json::to_vec_pretty(self)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let temp = path.with_file_name(format!(".{name}.{}.tmp", std::process::id()));
        let written = write_private(&temp, &body).and_then(|()| std::fs::rename(&temp, path));
        if written.is_err() {
            drop(std::fs::remove_file(&temp));
        }
        Ok(written?)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let body = std::fs::read(path)?;
        Ok(serde_json::from_slice(&body)?)
    }
}

fn write_private(path: &Path, body: &[u8]) -> std::io::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(body)?;
    file.sync_all()
}

impl Session {
    pub async fn storage_state(&mut self) -> Result<StorageState, Error> {
        let cookies = self.cookies().await?;
        let mut origins: BTreeSet<String> = cookies.iter().filter_map(cookie_origin).collect();
        origins.extend(
            self.pages()
                .await?
                .iter()
                .filter_map(|page| origin_of(&page.url)),
        );
        let mut stored = Vec::new();
        for origin in origins {
            let entries = self.on_origin(&origin, None).await?;
            if !entries.is_empty() {
                stored.push(OriginStorage {
                    origin,
                    local_storage: entries,
                });
            }
        }
        Ok(StorageState {
            cookies,
            origins: stored,
        })
    }

    pub(crate) async fn apply_storage_state(&mut self, state: &StorageState) -> Result<(), Error> {
        if !state.cookies.is_empty() {
            self.add_cookies(state.cookies.clone()).await?;
        }
        for origin in &state.origins {
            if !origin.local_storage.is_empty() {
                drop(
                    self.on_origin(&origin.origin, Some(&origin.local_storage))
                        .await?,
                );
            }
        }
        Ok(())
    }

    async fn on_origin(
        &mut self,
        origin: &str,
        write: Option<&[StorageEntry]>,
    ) -> Result<Vec<StorageEntry>, Error> {
        let home = self.active_page().clone();
        let temp = self.new_page(None).await?;
        let result = self.storage_in_temp(origin, write).await;
        let closed = self.close_page(&temp).await;
        let selected = self.select_page(&home).await;
        let entries = result?;
        closed?;
        selected?;
        Ok(entries)
    }

    async fn storage_in_temp(
        &mut self,
        origin: &str,
        write: Option<&[StorageEntry]>,
    ) -> Result<Vec<StorageEntry>, Error> {
        let pattern = UrlMatch::Glob(format!("{origin}/**"));
        let headers = BTreeMap::from([("content-type".to_owned(), "text/html".to_owned())]);
        self.route(
            pattern.clone(),
            RouteAction::Fulfill {
                status: 200,
                headers,
                body: Vec::new(),
            },
        )
        .await?;
        let visited = self.goto(&format!("{origin}/")).await;
        let script = match write {
            Some(entries) => write_script(entries)?,
            None => READ_STORAGE.to_owned(),
        };
        let value = match visited {
            Ok(()) => self.evaluate_in(None, World::Isolated, &script).await,
            Err(error) => Err(error),
        };
        self.unroute(pattern).await?;
        match value? {
            Value::String(text) if write.is_none() => Ok(serde_json::from_str(&text)?),
            _ => Ok(Vec::new()),
        }
    }
}

fn write_script(entries: &[StorageEntry]) -> Result<String, Error> {
    let json = serde_json::to_string(entries)?;
    Ok(format!(
        "{json}.forEach(e=>localStorage.setItem(e.name,e.value)),null"
    ))
}

fn cookie_origin(cookie: &Cookie) -> Option<String> {
    let domain = cookie.domain.as_deref()?.trim_start_matches('.');
    if domain.is_empty() {
        return None;
    }
    let scheme = if cookie.secure { HTTPS } else { HTTP };
    Some(format!("{scheme}{domain}"))
}

pub(crate) fn origin_of(url: &str) -> Option<String> {
    let scheme = [HTTPS, HTTP]
        .into_iter()
        .find(|scheme| url.starts_with(scheme))?;
    let rest = &url[scheme.len()..];
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = rest[..end]
        .rsplit_once('@')
        .map_or(&rest[..end], |(_, h)| h);
    (!host.is_empty()).then(|| format!("{scheme}{host}"))
}
