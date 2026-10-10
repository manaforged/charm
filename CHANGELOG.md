# Changelog

The minimum supported Rust version is 1.96.

charm-cdp is experimental: any `0.x` release can change the API and its behaviour, and each change is listed here.

## Unreleased

### Added

- `Session::launch` starts the installed Chrome with a temporary or persistent profile, headless by default, and connects over the DevTools protocol. `resolve()` finds Chrome, Chromium, or Chrome Canary, or the executable in `CHROME_EXE` or `CHROMIUM_EXE`.
- The transport refuses `Runtime.enable`, `Page.enable`, `Target.setAutoAttach`, `Target.sendMessageToTarget`, `Target.exposeDevToolsProtocol`, and `Target.autoAttachRelated` with `Error::Blocked`. `FORBIDDEN_METHODS` lists them, and `Session::sent()` returns every method a session has sent.
- Navigation: `goto`, `open`, `reload`, `go_back`, `go_forward`, `url`, `title`, `content`, `add_script_tag`, and `screenshot` with PNG, JPEG, or WebP output, a clip, or the full page.
- Locators: CSS, XPath (`xpath=` prefix), role, text, label, and test id selectors, with exact, substring, or regular-expression text matching through `TextMatch`. `nth`, `first`, `count`, and `all`.
- Actions that wait for an actionable element: `click`, `hover`, `fill`, `type_text`, and `press_sequentially`. `settle` and `try_click` report why an element is not actionable as a `Blocker`.
- Reads: `inner_text`, `text_content`, `input_value`, `attribute`, `bounding_box`, and the `is_*` state checks.
- Raw input: `mouse_move`, `mouse_click`, `wheel`, `scroll`, and `press` with modifier chords such as `Control+a`.
- Retrying assertions: `expect` for a locator (visibility, state, text, value, attribute, count) and `expect_page` for the title and URL. A failed assertion returns `Error::Assertion` with the expected and received values.
- Waits: `wait_for_selector`, `wait_for_function`, `wait_for_load_state` (including `LoadState::NetworkIdle`), and `wait_for_timeout`.
- Frames: `frame_locator` reaches same-origin and cross-origin iframes, and `frames` lists the frame tree.
- Network: `route` and `unroute` fulfill, abort, or continue requests that match a `UrlMatch`. `wait_for_request`, `wait_for_response`, and `response_body` read traffic after `network_on`. `block_urls`, cookie reads, `add_cookies`, and `clear_cookies`.
- Proxies: `Launch::proxy` accepts a URL with a user name and password and answers the proxy's authentication challenge.
- `Timing` holds every timeout and poll interval in one typed value.
- Launch errors: `Error::Spawn` when the executable cannot start, and `Error::Exited` with the exit status and the browser's stderr when it quits before DevTools is ready.
- The `human` module generates cursor paths and keystroke timing from a seeded random number generator, for callers that dispatch their own input.
- Agent browsing: `snapshot` returns the accessibility tree as text with a ref on each element, and `SnapshotOptions` limits it by interactivity, depth, scope, node count, and size. Refs work as selectors through `Selector::Ref` and the `ref=` prefix, inside same-origin and cross-origin frames, and a ref that is gone returns `Error::StaleRef`. `Snapshot::diff` reports the lines that changed.
- `Action` deserializes a model's JSON tool call, and `Session::act` runs it and returns an `Outcome` with the URL, title, new tabs, dialogs, console error count, and a full or delta snapshot chosen by `ActReport`.
- `text` reads the page as Markdown or plain text, up to a character limit.
- Tabs: `pages`, `active_page`, `new_page`, `select_page`, `close_page`, `page_info`, and `wait_for_page` for popups.
- `Session::connect` attaches to a running browser by port or websocket URL. `close` on an attached browser disconnects and leaves the browser running.
- Logins: `storage_state` exports cookies and `localStorage`, `StorageState::save` writes it with mode `0600` on Unix, and `Launch::storage_state` loads it into a new browser. `Session::launch` returns `Error::ProfileInUse` when another browser holds the `Launch::profile` directory.
- Opt-in events: `Launch::events` and `Session::set_events` allow `Page.enable` or `Runtime.enable` for one session. With them, `Launch::dialogs` answers dialogs by `DialogPolicy`, and `dialogs`, `console`, and `dropped_records` read what happened.
- Downloads and files: `set_downloads`, `wait_for_download`, and `downloads`; `set_input_files`; and `on_file_chooser` to answer the file picker.
- Form input: `select_option`, `check`, `uncheck`, `set_checked`, `dblclick`, `click_with` with `ClickOptions` and `Modifiers`, `click_at`, `drag_and_drop`, `drag_to`, `drag_to_point`, `focus`, `blur`, `clear`, and `Locator::press`.
- Clicks check that the pointer event reached the element and retry until `Timing::wait`. `fill` and `type_text` return `Blocker::NotEditable` for an element that does not accept text.
- Locators: `filter` with `Filter`, `and`, `or`, `last`, chained `locator`, and `with_options` with `ActionOptions` for a per-locator timeout or `force`.
- Assertions: `to_be_editable`, `to_be_empty`, `to_be_in_viewport`, `to_have_class`, `to_have_id`, `to_have_css`, `to_have_js_property`, and `to_match_aria_snapshot`.
- Emulation: `emulate` with `Device` presets, `set_viewport`, `clear_viewport`, `set_user_agent`, `set_locale`, `set_timezone`, `set_geolocation`, `set_color_scheme`, `set_throttle`, `set_offline`, `set_extra_headers`, and `set_javascript_enabled`.
- `pdf` prints the page with `PdfOptions`.
- Routes apply to every tab of a session.
- `Timing::network_idle` sets how long the network must be quiet for `LoadState::NetworkIdle`.
- `Session::reply_failures` and `Session::dropped_events` count rejected paused-request replies and events dropped from a full queue.
