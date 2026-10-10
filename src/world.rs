use crate::error::{Error, Fault};
use crate::human::{CursorOptions, Point, Rng, Step, Target, path};
use crate::session::Session;
use serde_json::{Value, json};

const WORLD_NAME: &str = "charm";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum World {
    Main,
    Isolated,
}

#[derive(Debug)]
pub(crate) struct Human {
    rng: Rng,
    cursor: CursorOptions,
}

impl Session {
    pub(crate) async fn evaluate_in(
        &mut self,
        session_id: Option<&str>,
        world: World,
        expression: &str,
    ) -> Result<Value, Error> {
        let result = self
            .evaluate_raw(session_id, world, expression, true)
            .await?;
        let remote = result.get("result").cloned().unwrap_or(Value::Null);
        Ok(remote.get("value").cloned().unwrap_or(remote))
    }

    pub(crate) async fn evaluate_handle_in(
        &mut self,
        session_id: Option<&str>,
        world: World,
        expression: &str,
    ) -> Result<String, Error> {
        let result = self
            .evaluate_raw(session_id, world, expression, false)
            .await?;
        result
            .pointer("/result/objectId")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| Error::Protocol {
                method: "Runtime.evaluate".into(),
                code: None,
                message: "the expression did not return an element".into(),
            })
    }

    async fn evaluate_raw(
        &mut self,
        session_id: Option<&str>,
        world: World,
        expression: &str,
        by_value: bool,
    ) -> Result<Value, Error> {
        let target = session_id.map_or_else(|| self.session_id().to_owned(), str::to_owned);
        let params = json!({
            "expression": expression,
            "returnByValue": by_value,
            "awaitPromise": true,
        });
        let result = match world {
            World::Main => self.send_on(&target, "Runtime.evaluate", params).await?,
            World::Isolated => self.evaluate_isolated(&target, params).await?,
        };
        if let Some(exception) = result.get("exceptionDetails") {
            return Err(Error::script(exception));
        }
        Ok(result)
    }

    async fn evaluate_isolated(&mut self, target: &str, mut params: Value) -> Result<Value, Error> {
        let cached = self.tabs_mut().active.worlds.get(target).copied();
        params["contextId"] = json!(self.isolated_context(target).await?);
        match self
            .send_on(target, "Runtime.evaluate", params.clone())
            .await
        {
            Err(error) if cached.is_some() && error.fault() == Fault::ContextGone => {
                self.forget_world(target);
                params["contextId"] = json!(self.isolated_context(target).await?);
                self.send_on(target, "Runtime.evaluate", params).await
            }
            other => other,
        }
    }

    pub(crate) fn forget_world(&mut self, target: &str) {
        self.tabs_mut().active.worlds.remove(target);
    }

    pub(crate) async fn isolated_context(&mut self, target: &str) -> Result<i64, Error> {
        if let Some(id) = self.tabs_mut().active.worlds.get(target) {
            return Ok(*id);
        }
        let tree = self.send_on(target, "Page.getFrameTree", json!({})).await?;
        let frame_id = tree
            .pointer("/frameTree/frame/id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| Error::Frame("Page.getFrameTree missing frame id".into()))?;
        let created = self
            .send_on(
                target,
                "Page.createIsolatedWorld",
                json!({
                    "frameId": frame_id,
                    "worldName": WORLD_NAME,
                    "grantUniveralAccess": false,
                }),
            )
            .await?;
        let context = created
            .get("executionContextId")
            .and_then(Value::as_i64)
            .ok_or_else(|| Error::Frame("Page.createIsolatedWorld missing context id".into()))?;
        self.tabs_mut()
            .active
            .worlds
            .insert(target.to_owned(), context);
        Ok(context)
    }

    pub fn set_human_input(&mut self, cursor: Option<CursorOptions>) {
        self.tabs_mut().human = cursor.map(|cursor| Human {
            rng: Rng::from_entropy(),
            cursor,
        });
    }

    pub(crate) fn human_path(&mut self, from: Point, to: Point) -> Option<Vec<Step>> {
        let human = self.tabs_mut().human.as_mut()?;
        Some(path(from, Target::Point(to), &human.cursor, &mut human.rng))
    }
}
