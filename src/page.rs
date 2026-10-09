use crate::error::Error;
use crate::session::Session;
use crate::wait::LoadState;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
pub enum ScriptTag {
    Url(String),
    Content(String),
}

fn decode_base64(data: &str) -> Result<Vec<u8>, Error> {
    let mut out = Vec::with_capacity(data.len() / 4 * 3);
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for byte in data.bytes() {
        let sextet = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\n' | b'\r' => continue,
            _ => {
                return Err(Error::Protocol {
                    method: "Page.captureScreenshot".into(),
                    message: "invalid base64 data".into(),
                });
            }
        };
        buffer = (buffer << 6) | u32::from(sextet);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.extend_from_slice(&((buffer >> bits) & 0xff).to_be_bytes()[3..]);
        }
    }
    Ok(out)
}

impl Session {
    pub async fn reload(&mut self) -> Result<(), Error> {
        drop(self.send("Page.reload", json!({})).await?);
        let timeout = self.timing().navigate;
        self.load_state_within(LoadState::Load, timeout).await
    }

    async fn evaluate_text(&mut self, expression: &str) -> Result<String, Error> {
        Ok(self
            .evaluate(expression)
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
                message: "missing data".into(),
            })?;
        decode_base64(data)
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
