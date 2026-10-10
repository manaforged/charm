use super::Locator;
use crate::dom::REGISTER_REF;
use crate::error::{Error, Fault};
use crate::frames::Context;
use crate::selector::{REF_REGISTRY, Selector};
use crate::session::Session;
use crate::snapshot::NodeRef;
use serde_json::{Value, json};

impl Locator<'_> {
    pub(crate) async fn bind_ref(&mut self) -> Result<Option<Context>, Error> {
        let Selector::Ref(raw) = &self.selector else {
            return Ok(None);
        };
        let raw = raw.clone();
        let target = NodeRef::parse(&raw)
            .and_then(|node_ref| self.session.refs().get(node_ref).cloned())
            .ok_or_else(|| Error::StaleRef(raw.clone()))?;
        let session = target
            .session
            .clone()
            .unwrap_or_else(|| self.session.session_id().to_owned());
        let object = self
            .session
            .resolve_ref(&session, target.backend, &raw)
            .await?;
        let registered = self
            .session
            .send_on(
                &session,
                "Runtime.callFunctionOn",
                json!({
                    "objectId": object,
                    "functionDeclaration": REGISTER_REF,
                    "arguments": [{ "value": raw }, { "value": REF_REGISTRY }],
                    "returnByValue": true,
                }),
            )
            .await;
        drop(
            self.session
                .send_on(
                    &session,
                    "Runtime.releaseObject",
                    json!({ "objectId": object }),
                )
                .await,
        );
        if let Some(exception) = registered?.get("exceptionDetails") {
            return Err(Error::script(exception));
        }
        Ok(Some(Context::for_session(target.session)))
    }
}

impl Session {
    async fn resolve_ref(
        &mut self,
        session: &str,
        backend: i64,
        raw: &str,
    ) -> Result<String, Error> {
        let mut retried = false;
        loop {
            let context = self.isolated_context(session).await?;
            let resolved = self
                .send_on(
                    session,
                    "DOM.resolveNode",
                    json!({ "backendNodeId": backend, "executionContextId": context }),
                )
                .await;
            match resolved {
                Ok(reply) => {
                    return reply
                        .pointer("/object/objectId")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                        .ok_or_else(|| Error::StaleRef(raw.to_owned()));
                }
                Err(error) if !retried && error.fault() == Fault::ContextGone => {
                    self.forget_world(session);
                    retried = true;
                }
                Err(Error::Protocol { .. }) => return Err(Error::StaleRef(raw.to_owned())),
                Err(error) => return Err(error),
            }
        }
    }
}
