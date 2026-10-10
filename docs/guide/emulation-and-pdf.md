# Emulation and PDF

The emulation methods change how the page sees the browser: screen size, device, language, time zone, location, color scheme, network speed, and request headers. Each method sends one DevTools command to the active tab. Whether a setting carries over to tabs opened later is not covered by a test; call the method again after `new_page` or `select_page` if you need it there.

## Devices

`emulate(device)` sets the viewport, touch support, and user agent of a preset device.

```rust
use charm::Device;

session.emulate(Device::Iphone15).await?;
session.goto("https://example.com/").await?;
```

| `Device` | Name for `Device::named` |
| --- | --- |
| `Device::Iphone15` | `iPhone 15` |
| `Device::Pixel8` | `Pixel 8` |
| `Device::Ipad` | `iPad` |
| `Device::Desktop1080p` | `Desktop 1080p` |

`Device::named(name)` finds a preset by name, ignoring case. `Device::spec()` returns its `DeviceSpec` with the `name`, `viewport`, `user_agent`, and `touch` flag. The live tests cover `Device::Iphone15`.

## Settings

| Method | Effect |
| --- | --- |
| `set_viewport(Viewport { width, height, device_scale_factor, mobile })` | Sets the layout size in CSS pixels and the pixel ratio. |
| `clear_viewport()` | Removes the viewport override. |
| `set_user_agent(ua)` | Replaces the `User-Agent` header and `navigator.userAgent`. |
| `set_locale(locale)` | Sets the locale for `Intl` and `navigator.language`, for example `"de-DE"`. |
| `set_timezone(zone)` | Sets the IANA time zone, for example `"Europe/Berlin"`. |
| `set_geolocation(Some(Geolocation { latitude, longitude, accuracy }))` | Sets the position the Geolocation API returns. `None` removes it. The page still needs the geolocation permission. |
| `set_color_scheme(ColorScheme::Dark)` | Sets `prefers-color-scheme` to `Dark`, `Light`, or `NoPreference`. |
| `set_throttle(Throttle::Slow3g)` | Slows the network. The presets are `None`, `Slow3g`, `Fast3g`, and `Slow4g`; `Throttle::Custom(Conditions { latency_ms, download_bytes_per_sec, upload_bytes_per_sec })` sets your own. |
| `set_offline(true)` | Fails every request as if the network were down. |
| `set_extra_headers(headers)` | Adds these headers to every request. |
| `set_javascript_enabled(false)` | Stops scripts from running. charm's own reads also use scripts, so most locators and waits stop working until you turn scripts back on. |

`Throttle::conditions()` returns the numbers behind a preset.

Only `emulate(Device::Iphone15)` is covered by a test. The other settings are not covered by a test.

## PDF

`pdf(options)` prints the active tab to PDF and returns the bytes. It works in headless mode.

```rust
use charm::{Margins, PaperSize, PdfOptions};

let bytes = session
    .pdf(PdfOptions {
        paper: PaperSize::A4,
        print_background: true,
        margins: Some(Margins { top: 0.5, right: 0.5, bottom: 0.5, left: 0.5 }),
        ..PdfOptions::default()
    })
    .await?;
std::fs::write("page.pdf", bytes)?;
```

| Field | Default | Effect |
| --- | --- | --- |
| `paper` | `PaperSize::Letter` | `Letter`, `Legal`, `Tabloid`, `A3`, `A4`, or `A5`. `PaperSize::inches()` returns the size. |
| `landscape` | `false` | Turns the page sideways. |
| `print_background` | `false` | Prints background colors and images. |
| `scale` | `None` | Scales the content. Chrome's default is 1. |
| `margins` | `None` | Margins in inches. `None` keeps Chrome's default margins. |
| `page_ranges` | `None` | Pages to print, for example `"1-3, 5"`. |

The live tests cover `pdf(PdfOptions::default())`.
