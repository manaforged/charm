use crate::selector::Selector;
use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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
    #[error("browser exited with code {} before DevTools was ready: {stderr}", code.map_or_else(|| "none".to_string(), |c| c.to_string()))]
    Exited { code: Option<i32>, stderr: String },
    #[error("CDP websocket closed")]
    Closed,
    #[error("expected {expected}, received {received}{}", selector.as_ref().map(|s| format!(" ({s:?})")).unwrap_or_default())]
    Assertion {
        expected: String,
        received: String,
        selector: Option<Selector>,
    },
    #[error("not actionable ({reason:?}{}): {selector:?}", covered_by.as_ref().map(|c| format!(", covered by {c}")).unwrap_or_default())]
    NotActionable {
        reason: Option<crate::dom::Blocker>,
        covered_by: Option<String>,
        selector: Selector,
    },
    #[error("frame: {0}")]
    Frame(String),
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
