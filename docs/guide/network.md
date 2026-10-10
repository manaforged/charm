# Network

charm covers four network jobs: routing requests, waiting for traffic, reading response bodies, and managing cookies.

## URL matching

Routes and network waits take a `UrlMatch`:

| Variant | Matches |
| --- | --- |
| `UrlMatch::Exact(url)` | The full URL. |
| `UrlMatch::Contains(part)` | Any URL that contains `part`. |
| `UrlMatch::Glob(pattern)` | A glob: `*` matches within one path segment, `**` matches across `/`, and `?` matches one character. |

`UrlMatch::matches(url)` runs the same test in Rust, so you can check a pattern without a browser.

## Routes

`route(url, action)` handles every request whose URL matches. Routes are checked in the order you added them, and the first match wins. A request that matches no route continues unchanged. Routes apply to every tab of the session, including tabs opened after the call.

```rust
use charm::{ErrorReason, RouteAction, UrlMatch};
use std::collections::BTreeMap;

session
    .route(
        UrlMatch::Contains("/api".into()),
        RouteAction::Fulfill {
            status: 200,
            headers: BTreeMap::from([("content-type".into(), "application/json".into())]),
            body: b"{\"routed\":true}".to_vec(),
        },
    )
    .await?;
session
    .route(UrlMatch::Contains("/ads/".into()), RouteAction::Abort(ErrorReason::BlockedByClient))
    .await?;
```

| Action | Effect |
| --- | --- |
| `Fulfill { status, headers, body }` | Answers the request without sending it. |
| `Abort(ErrorReason)` | Fails the request with that network error. The page sees a failed fetch or a broken resource. |
| `Continue { url, method, headers, post_data }` | Sends the request, with each field that is `Some` replacing the original. |

`unroute(url)` removes every route whose `UrlMatch` equals `url`.

Routes use the `Fetch` domain. Paused requests are answered while charm reads from the browser, which happens during any `Session` call. If your program spends a long time outside charm calls, a page request can wait until the next call.

`block_urls(&patterns)` is a lighter way to block requests. It passes the patterns to `Network.setBlockedURLs`, which uses Chrome's own wildcard syntax.

## Waiting for requests and responses

Call `network_on()` before the action that starts the request. It sends `Network.enable`, and charm records network events from that point. A request that happened earlier is not recorded.

```rust
use charm::UrlMatch;

session.network_on().await?;
session.click("#load").await?;
let response = session.wait_for_response(UrlMatch::Contains("/data".into())).await?;
assert_eq!(response.status, 200);
let body = session.response_body(&response).await?;
```

`wait_for_request` returns an `ObservedRequest` with the `request_id`, `url`, `method`, `headers`, and `resource_type`. `wait_for_response` returns an `ObservedResponse` with the `request_id`, `url`, `status`, `headers`, and `resource_type`. Header names are lowercase. Each wait takes the first recorded event that matches and removes it, so two waits for the same URL return two different requests. Both waits call `network_on` for you, but call it yourself first so that the event is not missed.

`response_body(&response)` waits until the response has finished loading, then returns the body as bytes. Base64 bodies are decoded.

charm keeps up to 4096 unread events. When the queue is full, charm drops the oldest event, and `dropped_events()` counts it. On long sessions with network events on, read the events you need, or call `take_events()` to drain them.

## Cookies

| Method | Effect |
| --- | --- |
| `cookies()` | Every cookie in the browser, as `Vec<Cookie>`. |
| `cookie_named(name)` | The first `Cookie` with that name, or `None`. |
| `add_cookies(vec![Cookie { .. }])` | Sets typed cookies. |
| `clear_cookies()` | Deletes every cookie. |

```rust
use charm::{Cookie, SameSite};

session
    .add_cookies(vec![Cookie {
        name: "session".into(),
        value: "abc".into(),
        url: Some("https://example.com/".into()),
        secure: true,
        same_site: Some(SameSite::Lax),
        ..Cookie::default()
    }])
    .await?;
```

A cookie needs a `url`, or a `domain` and `path`. To keep cookies between runs, launch with `Launch::profile(dir)`, or save a storage state. See [Logins](logins.md).

## Headers, throttling, and offline

`set_extra_headers`, `set_throttle`, and `set_offline` change every request of the active tab. See [Emulation and PDF](emulation-and-pdf.md).
