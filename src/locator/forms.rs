use super::Locator;
use crate::dom::Read;
use crate::error::Error;
use crate::input::ClickOptions;
use crate::selector::Selector;
use crate::session::{Session, World};
use serde_json::{Value, json};
use std::path::PathBuf;

const SELECT: &str = "(e,by,want,commit)=>{if(!e||e.tagName!=='SELECT')return null;const opts=[...e.options];const pick=want.map(w=>opts.find((o,i)=>by==='value'?o.value===w:by==='label'?o.label===w:i===w));if(pick.some(o=>!o))return null;if(!commit)return true;const keep=e.multiple?pick:pick.slice(0,1);opts.forEach(o=>{o.selected=keep.includes(o)});e.dispatchEvent(new Event('input',{bubbles:true}));e.dispatchEvent(new Event('change',{bubbles:true}));return keep.map(o=>o.value);}";

const CHECKABLE: &str = "e=>{if(!e)return false;const r=(e.getAttribute('role')||'').toLowerCase();const t=(e.type||'').toLowerCase();return (e.tagName==='INPUT'&&(t==='checkbox'||t==='radio'))||['checkbox','switch','radio','menuitemcheckbox'].includes(r);}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectBy {
    Value(Vec<String>),
    Label(Vec<String>),
    Index(Vec<usize>),
}

impl SelectBy {
    fn args(&self) -> (&'static str, Value) {
        match self {
            Self::Value(values) => ("value", json!(values)),
            Self::Label(labels) => ("label", json!(labels)),
            Self::Index(indexes) => ("index", json!(indexes)),
        }
    }
}

impl Locator<'_> {
    async fn ready(&mut self, read: Read<'_>) -> Result<(), Error> {
        let element = self.element_expr();
        let ready = format!(
            "({}&&{})",
            read.expr(&element),
            Read::Enabled.expr(&element)
        );
        let timeout = self.wait();
        drop(self.poll(&ready, timeout).await?);
        Ok(())
    }

    async fn run(&mut self, body: &str) -> Result<Value, Error> {
        let expression = format!("({body})({})", self.element_expr());
        self.eval(&expression).await
    }

    pub async fn select_option(&mut self, by: SelectBy) -> Result<Vec<String>, Error> {
        self.ready(Read::Visible).await?;
        let (kind, want) = by.args();
        let element = self.element_expr();
        let call = |commit: bool| format!("({SELECT})({element},'{kind}',{want},{commit})");
        let timeout = self.wait();
        drop(self.poll(&call(false), timeout).await?);
        let picked = self.eval(&call(true)).await?;
        Ok(serde_json::from_value::<Option<Vec<String>>>(picked)?.unwrap_or_default())
    }

    pub async fn check(&mut self) -> Result<(), Error> {
        self.set_checked(true).await
    }

    pub async fn uncheck(&mut self) -> Result<(), Error> {
        self.set_checked(false).await
    }

    pub async fn set_checked(&mut self, checked: bool) -> Result<(), Error> {
        self.ready(Read::Visible).await?;
        if self.run(CHECKABLE).await? != Value::Bool(true) {
            return Err(self.mismatch("checkbox, radio, or switch", "other element"));
        }
        if self.is_checked().await? == checked {
            return Ok(());
        }
        self.click().await?;
        match self.is_checked().await? {
            state if state == checked => Ok(()),
            state => Err(self.mismatch(&format!("checked={checked}"), &format!("checked={state}"))),
        }
    }

    fn mismatch(&self, expected: &str, received: &str) -> Error {
        Error::Assertion {
            expected: expected.to_owned(),
            received: received.to_owned(),
            selector: Some(self.selector.clone()),
        }
    }

    pub async fn set_input_files(&mut self, files: Vec<PathBuf>) -> Result<(), Error> {
        self.ready(Read::Attached).await?;
        let files = files
            .iter()
            .map(|file| std::path::absolute(file).map(|path| path.display().to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let context = self.context().await?;
        let target = context
            .session_id()
            .map_or_else(|| self.session.session_id().to_owned(), str::to_owned);
        let element = context.wrap(&self.element_expr());
        let object_id = self
            .session
            .evaluate_handle_in(context.session_id(), World::Isolated, &element)
            .await?;
        let params = json!({ "files": files, "objectId": object_id });
        drop(
            self.session
                .send_on(&target, "DOM.setFileInputFiles", params)
                .await?,
        );
        Ok(())
    }

    pub async fn focus(&mut self) -> Result<(), Error> {
        self.ready(Read::Attached).await?;
        drop(self.run("e=>{e.focus();return true}").await?);
        Ok(())
    }

    pub async fn blur(&mut self) -> Result<(), Error> {
        self.ready(Read::Attached).await?;
        drop(self.run("e=>{e.blur();return true}").await?);
        Ok(())
    }

    pub async fn click_with(&mut self, options: ClickOptions) -> Result<(), Error> {
        self.click_checked(options).await
    }

    pub async fn dblclick(&mut self) -> Result<(), Error> {
        self.click_with(ClickOptions {
            count: 2,
            ..ClickOptions::default()
        })
        .await
    }

    pub async fn press(&mut self, key: &str) -> Result<(), Error> {
        self.focus().await?;
        self.session.press(key).await
    }

    pub async fn clear(&mut self) -> Result<(), Error> {
        self.fill("").await
    }

    pub async fn drag_to(&mut self, target: impl Into<Selector>) -> Result<(), Error> {
        let from = self.actionable_box().await?.center();
        let html5 = self.run("e=>!!e&&e.draggable===true").await? == Value::Bool(true);
        let frames = self.frames.clone();
        let to = Locator::framed(&mut *self.session, target.into(), frames)
            .actionable_box()
            .await?
            .center();
        self.session.drag(from, to, html5).await
    }

    pub async fn drag_to_point(&mut self, x: f64, y: f64) -> Result<(), Error> {
        let from = self.actionable_box().await?.center();
        let html5 = self.run("e=>!!e&&e.draggable===true").await? == Value::Bool(true);
        self.session.drag(from, (x, y), html5).await
    }
}

impl Session {
    pub async fn select_option(
        &mut self,
        selector: impl Into<Selector>,
        by: SelectBy,
    ) -> Result<Vec<String>, Error> {
        self.locator(selector).select_option(by).await
    }

    pub async fn check(&mut self, selector: impl Into<Selector>) -> Result<(), Error> {
        self.locator(selector).check().await
    }

    pub async fn uncheck(&mut self, selector: impl Into<Selector>) -> Result<(), Error> {
        self.locator(selector).uncheck().await
    }

    pub async fn set_input_files(
        &mut self,
        selector: impl Into<Selector>,
        files: Vec<PathBuf>,
    ) -> Result<(), Error> {
        self.locator(selector).set_input_files(files).await
    }

    pub async fn focus(&mut self, selector: impl Into<Selector>) -> Result<(), Error> {
        self.locator(selector).focus().await
    }

    pub async fn dblclick(&mut self, selector: impl Into<Selector>) -> Result<(), Error> {
        self.locator(selector).dblclick().await
    }

    pub async fn drag_and_drop(
        &mut self,
        source: impl Into<Selector>,
        target: impl Into<Selector>,
    ) -> Result<(), Error> {
        self.locator(source).drag_to(target).await
    }
}
