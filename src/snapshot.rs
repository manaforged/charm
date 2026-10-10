use crate::pages::PageId;
use crate::selector::{Selector, quote};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

mod ax;
mod capture;
mod diff;
mod format;
mod walk;

const TRUNCATED: &str = "- truncated\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotOptions {
    pub frames: bool,
    pub max_nodes: usize,
    pub interesting_only: bool,
    pub interactive_only: bool,
    pub depth: Option<usize>,
    pub scope: Option<Selector>,
    pub max_chars: Option<usize>,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            frames: true,
            max_nodes: 2000,
            interesting_only: true,
            interactive_only: false,
            depth: None,
            scope: None,
            max_chars: None,
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
    page: PageId,
    title: String,
    url: String,
    nodes: Vec<SnapshotNode>,
    truncated: bool,
    max_chars: Option<usize>,
}

impl Snapshot {
    #[must_use]
    pub fn page(&self) -> &PageId {
        &self.page
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    #[must_use]
    pub fn nodes(&self) -> &[SnapshotNode] {
        &self.nodes
    }

    #[must_use]
    pub fn truncated(&self) -> bool {
        self.truncated
    }

    #[must_use]
    pub fn header(&self) -> String {
        format!("page {} {} {}", self.page, quote(&self.title), self.url)
    }

    #[must_use]
    pub fn text(&self) -> String {
        let mut body = String::new();
        for node in &self.nodes {
            body.push_str(&"  ".repeat(node.depth));
            body.push_str(&node.to_string());
            body.push('\n');
        }
        let mut cut = self.truncated;
        if let Some(max) = self.max_chars
            && body.len() > max
        {
            let edge = (0..=max)
                .rev()
                .find(|at| body.is_char_boundary(*at))
                .unwrap_or(0);
            let end = body
                .get(..edge)
                .and_then(|head| head.rfind('\n'))
                .map_or(0, |at| at + 1);
            body.truncate(end);
            cut = true;
        }
        let mut out = self.header();
        out.push('\n');
        out.push_str(&body);
        if cut {
            out.push_str(TRUNCATED);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RefTarget {
    pub(crate) backend: i64,
    pub(crate) session: Option<String>,
    pub(crate) frame: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct RefMap {
    targets: BTreeMap<NodeRef, RefTarget>,
    issued: u32,
    last: Option<Snapshot>,
}

impl RefMap {
    pub(crate) fn get(&self, node_ref: NodeRef) -> Option<&RefTarget> {
        self.targets.get(&node_ref)
    }

    pub(crate) fn last(&self) -> Option<&Snapshot> {
        self.last.as_ref()
    }

    fn issuer(&self) -> RefIssuer {
        RefIssuer {
            known: self
                .targets
                .iter()
                .map(|(node_ref, target)| (target.clone(), *node_ref))
                .collect(),
            issued: self.issued,
            targets: BTreeMap::new(),
        }
    }

    fn commit(&mut self, issuer: RefIssuer, snapshot: Snapshot) {
        self.targets = issuer.targets;
        self.issued = issuer.issued;
        self.last = Some(snapshot);
    }
}

struct RefIssuer {
    known: HashMap<RefTarget, NodeRef>,
    issued: u32,
    targets: BTreeMap<NodeRef, RefTarget>,
}

impl RefIssuer {
    fn issue(&mut self, target: RefTarget) -> NodeRef {
        let node_ref = match self.known.get(&target) {
            Some(node_ref) => *node_ref,
            None => {
                self.issued = self.issued.saturating_add(1);
                NodeRef(self.issued)
            }
        };
        self.targets.insert(node_ref, target);
        node_ref
    }
}
