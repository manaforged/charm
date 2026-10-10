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
| `Selector::Ref(id)` | The element with that ref in the latest [snapshot](agents.md) | `"ref=e5"` |

`Selector::parse(raw)` applies the same string rule: `ref=` gives `Ref`, `xpath=` gives `XPath`, and anything else gives `Css`. A plain `&str` converts with the same rule. `Selector::parse_target(raw)` also accepts a bare `e5` or `@e5` as a ref; `Action` targets use it.

The chaining methods below build the other variants: `Within`, `Filtered`, `And`, `Or`, and `Nth`.

## Text matching

`TextMatch` has three forms:

| Form | Build it with | Matches |
| --- | --- | --- |
| `Exact(text)` | `TextMatch::new(text, true)` or `"text".into()` | The whole text |
| `Contains(text)` | `TextMatch::new(text, false)` | A substring, ignoring case |
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

`all()` returns `0..count` as a `Range<usize>`, which you can use in the same loop. `first()` goes back to the first match, and `last()` picks the last one.

## Narrowing and combining

Each of these methods takes the locator and returns a new one, so you can chain them.

| Method | Matches |
| --- | --- |
| `locator(inner)` | Elements that match `inner` inside each match of this locator. |
| `filter(Filter { .. })` | Matches of this locator that pass the filter. |
| `and(other)` | Elements that match both this locator and `other`. |
| `or(other)` | Elements that match either. |
| `last()` | The last match. |

`Filter` has five optional fields. Set the ones you need and leave the rest at `Filter::default()`:

| Field | Keeps an element when |
| --- | --- |
| `has_text: Some(m)` | Its text matches `m`. |
| `has_not_text: Some(m)` | Its text does not match `m`. |
| `has: Some(selector)` | It contains an element that matches `selector`. |
| `has_not: Some(selector)` | It contains no element that matches `selector`. |
| `visible: Some(true)` | It is visible. `Some(false)` keeps hidden elements. |

```rust
use charm::{Filter, TextMatch};

let pear = Filter { has_text: Some(TextMatch::Contains("pear".into())), ..Filter::default() };
session.locator("li").filter(pear).locator("button").click().await?;

let headings_or_items = session.locator("h1").or("li").count().await?;
```

## Action options

`with_options(ActionOptions { force, timeout })` changes how this locator's actions wait.

| Field | Effect |
| --- | --- |
| `timeout: Some(duration)` | Uses `duration` instead of `Timing::wait` for this locator's actions and waits. |
| `force: true` | Skips the actionability checks. charm waits only until the element is attached, scrolls it to the center, and acts. A forced click does not check that it landed. |

```rust
use charm::ActionOptions;
use std::time::Duration;

let options = ActionOptions { timeout: Some(Duration::from_secs(2)), ..ActionOptions::default() };
session.locator("#save").with_options(options).click().await?;
```

`ActionOptions` is not covered by a test.

## Editable elements

`fill`, `type_text`, and `clear` need an element that accepts text: an enabled input, textarea, or `contenteditable` element that is not read-only. Otherwise they return `Error::NotActionable` with `reason: Some(Blocker::NotEditable)`.

## Frames

A locator from `frame_locator` reaches into same-origin and cross-origin iframes, and a ref from a snapshot does the same. See [Frames](frames.md).

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
