# Logins

A site keeps a login in cookies and in `localStorage`. charm can keep both between runs in two ways: a persistent profile directory, or a storage state file.

## A persistent profile

`Launch::profile(dir)` uses `dir` as Chrome's user data directory. Cookies, storage, and saved site settings stay in it after `close`. Chrome writes cookies to disk only every 30 seconds, so `close` also saves them to `charm-cookies.json` in the profile, and the next launch on that profile loads them. `Session::keep_cookies()` writes that file at any time, so a login survives a browser that is killed instead of closed.

```rust
use charm::{Launch, Session};

let launch = Launch::installed()?.profile("./browser-profile");
let mut session = Session::launch(launch).await?;
```

Chrome allows one browser per profile. If another browser holds the profile, `Session::launch` returns `Error::ProfileInUse { path }` before it starts Chrome. Close the other browser, or use another directory.

```rust
use charm::{Error, Launch, Session};

match Session::launch(Launch::installed()?.profile("./browser-profile")).await {
    Err(Error::ProfileInUse { path }) => eprintln!("{} is open in another browser", path.display()),
    other => drop(other?),
}
```

## Storage state

`storage_state()` exports the cookies of the browser and the `localStorage` of each origin that has a cookie or an open tab. It returns a `StorageState { cookies, origins }`. `save(path)` writes it as JSON, and `StorageState::load(path)` reads it back. `Launch::storage_state(state)` puts the cookies and the storage into a new browser before the first navigation.

```rust
use charm::{Launch, Session, StorageState};

let state = session.storage_state().await?;
state.save("state.json")?;
session.close().await?;

let loaded = StorageState::load("state.json")?;
let mut session = Session::launch(Launch::installed()?.storage_state(loaded)).await?;
```

`sessionStorage` and IndexedDB are not part of the state.

### The file is a secret

A storage state file holds session cookies and tokens. Anyone who reads it can act as the logged-in user. On Unix, `save` creates the file with mode `0600`, so only the owner can read it. Keep it out of version control, out of logs, and out of shared directories, and delete it when you no longer need it.

A profile directory holds the same secrets. Treat it the same way.

## Log in by hand, then continue

Some sites need a person to log in: a password manager, a second factor, or a CAPTCHA. Open a visible window, wait for the person to finish, and save the state for later runs.

```rust
use charm::{Launch, Session, Timing, expect_page, TextMatch};
use std::time::Duration;

let timing = Timing { wait: Duration::from_secs(300), ..Timing::default() };
let launch = Launch::installed()?.headed().timing(timing);
let mut session = Session::launch(launch).await?;
session.goto("https://app.example.com/login").await?;

expect_page(&mut session)
    .to_have_url(TextMatch::Contains("/dashboard".into()))
    .await?;

session.storage_state().await?.save("state.json")?;
session.close().await?;
```

The long `Timing::wait` gives the person five minutes. The assertion returns when the URL shows the page after the login. Later runs launch headless with `Launch::storage_state(StorageState::load("state.json")?)`.

`expect_page(..).to_have_url` takes a `TextMatch`. A `Contains` match ignores case.

The export, the reload into a new browser, and the profile lock are covered by the live tests. The headed wait for a person is not covered by a test.

[`examples/keep_login.rs`](https://github.com/manaforged/charm/blob/main/examples/keep_login.rs) exports a state and relaunches with it.
