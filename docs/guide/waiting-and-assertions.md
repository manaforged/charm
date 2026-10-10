# Waiting and assertions

charm does not subscribe to page events, so every wait is a poll: charm evaluates an expression in the page every `Timing::poll` (120 ms by default) until it holds or the time runs out. Most waits use `Timing::wait` (30 seconds by default). See [Launch options and timing](launch-and-timing.md) to change them.

## Assertions

`expect` takes a mutable reference to a locator. Each matcher retries until it passes or `Timing::wait` runs out, then returns `Error::Assertion { expected, received, selector }`.

```rust
use charm::{Error, TextMatch, expect};

expect(&mut session.locator("#status")).to_have_text("Saved").await?;
expect(&mut session.locator("#hidden")).to_be_hidden().await?;
expect(&mut session.locator("li")).to_have_count(3).await?;
expect(&mut session.locator("#total")).to_have_text(TextMatch::regex("^\\$\\d+\\.\\d{2}$", "")).await?;

let failed = expect(&mut session.locator("h1")).to_have_text("Goodbye").await;
assert!(matches!(failed, Err(Error::Assertion { .. })));
```

| Matcher | Passes when |
| --- | --- |
| `to_be_attached()` | The element is in the DOM. |
| `to_be_visible()`, `to_be_hidden()` | The element is visible, or not. |
| `to_be_enabled()`, `to_be_disabled()` | The element is enabled, or not. |
| `to_be_checked()` | The checkbox or radio is checked. |
| `to_be_focused()` | The element has focus. |
| `to_have_text(m)` | The element's text matches `m`. |
| `to_contain_text(m)` | The element's text contains `m`. |
| `to_have_value(m)` | The input value matches `m`. |
| `to_have_attribute(name, m)` | The attribute is present and matches `m`. |
| `to_have_count(n)` | The selector matches exactly `n` elements. |
| `to_be_editable()` | The element accepts text input. |
| `to_be_empty()` | The element has no text, or the input has no value. |
| `to_be_in_viewport()` | Part of the element is inside the viewport. |
| `to_have_class(m)` | The `class` attribute matches `m`. |
| `to_have_id(m)` | The `id` matches `m`. |
| `to_have_css(property, m)` | The computed value of the CSS property matches `m`. |
| `to_have_js_property(name, &value)` | The element's JavaScript property equals the `serde_json::Value`, compared as JSON. |
| `to_match_aria_snapshot(template)` | The element's accessibility tree matches the template. See below. |

`m` is anything that converts into a `TextMatch`. A `&str` is an exact match, except in `to_contain_text`, where it is a substring match. `to_have_text`, `to_contain_text`, and `to_have_class` collapse runs of whitespace and trim the ends of both sides before they compare; a regular expression is tested against the collapsed text. `to_have_value`, `to_have_attribute`, `to_have_id`, and `to_have_css` compare the value as it is. An exact match is case-sensitive. A `Contains` match ignores case, in assertions and in locators. A regular expression ignores case only with the `i` flag.

## Matching an aria snapshot

`to_match_aria_snapshot(template)` compares the element's accessibility tree with a template in the same line format that [`snapshot`](agents.md) prints. It retries until the tree matches or `Timing::wait` runs out. On failure, `Error::Assertion` holds the template and the tree it last saw.

```rust
use charm::expect;

expect(&mut session.locator("ul"))
    .to_match_aria_snapshot("- list\n  - listitem\n    - button \"Buy\"")
    .await?;
```

The rules:

- Each line is `- role`, then an optional name, optional attributes, and an optional `: value`. Indentation nests a line under the line above it.
- A name in double quotes matches the whole accessible name, after whitespace is collapsed. A name in slashes, such as `/^Buy/i`, is a regular expression. A line with no name matches any name.
- For `- text: words`, the text is a case-insensitive substring match.
- The attributes are `[checked]`, `[checked=false]`, `[checked=mixed]`, `[unchecked]`, `[disabled]`, `[expanded]`, `[collapsed]`, `[selected]`, `[level=N]`, and `[url=...]`. A `[ref=...]` attribute is ignored, so you can paste snapshot lines.
- Children match in order, and other children may sit between them. To require exactly these children, add a line `- /children: equal` at the start of the block. `- /children: deep-equal` applies the exact rule to every level below.
- The template can match any part of the element's tree, not only its root.

A template line that does not follow these rules fails at once with `Error::Assertion`.

For the page, use `expect_page`:

```rust
use charm::{TextMatch, expect_page};

expect_page(&mut session).to_have_title("Checkout").await?;
expect_page(&mut session).to_have_url(TextMatch::regex("/orders/\\d+$", "")).await?;
```

Both expectation types implement `std::ops::Not`. `!expect(&mut locator)` inverts the next matcher, so it waits until the condition stops holding.

## Waits

| Method | Waits until | Limit |
| --- | --- | --- |
| `wait_for_selector(selector)` | An element matches. | `Timing::wait` |
| `wait_for_function(expression)` | The JavaScript expression is truthy. Promises are awaited. | `Timing::wait` |
| `wait_for_load_state(LoadState::Load)` | `document.readyState` is `complete`. | `Timing::wait` |
| `wait_for_load_state(LoadState::DomContentLoaded)` | `document.readyState` is not `loading`. | `Timing::wait` |
| `wait_for_load_state(LoadState::NetworkIdle)` | `document.readyState` is `complete`, and then no request has been in flight for `Timing::network_idle` (500 ms). It calls `network_on()` first. | `Timing::wait` |
| `wait_for_page()` | A new tab opens. See [Tabs and connect](tabs-and-connect.md). | `Timing::wait` |
| `wait_for_download()` | A download completes. See [Downloads and files](downloads-and-files.md). | `Timing::wait` |
| `wait_for_timeout(duration)` | The duration passes. | None |

`wait_for_function` returns the value as `serde_json::Value`. An object or array result comes back as `true`; return a primitive when you need the value.

```rust
session
    .wait_for_function("document.getElementById('out').textContent === 'hi Ada'")
    .await?;
```

A wait that runs out returns `Error::Timeout` with the expression and the last page error, if there was one.

## Navigation waits

`goto(url)` navigates and waits for `DOMContentLoaded`, up to `Timing::navigate` (15 seconds). If the page is still on `about:blank` after `Timing::settle`, it navigates again, until `Timing::navigate` runs out. `open(url)` navigates and returns at once. `reload()` waits until the new document has loaded. `go_back()` and `go_forward()` move through the tab's history.

`NetworkIdle` counts requests that the tab sends after `network_on()`. A page that polls or keeps a connection open may never become idle; wait for an element instead.
