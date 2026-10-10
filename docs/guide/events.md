# Events

By default charm subscribes to no page or runtime events (see [The no-subscription design](no-subscription.md)). Some jobs need them: answering dialogs, reading console messages, and answering the file picker. `Events` turns them on, one group at a time.

## Opting in

```rust
use charm::{DialogPolicy, Events, Launch, Session};

let launch = Launch::installed()?
    .events(Events::all())
    .dialogs(DialogPolicy::Accept);
let mut session = Session::launch(launch).await?;
```

| Field | Sends | Turns on |
| --- | --- | --- |
| `Events::page` | `Page.enable` | Dialog handling, `dialogs()`, `on_file_chooser`, and `Outcome::dialogs` |
| `Events::runtime` | `Runtime.enable` | `console()` and `Outcome::console` |

`Events::all()` sets both. `Events::default()` sets neither. The opt-in applies to every tab, including tabs opened later.

`Session::set_events(events, policy)` changes the opt-in on a running session, for example after `Session::connect`. It is not covered by a test.

With an opt-in, the matching method is allowed and appears in `sent()`. The other methods in `FORBIDDEN_METHODS` stay blocked.

## What it costs

`Page.enable` and `Runtime.enable` are the two methods that charm leaves off by default. A page can detect a client that has `Runtime.enable` on, for example through the way Chrome serializes console arguments. Turn on only the group you need, and only for the sessions that need it. On sites that check for automation, test with events off first.

## Dialogs

`alert`, `confirm`, `prompt`, and `beforeunload` dialogs stop the page until someone answers them. With the `page` opt-in, charm answers each dialog as it opens, by the `DialogPolicy` you passed to `Launch::dialogs`:

| Policy | Answer |
| --- | --- |
| `DialogPolicy::Dismiss` (the default) | Presses Cancel. |
| `DialogPolicy::Accept` | Presses OK. |
| `DialogPolicy::AcceptWith(text)` | Presses OK and types `text` into a prompt. |

`dialogs()` returns the dialogs answered since the last call and clears the list. Each `Dialog` has a `kind` (`DialogKind::Alert`, `Confirm`, `Prompt`, or `BeforeUnload`), the `message`, the page `url`, and the `default_prompt`.

```rust
use charm::DialogKind;

session.click("#delete").await?;
for dialog in session.dialogs() {
    if dialog.kind == DialogKind::Confirm {
        println!("answered: {}", dialog.message);
    }
}
```

Without the `page` opt-in, charm does not see or answer dialogs, and an open dialog keeps blocking the page. If a page you drive opens dialogs, turn on `page`.

The live test covers `Accept` on an `alert`. `Dismiss` and `AcceptWith` are not covered by a test.

## Console messages

With the `runtime` opt-in, `console()` returns the console messages and uncaught exceptions since the last call and clears the list. Each `ConsoleMessage` has a `level`, the `text`, and the `url` and `line` of the call.

| `ConsoleLevel` | From |
| --- | --- |
| `Debug` | `console.debug`, `console.trace` |
| `Info` | `console.info` |
| `Log` | `console.log` and other calls |
| `Warning` | `console.warn` |
| `Error` | `console.error`, a failed `console.assert` |
| `Exception` | An uncaught exception |

## Limits

charm keeps up to 1024 dialogs, 1024 console messages, and 1024 downloads. When a list is full, the oldest record is dropped. `dropped_records()` returns how many records were dropped this way. Read the lists often on long sessions.

`dropped_events()` is a different count: raw CDP events dropped from the unread event queue, which holds up to 4096 events.

`act` drains `dialogs()` and `console()` after it runs its action, so its outcome also holds any records you had not read. Call them before `act` to keep earlier records out of the outcome.
