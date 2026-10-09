use crate::selector::Selector;
use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("refused CDP method {0}: subscription surface")]
    Blocked(&'static str),
    #[error("browser executable not found")]
    MissingExe,
    #[error("spawn {path}: {source}")]
    Spawn {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("browser exited with {status} before DevTools was ready: {stderr}")]
    Exited { status: String, stderr: String },
    #[error("expected {expected}, received {received}{}", selector.as_ref().map(|s| format!(" ({s:?})")).unwrap_or_default())]
    Assertion {
        expected: String,
        received: String,
        selector: Option<Selector>,
    },
    #[error("{0}")]
    Connect(String),
    #[error("{method}: {message}")]
    Protocol { method: String, message: String },
    #[error("timeout: {0}")]
    Timeout(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Ws(Box<tokio_tungstenite::tungstenite::Error>),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Ws(Box::new(error))
    }
}
