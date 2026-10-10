use super::Locator;
use crate::error::Error;
use crate::selector::{MATCH_TEXT, TextMatch};
use crate::snapshot::{Checked, SnapshotNode, SnapshotOptions};
use serde_json::{Map, Value, json};

impl Locator<'_> {
    pub(crate) async fn aria_tree(&mut self) -> Result<Vec<SnapshotNode>, Error> {
        let element = self.element_expr();
        let target = self.backend_of(&element).await?;
        let snapshot = self.session.snapshot(SnapshotOptions::default()).await?;
        let nodes = snapshot.nodes();
        let refs = self.session.refs();
        let root = nodes.iter().position(|node| {
            node.node_ref
                .and_then(|node_ref| refs.get(node_ref))
                .is_some_and(|found| *found == target)
        });
        let Some(start) = root.and_then(|index| nodes.get(index..)) else {
            return Ok(nodes.to_vec());
        };
        let base = start.first().map_or(0, |node| node.depth);
        Ok(start
            .iter()
            .enumerate()
            .take_while(|(index, node)| *index == 0 || node.depth > base)
            .map(|(_, node)| {
                let mut node = node.clone();
                node.depth = node.depth.saturating_sub(base);
                node
            })
            .collect())
    }

    pub(crate) async fn pause(&mut self) -> Result<(), Error> {
        let poll = self.session.timing().poll;
        self.session.idle(poll).await
    }
}

const TEXT_ROLE: &str = "text";
const CHILDREN: &str = "/children:";

pub(crate) const ARIA_MATCH: &str = r"(function(matchText,tpl,roots){
  function same(t,a,deep){
    if(t.role!==a.role)return false;
    if(t.name&&!matchText(a.name,t.name,false))return false;
    if(t.value&&!matchText(a.value==null?'':a.value,t.value,false))return false;
    for(var k in t.attrs){if(a[k]!==t.attrs[k])return false;}
    return kids(t,a,deep);
  }
  function kids(t,a,deep){
    var mode=deep?'deep':t.mode;
    if(mode==='contain'){var i=0;for(var j=0;j<a.children.length&&i<t.children.length;j++){if(same(t.children[i],a.children[j],false))i++;}return i===t.children.length;}
    if(a.children.length!==t.children.length)return false;
    for(var k=0;k<t.children.length;k++){if(!same(t.children[k],a.children[k],mode==='deep'))return false;}
    return true;
  }
  function any(a){if(kids(tpl,a,false))return true;for(var i=0;i<a.children.length;i++){if(any(a.children[i]))return true;}return false;}
  return any({children:roots});
})";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Children {
    Contain,
    Equal,
    DeepEqual,
}

impl Children {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "contain" => Some(Self::Contain),
            "equal" => Some(Self::Equal),
            "deep-equal" => Some(Self::DeepEqual),
            _ => None,
        }
    }

    fn js(self) -> &'static str {
        match self {
            Self::Contain => "contain",
            Self::Equal => "equal",
            Self::DeepEqual => "deep",
        }
    }
}

fn invalid(line: &str) -> Error {
    Error::Assertion {
        expected: "aria snapshot template line `- role \"name\" [attr]`".to_owned(),
        received: line.to_owned(),
        selector: None,
    }
}

fn closing(rest: &str, delimiter: char) -> Option<usize> {
    let mut escaped = false;
    for (at, ch) in rest.char_indices().skip(1) {
        match ch {
            '\\' if !escaped => escaped = true,
            ch if ch == delimiter && !escaped => return Some(at),
            _ => escaped = false,
        }
    }
    None
}

fn quoted(rest: &str) -> Option<(TextMatch, String)> {
    let end = closing(rest, '"')?;
    let (quoted, tail) = rest.split_at(end + 1);
    let text = serde_json::from_str::<String>(quoted).ok()?;
    Some((TextMatch::Exact(text), tail.to_owned()))
}

fn pattern(rest: &str) -> Option<(TextMatch, String)> {
    let end = closing(rest, '/')?;
    let pattern = rest.get(1..end)?;
    let tail = rest.get(end + 1..)?;
    let flags: String = tail.chars().take_while(char::is_ascii_alphabetic).collect();
    let tail = tail.get(flags.len()..)?.to_owned();
    Some((TextMatch::regex(pattern, &flags), tail))
}

fn text_match(rest: &str, line: &str) -> Result<(Option<TextMatch>, String), Error> {
    let parsed = match rest.chars().next() {
        Some('"') => quoted(rest),
        Some('/') => pattern(rest),
        _ => return Ok((None, rest.to_owned())),
    };
    let (found, tail) = parsed.ok_or_else(|| invalid(line))?;
    Ok((Some(found), tail))
}

const FLAGS: [&str; 3] = ["disabled", "selected", "expanded"];

fn flag(value: Option<&str>) -> Option<Value> {
    match value {
        None | Some("true") => Some(Value::Bool(true)),
        Some("false") => Some(Value::Bool(false)),
        Some(_) => None,
    }
}

fn checked_state(value: Option<&str>) -> Option<Value> {
    match value {
        None | Some("true") => Some(json!("true")),
        Some(state @ ("false" | "mixed")) => Some(json!(state)),
        Some(_) => None,
    }
}

fn attr_pair(key: &str, value: Option<&str>) -> Option<(&'static str, Value)> {
    if let Some(name) = FLAGS.into_iter().find(|name| *name == key) {
        return Some((name, flag(value)?));
    }
    match (key, value) {
        ("checked", value) => Some(("checked", checked_state(value)?)),
        ("unchecked", None) => Some(("checked", json!("false"))),
        ("collapsed", None) => Some(("expanded", Value::Bool(false))),
        ("level", Some(level)) => Some(("level", json!(level.parse::<u64>().ok()?))),
        ("url", Some(url)) => Some(("url", json!(url))),
        _ => None,
    }
}

fn attrs(mut rest: String, line: &str) -> Result<(Map<String, Value>, String), Error> {
    let mut attrs = Map::new();
    while let Some(open) = rest.trim_start().strip_prefix('[') {
        let (raw, tail) = open.split_once(']').ok_or_else(|| invalid(line))?;
        let (key, value) = match raw.split_once('=') {
            Some((key, value)) => (key.trim(), Some(value.trim())),
            None => (raw.trim(), None),
        };
        if key != "ref" {
            let (key, value) = attr_pair(key, value).ok_or_else(|| invalid(line))?;
            attrs.insert(key.to_owned(), value);
        }
        rest = tail.to_owned();
    }
    Ok((attrs, rest))
}

fn value(rest: &str, line: &str) -> Result<Option<TextMatch>, Error> {
    let Some(tail) = rest.trim().strip_prefix(':').map(str::trim) else {
        return Ok(None);
    };
    if tail.is_empty() {
        return Ok(None);
    }
    let (found, literal) = text_match(tail, line)?;
    Ok(Some(found.unwrap_or(TextMatch::Exact(literal))))
}

fn text_name(name: Option<TextMatch>, value: Option<TextMatch>) -> Option<TextMatch> {
    name.or(value).map(|text| match text {
        TextMatch::Exact(text) => TextMatch::Contains(text),
        other => other,
    })
}

fn node(line: &str) -> Result<Value, Error> {
    let body = line
        .strip_prefix('-')
        .ok_or_else(|| invalid(line))?
        .trim_start();
    let split = body
        .find(|ch: char| ch.is_whitespace() || ch == '[' || ch == ':')
        .unwrap_or(body.len());
    let (role, rest) = body.split_at(split);
    let (name, rest) = text_match(rest.trim_start(), line)?;
    let (attrs, rest) = attrs(rest, line)?;
    let value = value(&rest, line)?;
    let (name, value) = if role == TEXT_ROLE {
        (text_name(name, value), None)
    } else {
        (name, value)
    };
    Ok(json!({
        "role": role,
        "name": name.as_ref().map(TextMatch::query),
        "value": value.as_ref().map(TextMatch::query),
        "attrs": attrs,
        "mode": Children::Contain.js(),
        "children": [],
    }))
}

fn children_mode(line: &str) -> Option<&str> {
    line.strip_prefix('-')?
        .trim_start()
        .strip_prefix(CHILDREN)
        .map(str::trim)
}

fn block(
    lines: &[(usize, &str)],
    at: &mut usize,
    indent: usize,
) -> Result<(Vec<Value>, Children), Error> {
    let mut out = Vec::new();
    let mut mode = Children::Contain;
    while let Some(&(depth, line)) = lines.get(*at) {
        if depth < indent {
            break;
        }
        *at += 1;
        if let Some(raw) = children_mode(line) {
            mode = Children::parse(raw).ok_or_else(|| invalid(line))?;
            continue;
        }
        let mut parsed = node(line)?;
        if let Some(&(next, _)) = lines.get(*at).filter(|(next, _)| *next > depth) {
            let (children, child_mode) = block(lines, at, next)?;
            parsed["children"] = Value::Array(children);
            parsed["mode"] = json!(child_mode.js());
        }
        out.push(parsed);
    }
    Ok((out, mode))
}

pub(crate) fn aria_template(text: &str) -> Result<Value, Error> {
    let lines: Vec<(usize, &str)> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| (line.len() - line.trim_start().len(), line.trim()))
        .collect();
    let (children, mode) = block(&lines, &mut 0, 0)?;
    Ok(json!({ "role": "", "attrs": {}, "mode": mode.js(), "children": children }))
}

fn checked(state: Checked) -> &'static str {
    match state {
        Checked::True => "true",
        Checked::False => "false",
        Checked::Mixed => "mixed",
    }
}

fn forest(nodes: &[SnapshotNode], at: &mut usize, depth: usize) -> Vec<Value> {
    let mut out = Vec::new();
    while let Some(node) = nodes.get(*at) {
        if node.depth < depth {
            break;
        }
        *at += 1;
        let children = forest(nodes, at, node.depth + 1);
        out.push(json!({
            "role": node.role,
            "name": node.name,
            "value": node.value,
            "checked": node.checked.map(checked),
            "disabled": node.disabled,
            "selected": node.selected,
            "expanded": node.expanded,
            "level": node.level,
            "url": node.url,
            "children": children,
        }));
    }
    out
}

pub(crate) fn aria_probe(template: &Value, nodes: &[SnapshotNode]) -> String {
    let roots = Value::Array(forest(nodes, &mut 0, 0));
    format!("({ARIA_MATCH})({MATCH_TEXT},{template},{roots})")
}

pub(crate) fn aria_text(nodes: &[SnapshotNode]) -> String {
    nodes
        .iter()
        .map(|node| format!("{}{node}\n", "  ".repeat(node.depth)))
        .collect()
}
