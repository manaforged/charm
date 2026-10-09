# Frames

Use `frame_locator` to reach an element inside an iframe. It takes a selector for the `<iframe>` element and ends with `.locator(...)` for the element inside it.

```rust
use charm::expect;

let mut same = session.frame_locator("#same").locator("#sbtn");
same.click().await?;
expect(&mut same).to_have_text("done").await?;

session.frame_locator("#cross").locator("#inner").click().await?;
expect(&mut session.frame_locator("#cross").locator("#fout")).to_have_text("clicked").await?;
```

The result is an ordinary `Locator`, so every action, read, and assertion works on it. Chain `frame_locator` on a `FrameLocator` to go through nested iframes:

```rust
session.frame_locator("#outer").frame_locator("#inner").locator("button").click().await?;
```

## Same-origin and cross-origin frames

A same-origin iframe runs in the page's process, and charm evaluates its locator through the iframe's `contentDocument`.

Chrome runs a cross-origin iframe in a separate target. charm finds that target with `Target.getTargets` and attaches to it with `Target.attachToTarget` when a locator needs it. It does not use `Target.setAutoAttach`. Mouse events still go to the page, at the element's position in the page's coordinates, so a click inside a cross-origin iframe is a real click at that point.

## Listing frames

`frames()` returns the frame tree as a flat `Vec<FrameInfo>`, parent before children. Each entry has the frame `id`, its `url`, its `name` if it has one, and its `parent_id` (`None` for the main frame).

```rust
for frame in session.frames().await? {
    println!("{} {} parent={:?}", frame.id, frame.url, frame.parent_id);
}
```
