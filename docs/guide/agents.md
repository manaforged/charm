# Agents

This page is for programs where a language model decides what to do in the browser. The model reads a text snapshot of the page, picks an element by its ref, and sends one action. charm runs the action and returns what changed.

## The loop

1. Call `snapshot(SnapshotOptions::default())` and give `Snapshot::text()` to the model.
2. The model answers with one action as JSON. Each element it can act on has a ref such as `e5`.
3. Parse the JSON into an `Action` and call `act(action, ActReport::Delta)`.
4. Give the model the `Outcome`: the new URL and title, new tabs, dialogs, and the snapshot delta.
5. Repeat from step 2. Take a full snapshot again when the delta is not enough.

```rust
use charm::{ActReport, Action, Launch, Session, SnapshotOptions};

let mut session = Session::launch(Launch::installed()?).await?;
session.goto("https://example.com/").await?;
let snapshot = session.snapshot(SnapshotOptions::default()).await?;
println!("{}", snapshot.text());

let call = r#"{"action":"click","target":"e3"}"#;
let action: Action = serde_json::from_str(call)?;
let outcome = session.act(action, ActReport::Delta).await?;
println!("{} -> {}", outcome.url_before, outcome.url_after);
if let Some(delta) = outcome.snapshot {
    println!("{delta}");
}
```

[`examples/agent_loop.rs`](https://github.com/manaforged/charm/blob/main/examples/agent_loop.rs) is a runnable version.

## The snapshot

`snapshot` reads Chrome's accessibility tree and prints one line per node. The first line names the tab, the title, and the URL:

```text
page 9A1F... "login" data:text/html,...
- heading "Sign in" [level=1] [ref=e2]
- form "Login" [ref=e3]
  - textbox "Email" [ref=e4]
  - textbox "Password" [ref=e5]
  - checkbox "Remember me" [unchecked] [ref=e6]
  - button "Sign in" [ref=e8]
- link "Forgot password?" [url=...] [ref=e9]
```

This sample is shortened. The exact roles and names come from Chrome.

Each line is `- role "name"`, then tags in square brackets, then `: "value"` for controls that have a value. The tags are `checked`, `unchecked`, `checked=mixed`, `disabled`, `expanded`, `collapsed`, `selected`, `level=N`, `url=...`, and `ref=eN`. Indentation shows nesting. Text that is not already part of the parent's name prints as a `text` line, without a ref. Iframes print as an `iframe` line with the frame's content below it. The value of a password field always prints as `•••`.

`Snapshot` also gives the parsed form: `nodes()` returns `&[SnapshotNode]` with `role`, `name`, `node_ref`, `checked`, `disabled`, `expanded`, `selected`, `level`, `value`, `url`, and `depth`. `page()`, `title()`, `url()`, `header()`, and `truncated()` read the rest.

## Snapshot options

| Field | Default | Effect |
| --- | --- | --- |
| `frames` | `true` | Includes the content of iframes, same-origin and cross-origin, under an `iframe` line. |
| `max_nodes` | `2000` | Stops after this many nodes and ends the text with `- truncated`. |
| `interesting_only` | `true` | Skips ignored and generic wrapper nodes, and keeps their children. |
| `interactive_only` | `false` | Keeps controls (buttons, links, inputs, options, tabs, menu items), headings, landmarks, forms, dialogs, and alerts. Drops text lines and other nodes. |
| `depth` | `None` | Stops below this nesting depth. |
| `scope` | `None` | Starts the tree at the element that matches this selector. |
| `max_chars` | `None` | Cuts the text at a line boundary after this many bytes and adds `- truncated`. |

To keep model input small:

- Start with `interactive_only: true` when the model only needs to find a control.
- Use `scope` with a selector or a ref to read one form, dialog, or list.
- Set `max_chars` so one large page cannot fill the context.
- After the first snapshot, send deltas from `act` instead of full snapshots.

```rust
use charm::{Selector, SnapshotOptions};

let options = SnapshotOptions {
    interactive_only: true,
    scope: Some(Selector::parse("form")),
    max_chars: Some(4000),
    ..SnapshotOptions::default()
};
let text = session.snapshot(options).await?.text();
```

## Refs

Every node line except `text` lines carries a ref. A ref names one element from a snapshot. It works anywhere a selector works:

```rust
use charm::Selector;

session.fill(Selector::parse("ref=e3"), "ada@example.com").await?;
session.click(Selector::Ref("e8".into())).await?;
```

`Selector::parse` turns `ref=e3` into `Selector::Ref`. `Selector::parse_target`, which `Action` uses for its `target` fields, also accepts `e3` and `@e3`.

The same element keeps the same ref in later snapshots of the same tab. Each snapshot replaces the set of valid refs for that tab. If a ref is not in the latest snapshot, or its element was removed, the call returns `Error::StaleRef(ref)`. Take a new snapshot and pick again.

Refs reach into iframes. A ref for a button in a same-origin or cross-origin frame clicks that button; the live tests cover both cases.

## Actions as JSON

`Action` deserializes from JSON, so a tool call from the model parses straight into it. The `action` field names the variant. Every `target` is a ref or a selector string.

| Action | JSON |
| --- | --- |
| Click | `{"action":"click","target":"e8"}` |
| Click with options | `{"action":"click","target":"e8","options":{"button":"right","count":2,"modifiers":["shift"],"position":[4,4]}}` |
| Fill | `{"action":"fill","target":"e3","text":"ada@example.com"}` |
| Type key by key | `{"action":"type","target":"e3","text":"abc"}` |
| Press a key | `{"action":"press","key":"Enter"}`, or with `"target":"e3"` to focus first |
| Select options | `{"action":"select","target":"e6","by":{"value":["pro"]}}`; `by` also takes `label` or `index` |
| Check or uncheck | `{"action":"check","target":"e5"}`, `{"action":"check","target":"e5","checked":false}` |
| Upload files | `{"action":"upload","target":"#file","files":["report.pdf"]}` |
| Hover | `{"action":"hover","target":"e9"}` |
| Drag | `{"action":"drag","from":"e10","to":"e11"}` |
| Scroll | `{"action":"scroll","delta_y":600}`, or with `"target"` to scroll over an element |
| Navigate | `{"action":"navigate","url":"https://example.com/"}` |
| History | `{"action":"back"}`, `{"action":"forward"}`, `{"action":"reload"}` |
| Wait | `{"action":"wait","for":{"selector":"e12"}}`; `for` also takes `text`, `url`, `url_exact`, `url_glob`, `load`, or `ms` |
| Tabs | `{"action":"tab","new":"https://example.com/"}`, `{"action":"tab","select":"<page id>"}`, `{"action":"tab","close":"<page id>"}` |

Details:

- `modifiers` takes `Alt`, `Control`, `Meta`, and `Shift`. `button` takes `left`, `right`, or `middle`.
- `wait` with `text` waits for an element whose text contains the string. `url` is a substring match. `load` takes `load`, `domcontentloaded`, or `networkidle`. `ms` is capped at `Timing::wait`.
- `check` without `checked` checks the box.

You can also build an `Action` in Rust. The variants are `#[non_exhaustive]`, so match with a `_` arm.

Only the `click` action through JSON is covered by a test. The other variants call the same `Session` methods that this guide documents.

## The outcome

`act` returns an `Outcome`:

| Field | Holds |
| --- | --- |
| `url_before`, `url_after` | The active tab's URL before and after the action. |
| `title` | The title after the action. |
| `new_pages` | Tabs that opened during the action. |
| `dialogs` | Dialogs answered since the last read, or `None` when page events are off. |
| `console` | Console messages and uncaught exceptions since the last read, or `None` when runtime events are off. |
| `snapshot` | The report you asked for with `ActReport`. |

`act` hands you the dialogs and console messages recorded since you last read them, so nothing recorded before the call is lost. `None` means the matching events are off, not that nothing happened. See [Events](events.md).

`ActReport` picks the report:

| Value | `Outcome::snapshot` |
| --- | --- |
| `ActReport::None` | `None`. No snapshot is taken. |
| `ActReport::Delta` | The lines that changed since the last snapshot of this tab. With no earlier snapshot, the full text. |
| `ActReport::Full` | The full snapshot text. |

A delta starts with the header and `(delta, unchanged N)`, then one line per change: `+` for a new node, `-` for a removed node, and `~` for a node whose line changed.

```text
page 4F2C... "t" data:text/html,... (delta, unchanged 2)
+ text "done"
```

`Snapshot::diff(&previous)` computes the same delta from two snapshots you keep yourself.

## Reading the page as text

To answer "what does this page say", read text instead of the tree. `text(TextFormat::Markdown, max_chars)` returns the main document as Markdown, with headings, lists, links, and tables. `TextFormat::Text` returns the rendered text. Both stop at `max_chars` characters. Neither includes iframe content or form control values.

```rust
use charm::TextFormat;

let markdown = session.text(TextFormat::Markdown, 10_000).await?;
```

## Errors the model can act on

| Error | Tell the model |
| --- | --- |
| `StaleRef(ref)` | Take a new snapshot. The ref is gone. |
| `NotActionable { reason, covered_by, .. }` | The element is hidden, disabled, not editable, or covered. `covered_by` names the element on top. |
| `Timeout(what)` | The wait ran out. |
| `Script { kind: ScriptError::Syntax, .. }` | The selector or regular expression is invalid. |

The `Display` text of each error is short and can go to the model as it is.
