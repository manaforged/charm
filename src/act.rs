use crate::error::Error;
use crate::events::{ConsoleMessage, Dialog};
use crate::expect::expect_page;
use crate::input::ClickOptions;
use crate::locator::SelectBy;
use crate::network::UrlMatch;
use crate::pages::PageId;
use crate::selector::{Selector, TextMatch};
use crate::session::Session;
use crate::snapshot::SnapshotOptions;
use crate::wait::LoadState;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

mod wire;
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
#[non_exhaustive]
pub enum Action {
    Navigate {
        url: String,
    },
    Back,
    Forward,
    Reload,
    Click {
        #[serde(deserialize_with = "wire::selector")]
        target: Selector,
        #[serde(default, deserialize_with = "wire::click")]
        options: ClickOptions,
    },
    Fill {
        #[serde(deserialize_with = "wire::selector")]
        target: Selector,
        text: String,
    },
    Type {
        #[serde(deserialize_with = "wire::selector")]
        target: Selector,
        text: String,
    },
    Press {
        key: String,
        #[serde(default, deserialize_with = "wire::maybe_selector")]
        target: Option<Selector>,
    },
    Select {
        #[serde(deserialize_with = "wire::selector")]
        target: Selector,
        #[serde(deserialize_with = "wire::select_by")]
        by: SelectBy,
    },
    Check {
        #[serde(deserialize_with = "wire::selector")]
        target: Selector,
        #[serde(default = "wire::checked")]
        checked: bool,
    },
    Upload {
        #[serde(deserialize_with = "wire::selector")]
        target: Selector,
        files: Vec<PathBuf>,
    },
    Hover {
        #[serde(deserialize_with = "wire::selector")]
        target: Selector,
    },
    Drag {
        #[serde(deserialize_with = "wire::selector")]
        from: Selector,
        #[serde(deserialize_with = "wire::selector")]
        to: Selector,
    },
    Scroll {
        delta_y: f64,
        #[serde(default, deserialize_with = "wire::maybe_selector")]
        target: Option<Selector>,
    },
    Wait {
        #[serde(rename = "for")]
        for_: WaitFor,
    },
    Tab(TabAction),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(from = "wire::WaitWire")]
#[non_exhaustive]
pub enum WaitFor {
    Selector(Selector),
    Text(TextMatch),
    Url(UrlMatch),
    LoadState(LoadState),
    Duration(Duration),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum TabAction {
    New(Option<String>),
    Select(PageId),
    Close(PageId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ActReport {
    None,
    #[default]
    Delta,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Outcome {
    pub url_before: String,
    pub url_after: String,
    pub title: String,
    pub new_pages: Vec<PageId>,
    pub dialogs: Option<Vec<Dialog>>,
    pub console: Option<Vec<ConsoleMessage>>,
    pub snapshot: Option<String>,
}

impl Session {
    pub async fn act(&mut self, action: Action, report: ActReport) -> Result<Outcome, Error> {
        let url_before = self.url().await?;
        let before: BTreeSet<PageId> = self
            .pages()
            .await?
            .into_iter()
            .map(|page| page.id)
            .collect();
        self.perform(action).await?;
        let new_pages = self
            .pages()
            .await?
            .into_iter()
            .map(|page| page.id)
            .filter(|id| !before.contains(id))
            .collect();
        let watched = self.transport_mut().observer.events;
        let dialogs = watched.page.then(|| self.dialogs());
        let console = watched.runtime.then(|| self.console());
        let snapshot = self.report(report).await?;
        Ok(Outcome {
            url_before,
            url_after: self.url().await?,
            title: self.title().await?,
            new_pages,
            dialogs,
            console,
            snapshot,
        })
    }

    async fn report(&mut self, report: ActReport) -> Result<Option<String>, Error> {
        let previous = match report {
            ActReport::None => return Ok(None),
            ActReport::Full => None,
            ActReport::Delta => self.refs().last().cloned(),
        };
        let snapshot = self.snapshot(SnapshotOptions::default()).await?;
        Ok(Some(match previous {
            Some(previous) => snapshot.diff(&previous),
            None => snapshot.text(),
        }))
    }

    async fn perform(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::Navigate { url } => self.goto(&url).await,
            Action::Back => self.go_back().await,
            Action::Forward => self.go_forward().await,
            Action::Reload => self.reload().await,
            Action::Wait { for_ } => self.wait_for(for_).await,
            Action::Tab(tab) => self.tab(tab).await,
            element => self.element_action(element).await,
        }
    }

    async fn element_action(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::Click { target, options } => self.locator(target).click_with(options).await,
            Action::Fill { target, text } => self.fill(target, &text).await,
            Action::Type { target, text } => self.type_text(target, &text).await,
            Action::Press { key, target } => self.press_on(&key, target).await,
            Action::Select { target, by } => self.select_option(target, by).await.map(drop),
            Action::Check { target, checked } => self.locator(target).set_checked(checked).await,
            Action::Upload { target, files } => self.set_input_files(target, files).await,
            Action::Hover { target } => self.hover(target).await,
            Action::Drag { from, to } => self.drag_and_drop(from, to).await,
            Action::Scroll { delta_y, target } => self.scroll_on(delta_y, target).await,
            other => Err(Error::Protocol {
                method: "act".into(),
                code: None,
                message: format!("action not routed: {other:?}"),
            }),
        }
    }

    async fn press_on(&mut self, key: &str, target: Option<Selector>) -> Result<(), Error> {
        match target {
            Some(target) => self.locator(target).press(key).await,
            None => self.press(key).await,
        }
    }

    async fn scroll_on(&mut self, delta_y: f64, target: Option<Selector>) -> Result<(), Error> {
        if let Some(target) = target {
            self.hover(target).await?;
        }
        self.scroll(delta_y).await
    }

    async fn wait_for(&mut self, until: WaitFor) -> Result<(), Error> {
        match until {
            WaitFor::Selector(selector) => self.wait_for_selector(selector).await,
            WaitFor::Text(text) => self.wait_for_selector(Selector::Text(text)).await,
            WaitFor::Url(url) => expect_page(self).to_have_url(url.text_match()).await,
            WaitFor::LoadState(state) => self.wait_for_load_state(state).await,
            WaitFor::Duration(duration) => {
                let capped = duration.min(self.timing().wait);
                self.wait_for_timeout(capped).await;
                Ok(())
            }
        }
    }

    async fn tab(&mut self, tab: TabAction) -> Result<(), Error> {
        match tab {
            TabAction::New(url) => self.new_page(url.as_deref()).await.map(drop),
            TabAction::Select(id) => self.select_page(&id).await,
            TabAction::Close(id) => self.close_page(&id).await,
        }
    }
}
