use std::path::PathBuf;

#[must_use]
pub fn resolve() -> Option<PathBuf> {
    env_override().or_else(|| chrome_paths().into_iter().find(|path| path.is_file()))
}

fn env_override() -> Option<PathBuf> {
    env_file("CHROME_EXE").or_else(|| env_file("CHROMIUM_EXE"))
}

fn env_file(key: &str) -> Option<PathBuf> {
    let raw = std::env::var(key).ok().filter(|s| !s.is_empty())?;
    let path = PathBuf::from(raw);
    path.is_file().then_some(path)
}

fn chrome_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if cfg!(target_os = "macos") {
        paths.push(PathBuf::from(
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        ));
        paths.push(PathBuf::from(
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ));
        paths.push(PathBuf::from(
            "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
        ));
    }
    if cfg!(windows) {
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        let pf = std::env::var_os("PROGRAMFILES").map(PathBuf::from);
        let pf86 = std::env::var_os("PROGRAMFILES(X86)").map(PathBuf::from);
        for root in [local, pf, pf86].into_iter().flatten() {
            paths.push(root.join("Google/Chrome/Application/chrome.exe"));
            paths.push(root.join("Chromium/Application/chrome.exe"));
        }
    }
    if cfg!(target_os = "linux") {
        for name in [
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/snap/bin/chromium",
        ] {
            paths.push(PathBuf::from(name));
        }
    }
    paths
}
