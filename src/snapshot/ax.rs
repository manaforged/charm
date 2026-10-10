use super::Checked;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub(super) struct AxTree {
    pub(super) nodes: Vec<AxNode>,
}

#[derive(Debug, Deserialize)]
pub(super) struct AxNode {
    #[serde(rename = "nodeId")]
    node_id: String,
    #[serde(default)]
    pub(super) ignored: bool,
    role: Option<AxValue>,
    name: Option<AxValue>,
    value: Option<AxValue>,
    #[serde(default)]
    properties: Vec<AxProperty>,
    #[serde(default, rename = "childIds")]
    pub(super) child_ids: Vec<String>,
    #[serde(rename = "backendDOMNodeId")]
    pub(super) backend: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct AxValue {
    #[serde(default)]
    value: Value,
}

#[derive(Debug, Deserialize)]
struct AxProperty {
    name: String,
    value: AxValue,
}

impl AxValue {
    fn text(&self) -> String {
        match &self.value {
            Value::String(text) => text.split_whitespace().collect::<Vec<_>>().join(" "),
            Value::Null => String::new(),
            other => other.to_string(),
        }
    }
}

impl AxNode {
    pub(super) fn role(&self) -> &str {
        self.role
            .as_ref()
            .and_then(|role| role.value.as_str())
            .unwrap_or_default()
    }

    pub(super) fn name(&self) -> String {
        self.name.as_ref().map(AxValue::text).unwrap_or_default()
    }

    pub(super) fn value(&self) -> Option<String> {
        self.value
            .as_ref()
            .map(AxValue::text)
            .filter(|value| !value.is_empty())
    }

    pub(super) fn property(&self, name: &str) -> Option<&Value> {
        self.properties
            .iter()
            .find(|property| property.name == name)
            .map(|property| &property.value.value)
    }

    pub(super) fn flag(&self, name: &str) -> bool {
        matches!(self.property(name), Some(Value::Bool(true)))
            || self.property(name).and_then(Value::as_str) == Some("true")
    }

    pub(super) fn checked(&self) -> Option<Checked> {
        self.property("checked").map(|raw| match raw.as_str() {
            Some("true") => Checked::True,
            Some("mixed") => Checked::Mixed,
            _ => Checked::False,
        })
    }
}

#[derive(Debug)]
pub(super) struct Tree {
    nodes: Vec<AxNode>,
    index: HashMap<String, usize>,
    pub(super) session: Option<String>,
    pub(super) frame: Option<String>,
}

impl Tree {
    pub(super) fn new(
        raw: Value,
        session: Option<String>,
        frame: Option<String>,
    ) -> Result<Self, serde_json::Error> {
        let parsed: AxTree = serde_json::from_value(raw)?;
        let index = parsed
            .nodes
            .iter()
            .enumerate()
            .map(|(at, node)| (node.node_id.clone(), at))
            .collect();
        Ok(Self {
            nodes: parsed.nodes,
            index,
            session,
            frame,
        })
    }

    pub(super) fn root(&self) -> Option<&AxNode> {
        self.nodes.first()
    }

    pub(super) fn child(&self, id: &str) -> Option<&AxNode> {
        self.index.get(id).and_then(|at| self.nodes.get(*at))
    }

    pub(super) fn by_backend(&self, backend: i64) -> Option<&AxNode> {
        self.nodes.iter().find(|node| node.backend == Some(backend))
    }

    pub(super) fn filled_textboxes(&self) -> impl Iterator<Item = i64> + '_ {
        self.nodes
            .iter()
            .filter(|node| node.role() == "textbox" && node.value().is_some())
            .filter_map(|node| node.backend)
    }
}
