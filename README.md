# charm-cdp

Browser automation over the Chrome DevTools Protocol, written in Rust.

- **Waits for you**: locators wait until an element is visible, stable, and clickable, and assertions retry.
- **No downloads**: no Node runtime and no bundled browser. It drives the Chrome, Edge, or Brave on the machine.
- **Quiet**: it never turns on runtime or page events (`Runtime.enable`, `Page.enable`) unless you ask.
- **Agent-ready**: accessibility snapshots with element refs, and a JSON action format for models.
- **Cleans up**: a browser it launches exits with your program, even after a crash.

[Getting started](docs/guide/getting-started.md) | [Guides](#guides) | [Examples](examples) | [API by job](AGENTS.md)

Status: unreleased. The API is not stable and will change before the first release. External pull requests are not accepted yet; open an [issue](https://github.com/manaforged/charm/issues) for bugs and requests.

## Requirements

- Rust 1.96 or newer.
- An installed Chrome or Chromium. `Launch::installed()` finds Google Chrome, Chromium, Microsoft Edge, Brave, or Chrome Canary in their standard install paths on macOS, Windows, and Linux. For another build or a build in another path, set `CHROME_EXE` (or `CHROMIUM_EXE`) to the executable, or pass the path to `Launch::new`.
- On Linux, Ubuntu 23.10 and later restrict unprivileged user namespaces with AppArmor, and Chrome's sandbox can fail to start. If the launch fails with `Error::Exited`, add `Launch::arg("--no-sandbox")` on that host.

## Install

The crate is not published on crates.io yet. Until the first release, add it as a git dependency pinned to a commit:

```toml
[dependencies]
charm-cdp = { git = "https://github.com/manaforged/charm", rev = "<commit>" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Replace `<commit>` with the commit hash you tested against. The library name is `charm`, so you import it as `use charm::...`.

## Usage

```rust
use charm::{Launch, Session, expect};

#[tokio::main]
async fn main() -> Result<(), charm::Error> {
    let mut session = Session::launch(Launch::installed()?).await?;

    session
        .goto("data:text/html,<input id=name aria-label=Name><button id=go onclick=\"document.getElementById('out').textContent='hi '+document.getElementById('name').value\">Go</button><div id=out></div>")
        .await?;
    session.fill("#name", "Ada").await?;
    session.click("#go").await?;
    expect(&mut session.locator("#out")).to_have_text("hi Ada").await?;
    session.close().await
}
```

`click` waits until the element is attached, visible, stable, enabled, and not covered, then sends a real mouse event at its center. `expect` retries until the text matches or `Timing::wait` runs out, and then returns `Error::Assertion` with the expected and received values.

Four runnable programs are in [`examples/`](examples): `cargo run --example search_form`, `cargo run --example intercept`, `cargo run --example agent_loop`, and `cargo run --example keep_login`.

## For agents

When a language model drives the browser, give it a snapshot instead of HTML. `snapshot` returns the page's accessibility tree as text, with a ref on each element. The model answers with a JSON action that names a ref, and `act` runs it and returns what changed.

```rust
use charm::{ActReport, Action, SnapshotOptions};

let view = session.snapshot(SnapshotOptions::default()).await?.text();
let action: Action = serde_json::from_str(r#"{"action":"click","target":"e5"}"#)?;
let outcome = session.act(action, ActReport::Delta).await?;
println!("{}", outcome.snapshot.unwrap_or_default());
```

A snapshot line looks like `- button "Sign in" [ref=e5]`. A ref also works as a selector: `session.click(Selector::parse("ref=e5"))`. If the element is gone, the call returns `Error::StaleRef`, and the model takes a new snapshot. [Agents](docs/guide/agents.md) has the full action format and the options that keep snapshots small.

## No event subscriptions by default

`FORBIDDEN_METHODS` lists the methods charm does not send. `Session::send` and `Session::browser_send` return `Error::Blocked` for them, so your own raw CDP calls cannot send them by mistake either. `Session::sent()` returns the set of every method name the session has sent. Assert against it in your tests:

```rust
for method in charm::FORBIDDEN_METHODS {
    assert!(!session.sent().contains(method), "{method} was sent");
}
```

Without those subscriptions, charm reads page state by evaluating expressions and polling, not by listening for runtime or page events. The one exception is the [event opt-in](docs/guide/events.md): `Launch::events` allows `Page.enable` or `Runtime.enable` for that session, and `sent()` then shows them. [The no-subscription design](docs/guide/no-subscription.md) explains what that means for your code.

## Limits

- Chromium-based browsers only: Chrome, Chromium, Edge, Brave. No Firefox or WebKit.
- No trace viewer, video recording, test runner, or code generator. Tests run under `cargo test`.
- Dialogs, console messages, and the file chooser work only with the event opt-in.
- There is one browser context per session. Tabs share cookies and storage.
- Network events (`wait_for_request`, `wait_for_response`) start at `network_on()`. A request that finished before that call is not seen.
- `goto` waits for `DOMContentLoaded`, not for `load` or network idle. Call `wait_for_load_state(LoadState::Load)` when you need it.

## Guides

- [Getting started](docs/guide/getting-started.md)
- [Locators and selectors](docs/guide/locators.md)
- [Input and human motion](docs/guide/input.md)
- [Waiting and assertions](docs/guide/waiting-and-assertions.md)
- [Agents](docs/guide/agents.md)
- [Frames](docs/guide/frames.md)
- [Network](docs/guide/network.md)
- [Tabs and connect](docs/guide/tabs-and-connect.md)
- [Logins](docs/guide/logins.md)
- [Downloads and files](docs/guide/downloads-and-files.md)
- [Events](docs/guide/events.md)
- [Emulation and PDF](docs/guide/emulation-and-pdf.md)
- [Launch options and timing](docs/guide/launch-and-timing.md)
- [The no-subscription design](docs/guide/no-subscription.md)
- [AGENTS.md](AGENTS.md): the API by job, for coding agents

## License

MIT. See [LICENSE](LICENSE).
