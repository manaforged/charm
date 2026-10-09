use crate::error::{Error, ScriptError};
use crate::session::{Session, World};
use serde_json::Value;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum LoadState {
    #[default]
    Load,
    DomContentLoaded,
}

impl LoadState {
    fn expression(self) -> &'static str {
        match self {
            Self::Load => "document.readyState === 'complete'",
            Self::DomContentLoaded => "document.readyState !== 'loading'",
        }
    }
}

fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|n| n != 0.0 && !n.is_nan()),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

impl Session {
    pub(crate) async fn poll_expression_in(
        &mut self,
        session_id: Option<&str>,
        world: World,
        expression: &str,
        timeout: Duration,
    ) -> Result<Value, Error> {
        let wrapped = format!(
            "(async()=>{{const __r=await ({expression});return (__r&&typeof __r==='object')?true:__r;}})()"
        );
        let deadline = Instant::now() + timeout;
        let mut last = String::new();
        loop {
            match self.evaluate_in(session_id, world, &wrapped).await {
                Ok(value) if truthy(&value) => return Ok(value),
                Ok(_) => {}
                Err(
                    error @ Error::Script {
                        kind: ScriptError::Syntax,
                        ..
                    },
                ) => return Err(error),
                Err(Error::Protocol { message, .. } | Error::Script { message, .. }) => {
                    last = message;
                }
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                return Err(Error::Timeout(format!("{expression} {last}")));
            }
            self.idle(self.timing().poll).await?;
        }
    }

    pub async fn wait_for_function(&mut self, expression: &str) -> Result<Value, Error> {
        let timeout = self.timing().wait;
        self.poll_expression_in(None, World::Main, expression, timeout)
            .await
    }

    pub async fn wait_for_load_state(&mut self, state: LoadState) -> Result<(), Error> {
        let timeout = self.timing().wait;
        self.load_state_within(state, timeout).await
    }

    pub(crate) async fn load_state_within(
        &mut self,
        state: LoadState,
        timeout: Duration,
    ) -> Result<(), Error> {
        drop(
            self.poll_expression_in(None, World::Isolated, state.expression(), timeout)
                .await?,
        );
        Ok(())
    }

    pub async fn wait_for_timeout(&mut self, duration: Duration) {
        drop(self.idle(duration).await);
    }
}
