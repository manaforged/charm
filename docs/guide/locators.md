# Locators and selectors

A `Locator` names an element on the page. It is lazy: charm looks the element up again each time you call an action or a read on it, so a locator stays valid when the page re-renders.

## Building a locator

```rust
session.locator("#submit").click().await?;
session.get_by_label("Email", true).fill("ada@example.com").await?;
session.get_by_role("button", Some("Save"), true).click().await?;
let banner = session.get_by_text("Welcome", false).inner_text().await?;
let rows = session.get_by_test_id("order-row").count().await?;
```

`exact: true` matches the whole text. `exact: false` matches a case-insensitive substring.

A `Locator` borrows the session mutably. Finish with one locator before you call another `Session` method.

## Selector kinds

Every method that takes `impl Into<Selector>` accepts a `&str`, a `String`, or a `Selector`.

| Selector | Matches | From a string |
| --- | --- | --- |
| `Selector::Css(css)` | `document.querySelectorAll(css)` | Any string without a prefix |
| `Selector::XPath(path)` | An XPath expression | `"xpath=//button[1]"` |
| `Selector::Role { role, name }` | Elements with that ARIA role, and an accessible name if `name` is set | No |
| `Selector::Text(TextMatch)` | Elements by their text | No |
| `Selector::Label(TextMatch)` | Form controls by their accessible name: a `<label>`, `aria-label`, or `aria-labelledby` | No |
| `Selector::TestId(id)` | Elements with that `data-testid` | No |

`Selector::parse(raw)` applies the same string rule: `xpath=` gives `XPath`, anything else gives `Css`.

## Text matching

`TextMatch` has three forms:

| Form | Build it with | Matches |
| --- | --- | --- |
| `Exact(text)` | `TextMatch::new(text, true)` or `"text".into()` | The whole text |
| `Contains(text)` | `TextMatch::new(text, false)` | A substring. In a locator the match ignores case; in an assertion it does not. |
| `Regex { pattern, flags }` | `TextMatch::regex(pattern, flags)` | A JavaScript regular expression |

The pattern runs in the page as `new RegExp(pattern, flags)`, so it uses JavaScript syntax and flags such as `"i"`.

```rust
use charm::{Selector, TextMatch};

let heading = Selector::Text(TextMatch::regex("^Welcome \\d+$", ""));
assert_eq!(session.locator(heading).count().await?, 1);

let name = Some(TextMatch::regex("^save", "i"));
session.locator(Selector::Role { role: "button".into(), name }).click().await?;
```

## Several matches

Actions and reads use the first match. To pick another, use `nth`:

```rust
let total = session.locator("li").count().await?;
for index in 0..total {
    let text = session.locator("li").nth(index).inner_text().await?;
    println!("{index}: {text:?}");
}
```

`all()` returns `0..count` as a `Range<usize>`, which you can use in the same loop. `first()` goes back to the first match.

## Reads

| Method | Returns |
| --- | --- |
| `inner_text()` | `Option<String>`, the rendered text |
| `text_content()` | `Option<String>`, the DOM text |
| `input_value()` | `Option<String>`, the value of an input, textarea, or select |
| `attribute(name)` | `Option<String>` |
| `bounding_box()` | `Option<BoundingBox>` in viewport pixels |
| `is_attached()`, `is_visible()`, `is_enabled()`, `is_checked()`, `is_focused()` | `bool` |
| `count()` | `usize` |

A read returns `None` when nothing matches. It does not wait. To wait, use an [assertion](waiting-and-assertions.md).
