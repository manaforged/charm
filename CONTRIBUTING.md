# Contributing

charm-cdp does not accept external pull requests until its API is more stable. Only accounts with write access can open a pull request. To report a bug or request a feature, open an [issue](https://github.com/manaforged/charm/issues). To report a vulnerability, follow [SECURITY.md](SECURITY.md).

A useful bug report names the Chrome build (`chrome --version`), the operating system, the smallest program that shows the problem, and the `Error` value it returned.

The rest of this guide is the maintainers' workflow.

## Before you open a pull request

1. Run the default gates:

   ```sh
   ./scripts/verify.sh
   ```

   With no flag, the script runs `comments` (the comment lint), `fmt`, `clippy` with warnings denied, `test`, `msrv` (`cargo check` on the `rust-version` in `Cargo.toml`), `api` (`cargo truesight check`), and `deny` (`cargo deny check`). `--quick` runs only `comments`, `fmt`, `clippy`, and `test`. `--only` takes a comma-separated list of gates.

2. If the change touches the transport, a locator, input, frames, or the network code, add the live suite:

   ```sh
   ./scripts/verify.sh --live
   ```

   The live gate runs `cargo test --test live -- --ignored` against the installed Chrome. Set `CHROME_EXE` to choose the browser. On a Linux host where Chrome's sandbox cannot start, set `CHARM_CHROME_NO_SANDBOX=1`.

3. Read the result. A failed gate exits 1. When a gate skips itself because its tool is not installed, the run ends with "Verify incomplete" and exit code 2, and it does not count as a pass. In CI a skip is a failure.

4. If the change adds, removes, or changes a public item, run `cargo truesight sync` and commit `api/` and `docs/reference/`.

5. Add a line under `Unreleased` in `CHANGELOG.md` when the change is visible to a user of the crate.

The gates need Python 3, git, and rustup. Install `cargo-truesight` with `cargo install --locked --git https://github.com/manaforged/truesight` and `cargo-deny` with `cargo install --locked cargo-deny`.

## Style

Rust code is formatted by `rustfmt` with the repository settings. The comment lint allows one-line rustdoc and `// SAFETY:` comments only. Unit tests live in a `*_tests.rs` file beside the module. Tests that launch Chrome live in `tests/live.rs` and are marked `#[ignore]`.

Keep the guarantee: no code path may send a method in `FORBIDDEN_METHODS`. A live test that drives a new feature asserts that `Session::sent()` contains none of them.
