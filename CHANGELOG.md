# Changelog

The minimum supported Rust version is 1.96.

charm-cdp is experimental: any `0.x` release can change the API and its behaviour, and each change is listed here.

## Unreleased

### Added

- `Session::launch` starts the installed Chrome with a temporary or persistent profile, headless by default, and connects over the DevTools protocol. `resolve()` finds Chrome, Chromium, or Chrome Canary, or the executable in `CHROME_EXE` or `CHROMIUM_EXE`.
- The transport refuses `Runtime.enable`, `Page.enable`, and `Target.setAutoAttach` with `Error::Blocked`. `FORBIDDEN_METHODS` lists them, and `Session::sent()` returns every method a session has sent.
- Navigation: `goto`, `open`, `reload`, `url`, `title`, `content`, `add_script_tag`, and `screenshot` with PNG, JPEG, or WebP output, a clip, or the full page.
- Locators: CSS, XPath (`xpath=` prefix), role, text, label, and test id selectors, with exact, substring, or regular-expression text matching through `TextMatch`. `nth`, `first`, `count`, and `all`.
- Actions that wait for an actionable element: `click`, `hover`, `fill`, `type_text`, and `press_sequentially`. `settle` and `try_click` report why an element is not actionable as a `Blocker`.
- Reads: `inner_text`, `text_content`, `input_value`, `attribute`, `bounding_box`, and the `is_*` state checks.
- Raw input: `mouse_move`, `mouse_click`, `wheel`, `scroll`, and `press` with modifier chords such as `Control+a`.
- Retrying assertions: `expect` for a locator (visibility, state, text, value, attribute, count) and `expect_page` for the title and URL. A failed assertion returns `Error::Assertion` with the expected and received values.
- Waits: `wait_for_selector`, `wait_for_function`, `wait_for_load_state`, and `wait_for_timeout`.
- Frames: `frame_locator` reaches same-origin and cross-origin iframes, and `frames` lists the frame tree.
- Network: `route` and `unroute` fulfill, abort, or continue requests that match a `UrlMatch`. `wait_for_request`, `wait_for_response`, and `response_body` read traffic after `network_on`. `block_urls`, cookie reads, `add_cookies`, and `clear_cookies`.
- Proxies: `Launch::proxy` accepts a URL with a user name and password and answers the proxy's authentication challenge.
- `Timing` holds every timeout and poll interval in one typed value.
- Launch errors: `Error::Spawn` when the executable cannot start, and `Error::Exited` with the exit status and the browser's stderr when it quits before DevTools is ready.
- The `human` module generates cursor paths and keystroke timing from a seeded random number generator, for callers that dispatch their own input.
