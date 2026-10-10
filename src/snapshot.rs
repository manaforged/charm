use crate::error::Error;
use crate::session::Session;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fmt;

mod format;

const ROOT_ROLE: &str = "RootWebArea";
const TEXT_ROLE: &str = "StaticText";
const TEXT_LABEL: &str = "text";
const FRAME_LABEL: &str = "frame";
const SKIPPED: &[&str] = &[
    "generic",
    "none",
    "presentation",
    "InlineTextBox",
    "LineBreak",
    "Ignored",
    "IgnoredRole",
];
const AX_TREE: &str = "Accessibility.getFullAXTree";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotOptions {
    pub frames: bool,
    pub max_nodes: usize,
    pub interesting_only: bool,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            frames: true,
            max_nodes: 2000,
            interesting_only: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeRef(u32);

impl NodeRef {
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        raw.strip_prefix('e')?
            .parse()
            .ok()
            .filter(|n| *n > 0)
            .map(Self)
    }
}

impl fmt::Display for NodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Checked {
    True,
    False,
    Mixed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SnapshotNode {
    pub depth: usize,
    pub role: String,
    pub name: String,
    pub node_ref: Option<NodeRef>,
    pub checked: Option<Checked>,
    pub disabled: bool,
    pub expanded: Option<bool>,
    pub selected: bool,
    pub level: Option<u64>,
    pub value: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    nodes: Vec<SnapshotNode>,
    truncated: bool,
}

impl Snapshot {
    #[must_use]
    pub fn nodes(&self) -> &[SnapshotNode] {
        &self.nodes
    }

    #[must_use]
    pub fn truncated(&self) -> bool {
        self.truncated
    }

    #[must_use]
    pub fn text(&self) -> String {
        let mut out = String::new();
        for node in &self.nodes {
            out.push_str(&"  ".repeat(node.depth));
            out.push_str(&node.to_string());
            out.push('\n');
        }
        if self.truncated {
            out.push_str("- truncated\n");
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RefTarget {
    pub(crate) backend: i64,
    pub(crate) session: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct RefMap {
    targets: Vec<RefTarget>,
}

impl RefMap {
    pub(crate) fn get(&self, node_ref: NodeRef) -> Option<&RefTarget> {
        let index = usize::try_from(node_ref.0).ok()?.checked_sub(1)?;
        self.targets.get(index)
    }

    fn replace(&mut self, targets: Vec<RefTarget>) {
        self.targets = targets;
    }
}

#[derive(Debug, Deserialize)]
struct AxTree {
    nodes: Vec<AxNode>,
}

#[derive(Debug, Deserialize)]
struct AxNode {
    #[serde(rename = "nodeId")]
    node_id: String,
    #[serde(default)]
    ignored: bool,
    role: Option<AxValue>,
    name: Option<AxValue>,
    value: Option<AxValue>,
    #[serde(default)]
    properties: Vec<AxProperty>,
    #[serde(default, rename = "childIds")]
    child_ids: Vec<String>,
    #[serde(rename = "backendDOMNodeId")]
    backend: Option<i64>,
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
    fn role(&self) -> &str {
        self.role
            .as_ref()
            .and_then(|role| role.value.as_str())
            .unwrap_or_default()
    }

    fn name(&self) -> String {
        self.name.as_ref().map(AxValue::text).unwrap_or_default()
    }

    fn property(&self, name: &str) -> Option<&Value> {
        self.properties
            .iter()
            .find(|property| property.name == name)
            .map(|property| &property.value.value)
    }

    fn flag(&self, name: &str) -> bool {
        matches!(self.property(name), Some(Value::Bool(true)))
            || self.property(name).and_then(Value::as_str) == Some("true")
    }

    fn checked(&self) -> Option<Checked> {
        self.property("checked").map(|raw| match raw.as_str() {
            Some("true") => Checked::True,
            Some("mixed") => Checked::Mixed,
            _ => Checked::False,
        })
    }
}

struct Walk {
    options: SnapshotOptions,
    session: Option<String>,
    nodes: Vec<SnapshotNode>,
    targets: Vec<RefTarget>,
    truncated: bool,
}

impl Walk {
    fn add(&mut self, tree: &AxTree, session: Option<String>, depth: usize) {
        self.session = session;
        let index: HashMap<&str, &AxNode> = tree
            .nodes
            .iter()
            .map(|node| (node.node_id.as_str(), node))
            .collect();
        if let Some(root) = tree.nodes.first() {
            self.visit(&index, root, depth, "");
        }
    }

    fn full(&mut self) -> bool {
        if self.nodes.len() >= self.options.max_nodes {
            self.truncated = true;
        }
        self.truncated
    }

    fn visit(&mut self, index: &HashMap<&str, &AxNode>, node: &AxNode, depth: usize, parent: &str) {
        if self.full() {
            return;
        }
        let role = node.role();
        let name = node.name();
        if role == TEXT_ROLE {
            self.push_text(depth, name, parent);
            return;
        }
        let transparent = role == ROOT_ROLE
            || (self.options.interesting_only && (node.ignored || SKIPPED.contains(&role)));
        let child_depth = if transparent {
            depth
        } else {
            self.push_node(node, depth, &name);
            depth + 1
        };
        let label = if transparent { parent.to_owned() } else { name };
        for child in &node.child_ids {
            if let Some(next) = index.get(child.as_str()) {
                self.visit(index, next, child_depth, &label);
            }
        }
    }

    fn push_text(&mut self, depth: usize, name: String, parent: &str) {
        if name.is_empty() || parent.contains(&name) {
            return;
        }
        if let Some(last) = self.nodes.last_mut()
            && last.depth == depth
            && last.role == TEXT_LABEL
        {
            last.name.push(' ');
            last.name.push_str(&name);
            return;
        }
        self.nodes.push(blank(depth, TEXT_LABEL, name));
    }

    fn push_node(&mut self, node: &AxNode, depth: usize, name: &str) {
        let node_ref = node.backend.map(|backend| {
            self.targets.push(RefTarget {
                backend,
                session: self.session.clone(),
            });
            NodeRef(u32::try_from(self.targets.len()).unwrap_or(u32::MAX))
        });
        let level = node.property("level").and_then(Value::as_u64);
        let url = node
            .property("url")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let value = node
            .value
            .as_ref()
            .map(AxValue::text)
            .filter(|value| !value.is_empty());
        self.nodes.push(SnapshotNode {
            node_ref,
            checked: node.checked(),
            disabled: node.flag("disabled"),
            expanded: node.property("expanded").and_then(Value::as_bool),
            selected: node.flag("selected"),
            level,
            value,
            url,
            ..blank(depth, node.role(), name.to_owned())
        });
    }
}

fn blank(depth: usize, role: &str, name: String) -> SnapshotNode {
    SnapshotNode {
        depth,
        role: role.to_owned(),
        name,
        ..SnapshotNode::default()
    }
}

impl Session {
    pub async fn snapshot(&mut self, options: SnapshotOptions) -> Result<Snapshot, Error> {
        let mut walk = Walk {
            options,
            session: None,
            nodes: Vec::new(),
            targets: Vec::new(),
            truncated: false,
        };
        let page: AxTree = serde_json::from_value(self.send(AX_TREE, json!({})).await?)?;
        walk.add(&page, None, 0);
        if options.frames {
            let sessions: Vec<String> = self.frame_sessions().values().cloned().collect();
            for session in sessions {
                let tree = match self.send_on(&session, AX_TREE, json!({})).await {
                    Ok(tree) => tree,
                    Err(Error::Protocol { .. }) => continue,
                    Err(error) => return Err(error),
                };
                if walk.full() {
                    break;
                }
                walk.nodes.push(blank(0, FRAME_LABEL, String::new()));
                walk.add(&serde_json::from_value(tree)?, Some(session), 1);
            }
        }
        self.refs().replace(walk.targets);
        Ok(Snapshot {
            nodes: walk.nodes,
            truncated: walk.truncated,
        })
    }
}
