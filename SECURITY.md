# Security

## Reporting a vulnerability

Report vulnerabilities privately through [GitHub security advisories](https://github.com/manaforged/charm-cdp/security/advisories/new). Do not open a public issue for a vulnerability.

You will get an acknowledgement within 7 days of your report. One maintainer reviews reports and replies in the advisory thread.

## Supported versions

charm-cdp has no release yet. Security fixes go to the `main` branch.

## Scope

- The `charm-cdp` crate: its process launch, its DevTools connection, its request interception, and the data it reads from the page.
- A method in `FORBIDDEN_METHODS` reaching the browser through any public API.

## Out of scope

- Vulnerabilities in Chrome, Chromium, or another browser that charm launches. Report them to the browser vendor.
- A site that tells a charm session apart from a person. That is not a security defect.

## Things to know

- The browser listens for DevTools on `127.0.0.1` with a random port. Any local process that can read the profile directory can find that port and control the browser while the session runs.
- `Launch::arg("--no-sandbox")` turns off Chrome's sandbox. Use it only on hosts where the sandbox cannot start, and only for pages you trust.
- Without `Launch::profile`, charm creates a temporary profile and deletes it on `Session::close`. A session that ends without `close` can leave that directory behind.
- A proxy URL that carries a user name and password (`http://user:pass@host:port`) is answered through request interception. charm does not pass the credentials on the command line.
