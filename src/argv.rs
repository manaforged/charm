use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Launch {
    exe: PathBuf,
    user_data_dir: Option<PathBuf>,
    headless: bool,
    start_url: String,
    offscreen: bool,
    proxy: Option<String>,
}

impl Launch {
    #[must_use]
    pub fn new(exe: impl Into<PathBuf>) -> Self {
        Self {
            exe: exe.into(),
            user_data_dir: None,
            headless: true,
            start_url: "about:blank".into(),
            offscreen: false,
            proxy: None,
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
        if let Some(proxy) = &self.proxy {
            args.push(format!("--proxy-server={}", proxy_server(proxy)));
        }
        if self.offscreen && cfg!(windows) {
            args.push("--window-position=-32000,-32000".into());
        }
        args.push(self.start_url.clone());
        args
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
