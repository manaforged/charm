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
    #[error("{0}")]
    Connect(String),
    #[error("{method}: {message}")]
    Protocol { method: String, message: String },
    #[error("timeout: {0}")]
    Timeout(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
