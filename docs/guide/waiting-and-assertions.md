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

`m` is anything that converts into a `TextMatch`. A `&str` is an exact match. `to_have_text` and `to_contain_text` collapse runs of whitespace and trim the ends of both sides before they compare; a regular expression is tested against the collapsed text. `to_have_value` and `to_have_attribute` compare the value as it is. All text comparisons in assertions are case-sensitive unless a regular expression uses the `i` flag.

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
| `wait_for_timeout(duration)` | The duration passes. | None |

`wait_for_function` returns the value as `serde_json::Value`. An object or array result comes back as `true`; return a primitive when you need the value.

```rust
session
    .wait_for_function("document.getElementById('out').textContent === 'hi Ada'")
    .await?;
```

A wait that runs out returns `Error::Timeout` with the expression and the last page error, if there was one.

## Navigation waits

`goto(url)` navigates and waits for `DOMContentLoaded`, up to `Timing::navigate` (15 seconds). If the page is still on `about:blank` after `Timing::settle`, it navigates again, until `Timing::navigate` runs out. `open(url)` navigates and returns at once. `reload()` waits for `load`.
