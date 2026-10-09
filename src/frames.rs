use crate::dom::BoundingBox;
use crate::error::Error;
use crate::locator::Locator;
use crate::selector::{Selector, quote};
use crate::session::{Session, World};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FrameInfo {
    pub id: String,
    pub url: String,
    pub name: Option<String>,
    pub parent_id: Option<String>,
}

pub struct FrameLocator<'a> {
    session: &'a mut Session,
    chain: Vec<Selector>,
}

impl<'a> FrameLocator<'a> {
    #[must_use]
    pub fn frame_locator(mut self, selector: impl Into<Selector>) -> Self {
        self.chain.push(selector.into());
        self
    }

    #[must_use]
    pub fn locator(self, inner: impl Into<Selector>) -> Locator<'a> {
        Locator::framed(self.session, inner.into(), self.chain)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Context {
    session_id: Option<String>,
    window: Option<String>,
    offset: (f64, f64),
}

impl Context {
    pub(crate) fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub(crate) fn wrap(&self, expression: &str) -> String {
        match &self.window {
            None => expression.to_owned(),
            Some(window) => format!("(async()=>await ({window}).eval({}))()", quote(expression)),
        }
    }

    pub(crate) fn shift(&self, bounds: BoundingBox) -> BoundingBox {
        BoundingBox {
            x: bounds.x + self.offset.0,
            y: bounds.y + self.offset.1,
            ..bounds
        }
    }

    fn child_window(&self, element: &str) -> String {
        let content = format!("({element}).contentWindow");
        match &self.window {
            None => content,
            Some(window) => format!("({window}).eval({})", quote(&content)),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrameMeta {
    x: f64,
    y: f64,
    src: String,
    origin: String,
    same_origin: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TargetKind {
    Iframe,
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetInfo {
    target_id: String,
    #[serde(rename = "type")]
    kind: TargetKind,
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrameNode {
    frame: FrameRecord,
    #[serde(default)]
    child_frames: Vec<FrameNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrameRecord {
    id: String,
    url: String,
    name: Option<String>,
    parent_id: Option<String>,
}

fn meta_expr(element: &str) -> String {
    format!(
        "(()=>{{const el={element};if(!el)return null;let origin='';try{{origin=new URL(el.src||'',location.href).origin}}catch(e){{}}let doc=null;try{{doc=el.contentDocument}}catch(e){{}}const same=!el.src||origin==='null'||origin===location.origin;if(same!==!!doc)return null;const v=el.getBoundingClientRect();if(v.bottom<0||v.right<0||v.top>innerHeight||v.left>innerWidth)el.scrollIntoView({{block:'center',inline:'center'}});const r=el.getBoundingClientRect();const cs=getComputedStyle(el);const n=v=>parseFloat(v)||0;return {{x:r.x+n(cs.borderLeftWidth)+n(cs.paddingLeft),y:r.y+n(cs.borderTopWidth)+n(cs.paddingTop),src:el.src||'',origin,sameOrigin:same}};}})()"
    )
}

fn origin_of(url: &str) -> Option<&str> {
    let (scheme, rest) = url.split_once("://")?;
    let host = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    url.get(..scheme.len() + 3 + host)
}

fn match_target<'t>(meta: &FrameMeta, iframes: &'t [TargetInfo]) -> Option<&'t TargetInfo> {
    let by_url = (!meta.src.is_empty())
        .then(|| {
            iframes.iter().find(|t| t.url == meta.src).or_else(|| {
                iframes.iter().find(|t| {
                    !t.url.is_empty()
                        && (t.url.starts_with(&meta.src) || meta.src.starts_with(&t.url))
                })
            })
        })
        .flatten();
    by_url.or_else(|| {
        let mut same = iframes
            .iter()
            .filter(|t| origin_of(&t.url) == Some(meta.origin.as_str()));
        match (same.next(), same.next()) {
            (Some(only), None) => Some(only),
            _ => None,
        }
    })
}

impl Session {
    #[must_use]
    pub fn frame_locator(&mut self, selector: impl Into<Selector>) -> FrameLocator<'_> {
        FrameLocator {
            session: self,
            chain: vec![selector.into()],
        }
    }

    pub async fn frames(&mut self) -> Result<Vec<FrameInfo>, Error> {
        let tree = self.send("Page.getFrameTree", json!({})).await?;
        let root: FrameNode =
            serde_json::from_value(tree.get("frameTree").cloned().unwrap_or(Value::Null))?;
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            out.push(FrameInfo {
                id: node.frame.id,
                url: node.frame.url,
                name: node.frame.name,
                parent_id: node.frame.parent_id,
            });
            stack.extend(node.child_frames.into_iter().rev());
        }
        Ok(out)
    }

    pub(crate) async fn read_in<T: DeserializeOwned>(
        &mut self,
        session_id: Option<&str>,
        expression: &str,
    ) -> Result<T, Error> {
        Ok(serde_json::from_value(
            self.evaluate_in(session_id, World::Isolated, expression)
                .await?,
        )?)
    }

    pub(crate) async fn frame_context(&mut self, chain: &[Selector]) -> Result<Context, Error> {
        let mut context = Context::default();
        for iframe in chain {
            context = self.enter_frame(context, iframe).await?;
        }
        Ok(context)
    }

    async fn enter_frame(&mut self, parent: Context, iframe: &Selector) -> Result<Context, Error> {
        let element = iframe.one_expr(None);
        let probe = parent.wrap(&meta_expr(&element));
        let timeout = self.timing().wait;
        drop(
            self.poll_expression_in(parent.session_id(), World::Isolated, &probe, timeout)
                .await?,
        );
        let meta: Option<FrameMeta> = self.read_in(parent.session_id(), &probe).await?;
        let meta = meta.ok_or_else(|| Error::Frame(format!("frame detached: {iframe:?}")))?;
        let offset = (parent.offset.0 + meta.x, parent.offset.1 + meta.y);
        if meta.same_origin {
            return Ok(Context {
                window: Some(parent.child_window(&element)),
                session_id: parent.session_id,
                offset,
            });
        }
        let session_id = self.attach_frame(&meta, iframe).await?;
        Ok(Context {
            session_id: Some(session_id),
            window: None,
            offset,
        })
    }

    async fn attach_frame(&mut self, meta: &FrameMeta, iframe: &Selector) -> Result<String, Error> {
        let listed = self.browser_send("Target.getTargets", json!({})).await?;
        let infos: Vec<TargetInfo> = serde_json::from_value(
            listed
                .get("targetInfos")
                .cloned()
                .unwrap_or_else(|| Value::Array(Vec::new())),
        )?;
        let iframes: Vec<TargetInfo> = infos
            .into_iter()
            .filter(|info| info.kind == TargetKind::Iframe)
            .collect();
        let target = match_target(meta, &iframes).ok_or_else(|| {
            Error::Frame(format!(
                "no out-of-process frame target for {iframe:?} (src {}, origin {})",
                meta.src, meta.origin
            ))
        })?;
        if let Some(cached) = self.frame_sessions().get(&target.target_id) {
            return Ok(cached.clone());
        }
        let attached = self
            .browser_send(
                "Target.attachToTarget",
                json!({ "targetId": target.target_id, "flatten": true }),
            )
            .await?;
        let session_id = attached
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| Error::Protocol {
                method: "Target.attachToTarget".into(),
                code: None,
                message: "missing sessionId".into(),
            })?;
        self.frame_sessions()
            .insert(target.target_id.clone(), session_id.clone());
        Ok(session_id)
    }
}
