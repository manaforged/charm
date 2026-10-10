use crate::error::Error;
use crate::expect::expect_page;
use crate::selector::TextMatch;
use crate::session::{Session, World};
use crate::wait::LoadState;
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Instant;

const DOCUMENT_ID: &str = "String(performance.timeOrigin)";
const ARM_UNLOAD: &str = "(() => { window.__charmUnload = false; addEventListener('beforeunload', () => { window.__charmUnload = true; }, { once: true }); return String(performance.timeOrigin); })()";
const UNLOADING: &str = "window.__charmUnload === true";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ImageFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
}

impl ImageFormat {
    fn name(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Webp => "webp",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Clip {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScreenshotOptions {
    pub format: ImageFormat,
    pub quality: Option<u8>,
    pub full_page: bool,
    pub clip: Option<Clip>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScriptTag {
    Url(String),
    Content(String),
}

impl Session {
    pub async fn reload(&mut self) -> Result<(), Error> {
        let before = self.document_id().await?;
        drop(self.send("Page.reload", json!({})).await?);
        self.await_document(&before, LoadState::Load, "Page.reload")
            .await
    }

    pub async fn open(&mut self, url: &str) -> Result<(), Error> {
        self.navigate(url).await?;
        Ok(())
    }

    pub async fn goto(&mut self, url: &str) -> Result<(), Error> {
        let before = self.document_id().await?;
        if self.navigate(url).await? {
            self.await_document(&before, LoadState::DomContentLoaded, "Page.navigate")
                .await?;
        }
        Ok(())
    }

    async fn navigate(&mut self, url: &str) -> Result<bool, Error> {
        let nav = self.send("Page.navigate", json!({ "url": url })).await?;
        if let Some(error) = nav.get("errorText").and_then(Value::as_str)
            && error != "net::ERR_ABORTED"
        {
            return Err(Error::Protocol {
                method: "Page.navigate".into(),
                code: None,
                message: error.to_string(),
            });
        }
        Ok(nav.get("loaderId").and_then(Value::as_str).is_some())
    }

    pub(crate) async fn arm_unload(&mut self) -> Result<String, Error> {
        self.evaluate_text(ARM_UNLOAD).await
    }

    pub(crate) async fn follow(&mut self, before: &str, method: &str) -> Result<(), Error> {
        let grace = self.timing().follow;
        let leaving = match self
            .poll_expression_in(None, World::Isolated, UNLOADING, grace)
            .await
        {
            Ok(_) => true,
            Err(Error::Timeout(_)) => false,
            Err(_) => true,
        };
        let replaced = leaving || self.document_id().await.map_or(true, |now| now != before);
        if replaced {
            self.await_document(before, LoadState::DomContentLoaded, method)
                .await?;
        }
        Ok(())
    }

    async fn document_id(&mut self) -> Result<String, Error> {
        self.evaluate_text(DOCUMENT_ID).await
    }

    async fn await_document(
        &mut self,
        before: &str,
        state: LoadState,
        method: &str,
    ) -> Result<(), Error> {
        let expression = format!("{DOCUMENT_ID} !== {}", serde_json::to_string(before)?);
        let deadline = Instant::now() + self.timing().navigate;
        let waited = match self
            .poll_expression_in(None, World::Isolated, &expression, self.timing().navigate)
            .await
        {
            Ok(_) => {
                let left = deadline.saturating_duration_since(Instant::now());
                self.load_state_within(state, left).await
            }
            Err(error) => Err(error),
        };
        match waited {
            Err(Error::Timeout(_)) => Err(Error::Timeout(method.into())),
            other => other,
        }
    }

    async fn evaluate_text(&mut self, expression: &str) -> Result<String, Error> {
        Ok(self
            .evaluate_in(None, World::Isolated, expression)
            .await?
            .as_str()
            .unwrap_or_default()
            .to_string())
    }

    pub async fn title(&mut self) -> Result<String, Error> {
        self.evaluate_text("document.title").await
    }

    pub async fn url(&mut self) -> Result<String, Error> {
        self.evaluate_text("location.href").await
    }

    pub async fn content(&mut self) -> Result<String, Error> {
        self.evaluate_text("document.documentElement.outerHTML")
            .await
    }

    pub async fn screenshot(&mut self, options: ScreenshotOptions) -> Result<Vec<u8>, Error> {
        let mut params = json!({
            "format": options.format.name(),
            "captureBeyondViewport": options.full_page,
        });
        if let Some(quality) = options.quality {
            params["quality"] = json!(quality);
        }
        if let Some(clip) = options.clip {
            params["clip"] = json!({
                "x": clip.x,
                "y": clip.y,
                "width": clip.width,
                "height": clip.height,
                "scale": clip.scale,
            });
        }
        let shot = self.send("Page.captureScreenshot", params).await?;
        let data = shot
            .get("data")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::Protocol {
                method: "Page.captureScreenshot".into(),
                code: None,
                message: "missing data".into(),
            })?;
        crate::network::decode_base64(data, "Page.captureScreenshot")
    }

    pub async fn add_script_tag(&mut self, tag: ScriptTag) -> Result<(), Error> {
        let expression = match tag {
            ScriptTag::Url(url) => format!(
                "new Promise((res, rej) => {{ const s = document.createElement('script'); s.src = {}; s.onload = () => res(true); s.onerror = () => rej(new Error('addScriptTag load failed: ' + s.src)); (document.head || document.documentElement).appendChild(s); }})",
                serde_json::to_string(&url)?
            ),
            ScriptTag::Content(content) => format!(
                "(() => {{ const s = document.createElement('script'); s.textContent = {}; (document.head || document.documentElement).appendChild(s); return true; }})()",
                serde_json::to_string(&content)?
            ),
        };
        drop(self.evaluate(&expression).await?);
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct History {
    current_index: i64,
    entries: Vec<HistoryEntry>,
}

#[derive(Deserialize)]
struct HistoryEntry {
    id: i64,
    url: String,
}

impl Session {
    pub async fn go_back(&mut self) -> Result<(), Error> {
        self.history(-1).await
    }

    pub async fn go_forward(&mut self) -> Result<(), Error> {
        self.history(1).await
    }

    async fn history(&mut self, step: i64) -> Result<(), Error> {
        let history: History =
            serde_json::from_value(self.send("Page.getNavigationHistory", json!({})).await?)?;
        let Some(entry) = history
            .current_index
            .checked_add(step)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| history.entries.get(index))
        else {
            return Ok(());
        };
        let params = json!({ "entryId": entry.id });
        drop(self.send("Page.navigateToHistoryEntry", params).await?);
        expect_page(self)
            .to_have_url(TextMatch::Exact(entry.url.clone()))
            .await?;
        self.wait_for_load_state(LoadState::DomContentLoaded).await
    }
}
