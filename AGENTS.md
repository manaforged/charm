# charm-cdp for coding agents

Point your coding agent at this file when it writes Rust code that drives Chrome with charm-cdp. Every symbol below exists in the crate. The full list of public items is in [`api/charm-cdp.txt`](api/charm-cdp.txt).

## Install

The crate is not published on crates.io yet. Add it as a git dependency pinned to a commit, plus tokio:

```toml
[dependencies]
charm-cdp = { git = "https://github.com/manaforged/charm-cdp", rev = "<commit>" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The library name is `charm`. Import with `use charm::{...}`. Every browser call is `async` and returns `Result<_, charm::Error>`.

The host needs an installed Chrome or Chromium. `Launch::installed()` finds it, or reads `CHROME_EXE`. On Linux hosts where Chrome's sandbox cannot start, `Session::launch` returns `Error::Exited`; add `Launch::arg("--no-sandbox")`.

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
| `Session` | One browser process and one page. Owns the DevTools connection. |
| `Launch` | Builder for the browser command line: profile, headed, proxy, timing, extra args. |
| `Timing` | Every timeout and poll interval. `Timing::default()` holds the defaults. |
| `Locator<'_>` | A lazy reference to an element. Borrows the session mutably. |
| `Selector` | `Css`, `XPath`, `Role`, `Text`, `TestId`, `Label`. A `&str` converts to `Css`, or to `XPath` with an `xpath=` prefix. |
| `TextMatch` | `Exact`, `Contains`, or `Regex { pattern, flags }`. A `&str` converts to `Exact`. |
| `FrameLocator<'_>` | A path through one or more iframes. Ends in `.locator(...)`. |
| `UrlMatch` | `Exact`, `Contains`, or `Glob` for routes and network waits. |
| `RouteAction` | `Fulfill`, `Abort(ErrorReason)`, or `Continue` with optional overrides. |
| `Error` | Every failure. Match on the variant, not the message. |
| `human` | Cursor path and keystroke timing generators. Pure functions; they do not touch a session. |

## Jobs to symbols

Methods are on `Session` unless the row names another type.

| Job | Symbol |
| --- | --- |
| Find the browser | `Launch::installed()`, or `resolve()` for the path |
| Start the browser | `Session::launch(Launch::installed()?)` |
| Show the window | `Launch::headed()` |
| Keep cookies and storage between runs | `Launch::profile(dir)` |
| Use a proxy | `Launch::proxy("http://user:pass@host:port")` |
| Add a Chrome flag | `Launch::arg("--flag")` |
| Change timeouts | `Launch::timing(Timing { wait, ..Timing::default() })` |
| Close and clean up | `close()` |
| Check whether the browser died | `exited()` |
| Navigate and wait for the DOM | `goto(url)` |
| Navigate without waiting | `open(url)` |
| Reload | `reload()` |
| Read the URL, title, or HTML | `url()`, `title()`, `content()` |
| Find an element | `locator(selector)`, `get_by_role`, `get_by_text`, `get_by_label`, `get_by_test_id` |
| Match text with a regular expression | `Selector::Text(TextMatch::regex(pattern, flags))` |
| Pick one of many matches | `Locator::nth(i)`, `Locator::first()` |
| Count matches | `Locator::count()`, `Locator::all()` |
| Click, hover | `click(selector)`, `hover(selector)`, or the same on `Locator` |
| Click without failing on a blocked element | `Locator::try_click()` returns `ClickResult` |
| Find out why an element is not actionable | `Locator::settle()` returns `Actionable` with a `Blocker` |
| Replace an input's value | `fill(selector, text)` |
| Type key by key into an element | `type_text(selector, text)`, `Locator::type_text(text)` |
| Press a key or chord | `press("Enter")`, `press("Control+a")` |
| Move or click the mouse at a point | `mouse_move(x, y)`, `mouse_click(x, y, MouseButton::Left)` |
| Scroll | `scroll(delta_y)`, `wheel(delta_x, delta_y)` |
| Read element text or state | `Locator::inner_text`, `text_content`, `input_value`, `attribute`, `bounding_box`, `is_visible`, `is_enabled`, `is_checked`, `is_focused`, `is_attached` |
| Assert on an element | `expect(&mut locator).to_be_visible()`, `to_have_text`, `to_contain_text`, `to_have_value`, `to_have_attribute`, `to_have_count`, and the other `to_be_*` matchers |
| Assert on the page | `expect_page(&mut session).to_have_title(..)`, `to_have_url(..)` |
| Wait for an element | `wait_for_selector(selector)` |
| Wait for a JavaScript condition | `wait_for_function(expression)` |
| Wait for `load` | `wait_for_load_state(LoadState::Load)` |
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
| Send a raw CDP command | `send(method, params)` for the page, `browser_send` for the browser |
| Drain raw CDP events | `take_events()` |
| Prove no forbidden method was sent | `sent()` and `FORBIDDEN_METHODS` |
| Generate a cursor path | `human::path(from, Target::Rect(rect), &CursorOptions::default(), &mut Rng::new(seed))` |
| Generate keystroke timing | `human::keystrokes(text, &TypingOptions::default(), &mut Rng::new(seed))` |

## Patterns that work

- A `Locator` borrows the session mutably. Build it, use it, and drop it before you call another `Session` method. For assertions, write `expect(&mut session.locator("#id")).to_have_text("x").await?` in one statement.
- Prefer `expect(...)` over reading a value and comparing it. The matchers retry until `Timing::wait` runs out, so they absorb rendering delays.
- `fill` clicks the element, selects its content, deletes it, and types the new text with key events. It does not set `value` directly, so input handlers run.
- Call `network_on()` before the action that starts the request you want to wait for. Events from before that call are not recorded.
- Use `UrlMatch::Contains` for routes unless you need an exact URL. `UrlMatch::Glob` supports `*` and `**`.
- Do not call `send` with `Runtime.enable`, `Page.enable`, or `Target.setAutoAttach`. It returns `Error::Blocked`. Poll with `wait_for_function` or read with `evaluate` instead of subscribing to events.
- In tests, assert the guarantee at the end: `for m in FORBIDDEN_METHODS { assert!(!session.sent().contains(m)); }`.
- Always call `close()`. It shuts the browser down and deletes the temporary profile.

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
| `NotActionable { reason, covered_by, selector }` | The element never became visible, enabled, stable, and unobscured within the wait. |
| `Frame(message)` | A frame was detached or its target could not be found. |
| `Closed` | The DevTools connection closed mid-session. |
| `Blocked(method)` | The call named a forbidden method. |
| `Io`, `Ws`, `Json` | Lower-level failures. |
