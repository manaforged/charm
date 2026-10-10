use crate::selector::Selector;
use serde_json::Value;
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
    #[error("browser profile in use: {}", path.display())]
    ProfileInUse { path: PathBuf },
    #[error("ref {0} is stale: take a new snapshot")]
    StaleRef(String),
    #[error("{0}")]
    Connect(String),
    #[error("{method}: {message}{}", code.map(|c| format!(" (code {c})")).unwrap_or_default())]
    Protocol {
        method: String,
        code: Option<i64>,
        message: String,
    },
    #[error("page script {kind:?}: {message}")]
    Script { kind: ScriptError, message: String },
    #[error("timeout: {0}")]
    Timeout(String),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Ws(Box<tokio_tungstenite::tungstenite::Error>),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScriptError {
    Syntax,
    Type,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fault {
    ContextGone,
    Other,
}

const SERVER_ERROR: i64 = -32000;
const CONTEXT_GONE: &str = "Cannot find context with specified id";
const SYNTAX: &str = "SyntaxError";

impl ScriptError {
    fn parse(class: &str, description: &str) -> Self {
        match class {
            SYNTAX => Self::Syntax,
            "TypeError" => Self::Type,
            "DOMException" if description.starts_with(SYNTAX) => Self::Syntax,
            _ => Self::Other,
        }
    }
}

impl Error {
    pub(crate) fn protocol(method: &str, error: &Value) -> Self {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .map_or_else(|| error.to_string(), str::to_owned);
        let message = match error.get("data").and_then(Value::as_str) {
            Some(data) => format!("{message}: {data}"),
            None => message,
        };
        Self::Protocol {
            method: method.to_owned(),
            code: error.get("code").and_then(Value::as_i64),
            message,
        }
    }

    pub(crate) fn script(details: &Value) -> Self {
        let exception = details.get("exception").unwrap_or(&Value::Null);
        let class = exception
            .get("className")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let message = exception
            .get("description")
            .and_then(Value::as_str)
            .or_else(|| details.get("text").and_then(Value::as_str))
            .map_or_else(|| details.to_string(), str::to_owned);
        Self::Script {
            kind: ScriptError::parse(class, &message),
            message,
        }
    }

    pub(crate) fn fault(&self) -> Fault {
        match self {
            Self::Protocol {
                code: Some(SERVER_ERROR),
                message,
                ..
            } if message.starts_with(CONTEXT_GONE) => Fault::ContextGone,
            _ => Fault::Other,
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Ws(Box::new(error))
    }
}
