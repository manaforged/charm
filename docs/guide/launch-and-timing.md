# Launch options and timing

`Launch` builds the browser command line. `Session::launch(launch)` starts the process and connects.

```rust
use charm::{Launch, Session, Timing};
use std::time::Duration;

let launch = Launch::installed()?
    .profile("./profile")
    .proxy("http://user:pass@proxy.example:8080")
    .timing(Timing { wait: Duration::from_secs(10), ..Timing::default() })
    .arg("--lang=en-US");
let mut session = Session::launch(launch).await?;
```

## Builder methods

| Method | Effect |
| --- | --- |
| `Launch::installed()?` | Starts from the executable path. Headless, temporary profile, `about:blank`. |
| `headed()` | Shows the browser window. Without it, charm passes `--headless=new`. |
| `profile(dir)` | Uses `dir` as the user data directory and keeps it after `close`. Without it, charm creates a temporary directory and deletes it on `close`. If another browser holds `dir`, the launch returns `Error::ProfileInUse`. |
| `url(url)` | The first page the browser opens. The default is `about:blank`. |
| `proxy(url)` | Routes traffic through a proxy. A user name and password in the URL are removed from the command line and sent in answer to the proxy's authentication challenge. An empty string means no proxy. |
| `offscreen()` | On Windows, places a headed window off the visible screen. It has no effect on other systems. |
| `arg(flag)` | Adds one Chrome command-line flag after charm's own flags. Call it once per flag. |
| `timing(timing)` | Sets every timeout and poll interval. |
| `events(events)` | Opts in to page or runtime events. See [Events](events.md). |
| `dialogs(policy)` | Sets how charm answers dialogs when page events are on. The default is `DialogPolicy::Dismiss`. |
| `storage_state(state)` | Loads cookies and `localStorage` from a `StorageState` before the first navigation. See [Logins](logins.md). |

`Launch::args(user_data_dir)` returns the full argument list charm will pass, so you can log it or check it in a test. charm always passes `--remote-debugging-port=0`, `--no-first-run`, `--no-default-browser-check`, `--window-size=1536,864`, and `--remote-allow-origins=*`.

## Choosing the browser

`resolve()` reads `CHROME_EXE`, then `CHROMIUM_EXE`, then checks the standard install paths of Google Chrome, Chromium, Microsoft Edge, Brave, and Chrome Canary. It returns `None` when nothing is found. Any Chromium-based browser that supports `--remote-debugging-port` can be passed to `Launch::new`.

## Linux sandbox

Ubuntu 23.10 and later restrict unprivileged user namespaces through AppArmor. Chrome's sandbox can then fail, and the browser exits at once. charm reports it as `Error::Exited` with Chrome's stderr. On such a host, either install an AppArmor profile for the browser or turn the sandbox off:

```rust
let launch = Launch::installed()?.arg("--no-sandbox");
```

Turn the sandbox off only on hosts where it cannot start, and only for pages you trust.

## Timing

`Timing` holds every duration charm uses. Build one from the defaults and change only the fields you need. To change the wait for one locator only, use `Locator::with_options(ActionOptions { timeout: Some(..), .. })`; see [Locators](locators.md).

| Field | Default | Used for |
| --- | --- | --- |
| `connect` | 45 s | How long `Session::launch` waits for the DevTools endpoint, and how long `Session::connect` waits for `/json/version`. |
| `connect_poll` | 250 ms | How often it checks for the endpoint. |
| `command` | 30 s | How long one CDP command waits for its reply. |
| `drain` | 25 ms | How long charm reads pending browser messages before it returns events. |
| `navigate` | 15 s | The limit for `goto` and `reload`. |
| `settle` | 2.5 s | How long `goto` waits for the page to leave `about:blank` before it navigates again. |
| `poll` | 120 ms | The interval between checks in every wait and assertion. |
| `wait` | 30 s | The limit for assertions, `wait_for_*` (including `wait_for_page` and `wait_for_download`), frame lookup, network waits, and actions that wait for an actionable element. A click that does not land is retried until this limit. |
| `actionable` | 1 s | The limit for `Locator::settle` and `Locator::try_click`, and for the check that an element inside an iframe is the one under the pointer. |
| `network_idle` | 500 ms | How long no request may be in flight before `LoadState::NetworkIdle` holds. |
| `close` | 1.5 s | How long `close` waits for the browser to exit before it kills the process. |
| `follow` | 60 ms | How long `act` waits for a navigation its action starts, such as a form submit, before it reads the outcome. A navigation that starts in time is followed to its new page. |

On macOS and Linux a browser that `Session::launch` started also ends when the program that started it ends, even after `process::exit`, Ctrl-C, or a crash: charm holds a pipe to a small shell guard, and the guard stops Chrome when the pipe closes. On Windows the browser ends when the `Session` is dropped or closed.

## Errors

`Error` is `#[non_exhaustive]`: a `match` on it needs a `_` arm.

| Variant | Meaning |
| --- | --- |
| `Spawn { path, source }` | The operating system could not start `path`. |
| `Exited { code, stderr }` | The browser exited before DevTools was ready. `stderr` holds its output. |
| `Connect(message)` | The DevTools endpoint did not appear within `connect`, was malformed, or the websocket closed. |
| `Timeout(what)` | A wait, a navigation, or a command ran past its limit. |
| `Protocol { method, code, message }` | Chrome returned an error, or a navigation failed (for example `net::ERR_NAME_NOT_RESOLVED`). |
| `Script { kind, message }` | A script threw in the page. `kind` is `ScriptError::Syntax` for an invalid selector or regex, which fails at once, `ScriptError::Type` for a type error, and `ScriptError::Other` for the rest. |
| `NotActionable { reason, covered_by, selector }` | The element never became actionable within the wait. `reason` is a `Blocker`, for example `Obscured`, `Disabled`, or `NotEditable` for `fill` on a read-only input. A click that never reached the element also returns `Obscured`. |
| `StaleRef(ref)` | A snapshot ref is not in the latest snapshot of the active tab, or its element is gone. Take a new snapshot. |
| `ProfileInUse { path }` | Another browser holds the profile directory given to `Launch::profile`. |
| `Frame(message)` | A frame was detached or its target could not be found. |
| `Closed` | The DevTools connection closed mid-session, for example because the browser quit or crashed. |
| `Assertion { expected, received, selector }` | An `expect` matcher failed. |
| `Blocked(method)` | The call named a method in `FORBIDDEN_METHODS` that the session has not opted in to, or `on_file_chooser` ran without the `page` event opt-in. |
| `MissingExe` | `Launch::installed()` found no browser executable. |
| `Io`, `Ws`, `Json` | I/O, websocket, and JSON errors from the layers below. |

`Session::exited()` returns `true` when the browser process has ended, for example after a crash.
