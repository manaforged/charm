use crate::dom::RESOLVER;
use serde_json::{Value, json};

const XPATH_PREFIX: &str = "xpath=";
const RESOLVER_VAR: &str = "__c";

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    fn query(&self) -> Value {
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
    pub fn parse(raw: &str) -> Self {
        match raw.strip_prefix(XPATH_PREFIX) {
            Some(path) => Self::XPath(path.to_owned()),
            None => Self::Css(raw.to_owned()),
        }
    }

    fn semantic(&self) -> Value {
        match self {
            Self::Css(_) | Self::XPath(_) => Value::Null,
            Self::Role { role, name } => json!({
                "kind": "role",
                "role": role,
                "name": name.as_ref().map(TextMatch::query),
            }),
            Self::Text(text) => json!({ "kind": "text", "match": text.query() }),
            Self::TestId(id) => json!({ "kind": "testid", "id": id }),
            Self::Label(text) => json!({ "kind": "label", "match": text.query() }),
        }
    }

    fn element_expr(&self, index: Option<usize>) -> String {
        match (self, index) {
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
        format!("(function(){{var {RESOLVER_VAR}=({RESOLVER})();{body}}})()")
    }

    pub(crate) fn one_expr(&self, index: Option<usize>) -> String {
        match self {
            Self::Css(_) | Self::XPath(_) => self.element_expr(index),
            _ => Self::with_resolver(&format!("return {};", self.element_expr(index))),
        }
    }

    pub(crate) fn count_expr(&self) -> String {
        match self {
            Self::XPath(path) => format!(
                "document.evaluate({},document,null,XPathResult.ORDERED_NODE_SNAPSHOT_TYPE,null).snapshotLength",
                quote(path)
            ),
            Self::Css(css) => format!("document.querySelectorAll({}).length", quote(css)),
            _ => Self::with_resolver(&format!(
                "return {RESOLVER_VAR}.find({}).length;",
                self.semantic()
            )),
        }
    }

    pub(crate) fn actionable_expr(&self, index: Option<usize>) -> String {
        Self::with_resolver(&format!(
            "return {RESOLVER_VAR}.actionable({});",
            self.element_expr(index)
        ))
    }

    pub(crate) fn settled_expr(&self, index: Option<usize>) -> String {
        Self::with_resolver(&format!(
            "return {RESOLVER_VAR}.actionable({}).then(function(v){{return v.ok;}});",
            self.element_expr(index)
        ))
    }
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
