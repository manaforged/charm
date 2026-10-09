# The no-subscription design

`Runtime.enable`, `Page.enable`, and `Target.setAutoAttach` subscribe a DevTools client to streams of runtime, page, and target events. charm never sends them.

## What charm refuses

`FORBIDDEN_METHODS` is the list:

```rust
pub const FORBIDDEN_METHODS: [&str; 3] = ["Runtime.enable", "Page.enable", "Target.setAutoAttach"];
```

The transport checks every outgoing command against this list before it writes to the websocket. A match returns `Error::Blocked(method)` and nothing is sent. The check covers charm's own code and your calls through `Session::send` and `Session::browser_send`.

## Why

Each of the three methods turns on browser-side work for the client:

- `Runtime.enable` makes the browser report console calls, exceptions, and every new execution context.
- `Page.enable` turns on page lifecycle and frame events.
- `Target.setAutoAttach` makes the client attach to every new frame, worker, and popup as it is created.

charm leaves all three off, so the page runs with that reporting off. The guarantee is narrow on purpose: these three methods are never sent, and you can test it.

## How to check it

`Session::sent()` returns the set of method names the session has sent, page and browser commands together. Assert against it at the end of a test:

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
- There is no API for console messages, page errors, dialogs, popups, or new tabs, because each needs one of the refused subscriptions.
- Network events are opt-in. `network_on()` sends `Network.enable`, which is not on the list. Use it only when you need `wait_for_request`, `wait_for_response`, or `response_body`.

`take_events()` returns the raw events charm has received and not yet consumed, for domains you enable yourself with `send`.
