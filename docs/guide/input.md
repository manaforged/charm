# Input and human motion

charm sends input with `Input.dispatchMouseEvent` and `Input.dispatchKeyEvent`, the same path a physical mouse and keyboard take into the page. It does not call `element.click()` or set `value` from JavaScript.

## Element actions

| Method | What it does |
| --- | --- |
| `click()` | Waits until the element is actionable, then presses and releases the left button at the center of its box. |
| `dblclick()` | The same, with two presses. |
| `click_with(ClickOptions { .. })` | A click with another button, click count, held modifiers, or offset. |
| `check()`, `uncheck()`, `set_checked(bool)` | Clicks a checkbox, radio, or switch only if its state differs, then checks that the state changed. |
| `select_option(SelectBy)` | Picks options in a `<select>` and returns the values now selected. |
| `drag_to(target)`, `drag_to_point(x, y)` | Presses on the element, moves to the target, and releases. |
| `focus()`, `blur()` | Focuses or blurs the element from JavaScript. |
| `press(key)` | Focuses the element, then presses `key`. |
| `clear()` | `fill("")`. |
| `set_input_files(files)` | Sets the files of a file input. See [Downloads and files](downloads-and-files.md). |
| `hover()` | Waits until the element is actionable, then moves the mouse to its center. |
| `fill(text)` | Clicks the element, selects its content, presses Backspace, and types `text`. |
| `type_text(text)` | Clicks the element and types `text` key by key after the existing content. |

The same names on `Session` (`session.click("#go")`, `session.fill("#name", "Ada")`, `session.check("#keep")`, `session.select_option(..)`, `session.dblclick(..)`, `session.drag_and_drop(from, to)`, `session.focus(..)`, `session.uncheck(..)`, `session.set_input_files(..)`) build a locator from the selector and call the action.

### Clicks that land

Before a click, charm listens for the next `pointerdown` on the element. After the click, it checks that the event reached the element or one of its children. If another element took the click, for example an overlay that appeared at the last moment, charm waits `Timing::poll` and tries again. When `Timing::wait` runs out, it returns `Error::NotActionable` with `reason: Some(Blocker::Obscured)` and `covered_by` naming the element on top. A click with `ActionOptions { force: true, .. }` skips this check.

### Forms

```rust
use charm::SelectBy;

session.check("#remember").await?;
let picked = session.select_option("#plan", SelectBy::Value(vec!["pro".into()])).await?;
assert_eq!(picked, vec!["pro".to_owned()]);
```

`SelectBy::Value`, `SelectBy::Label`, and `SelectBy::Index` each take a list, so a multi-select can pick several options. `select_option` waits until every requested option exists.

### Click options and modifiers

```rust
use charm::{ClickOptions, Modifiers, MouseButton};

let options = ClickOptions {
    button: MouseButton::Left,
    modifiers: Modifiers::CONTROL | Modifiers::SHIFT,
    ..ClickOptions::default()
};
session.locator("a.result").click_with(options).await?;
```

| `ClickOptions` field | Effect |
| --- | --- |
| `button` | `MouseButton::Left`, `Right`, or `Middle`. |
| `count` | The number of presses. `2` is a double click. |
| `modifiers` | Keys held during the click: `Modifiers::ALT`, `CONTROL`, `META`, `SHIFT`, combined with the bitwise OR operator. `Modifiers::NONE` holds none. |
| `position` | An `(x, y)` offset from the top-left corner of the element. `None` clicks the center. |

`session.click_at(x, y, options)` sends the same click at a viewport point without a locator.

`dblclick`, `click_with`, `Modifiers`, and the drag methods are not covered by a test.

### Actionability

An element is actionable when it is attached, has a non-zero size, is visible, is inside the viewport, is not moving, is enabled, and is the top element at its center point. charm scrolls it into view as needed. `click` waits up to `Timing::wait` and then returns `Error::NotActionable` with the reason. `fill` and `type_text` also need an editable element.

To check without failing, use `settle` or `try_click`. Both wait only up to `Timing::actionable`:

```rust
use charm::Blocker;

let result = session.locator("#buy").try_click().await?;
if !result.clicked {
    match result.reason {
        Some(Blocker::Obscured) => println!("covered by {:?}", result.covered_by),
        Some(Blocker::Disabled) => println!("disabled"),
        other => println!("not clicked: {other:?}"),
    }
}
```

`Blocker` has the variants `NoMatch`, `Detached`, `ZeroSize`, `NotVisible`, `Offscreen`, `Unstable`, `Disabled`, `NotEditable`, and `Obscured`.

## Raw input

| Method | What it does |
| --- | --- |
| `mouse_move(x, y)` | Moves the mouse to a viewport point. |
| `mouse_click(x, y, MouseButton::Left)` | Moves, presses, and releases. `Right` and `Middle` also exist. |
| `wheel(delta_x, delta_y)` | Sends one wheel event at the center of the viewport. |
| `scroll(delta_y)` | A vertical wheel event. |
| `press(combo)` | Presses a key or a chord. |

`press` takes a key name or a single character, optionally after modifiers joined with `+`: `"Enter"`, `"a"`, `"Control+a"`, `"Shift+Tab"`, `"Control+Shift+ArrowLeft"`. The modifiers are `Alt`, `Control`, `Meta`, and `Shift`. The named keys are `Enter`, `Tab`, `Backspace`, `Delete`, `Escape`, `Space`, `ArrowLeft`, `ArrowRight`, `ArrowUp`, `ArrowDown`, `Home`, and `End`.

## The `human` module

`charm::human` generates input that varies the way a person's does: curved cursor paths with sub-movements and corrections, and keystroke delays with bursts, pauses, and corrected typos. It is a set of pure functions. It does not touch a session, and `click` and `fill` do not use it. You replay its output with the raw input methods.

Every function takes a `human::Rng`. `Rng::new(seed)` gives the same output for the same seed, which makes a run repeatable. `Rng::from_entropy()` gives a different run each time.

### Cursor paths

`human::path(from, to, &options, &mut rng)` returns a `Vec<Step>`. Each `Step` has a `point` and an `after` delay. `to` is a `Target::Point` or a `Target::Rect`; for a rectangle, the path ends at a point inside it.

```rust
use charm::human::{self, CursorOptions, Point, Rect, Rng, Target};

let bounds = session.locator("#go").bounding_box().await?.expect("the button exists");
let target = Target::Rect(Rect { x: bounds.x, y: bounds.y, width: bounds.width, height: bounds.height });
let mut rng = Rng::new(7);
let steps = human::path(Point { x: 10.0, y: 10.0 }, target, &CursorOptions::default(), &mut rng);

for step in &steps {
    session.mouse_move(step.point.x, step.point.y).await?;
    session.wait_for_timeout(step.after).await;
}
let last = steps.last().expect("a path has steps").point;
session.mouse_click(last.x, last.y, charm::MouseButton::Left).await?;
```

`CursorOptions` holds the model's parameters: Fitts's-law timing (`fitts_a_ms`, `fitts_b_ms`), sub-movement counts and shares, path bow and skew, tremor, stalls, and the settle nudges at the end. `CursorOptions::default()` is the tuned starting point. `human::target_point(rect, padding_percentage, &options, &mut rng)` picks only the landing point inside a rectangle.

### Keystrokes

`human::keystrokes(text, &options, &mut rng)` returns a `Vec<Keystroke>`. Each has a `key` (`Key::Char`, `Space`, `Enter`, `Tab`, or `Backspace`) and a `before` delay. With the default options, some keystrokes are typos followed by `Backspace` and the right key, so the keys you replay produce `text` only when every typo is corrected.

```rust
use charm::human::{self, Key, Rng, TypingOptions};

session.locator("#name").click().await?;
let options = TypingOptions::default().ensure_correction();
for stroke in human::keystrokes("hello world", &options, &mut Rng::new(7)) {
    session.wait_for_timeout(stroke.before).await;
    let name = match stroke.key {
        Key::Char(c) => c.to_string(),
        Key::Space => "Space".into(),
        Key::Enter => "Enter".into(),
        Key::Tab => "Tab".into(),
        Key::Backspace => "Backspace".into(),
    };
    session.press(&name).await?;
}
```

`TypingOptions::ensure_correction()` sets the error detection rate to 1, so every typo is fixed. `TypingOptions::typo_free()` turns typos off. The other fields set the words-per-minute rate, burst and pause behaviour, key-distance effects, and fatigue.
