# The no-subscription design

`Runtime.enable`, `Page.enable`, and `Target.setAutoAttach` subscribe a DevTools client to streams of runtime, page, and target events. By default charm never sends them. `Runtime.enable` and `Page.enable` are sent only when you opt in to [events](events.md).

## What charm refuses

`FORBIDDEN_METHODS` is the list:

```rust
pub const FORBIDDEN_METHODS: &[&str] = &[
    "Runtime.enable",
    "Page.enable",
    "Target.setAutoAttach",
    "Target.sendMessageToTarget",
    "Target.exposeDevToolsProtocol",
    "Target.autoAttachRelated",
];
```

The transport checks every outgoing command against this list before it writes to the websocket. A match returns `Error::Blocked(method)` and nothing is sent. The check covers charm's own code and your calls through `Session::send` and `Session::browser_send`.

The one exception is the event opt-in. `Launch::events(Events { page, runtime })` or `Session::set_events` allows `Page.enable`, `Runtime.enable`, or both for that session. The other methods stay blocked. Every method that is sent is in `sent()`, so a test can still check what the session did.

## Why

Each of the first three methods turns on browser-side work for the client:

- `Runtime.enable` makes the browser report console calls, exceptions, and every new execution context.
- `Page.enable` turns on page lifecycle and frame events.
- `Target.setAutoAttach` makes the client attach to every new frame, worker, and popup as it is created.

The other three methods on the list are other ways to attach to targets or to expose the protocol to them. charm leaves all of them off, so the page runs with that reporting off. The guarantee is narrow on purpose: without an event opt-in, the methods on the list are never sent, and you can test it.

## How to check it

`Session::sent()` returns the set of method names the session has sent, page and browser commands together. Assert against it at the end of a test that does not opt in to events:

```rust
use charm::FORBIDDEN_METHODS;

for &method in FORBIDDEN_METHODS {
    assert!(!session.sent().contains(method), "{method} was sent");
}
```

The repository's live tests do this after driving forms, frames, routes, and response bodies.

## What it means for your code

Without those events, charm works by asking instead of listening:

- Waits and assertions poll. charm evaluates an expression every `Timing::poll` until it holds. There is no event for "element appeared" or "navigation finished".
- `evaluate` runs in the page's default context through `Runtime.evaluate`, which works without `Runtime.enable`. There is no list of execution contexts.
- `goto` sends `Page.navigate` and then polls `document.readyState`. It does not wait for lifecycle events.
- Cross-origin iframes are attached one at a time with `Target.attachToTarget`, when a locator needs one.
- New tabs and popups are found by polling the target list: `pages()` and `wait_for_page()` need no subscription.
- Console messages, page errors, dialogs, and the file picker need events. They work only after the [event opt-in](events.md).
- Downloads are reported by browser-level download events, which are not on the list.
- Network events are opt-in. `network_on()` sends `Network.enable`, which is not on the list. Use it only when you need `wait_for_request`, `wait_for_response`, or `response_body`.

`take_events()` returns the raw events charm has received and not yet consumed, for domains you enable yourself with `send`.

## Cancelling a call

One `Session` reads the DevTools socket only while one of its calls runs, and it answers paused requests while it reads. Do not drop a call's future part way, for example with `tokio::time::timeout` or a `select!` branch: a paused request read in that call may never be answered. Bound waits with `Timing` instead. `Session::reply_failures` counts paused-request replies that Chrome rejected; `Session::dropped_events` counts events dropped from a full queue.
