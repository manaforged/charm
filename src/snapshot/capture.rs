use super::ax::Tree;
use super::walk::{FRAME_LABEL, FrameTree, Secrets, Walk};
use super::{NodeRef, RefTarget, Snapshot, SnapshotOptions};
use crate::error::Error;
use crate::selector::Selector;
use crate::session::{Session, World};
use serde_json::{Value, json};
use std::collections::HashMap;

const AX_TREE: &str = "Accessibility.getFullAXTree";
const DESCRIBE: &str = "DOM.describeNode";
const FRAME_OWNER: &str = "DOM.getFrameOwner";
const PASSWORD: &str = "password";

fn missing(method: &str, message: &str) -> Error {
    Error::Protocol {
        method: method.into(),
        code: None,
        message: message.into(),
    }
}

impl Session {
    async fn on(
        &mut self,
        session: Option<&str>,
        method: &str,
        params: Value,
    ) -> Result<Option<Value>, Error> {
        let target = session.map_or_else(|| self.session_id().to_owned(), str::to_owned);
        match self.send_on(&target, method, params).await {
            Ok(value) => Ok(Some(value)),
            Err(Error::Protocol { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub async fn snapshot(&mut self, options: SnapshotOptions) -> Result<Snapshot, Error> {
        let main = Tree::new(self.send(AX_TREE, json!({})).await?, None, None)?;
        let (frames, remote) = self.frame_trees(options.frames).await?;
        let local = std::iter::once(&main).chain(frames.values().map(|frame| &frame.tree));
        let trees: Vec<&Tree> = local.chain(remote.iter()).collect();
        let secrets = self.secrets(&trees).await?;
        let scope = self.scope(options.scope.as_ref()).await?;
        let mut walk = Walk {
            options: &options,
            frames: &frames,
            secrets: &secrets,
            issuer: self.refs().issuer(),
            nodes: Vec::new(),
            truncated: false,
        };
        match scope {
            Some(target) => walk_scope(&mut walk, &trees, &target)?,
            None => walk_page(&mut walk, &main, &remote),
        }
        let Walk {
            issuer,
            nodes,
            truncated,
            ..
        } = walk;
        let snapshot = Snapshot {
            page: self.active_page().clone(),
            title: self.title().await?,
            url: self.url().await?,
            nodes,
            truncated,
            max_chars: options.max_chars,
        };
        self.refs().commit(issuer, snapshot.clone());
        Ok(snapshot)
    }

    async fn frame_trees(
        &mut self,
        enabled: bool,
    ) -> Result<(HashMap<i64, FrameTree>, Vec<Tree>), Error> {
        if !enabled {
            return Ok((HashMap::new(), Vec::new()));
        }
        Ok((
            self.same_origin_frames().await?,
            self.remote_frames().await?,
        ))
    }

    async fn scope(&mut self, selector: Option<&Selector>) -> Result<Option<RefTarget>, Error> {
        match selector {
            Some(selector) => Ok(Some(self.scope_target(selector).await?)),
            None => Ok(None),
        }
    }

    async fn send_or_skip(
        &mut self,
        session: Option<&str>,
        method: &str,
        params: Value,
    ) -> Result<Option<Value>, Error> {
        let target = session.map_or_else(|| self.session_id().to_owned(), str::to_owned);
        match self.send_on(&target, method, params).await {
            Ok(reply) => Ok(Some(reply)),
            Err(Error::Protocol { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    async fn same_origin_frames(&mut self) -> Result<HashMap<i64, FrameTree>, Error> {
        let mut out = HashMap::new();
        for frame in self.frames().await? {
            if frame.parent_id.is_none() {
                continue;
            }
            let Some(raw) = self
                .on(None, AX_TREE, json!({ "frameId": frame.id }))
                .await?
            else {
                continue;
            };
            let owner = self
                .on(None, FRAME_OWNER, json!({ "frameId": frame.id }))
                .await?;
            let Some(backend) = owner
                .as_ref()
                .and_then(|owner| owner.get("backendNodeId"))
                .and_then(Value::as_i64)
            else {
                continue;
            };
            let tree = Tree::new(raw, None, Some(frame.id))?;
            let name = frame.name.unwrap_or_default();
            out.insert(backend, FrameTree { name, tree });
        }
        Ok(out)
    }

    async fn remote_frames(&mut self) -> Result<Vec<Tree>, Error> {
        let sessions: Vec<String> = self.frame_sessions().values().cloned().collect();
        let mut out = Vec::new();
        for session in sessions {
            if let Some(raw) = self
                .send_or_skip(Some(&session), AX_TREE, json!({}))
                .await?
            {
                out.push(Tree::new(raw, Some(session), None)?);
            }
        }
        Ok(out)
    }

    async fn secrets(&mut self, trees: &[&Tree]) -> Result<Secrets, Error> {
        let filled: Vec<(Option<String>, i64)> = trees
            .iter()
            .flat_map(|tree| {
                tree.filled_textboxes()
                    .map(move |backend| (tree.session.clone(), backend))
            })
            .collect();
        let mut out = Secrets::new();
        for (session, backend) in filled {
            let described = self
                .on(
                    session.as_deref(),
                    DESCRIBE,
                    json!({ "backendNodeId": backend }),
                )
                .await?;
            if described.as_ref().is_none_or(is_password) {
                out.insert((session, backend));
            }
        }
        Ok(out)
    }

    async fn scope_target(&mut self, selector: &Selector) -> Result<RefTarget, Error> {
        if let Selector::Ref(raw) = selector {
            return NodeRef::parse(raw)
                .and_then(|node_ref| self.refs().get(node_ref).cloned())
                .ok_or_else(|| Error::StaleRef(raw.clone()));
        }
        let object = self
            .evaluate_handle_in(None, World::Isolated, &selector.one_expr(None))
            .await?;
        let described = self.send(DESCRIBE, json!({ "objectId": object })).await;
        drop(
            self.send("Runtime.releaseObject", json!({ "objectId": object }))
                .await,
        );
        let backend = described?
            .pointer("/node/backendNodeId")
            .and_then(Value::as_i64)
            .ok_or_else(|| missing(DESCRIBE, "missing backendNodeId"))?;
        Ok(RefTarget {
            backend,
            session: None,
            frame: None,
        })
    }
}

fn walk_scope(walk: &mut Walk<'_>, trees: &[&Tree], target: &RefTarget) -> Result<(), Error> {
    let found = trees
        .iter()
        .filter(|tree| tree.session == target.session)
        .find_map(|tree| tree.by_backend(target.backend).map(|node| (*tree, node)));
    let (tree, node) = found.ok_or_else(|| missing(AX_TREE, "scope element is not in the tree"))?;
    walk.add(tree, node, 0);
    Ok(())
}

fn walk_page(walk: &mut Walk<'_>, main: &Tree, remote: &[Tree]) {
    if let Some(root) = main.root() {
        walk.add(main, root, 0);
    }
    for tree in remote {
        if walk.full() {
            break;
        }
        walk.label(0, FRAME_LABEL, String::new());
        if let Some(root) = tree.root() {
            walk.add(tree, root, 1);
        }
    }
}

fn is_password(described: &Value) -> bool {
    described
        .pointer("/node/attributes")
        .and_then(Value::as_array)
        .is_some_and(|pairs| {
            pairs.chunks(2).any(|pair| {
                pair.first().and_then(Value::as_str) == Some("type")
                    && pair
                        .get(1)
                        .and_then(Value::as_str)
                        .is_some_and(|kind| kind.eq_ignore_ascii_case(PASSWORD))
            })
        })
}
