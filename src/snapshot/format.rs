use super::{Checked, SnapshotNode};
use crate::selector::quote;
use std::fmt;

impl SnapshotNode {
    fn tags(&self) -> impl Iterator<Item = String> {
        [
            self.checked.map(|checked| checked.tag().to_owned()),
            self.disabled.then(|| "disabled".to_owned()),
            self.expanded.map(expanded_tag),
            self.selected.then(|| "selected".to_owned()),
            self.level.map(|level| format!("level={level}")),
            self.url.as_ref().map(|url| format!("url={url}")),
            self.node_ref.map(|node_ref| format!("ref={node_ref}")),
        ]
        .into_iter()
        .flatten()
    }
}

fn expanded_tag(expanded: bool) -> String {
    let tag = if expanded { "expanded" } else { "collapsed" };
    tag.to_owned()
}

impl Checked {
    fn tag(self) -> &'static str {
        match self {
            Self::True => "checked",
            Self::Mixed => "checked=mixed",
            Self::False => "unchecked",
        }
    }
}

impl fmt::Display for SnapshotNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "- {}", self.role)?;
        if !self.name.is_empty() {
            write!(f, " {}", quote(&self.name))?;
        }
        for tag in self.tags() {
            write!(f, " [{tag}]")?;
        }
        if let Some(value) = &self.value {
            write!(f, ": {}", quote(value))?;
        }
        Ok(())
    }
}
