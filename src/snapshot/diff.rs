use super::{NodeRef, Snapshot, SnapshotNode};
use std::collections::{HashMap, HashSet};

const ADDED: char = '+';
const REMOVED: char = '-';
const CHANGED: char = '~';

fn push(out: &mut String, mark: char, line: &str) {
    out.push(mark);
    out.push(' ');
    out.push_str(line.strip_prefix("- ").unwrap_or(line));
    out.push('\n');
}

struct Delta {
    before: HashMap<NodeRef, String>,
    loose: HashMap<String, usize>,
    seen: HashSet<NodeRef>,
    body: String,
    unchanged: usize,
}

impl Delta {
    fn new(previous: &Snapshot) -> Self {
        let before = previous
            .nodes
            .iter()
            .filter_map(|node| node.node_ref.map(|node_ref| (node_ref, node.to_string())))
            .collect();
        let mut loose: HashMap<String, usize> = HashMap::new();
        for node in previous.nodes.iter().filter(|node| node.node_ref.is_none()) {
            *loose.entry(node.to_string()).or_default() += 1;
        }
        Self {
            before,
            loose,
            seen: HashSet::new(),
            body: String::new(),
            unchanged: 0,
        }
    }

    fn current(&mut self, node: &SnapshotNode) {
        let line = node.to_string();
        let Some(node_ref) = node.node_ref else {
            if self.take_loose(&line) {
                self.unchanged += 1;
            } else {
                push(&mut self.body, ADDED, &line);
            }
            return;
        };
        self.seen.insert(node_ref);
        match self.before.get(&node_ref) {
            Some(old) if *old == line => self.unchanged += 1,
            Some(_) => push(&mut self.body, CHANGED, &line),
            None => push(&mut self.body, ADDED, &line),
        }
    }

    fn vanished(&mut self, node: &SnapshotNode) {
        let line = node.to_string();
        let gone = match node.node_ref {
            Some(node_ref) => !self.seen.contains(&node_ref),
            None => self.take_loose(&line),
        };
        if gone {
            push(&mut self.body, REMOVED, &line);
        }
    }

    fn take_loose(&mut self, line: &str) -> bool {
        match self.loose.get_mut(line) {
            Some(count) if *count > 0 => {
                *count -= 1;
                true
            }
            _ => false,
        }
    }
}

impl Snapshot {
    #[must_use]
    pub fn diff(&self, previous: &Snapshot) -> String {
        let mut delta = Delta::new(previous);
        for node in &self.nodes {
            delta.current(node);
        }
        for node in &previous.nodes {
            delta.vanished(node);
        }
        format!(
            "{} (delta, unchanged {})\n{}",
            self.header(),
            delta.unchanged,
            delta.body
        )
    }
}
