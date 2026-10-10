mod actions;
mod aria;
mod chain;
mod drag;
mod forms;
mod reads;
mod refs;

pub(crate) use aria::{aria_probe, aria_template, aria_text};
pub use chain::ActionOptions;
pub use forms::SelectBy;

use crate::dom::{Actionable, Need};
use crate::error::Error;
use crate::frames::Context;
use crate::selector::{Selector, TextMatch};
use crate::session::{Session, World};
use serde::de::DeserializeOwned;
use std::ops::Range;
use std::time::Duration;

const HIDDEN: &str = "document.visibilityState === 'hidden'";

pub struct Locator<'a> {
    session: &'a mut Session,
    selector: Selector,
    index: Option<usize>,
    frames: Vec<Selector>,
    context: Option<Context>,
    options: ActionOptions,
}

impl Session {
    #[must_use]
    pub fn locator(&mut self, selector: impl Into<Selector>) -> Locator<'_> {
        Locator::framed(self, selector.into(), Vec::new())
    }

    #[must_use]
    pub fn get_by_role(&mut self, role: &str, name: Option<&str>, exact: bool) -> Locator<'_> {
        self.locator(Selector::Role {
            role: role.to_owned(),
            name: name.map(|text| TextMatch::new(text, exact)),
        })
    }

    #[must_use]
    pub fn get_by_text(&mut self, text: &str, exact: bool) -> Locator<'_> {
        self.locator(Selector::Text(TextMatch::new(text, exact)))
    }

    #[must_use]
    pub fn get_by_test_id(&mut self, id: &str) -> Locator<'_> {
        self.locator(Selector::TestId(id.to_owned()))
    }

    #[must_use]
    pub fn get_by_label(&mut self, text: &str, exact: bool) -> Locator<'_> {
        self.locator(Selector::Label(TextMatch::new(text, exact)))
    }

    pub async fn click(&mut self, selector: impl Into<Selector>) -> Result<(), Error> {
        self.locator(selector).click().await
    }

    pub async fn hover(&mut self, selector: impl Into<Selector>) -> Result<(), Error> {
        self.locator(selector).hover().await
    }

    pub async fn type_text(
        &mut self,
        selector: impl Into<Selector>,
        text: &str,
    ) -> Result<(), Error> {
        self.locator(selector).type_text(text).await
    }

    pub async fn fill(&mut self, selector: impl Into<Selector>, text: &str) -> Result<(), Error> {
        self.locator(selector).fill(text).await
    }

    pub async fn wait_for_selector(&mut self, selector: impl Into<Selector>) -> Result<(), Error> {
        let expr = format!("!!({})", selector.into().one_expr(None));
        let timeout = self.timing().wait;
        drop(
            self.poll_expression_in(None, World::Isolated, &expr, timeout)
                .await?,
        );
        Ok(())
    }
}

impl<'a> Locator<'a> {
    pub(crate) fn framed(
        session: &'a mut Session,
        selector: Selector,
        frames: Vec<Selector>,
    ) -> Self {
        Self {
            session,
            selector,
            index: None,
            frames,
            context: None,
            options: ActionOptions::default(),
        }
    }

    async fn context(&mut self) -> Result<Context, Error> {
        if let Some(context) = &self.context {
            return Ok(context.clone());
        }
        let context = match self.bind_ref().await? {
            Some(context) => context,
            None => self.session.frame_context(&self.frames).await?,
        };
        self.context = Some(context.clone());
        Ok(context)
    }

    async fn eval<T: DeserializeOwned>(&mut self, expression: &str) -> Result<T, Error> {
        let context = self.context().await?;
        self.session
            .read_in(context.session_id(), &context.wrap(expression))
            .await
    }

    pub(crate) fn element_expr(&self) -> String {
        self.selector.one_expr(self.index)
    }

    pub(crate) fn count_expr(&self) -> String {
        self.selector.count_expr()
    }

    pub(crate) fn timing(&self) -> crate::Timing {
        self.session.timing()
    }

    pub(crate) fn wait(&self) -> Duration {
        self.options.timeout.unwrap_or(self.session.timing().wait)
    }

    pub(crate) async fn poll(
        &mut self,
        expression: &str,
        timeout: Duration,
    ) -> Result<serde_json::Value, Error> {
        let context = self.context().await?;
        self.session
            .poll_expression_in(
                context.session_id(),
                World::Isolated,
                &context.wrap(expression),
                timeout,
            )
            .await
    }

    pub(crate) async fn value(&mut self, expression: &str) -> Result<serde_json::Value, Error> {
        self.eval(expression).await
    }

    #[must_use]
    pub fn first(mut self) -> Self {
        self.index = None;
        self
    }

    #[must_use]
    pub fn nth(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    #[must_use]
    pub fn selector(&self) -> &Selector {
        &self.selector
    }

    pub async fn count(&mut self) -> Result<usize, Error> {
        let expr = self.selector.count_expr();
        self.eval(&expr).await
    }

    pub async fn all(&mut self) -> Result<Range<usize>, Error> {
        Ok(0..self.count().await?)
    }

    async fn settle_within(&mut self, timeout: Duration, need: Need) -> Result<Actionable, Error> {
        let hidden = self
            .session
            .evaluate_in(None, World::Isolated, HIDDEN)
            .await?;
        if hidden.as_bool() == Some(true) {
            drop(
                self.session
                    .send("Page.bringToFront", serde_json::json!({}))
                    .await?,
            );
        }
        let context = self.context().await?;
        let frame_wait = self.session.timing().actionable;
        let settled = context.wrap(&self.selector.settled_expr(self.index, frame_wait, need));
        let mut verdict: Actionable = match self
            .session
            .poll_expression_in(context.session_id(), World::Isolated, &settled, timeout)
            .await
        {
            Ok(serde_json::Value::String(json)) => serde_json::from_str(&json)?,
            Ok(_) | Err(Error::Timeout(_)) => {
                let probe = self.selector.actionable_expr(self.index, frame_wait, need);
                self.eval(&probe).await?
            }
            Err(error) => return Err(error),
        };
        if !self.frames.is_empty() {
            self.context = None;
        }
        let context = self.context().await?;
        verdict.bounds = verdict.bounds.map(|bounds| context.shift(bounds));
        if let Some(cover) = verdict.covered_by.take() {
            verdict.covered_by = Some(self.cover_label(cover).await);
        }
        Ok(verdict)
    }

    pub async fn settle(&mut self) -> Result<Actionable, Error> {
        let timeout = self.session.timing().actionable;
        self.settle_within(timeout, Need::Pointer).await
    }
}
