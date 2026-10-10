use super::ax::{AxNode, Tree};
use super::{RefIssuer, RefTarget, SnapshotNode, SnapshotOptions};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

const ROOT_ROLE: &str = "RootWebArea";
const TEXT_ROLE: &str = "StaticText";
const IFRAME_ROLE: &str = "Iframe";
pub(super) const TEXT_LABEL: &str = "text";
pub(super) const FRAME_LABEL: &str = "frame";
const IFRAME_LABEL: &str = "iframe";
pub(super) const MASK: &str = "•••";
const SKIPPED: &[&str] = &[
    "generic",
    "none",
    "presentation",
    "InlineTextBox",
    "LineBreak",
    "Ignored",
    "IgnoredRole",
];
const INTERACTIVE: &[&str] = &[
    "button",
    "link",
    "textbox",
    "searchbox",
    "combobox",
    "listbox",
    "option",
    "checkbox",
    "radio",
    "switch",
    "slider",
    "spinbutton",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "tab",
    "treeitem",
    "heading",
    "banner",
    "main",
    "navigation",
    "contentinfo",
    "complementary",
    "form",
    "search",
    "region",
    "dialog",
    "alertdialog",
    "alert",
];

#[derive(Debug)]
pub(super) struct FrameTree {
    pub(super) name: String,
    pub(super) tree: Tree,
}

pub(super) type Secrets = HashSet<(Option<String>, i64)>;

pub(super) struct Walk<'a> {
    pub(super) options: &'a SnapshotOptions,
    pub(super) frames: &'a HashMap<i64, FrameTree>,
    pub(super) secrets: &'a Secrets,
    pub(super) issuer: RefIssuer,
    pub(super) nodes: Vec<SnapshotNode>,
    pub(super) truncated: bool,
}

impl Walk<'_> {
    pub(super) fn add(&mut self, tree: &Tree, root: &AxNode, depth: usize) {
        self.visit(tree, root, depth, "");
    }

    pub(super) fn full(&mut self) -> bool {
        if self.nodes.len() >= self.options.max_nodes {
            self.truncated = true;
        }
        self.truncated
    }

    pub(super) fn label(&mut self, depth: usize, role: &str, name: String) {
        self.nodes.push(blank(depth, role, name));
    }

    fn visit(&mut self, tree: &Tree, node: &AxNode, depth: usize, parent: &str) {
        if self.full() || self.options.depth.is_some_and(|max| depth > max) {
            return;
        }
        let role = node.role();
        let name = node.name();
        if role == TEXT_ROLE {
            if !self.options.interactive_only {
                self.push_text(depth, name, parent);
            }
            return;
        }
        if role == IFRAME_ROLE
            && tree.session.is_none()
            && self.visit_frame(tree, node, depth, &name)
        {
            return;
        }
        let transparent = self.transparent(role, node);
        let child_depth = if transparent {
            depth
        } else {
            self.push_node(tree, node, depth, role, name.clone());
            depth + 1
        };
        let label = if transparent { parent.to_owned() } else { name };
        for child in &node.child_ids {
            if let Some(next) = tree.child(child) {
                self.visit(tree, next, child_depth, &label);
            }
        }
    }

    fn visit_frame(&mut self, tree: &Tree, node: &AxNode, depth: usize, name: &str) -> bool {
        let frames = self.frames;
        let Some(frame) = node.backend.and_then(|backend| frames.get(&backend)) else {
            return false;
        };
        let label = if frame.name.is_empty() {
            name.to_owned()
        } else {
            frame.name.clone()
        };
        self.push_node(tree, node, depth, IFRAME_LABEL, label);
        if let Some(root) = frame.tree.root() {
            self.visit(&frame.tree, root, depth + 1, "");
        }
        true
    }

    fn transparent(&self, role: &str, node: &AxNode) -> bool {
        role == ROOT_ROLE
            || (self.options.interesting_only && (node.ignored || SKIPPED.contains(&role)))
            || (self.options.interactive_only && !INTERACTIVE.contains(&role))
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

    fn push_node(&mut self, tree: &Tree, node: &AxNode, depth: usize, role: &str, name: String) {
        let node_ref = node.backend.map(|backend| {
            self.issuer.issue(RefTarget {
                backend,
                session: tree.session.clone(),
                frame: tree.frame.clone(),
            })
        });
        let secret = node
            .backend
            .is_some_and(|backend| self.secrets.contains(&(tree.session.clone(), backend)));
        let value = node
            .value()
            .map(|value| if secret { MASK.to_owned() } else { value });
        self.nodes.push(SnapshotNode {
            node_ref,
            checked: node.checked(),
            disabled: node.flag("disabled"),
            expanded: node.property("expanded").and_then(Value::as_bool),
            selected: node.flag("selected"),
            level: node.property("level").and_then(Value::as_u64),
            value,
            url: node
                .property("url")
                .and_then(Value::as_str)
                .map(str::to_owned),
            ..blank(depth, role, name)
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
