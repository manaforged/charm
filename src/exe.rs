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

const MACOS: [&str; 5] = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
    "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
];

const WINDOWS_ROOTS: [&str; 3] = ["LOCALAPPDATA", "PROGRAMFILES", "PROGRAMFILES(X86)"];

const WINDOWS: [&str; 4] = [
    "Google/Chrome/Application/chrome.exe",
    "Chromium/Application/chrome.exe",
    "Microsoft/Edge/Application/msedge.exe",
    "BraveSoftware/Brave-Browser/Application/brave.exe",
];

const LINUX: [&str; 8] = [
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/snap/bin/chromium",
    "/usr/bin/microsoft-edge",
    "/usr/bin/microsoft-edge-stable",
    "/usr/bin/brave-browser",
];

fn chrome_paths() -> Vec<PathBuf> {
    if cfg!(target_os = "macos") {
        return MACOS.iter().map(PathBuf::from).collect();
    }
    if cfg!(windows) {
        return WINDOWS_ROOTS
            .iter()
            .filter_map(std::env::var_os)
            .map(PathBuf::from)
            .flat_map(|root| WINDOWS.iter().map(move |path| root.join(path)))
            .collect();
    }
    LINUX.iter().map(PathBuf::from).collect()
}
