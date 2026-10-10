# charm-cdp for coding agents

Point your coding agent at this file when it writes Rust code that drives Chrome with charm-cdp. Every symbol below exists in the crate. The full list of public items is in [`api/charm-cdp.txt`](api/charm-cdp.txt).

## Install

The crate is not published on crates.io yet. Add it as a git dependency pinned to a commit, plus tokio:

```toml
[dependencies]
charm-cdp = { git = "https://github.com/manaforged/charm", rev = "<commit>" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The library name is `charm`. Import with `use charm::{...}`. Every browser call is `async` and returns `Result<_, charm::Error>`.

The host needs an installed Chrome or Chromium. `Launch::installed()` finds it, or reads `CHROME_EXE`. On Linux hosts where Chrome's sandbox cannot start, `Session::launch` returns `Error::Exited`; add `Launch::arg("--no-sandbox")`.

## The agent loop

If a language model drives the browser, use snapshots and refs instead of writing selectors:

1. `session.snapshot(SnapshotOptions::default()).await?.text()` returns the accessibility tree, one line per node, with a ref such as `[ref=e5]` on each element.
2. The model answers with one action as JSON, for example `{"action":"click","target":"e5"}`.
3. `serde_json::from_str::<Action>(&json)?` parses it, and `session.act(action, ActReport::Delta).await?` runs it.
4. The returned `Outcome` has `url_before`, `url_after`, `title`, `new_pages`, `dialogs`, `console`, and `snapshot`, which holds only the lines that changed.

```rust
use charm::{ActReport, Action, Launch, Session, SnapshotOptions};

let mut session = Session::launch(Launch::installed()?).await?;
session.goto("https://example.com/").await?;
let view = session.snapshot(SnapshotOptions { interactive_only: true, ..SnapshotOptions::default() }).await?.text();
let action: Action = serde_json::from_str(r#"{"action":"fill","target":"e3","text":"ada@example.com"}"#)?;
let outcome = session.act(action, ActReport::Delta).await?;
```

A ref works anywhere a selector does: `session.click(Selector::parse("ref=e5"))`. `Error::StaleRef` means the ref is not in the latest snapshot of the active tab; take a new snapshot. The full JSON shape of `Action` is in [docs/guide/agents.md](docs/guide/agents.md).

## Quick path

```rust
use charm::{Launch, Session, expect_page};

#[tokio::main]
async fn main() -> Result<(), charm::Error> {
    let mut session = Session::launch(Launch::installed()?).await?;
    session.goto("https://example.com/").await?;
    expect_page(&mut session).to_have_title("Example Domain").await?;
    session.close().await
}
```

## Core types

| Symbol | What it is |
| --- | --- |
| `Session` | One browser and its tabs. Page calls go to the active tab. Owns the DevTools connection, and the process when it launched it. |
| `Launch` | Builder for the browser command line: profile, headed, proxy, timing, events, storage state, extra args. |
| `Timing` | Every timeout and poll interval. `Timing::default()` holds the defaults. |
| `Locator<'_>` | A lazy reference to an element. Borrows the session mutably. |
| `Selector` | `Css`, `XPath`, `Role`, `Text`, `TestId`, `Label`, `Ref`, and the chained forms `Within`, `Filtered`, `And`, `Or`, `Nth`. A `&str` converts to `Css`, to `XPath` with an `xpath=` prefix, or to `Ref` with a `ref=` prefix. |
| `TextMatch` | `Exact`, `Contains`, or `Regex { pattern, flags }`. A `&str` converts to `Exact`. |
| `FrameLocator<'_>` | A path through one or more iframes. Ends in `.locator(...)`. |
| `UrlMatch` | `Exact`, `Contains`, or `Glob` for routes and network waits. |
| `RouteAction` | `Fulfill`, `Abort(ErrorReason)`, or `Continue` with optional overrides. |
| `Snapshot` | An accessibility tree with refs. `text()`, `nodes()`, `diff(&previous)`. |
| `Action`, `Outcome`, `ActReport` | One agent action parsed from JSON, its result, and the report it carries. |
| `PageId`, `PageInfo` | A tab and its URL, title, and active flag. |
| `StorageState` | Cookies and `localStorage` to save, load, and launch with. |
| `Events`, `DialogPolicy` | The opt-in for page and runtime events, and how dialogs are answered. |
| `Error` | Every failure. Match on the variant, not the message. |
| `human` | Cursor path and keystroke timing generators. Pure functions; they do not touch a session. |

## Jobs to symbols

Methods are on `Session` unless the row names another type.

| Job | Symbol |
| --- | --- |
| Snapshot the page for a model | `snapshot(SnapshotOptions::default())`, then `Snapshot::text()` |
| Keep a snapshot small | `SnapshotOptions { interactive_only, depth, scope, max_chars, max_nodes, .. }` |
| Act on a ref | `Selector::parse("ref=e5")` or `Selector::Ref("e5".into())` in any selector argument |
| Run a model's JSON action | `serde_json::from_str::<Action>(json)`, then `act(action, ActReport::Delta)` |
| Diff two snapshots | `Snapshot::diff(&previous)` |
| Read the page as Markdown or text | `text(TextFormat::Markdown, max_chars)`, `text(TextFormat::Text, max_chars)` |
| Find the browser | `Launch::installed()`, or `resolve()` for the path |
| Start the browser | `Session::launch(Launch::installed()?)` |
| Attach to a running browser | `Session::connect(Endpoint::Port(9222), Timing::default())` or `Endpoint::WebSocket(url)`; `close()` leaves it running |
| Use another Chromium build | `Launch::new(path)` |
| Show the window | `Launch::headed()`; `Launch::offscreen()` places it off screen on Windows |
| Open a start URL | `Launch::url(url)` |
| Keep cookies and storage between runs | `Launch::profile(dir)`; `Error::ProfileInUse` if another browser holds it |
| Save and restore a login | `storage_state()`, `StorageState::save(path)`, `StorageState::load(path)`, `Launch::storage_state(state)` |
| Use a proxy | `Launch::proxy("http://user:pass@host:port")` |
| Add a Chrome flag | `Launch::arg("--flag")`; `Launch::args(dir)` shows the full command line |
| Change timeouts | `Launch::timing(Timing { wait, ..Timing::default() })` |
| Change the wait for one locator, or skip checks | `Locator::with_options(ActionOptions { timeout, force })` |
| Turn on dialogs, console, or file chooser events | `Launch::events(Events::all())`, `Launch::dialogs(DialogPolicy::Accept)`, or `set_events(events, policy)` |
| Read dialogs and console messages | `dialogs()`, `console()`, `dropped_records()` |
| Close and clean up | `close()` |
| Check whether the browser died | `exited()` |
| List, open, switch, and close tabs | `pages()`, `active_page()`, `new_page(Some(url))`, `select_page(&id)`, `close_page(&id)`, `page_info(&id)` |
| Wait for a popup or new tab | `wait_for_page()`; it becomes the active tab |
| Navigate and wait for the DOM | `goto(url)` |
| Navigate without waiting | `open(url)` |
| Reload, back, forward | `reload()`, `go_back()`, `go_forward()` |
| Read the URL, title, or HTML | `url()`, `title()`, `content()` |
| Find an element | `locator(selector)`, `get_by_role`, `get_by_text`, `get_by_label`, `get_by_test_id` |
| Match text with a regular expression | `Selector::Text(TextMatch::regex(pattern, flags))` |
| Pick one of many matches | `Locator::nth(i)`, `Locator::first()`, `Locator::last()` |
| Narrow or combine locators | `Locator::locator(inner)`, `filter(Filter { has_text, has, has_not, has_not_text, visible })`, `and(other)`, `or(other)` |
| Count matches | `Locator::count()`, `Locator::all()` |
| Click, hover | `click(selector)`, `hover(selector)`, or the same on `Locator`; clicks check that they landed and retry |
| Double-click, right-click, click with modifiers | `dblclick(selector)`, `Locator::click_with(ClickOptions { button, count, modifiers: Modifiers::SHIFT, position })`, `click_at(x, y, options)` |
| Check a box or pick an option | `check`, `uncheck`, `Locator::set_checked(bool)`, `select_option(selector, SelectBy::Value(..))` |
| Drag and drop | `drag_and_drop(from, to)`, `Locator::drag_to(target)`, `Locator::drag_to_point(x, y)` |
| Focus, blur, or clear an element | `focus(selector)`, `Locator::blur()`, `Locator::clear()` |
| Press a key on an element | `Locator::press(key)` |
| Upload files | `set_input_files(selector, files)`, or `on_file_chooser(files)` with `Events { page: true, .. }` |
| Download files | `set_downloads(dir)`, then `wait_for_download()`; `downloads()` lists them |
| Click without failing on a blocked element | `Locator::try_click()` returns `ClickResult` |
| Find out why an element is not actionable | `Locator::settle()` returns `Actionable` with a `Blocker` |
| Replace an input's value | `fill(selector, text)` |
| Type key by key into an element | `type_text(selector, text)`, `Locator::type_text(text)` |
| Press a key or chord | `press("Enter")`, `press("Control+a")` |
| Move or click the mouse at a point | `mouse_move(x, y)`, `mouse_click(x, y, MouseButton::Left)` |
| Scroll | `scroll(delta_y)`, `wheel(delta_x, delta_y)` |
| Read element text or state | `Locator::inner_text`, `text_content`, `input_value`, `attribute`, `bounding_box`, `is_visible`, `is_enabled`, `is_checked`, `is_focused`, `is_attached` |
| Assert on an element | `expect(&mut locator).to_be_visible()`, `to_have_text`, `to_contain_text`, `to_have_value`, `to_have_attribute`, `to_have_count`, `to_have_class`, `to_have_id`, `to_have_css`, `to_have_js_property`, `to_be_editable`, `to_be_empty`, `to_be_in_viewport`, and the other `to_be_*` matchers |
| Assert on the accessibility tree | `expect(&mut locator).to_match_aria_snapshot("- list\n  - listitem")` |
| Assert on the page | `expect_page(&mut session).to_have_title(..)`, `to_have_url(..)` |
| Wait for an element | `wait_for_selector(selector)` |
| Wait for a JavaScript condition | `wait_for_function(expression)` |
| Wait for `load` or network idle | `wait_for_load_state(LoadState::Load)`, `wait_for_load_state(LoadState::NetworkIdle)` |
| Sleep | `wait_for_timeout(duration)` |
| Run JavaScript | `evaluate(expression)` returns `serde_json::Value`; promises are awaited |
| Inject a script | `add_script_tag(ScriptTag::Content(..))` or `ScriptTag::Url(..)` |
| Work inside an iframe | `frame_locator("#frame").locator("#button")` |
| List frames | `frames()` |
| Mock, block, or rewrite a request | `route(UrlMatch, RouteAction)`, `unroute(UrlMatch)` |
| Block URLs by pattern | `block_urls(&patterns)` |
| Wait for a request or response | `network_on()` first, then `wait_for_request(UrlMatch)`, `wait_for_response(UrlMatch)` |
| Read a response body | `response_body(&response)` |
| Read cookies | `cookies()`, `cookie_named(name)` |
| Write or clear cookies | `add_cookies(vec![Cookie { .. }])`, `clear_cookies()` |
| Take a screenshot | `screenshot(ScreenshotOptions::default())` returns PNG bytes |
| Print to PDF | `pdf(PdfOptions::default())` returns PDF bytes |
| Emulate a device | `emulate(Device::Iphone15)`, `set_viewport(Viewport { .. })`, `clear_viewport()`, `set_user_agent(ua)` |
| Change locale, time zone, location, or color scheme | `set_locale`, `set_timezone`, `set_geolocation(Some(Geolocation { .. }))`, `set_color_scheme(ColorScheme::Dark)` |
| Slow down or cut the network | `set_throttle(Throttle::Slow3g)`, `set_offline(true)` |
| Add headers to every request | `set_extra_headers(headers)` |
| Turn JavaScript off | `set_javascript_enabled(false)` |
| Send a raw CDP command | `send(method, params)` for the page, `browser_send` for the browser |
| Drain raw CDP events | `take_events()`; `dropped_events()` counts events lost from a full queue |
| Count rejected paused-request replies | `reply_failures()` |
| Prove no forbidden method was sent | `sent()` and `FORBIDDEN_METHODS` |
| Generate a cursor path | `human::path(from, Target::Rect(rect), &CursorOptions::default(), &mut Rng::new(seed))` |
| Generate keystroke timing | `human::keystrokes(text, &TypingOptions::default(), &mut Rng::new(seed))` |

## Patterns that work

- A `Locator` borrows the session mutably. Build it, use it, and drop it before you call another `Session` method. For assertions, write `expect(&mut session.locator("#id")).to_have_text("x").await?` in one statement.
- Prefer `expect(...)` over reading a value and comparing it. The matchers retry until `Timing::wait` runs out, so they absorb rendering delays.
- `fill` clicks the element, selects its content, deletes it, and types the new text with key events. It does not set `value` directly, so input handlers run.
- Call `network_on()` before the action that starts the request you want to wait for. Events from before that call are not recorded.
- Use `UrlMatch::Contains` for routes unless you need an exact URL. `UrlMatch::Glob` supports `*` and `**`.
- Do not call `send` with a method in `FORBIDDEN_METHODS`. It returns `Error::Blocked`. Poll with `wait_for_function` or read with `evaluate` instead of subscribing to events. For dialogs, console messages, or the file picker, opt in with `Launch::events`; that allows `Page.enable` or `Runtime.enable` only.
- For an agent, send `ActReport::Delta` results instead of full snapshots after the first one, and use `interactive_only` or `scope` for large pages.
- A storage state file holds login cookies. `save` writes it with mode `0600` on Unix; keep it out of version control.
- In tests that do not opt in to events, assert the guarantee at the end: `for m in FORBIDDEN_METHODS { assert!(!session.sent().contains(m)); }`.
- Always call `close()`. For a launched browser it shuts the browser down and deletes the temporary profile. For a browser from `Session::connect` it only disconnects.

## Errors

| Variant | When |
| --- | --- |
| `MissingExe` | `Launch::installed()` found no browser. |
| `Spawn { path, source }` | The executable could not start. |
| `Exited { code, stderr }` | The browser quit before DevTools was ready. Read `stderr`; on Linux it is often the sandbox. |
| `Connect(_)` | The DevTools endpoint was missing, malformed, or closed. |
| `Timeout(_)` | A wait, navigation, or actionability check ran out of time. |
| `Assertion { expected, received, selector }` | An `expect` matcher failed. |
| `Protocol { method, code, message }` | Chrome returned an error for a command, or a navigation failed. |
| `Script { kind, message }` | A script threw in the page. `kind` is `ScriptError::Syntax` for an invalid selector or regex, which fails at once. |
| `NotActionable { reason, covered_by, selector }` | The element never became actionable within the wait. `reason` is a `Blocker` such as `Obscured`, `Disabled`, or `NotEditable`. |
| `StaleRef(ref)` | The ref is not in the latest snapshot of the active tab. Take a new snapshot. |
| `ProfileInUse { path }` | Another browser holds the `Launch::profile` directory. |
| `Frame(message)` | A frame was detached or its target could not be found. |
| `Closed` | The DevTools connection closed mid-session. |
| `Blocked(method)` | The call named a method in `FORBIDDEN_METHODS` without the event opt-in, or `on_file_chooser` ran without `Events { page: true, .. }`. |
| `Io`, `Ws`, `Json` | Lower-level failures. |
