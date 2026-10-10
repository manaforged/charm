# Tabs and connect

A `Session` drives one browser and can hold many tabs. One tab is active at a time. Every page call (`goto`, `click`, `snapshot`, `evaluate`, and the rest) goes to the active tab.

## Tabs

| Method | Effect |
| --- | --- |
| `pages()` | Lists every open tab as `PageInfo { id, url, title, active }`. |
| `active_page()` | Returns the `PageId` of the active tab. |
| `new_page(Some(url))` | Opens a tab, makes it active, and navigates it to `url` like `goto`. `None` opens a blank tab. |
| `select_page(&id)` | Makes `id` the active tab and brings it to the front. |
| `close_page(&id)` | Closes the tab. If it was active, another open tab becomes active, or charm opens a blank tab when none is left. |
| `wait_for_page()` | Waits up to `Timing::wait` for a tab that charm has not seen before, makes it active, and returns its id. |
| `page_info(&id)` | Returns the `PageInfo` of one tab. |

`PageId` is an opaque string. `as_str()` returns it, and it serializes as a plain JSON string, so a model can name a tab in an `Action`.

```rust
use charm::LoadState;

let first = session.active_page().clone();
session.click("#open-in-new-tab").await?;
let popup = session.wait_for_page().await?;
session.wait_for_load_state(LoadState::Load).await?;
println!("{}", session.title().await?);

session.select_page(&first).await?;
session.close_page(&popup).await?;
```

`wait_for_page` polls the browser's target list. It does not subscribe to target events, so it needs no event opt-in. Call it after the action that opens the tab.

Routes added with `route` apply to every tab, including tabs opened later. Snapshot refs belong to the tab that took the snapshot; a ref from another tab returns `Error::StaleRef`.

## Connect to a running browser

`Session::connect(endpoint, timing)` attaches to a Chrome that is already running with remote debugging on:

| Endpoint | Use |
| --- | --- |
| `Endpoint::Port(port)` | The `--remote-debugging-port` of a browser on this host. charm reads `http://127.0.0.1:{port}/json/version` to find the websocket URL. |
| `Endpoint::WebSocket(url)` | The browser websocket URL, for example from the `DevToolsActivePort` file or `/json/version`. |

```rust
use charm::{Endpoint, Session, Timing};

let mut session = Session::connect(Endpoint::Port(9222), Timing::default()).await?;
session.goto("https://example.com/").await?;
session.close().await?;
```

Start the browser yourself with a flag such as `--remote-debugging-port=9222` and a `--user-data-dir`. A browser started that way keeps the logins of that profile, so the session runs as that user.

The session makes the browser's first open tab active, and opens a tab when there is none. `Timing::connect` bounds the lookup of the websocket URL. To receive dialogs or console messages from an attached browser, call `set_events` after you connect; see [Events](events.md). Only `Endpoint::Port` is covered by a test.

## Who owns the browser

| Started by | `close()` |
| --- | --- |
| `Session::launch` | Asks the browser to quit, kills it after `Timing::close`, and deletes the temporary profile. A profile from `Launch::profile` is kept. |
| `Session::connect` | Closes the DevTools connection only. The browser and its tabs keep running. |

`exited()` reports `false` for an attached browser, because charm does not own its process.
