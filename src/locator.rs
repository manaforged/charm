mod reads;

use crate::dom::{Actionable, Blocker, BoundingBox, ClickResult, Read};
use crate::error::Error;
use crate::frames::Context;
use crate::input::MouseButton;
use crate::selector::{Selector, TextMatch};
use crate::session::Session;
use serde::de::DeserializeOwned;
use std::ops::Range;
use std::time::Duration;

pub struct Locator<'a> {
    session: &'a mut Session,
    selector: Selector,
    index: Option<usize>,
    frames: Vec<Selector>,
    context: Option<Context>,
}

impl Session {
    pub fn locator(&mut self, selector: impl Into<Selector>) -> Locator<'_> {
        Locator::framed(self, selector.into(), Vec::new())
    }

    pub fn get_by_role(&mut self, role: &str, name: Option<&str>, exact: bool) -> Locator<'_> {
        self.locator(Selector::Role {
            role: role.to_owned(),
            name: name.map(|text| TextMatch::new(text, exact)),
        })
    }

    pub fn get_by_text(&mut self, text: &str, exact: bool) -> Locator<'_> {
        self.locator(Selector::Text(TextMatch::new(text, exact)))
    }

    pub fn get_by_test_id(&mut self, id: &str) -> Locator<'_> {
        self.locator(Selector::TestId(id.to_owned()))
    }

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
        drop(self.poll_expression(&expr, timeout).await?);
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
        }
    }

    async fn context(&mut self) -> Result<Context, Error> {
        if let Some(context) = &self.context {
            return Ok(context.clone());
        }
        let context = self.session.frame_context(&self.frames).await?;
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

    pub(crate) async fn poll(
        &mut self,
        expression: &str,
        timeout: Duration,
    ) -> Result<serde_json::Value, Error> {
        let context = self.context().await?;
        self.session
            .poll_expression_in(context.session_id(), &context.wrap(expression), timeout)
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

    async fn settle_within(&mut self, timeout: Duration) -> Result<Actionable, Error> {
        let context = self.context().await?;
        let settled = context.wrap(&self.selector.settled_expr(self.index));
        let mut verdict: Actionable = match self
            .session
            .poll_expression_in(context.session_id(), &settled, timeout)
            .await
        {
            Ok(serde_json::Value::String(json)) => serde_json::from_str(&json)?,
            Ok(_) | Err(Error::Timeout(_)) => {
                self.eval(&self.selector.actionable_expr(self.index))
                    .await?
            }
            Err(error) => return Err(error),
        };
        verdict.bounds = verdict.bounds.map(|bounds| context.shift(bounds));
        Ok(verdict)
    }

    pub async fn settle(&mut self) -> Result<Actionable, Error> {
        let timeout = self.session.timing().actionable;
        self.settle_within(timeout).await
    }

    async fn actionable_box(&mut self) -> Result<BoundingBox, Error> {
        let timeout = self.session.timing().wait;
        let verdict = self.settle_within(timeout).await?;
        match (verdict.ok, verdict.bounds) {
            (true, Some(bounds)) => Ok(bounds),
            _ => Err(Error::Timeout(format!(
                "not actionable ({:?}{}): {:?}",
                verdict.reason.unwrap_or(Blocker::NoMatch),
                verdict
                    .covered_by
                    .map(|by| format!(", covered by {by}"))
                    .unwrap_or_default(),
                self.selector
            ))),
        }
    }

    pub async fn click(&mut self) -> Result<(), Error> {
        let (x, y) = self.actionable_box().await?.center();
        self.session.mouse_click(x, y, MouseButton::Left).await
    }

    pub async fn try_click(&mut self) -> Result<ClickResult, Error> {
        let verdict = self.settle().await?;
        match (verdict.ok, verdict.bounds) {
            (true, Some(bounds)) => {
                let (x, y) = bounds.center();
                self.session.mouse_click(x, y, MouseButton::Left).await?;
                Ok(ClickResult {
                    clicked: true,
                    reason: None,
                    covered_by: None,
                })
            }
            _ => Ok(ClickResult {
                clicked: false,
                reason: verdict.reason,
                covered_by: verdict.covered_by,
            }),
        }
    }

    pub async fn hover(&mut self) -> Result<(), Error> {
        let (x, y) = self.actionable_box().await?.center();
        self.session.mouse_move(x, y).await
    }

    pub async fn fill(&mut self, text: &str) -> Result<(), Error> {
        let element = self.selector.one_expr(self.index);
        let ready = format!(
            "({}&&{})",
            Read::Visible.expr(&element),
            Read::Enabled.expr(&element)
        );
        let timeout = self.session.timing().wait;
        drop(self.poll(&ready, timeout).await?);
        let select = format!(
            "(el=>{{el.focus();if(typeof el.select==='function'){{el.select();return true}}const r=document.createRange();r.selectNodeContents(el);const s=getSelection();s.removeAllRanges();s.addRange(r);return true}})({element})"
        );
        drop(self.eval::<serde_json::Value>(&select).await?);
        if text.is_empty() {
            return self.session.press("Backspace").await;
        }
        drop(
            self.session
                .send("Input.insertText", serde_json::json!({ "text": text }))
                .await?,
        );
        Ok(())
    }

    pub async fn type_text(&mut self, text: &str) -> Result<(), Error> {
        self.click().await?;
        self.session.type_chars(text).await
    }
}
