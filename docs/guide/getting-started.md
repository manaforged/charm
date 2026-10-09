# Getting started

This page takes you from an empty crate to a program that opens a page, fills a form, clicks a button, and checks the result.

## Add the dependency

charm-cdp is not published on crates.io yet. Add it as a git dependency pinned to a commit, and add tokio for the async runtime:

```toml
[dependencies]
charm-cdp = { git = "https://github.com/manaforged/charm", rev = "<commit>" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The library name is `charm`.

## Find the browser

charm drives a browser that is already installed. `resolve()` returns the first match from this list:

1. The path in `CHROME_EXE`, if that file exists.
2. The path in `CHROMIUM_EXE`, if that file exists.
3. Google Chrome, Chromium, Microsoft Edge, Brave, and Chrome Canary in their standard install paths for macOS, Windows, and Linux (including the Snap Chromium).

`resolve()` returns `None` when it finds nothing. To use another Chromium build, set `CHROME_EXE` or pass the path to `Launch::new` yourself.

## Write the program

```rust
use charm::{Launch, Session, expect};

const FORM: &str = "data:text/html,<title>demo</title><input id=name aria-label=Name><button id=go onclick=\"document.getElementById('out').textContent='hi '+document.getElementById('name').value\">Go</button><div id=out></div>";

#[tokio::main]
async fn main() -> Result<(), charm::Error> {
    let mut session = Session::launch(Launch::installed()?).await?;

    session.goto(FORM).await?;
    session.get_by_label("Name", true).fill("Ada").await?;
    session.click("#go").await?;
    expect(&mut session.locator("#out")).to_have_text("hi Ada").await?;

    session.close().await
}
```

Run it with `cargo run`. The browser starts headless with a temporary profile, and `close` deletes that profile.

## What each call does

- `Session::launch` starts the browser with `--remote-debugging-port=0`, reads the port Chrome picked, connects, and creates one page.
- `goto` navigates and waits until the document reaches `DOMContentLoaded`.
- `get_by_label` and `locator` build a `Locator`. Nothing runs until you call an action on it.
- `fill` and `click` wait until the element can receive input, then send real mouse and keyboard events.
- `expect(...).to_have_text(...)` retries until the text matches. When the time runs out, it returns `Error::Assertion`.

## When the launch fails

| Error | Cause and fix |
| --- | --- |
| `MissingExe` (from `Launch::installed()`) | No browser was found. Set `CHROME_EXE`. |
| `Spawn { path, source }` | The path exists but could not start. Check the path and its permissions. |
| `Exited { code, stderr }` | The browser quit before DevTools was ready. Read `stderr`. On Ubuntu 23.10 and later, AppArmor often blocks Chrome's sandbox; add `Launch::arg("--no-sandbox")` on that host. |
| `Connect(_)` | The browser started but its DevTools endpoint did not answer within `Timing::connect`. |

## Next steps

- [Locators and selectors](locators.md)
- [Waiting and assertions](waiting-and-assertions.md)
- [Launch options and timing](launch-and-timing.md)
