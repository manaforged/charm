use crate::dom::{EDITABLE, ENABLED, Need, RESOLVER};
use serde_json::{Value, json};
use std::time::Duration;

const XPATH_PREFIX: &str = "xpath=";
const RESOLVER_VAR: &str = "__c";
const REF_PREFIX: &str = "ref=";
pub(crate) const REF_REGISTRY: &str = "__charmRefs";

pub(crate) const MATCH_TEXT: &str = r"(function(actual,q,raw){
  function norm(t){var v=t==null?'':String(t);return raw?v:v.replace(/\s+/g,' ').trim();}
  var a=norm(actual);
  if(q.mode==='regex')return new RegExp(q.text,q.flags.replace(/[gy]/g,'')).test(a);
  var w=norm(q.text);
  return q.mode==='exact'?a===w:a.toLowerCase().indexOf(w.toLowerCase())>=0;
})";

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Selector {
    Css(String),
    XPath(String),
    Role {
        role: String,
        name: Option<TextMatch>,
    },
    Text(TextMatch),
    TestId(String),
    Label(TextMatch),
    Ref(String),
    Within {
        outer: Box<Selector>,
        inner: Box<Selector>,
    },
    Filtered {
        base: Box<Selector>,
        filter: Box<Filter>,
    },
    And(Box<Selector>, Box<Selector>),
    Or(Box<Selector>, Box<Selector>),
    Nth {
        base: Box<Selector>,
        at: Position,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    pub has_text: Option<TextMatch>,
    pub has_not_text: Option<TextMatch>,
    pub has: Option<Selector>,
    pub has_not: Option<Selector>,
    pub visible: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Position {
    Index(usize),
    Last,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextMatch {
    Exact(String),
    Contains(String),
    Regex { pattern: String, flags: String },
}

impl TextMatch {
    #[must_use]
    pub fn new(text: &str, exact: bool) -> Self {
        if exact {
            Self::Exact(text.to_owned())
        } else {
            Self::Contains(text.to_owned())
        }
    }

    #[must_use]
    pub fn regex(pattern: &str, flags: &str) -> Self {
        Self::Regex {
            pattern: pattern.to_owned(),
            flags: flags.to_owned(),
        }
    }

    pub(crate) fn test_expr(&self, value: &str, raw: bool) -> String {
        format!("{MATCH_TEXT}({value},{},{raw})", self.query())
    }

    pub(crate) fn query(&self) -> Value {
        match self {
            Self::Exact(text) => json!({ "mode": "exact", "text": text }),
            Self::Contains(text) => json!({ "mode": "contains", "text": text }),
            Self::Regex { pattern, flags } => {
                json!({ "mode": "regex", "text": pattern, "flags": flags })
            }
        }
    }
}

impl Selector {
    #[must_use]
    pub fn parse_target(raw: &str) -> Self {
        let bare = raw.strip_prefix('@').unwrap_or(raw);
        match crate::snapshot::NodeRef::parse(bare) {
            Some(_) => Self::Ref(bare.to_owned()),
            None => Self::parse(raw),
        }
    }

    #[must_use]
    pub fn parse(raw: &str) -> Self {
        if let Some(id) = raw.strip_prefix(REF_PREFIX) {
            return Self::Ref(id.to_owned());
        }
        match raw.strip_prefix(XPATH_PREFIX) {
            Some(path) => Self::XPath(path.to_owned()),
            None => Self::Css(raw.to_owned()),
        }
    }

    fn leaf(&self) -> bool {
        matches!(self, Self::Css(_) | Self::XPath(_) | Self::Ref(_))
    }

    pub(crate) fn ref_ids(&self) -> Vec<&str> {
        match self {
            Self::Ref(id) => vec![id.as_str()],
            Self::Within { outer: a, inner: b } | Self::And(a, b) | Self::Or(a, b) => {
                let mut ids = a.ref_ids();
                ids.extend(b.ref_ids());
                ids
            }
            Self::Filtered { base, filter } => {
                let mut ids = base.ref_ids();
                for nested in [&filter.has, &filter.has_not].into_iter().flatten() {
                    ids.extend(nested.ref_ids());
                }
                ids
            }
            Self::Nth { base, .. } => base.ref_ids(),
            _ => Vec::new(),
        }
    }

    fn semantic(&self) -> Value {
        match self {
            Self::Css(css) => json!({ "kind": "css", "css": css }),
            Self::XPath(path) => json!({ "kind": "xpath", "path": path }),
            Self::Ref(id) => json!({ "kind": "ref", "id": id, "registry": REF_REGISTRY }),
            Self::Role { role, name } => json!({
                "kind": "role",
                "role": role,
                "name": name.as_ref().map(TextMatch::query),
            }),
            Self::Text(text) => json!({ "kind": "text", "match": text.query() }),
            Self::TestId(id) => json!({ "kind": "testid", "id": id }),
            Self::Label(text) => json!({ "kind": "label", "match": text.query() }),
            Self::Within { outer, inner } => {
                json!({ "kind": "within", "outer": outer.semantic(), "inner": inner.semantic() })
            }
            Self::Filtered { base, filter } => json!({
                "kind": "filter",
                "base": base.semantic(),
                "hasText": filter.has_text.as_ref().map(TextMatch::query),
                "hasNotText": filter.has_not_text.as_ref().map(TextMatch::query),
                "has": filter.has.as_ref().map(Self::semantic),
                "hasNot": filter.has_not.as_ref().map(Self::semantic),
                "visible": filter.visible,
            }),
            Self::And(a, b) => json!({ "kind": "and", "a": a.semantic(), "b": b.semantic() }),
            Self::Or(a, b) => json!({ "kind": "or", "a": a.semantic(), "b": b.semantic() }),
            Self::Nth { base, at } => match at {
                Position::Index(index) => {
                    json!({ "kind": "nth", "base": base.semantic(), "index": index })
                }
                Position::Last => json!({ "kind": "nth", "base": base.semantic(), "last": true }),
            },
        }
    }

    fn element_expr(&self, index: Option<usize>) -> String {
        match (self, index) {
            (Self::Ref(id), _) => ref_expr(id),
            (Self::XPath(path), None) => format!(
                "document.evaluate({},document,null,XPathResult.FIRST_ORDERED_NODE_TYPE,null).singleNodeValue",
                quote(path)
            ),
            (Self::XPath(path), Some(i)) => format!(
                "(function(){{var s=document.evaluate({},document,null,XPathResult.ORDERED_NODE_SNAPSHOT_TYPE,null);return s.snapshotItem({i})||null;}})()",
                quote(path)
            ),
            (Self::Css(css), None) => format!("document.querySelector({})", quote(css)),
            (Self::Css(css), Some(i)) => {
                format!("(document.querySelectorAll({})[{i}]||null)", quote(css))
            }
            _ => format!(
                "({RESOLVER_VAR}.find({})[{}]||null)",
                self.semantic(),
                index.unwrap_or(0)
            ),
        }
    }

    fn with_resolver(body: &str) -> String {
        format!(
            "(function(){{var {RESOLVER_VAR}=({RESOLVER})({MATCH_TEXT},{EDITABLE},{ENABLED});{body}}})()"
        )
    }

    pub(crate) fn one_expr(&self, index: Option<usize>) -> String {
        if self.leaf() {
            return self.element_expr(index);
        }
        Self::with_resolver(&format!("return {};", self.element_expr(index)))
    }

    pub(crate) fn count_expr(&self) -> String {
        match self {
            Self::XPath(path) => format!(
                "document.evaluate({},document,null,XPathResult.ORDERED_NODE_SNAPSHOT_TYPE,null).snapshotLength",
                quote(path)
            ),
            Self::Css(css) => format!("document.querySelectorAll({}).length", quote(css)),
            Self::Ref(id) => format!("({}?1:0)", ref_expr(id)),
            _ => Self::with_resolver(&format!(
                "return {RESOLVER_VAR}.find({}).length;",
                self.semantic()
            )),
        }
    }

    pub(crate) fn actionable_expr(
        &self,
        index: Option<usize>,
        frame_wait: Duration,
        need: Need,
    ) -> String {
        Self::with_resolver(&format!(
            "return {RESOLVER_VAR}.actionable({},{},{});",
            self.element_expr(index),
            frame_wait.as_millis(),
            quote(need.js())
        ))
    }

    pub(crate) fn settled_expr(
        &self,
        index: Option<usize>,
        frame_wait: Duration,
        need: Need,
    ) -> String {
        Self::with_resolver(&format!(
            "return {RESOLVER_VAR}.actionable({},{},{}).then(function(v){{return v.ok?JSON.stringify(v):false;}});",
            self.element_expr(index),
            frame_wait.as_millis(),
            quote(need.js())
        ))
    }
}

fn ref_expr(id: &str) -> String {
    format!(
        "(function(){{var m=globalThis.{REF_REGISTRY};var e=m&&m[{}];return e&&e.isConnected?e:null;}})()",
        quote(id)
    )
}

pub(crate) fn quote(text: &str) -> String {
    Value::String(text.to_owned()).to_string()
}

impl From<&str> for TextMatch {
    fn from(text: &str) -> Self {
        Self::Exact(text.to_owned())
    }
}

impl From<String> for TextMatch {
    fn from(text: String) -> Self {
        Self::Exact(text)
    }
}

impl From<&str> for Selector {
    fn from(raw: &str) -> Self {
        Self::parse(raw)
    }
}

impl From<String> for Selector {
    fn from(raw: String) -> Self {
        Self::parse(&raw)
    }
}

impl From<&String> for Selector {
    fn from(raw: &String) -> Self {
        Self::parse(raw)
    }
}
