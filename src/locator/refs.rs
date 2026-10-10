use super::Locator;
use crate::dom::{COVER, REGISTER_REF};
use crate::error::{Error, Fault};
use crate::frames::Context;
use crate::selector::REF_REGISTRY;
use crate::session::{Session, World};
use crate::snapshot::{NodeRef, RefTarget};
use serde_json::{Value, json};

impl Locator<'_> {
    pub(crate) async fn bind_ref(&mut self) -> Result<Option<Context>, Error> {
        let ids: Vec<String> = self
            .selector
            .ref_ids()
            .into_iter()
            .map(str::to_owned)
            .collect();
        let mut context = None;
        for raw in ids {
            let bound = self.bind_one(&raw).await?;
            context.get_or_insert(bound);
        }
        Ok(context)
    }

    async fn bind_one(&mut self, raw: &str) -> Result<Context, Error> {
        let target = NodeRef::parse(raw)
            .and_then(|node_ref| self.session.refs().get(node_ref).cloned())
            .ok_or_else(|| Error::StaleRef(raw.to_owned()))?;
        let session = target
            .session
            .clone()
            .unwrap_or_else(|| self.session.session_id().to_owned());
        let object = self
            .session
            .resolve_ref(&session, target.backend, raw)
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
        self.session.release(&session, &object).await;
        if let Some(exception) = registered?.get("exceptionDetails") {
            return Err(Error::script(exception));
        }
        let offset = match (target.session.as_deref(), target.frame.as_deref()) {
            (Some(frame), _) => Some(self.session.frame_offset(frame).await?),
            (None, Some(frame)) => self.session.owner_offset(&session, frame).await?,
            (None, None) => None,
        };
        Ok(match offset {
            Some(offset) => Context::for_frame_session(session, offset),
            None => Context::for_session(target.session),
        })
    }

    pub(crate) async fn backend_of(&mut self, expression: &str) -> Result<RefTarget, Error> {
        let context = self.context().await?;
        let frame = context.session_id().map(str::to_owned);
        let target = frame
            .clone()
            .unwrap_or_else(|| self.session.session_id().to_owned());
        let object = self
            .session
            .evaluate_handle_in(frame.as_deref(), World::Isolated, &context.wrap(expression))
            .await?;
        let described = self
            .session
            .send_on(&target, "DOM.describeNode", json!({ "objectId": object }))
            .await;
        self.session.release(&target, &object).await;
        let backend = described?
            .pointer("/node/backendNodeId")
            .and_then(Value::as_i64)
            .ok_or_else(|| Error::Protocol {
                method: "DOM.describeNode".to_owned(),
                code: None,
                message: format!("no backend node for {expression}"),
            })?;
        Ok(RefTarget {
            backend,
            session: frame,
            frame: None,
        })
    }

    pub(crate) fn ref_for(&mut self, target: &RefTarget) -> Option<NodeRef> {
        let refs = self.session.refs();
        (1_u32..)
            .map_while(|n| {
                let node_ref = NodeRef::parse(&format!("e{n}"))?;
                refs.get(node_ref).map(|found| (node_ref, found))
            })
            .find(|(_, found)| *found == target)
            .map(|(node_ref, _)| node_ref)
    }

    pub(crate) async fn cover_label(&mut self, label: String) -> String {
        match self.backend_of(COVER).await {
            Ok(target) => match self.ref_for(&target) {
                Some(node_ref) => format!("{label} [ref={node_ref}]"),
                None => label,
            },
            Err(_) => label,
        }
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

    async fn release(&mut self, session: &str, object: &str) {
        drop(
            self.send_on(
                session,
                "Runtime.releaseObject",
                json!({ "objectId": object }),
            )
            .await,
        );
    }
}
