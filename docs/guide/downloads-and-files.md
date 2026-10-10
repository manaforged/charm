# Downloads and files

## Downloads

`set_downloads(dir)` tells the browser to save downloads in `dir`. charm creates the directory if it does not exist. `wait_for_download()` waits up to `Timing::wait` for a download to complete and returns a `Download`.

```rust
use charm::{Events, Launch, Session};

let launch = Launch::installed()?.events(Events { page: true, ..Events::default() });
let mut session = Session::launch(launch).await?;
session.set_downloads("downloads".into()).await?;
session.goto("https://example.com/reports").await?;
session.click("#export").await?;
let download = session.wait_for_download().await?;
println!("{} -> {:?}", download.suggested_filename, download.path);
```

| `Download` field | Holds |
| --- | --- |
| `url` | The URL the file came from. |
| `suggested_filename` | The name the site gave the file. |
| `path` | Where the file is on disk once it completed, under its suggested name. If that name is taken, charm adds ` (1)`, ` (2)`, and so on. |
| `state` | `DownloadState::InProgress`, `Completed`, or `Canceled`. |
| `guid` | Chrome's id for the download. |

`downloads()` returns the downloads recorded so far without waiting.

Call `set_downloads` before the click that starts the download. The live test runs with the `page` event opt-in; a download without it is not covered by a test.

## Uploads

`set_input_files(selector, files)` sets the files of an `<input type=file>` without opening the file picker. Relative paths are resolved against the current directory.

```rust
use std::path::PathBuf;

session.set_input_files("#avatar", vec![PathBuf::from("avatar.png")]).await?;
```

`Locator::set_input_files(files)` does the same on a locator.

## The file picker

Some pages open the picker from a button and never expose the input. `on_file_chooser(files)` answers every picker that opens after the call with `files`, until you call it again.

```rust
use charm::{Events, Launch, Session};
use std::path::PathBuf;

let launch = Launch::installed()?.events(Events { page: true, ..Events::default() });
let mut session = Session::launch(launch).await?;
session.goto("https://example.com/upload").await?;
session.on_file_chooser(vec![PathBuf::from("report.pdf")]).await?;
session.click("#choose-file").await?;
```

The picker is a page event, so `on_file_chooser` needs the `page` opt-in. Without it, the call returns `Error::EventsRequired("page")`. An empty list turns the interception off. See [Events](events.md) for what the opt-in costs.
