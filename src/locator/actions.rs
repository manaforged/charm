use super::Locator;
use crate::dom::{Actionable, Blocker, BoundingBox, ClickResult, Need, Read};
use crate::error::Error;
use crate::input::{ClickOptions, MouseButton};
use crate::session::World;
use serde_json::Value;
use std::time::Duration;

impl Locator<'_> {
    async fn await_frame_hit(&mut self, bounds: BoundingBox) -> Result<(), Error> {
        let (x, y) = bounds.center();
        let probe = format!(
            "(()=>{{const e=document.elementFromPoint({x},{y});return !!e&&e.tagName==='IFRAME';}})()"
        );
        let timeout = self.session.timing().actionable;
        match self
            .session
            .poll_expression_in(None, World::Isolated, &probe, timeout)
            .await
        {
            Ok(_) | Err(Error::Timeout(_)) => Ok(()),
            Err(error) => Err(error),
        }
    }

    fn blocked(&self, verdict: Actionable) -> Error {
        Error::NotActionable {
            reason: verdict.reason,
            covered_by: verdict.covered_by,
            selector: self.selector.clone(),
        }
    }

    async fn forced_box(&mut self) -> Result<BoundingBox, Error> {
        let element = self.element_expr();
        let timeout = self.wait();
        drop(self.poll(&Read::Attached.expr(&element), timeout).await?);
        let scroll = format!(
            "(e=>{{e.scrollIntoView({{block:'center',inline:'center'}});return true}})({element})"
        );
        drop(self.eval::<Value>(&scroll).await?);
        let local: Option<BoundingBox> = self.eval(&Read::BoundingBox.expr(&element)).await?;
        let context = self.context().await?;
        local
            .map(|bounds| context.shift(bounds))
            .ok_or_else(|| Error::NotActionable {
                reason: Some(Blocker::ZeroSize),
                covered_by: None,
                selector: self.selector.clone(),
            })
    }

    pub(crate) async fn actionable_box(&mut self) -> Result<BoundingBox, Error> {
        if self.options.force {
            return self.forced_box().await;
        }
        let timeout = self.wait();
        let verdict = self.settle_within(timeout, Need::Pointer).await?;
        match (verdict.ok, verdict.bounds) {
            (true, Some(bounds)) => {
                if !self.frames.is_empty() {
                    self.await_frame_hit(bounds).await?;
                }
                Ok(bounds)
            }
            _ => Err(self.blocked(verdict)),
        }
    }

    async fn ensure_editable(&mut self) -> Result<(), Error> {
        if self.options.force {
            let attached = Read::Attached.expr(&self.element_expr());
            let timeout = self.wait();
            drop(self.poll(&attached, timeout).await?);
            return Ok(());
        }
        let timeout = self.wait();
        let verdict = self.settle_within(timeout, Need::Editable).await?;
        if verdict.ok {
            return Ok(());
        }
        Err(self.blocked(verdict))
    }

    pub async fn click(&mut self) -> Result<(), Error> {
        self.click_checked(ClickOptions::default()).await
    }

    async fn arm_hit(&mut self) -> Result<(), Error> {
        let arm = format!(
            "(el=>{{globalThis.__charmHit=false;((el&&el.ownerDocument.defaultView)||window).addEventListener('pointerdown',e=>{{globalThis.__charmHit=!!el&&(el===e.target||el.contains(e.target));}},{{capture:true,once:true}});return true;}})({})",
            self.element_expr()
        );
        drop(self.eval::<Value>(&arm).await?);
        Ok(())
    }

    async fn missed(&mut self) -> Error {
        let covered_by = match self.settle_within(Duration::ZERO, Need::Pointer).await {
            Ok(verdict) => verdict.covered_by,
            Err(error) => return error,
        };
        Error::NotActionable {
            reason: Some(Blocker::Obscured),
            covered_by,
            selector: self.selector.clone(),
        }
    }

    pub(crate) async fn click_checked(&mut self, options: ClickOptions) -> Result<(), Error> {
        let deadline = tokio::time::Instant::now() + self.wait();
        loop {
            let bounds = self.actionable_box().await?;
            let (x, y) = options.position.map_or_else(
                || bounds.center(),
                |(dx, dy)| (bounds.x + dx, bounds.y + dy),
            );
            if self.options.force {
                return self.session.click_at(x, y, options).await;
            }
            self.arm_hit().await?;
            self.session.click_at(x, y, options).await?;
            if self.eval::<bool>("globalThis.__charmHit===true").await? {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(self.missed().await);
            }
            self.context = None;
            let pause = self.session.timing().poll;
            self.session.idle(pause).await?;
        }
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
        self.ensure_editable().await?;
        let element = self.element_expr();
        let select = format!(
            "(el=>{{el.focus();if(typeof el.select==='function'){{el.select();return true}}const r=document.createRange();r.selectNodeContents(el);const s=getSelection();s.removeAllRanges();s.addRange(r);return true}})({element})"
        );
        drop(self.eval::<Value>(&select).await?);
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
        self.ensure_editable().await?;
        self.click().await?;
        self.session.type_chars(text).await
    }
}
