mod reads;

use crate::dom::{Actionable, Blocker, BoundingBox, ClickResult};
use crate::error::Error;
use crate::input::MouseButton;
use crate::selector::Selector;
use crate::session::Session;
use std::ops::Range;
use std::time::Duration;

pub struct Locator<'a> {
    session: &'a mut Session,
    selector: Selector,
    index: Option<usize>,
}

impl Session {
    pub fn locator(&mut self, selector: impl Into<Selector>) -> Locator<'_> {
        Locator {
            session: self,
            selector: selector.into(),
            index: None,
        }
    }

    pub fn get_by_role(&mut self, role: &str, name: Option<&str>, exact: bool) -> Locator<'_> {
        self.locator(Selector::Role {
            role: role.to_owned(),
            name: name.map(str::to_owned),
            exact,
        })
    }

    pub fn get_by_text(&mut self, text: &str, exact: bool) -> Locator<'_> {
        self.locator(Selector::Text {
            text: text.to_owned(),
            exact,
        })
    }

    pub fn get_by_test_id(&mut self, id: &str) -> Locator<'_> {
        self.locator(Selector::TestId(id.to_owned()))
    }

    pub fn get_by_label(&mut self, text: &str, exact: bool) -> Locator<'_> {
        self.locator(Selector::Label {
            text: text.to_owned(),
            exact,
        })
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

impl Locator<'_> {
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
        self.session.read(&expr).await
    }

    pub async fn all(&mut self) -> Result<Range<usize>, Error> {
        Ok(0..self.count().await?)
    }

    async fn probe(&mut self) -> Result<Actionable, Error> {
        let expr = self.selector.actionable_expr(self.index);
        self.session.read(&expr).await
    }

    async fn settle_within(&mut self, timeout: Duration) -> Result<Actionable, Error> {
        let ready = self.selector.ready_expr(self.index);
        match self.session.poll_expression(&ready, timeout).await {
            Ok(_) | Err(Error::Timeout(_)) => self.probe().await,
            Err(error) => Err(error),
        }
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
        self.click().await?;
        let select = format!(
            "(el=>{{if(el&&typeof el.select==='function'){{el.select();return true}}if(el){{const r=document.createRange();r.selectNodeContents(el);const s=getSelection();s.removeAllRanges();s.addRange(r);return true}}return false}})({})",
            self.selector.one_expr(self.index)
        );
        drop(self.session.evaluate(&select).await?);
        self.session.press("Backspace").await?;
        self.session.type_chars(text).await
    }

    pub async fn press_sequentially(&mut self, text: &str) -> Result<(), Error> {
        self.click().await?;
        self.session.type_chars(text).await
    }

    pub async fn type_text(&mut self, text: &str) -> Result<(), Error> {
        self.press_sequentially(text).await
    }
}
