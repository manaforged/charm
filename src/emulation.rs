use crate::error::Error;
use crate::session::Session;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f64,
    pub mobile: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceSpec {
    pub name: &'static str,
    pub viewport: Viewport,
    pub touch: bool,
    pub user_agent: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Device {
    Iphone15,
    Pixel8,
    Ipad,
    Desktop1080p,
}

const DEVICES: [(Device, DeviceSpec); 4] = [
    (
        Device::Iphone15,
        DeviceSpec {
            name: "iPhone 15",
            viewport: Viewport {
                width: 393,
                height: 852,
                device_scale_factor: 3.0,
                mobile: true,
            },
            touch: true,
            user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
        },
    ),
    (
        Device::Pixel8,
        DeviceSpec {
            name: "Pixel 8",
            viewport: Viewport {
                width: 412,
                height: 915,
                device_scale_factor: 2.625,
                mobile: true,
            },
            touch: true,
            user_agent: "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Mobile Safari/537.36",
        },
    ),
    (
        Device::Ipad,
        DeviceSpec {
            name: "iPad",
            viewport: Viewport {
                width: 820,
                height: 1180,
                device_scale_factor: 2.0,
                mobile: true,
            },
            touch: true,
            user_agent: "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
        },
    ),
    (
        Device::Desktop1080p,
        DeviceSpec {
            name: "Desktop 1080p",
            viewport: Viewport {
                width: 1920,
                height: 1080,
                device_scale_factor: 1.0,
                mobile: false,
            },
            touch: false,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
        },
    ),
];

impl Device {
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        DEVICES
            .iter()
            .find(|(_, spec)| spec.name.eq_ignore_ascii_case(name))
            .map(|(device, _)| *device)
    }

    #[must_use]
    pub fn spec(self) -> DeviceSpec {
        DEVICES
            .iter()
            .find(|(device, _)| *device == self)
            .map_or(DEVICES[0].1, |(_, spec)| *spec)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geolocation {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ColorScheme {
    #[default]
    NoPreference,
    Light,
    Dark,
}

impl ColorScheme {
    fn name(self) -> &'static str {
        match self {
            Self::NoPreference => "no-preference",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Conditions {
    pub latency_ms: f64,
    pub download_bytes_per_sec: f64,
    pub upload_bytes_per_sec: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[non_exhaustive]
pub enum Throttle {
    #[default]
    None,
    Slow3g,
    Fast3g,
    Slow4g,
    Custom(Conditions),
}

const THROTTLES: [(Throttle, Conditions); 4] = [
    (
        Throttle::None,
        Conditions {
            latency_ms: 0.0,
            download_bytes_per_sec: -1.0,
            upload_bytes_per_sec: -1.0,
        },
    ),
    (
        Throttle::Slow3g,
        Conditions {
            latency_ms: 2000.0,
            download_bytes_per_sec: 50_000.0,
            upload_bytes_per_sec: 50_000.0,
        },
    ),
    (
        Throttle::Fast3g,
        Conditions {
            latency_ms: 562.5,
            download_bytes_per_sec: 180_000.0,
            upload_bytes_per_sec: 84_375.0,
        },
    ),
    (
        Throttle::Slow4g,
        Conditions {
            latency_ms: 150.0,
            download_bytes_per_sec: 400_000.0,
            upload_bytes_per_sec: 400_000.0,
        },
    ),
];

impl Throttle {
    #[must_use]
    pub fn conditions(self) -> Conditions {
        if let Self::Custom(conditions) = self {
            return conditions;
        }
        THROTTLES
            .iter()
            .find(|(preset, _)| *preset == self)
            .map_or(THROTTLES[0].1, |(_, c)| *c)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum PaperSize {
    #[default]
    Letter,
    Legal,
    Tabloid,
    A3,
    A4,
    A5,
}

impl PaperSize {
    #[must_use]
    pub fn inches(self) -> (f64, f64) {
        match self {
            Self::Letter => (8.5, 11.0),
            Self::Legal => (8.5, 14.0),
            Self::Tabloid => (11.0, 17.0),
            Self::A3 => (11.69, 16.54),
            Self::A4 => (8.27, 11.69),
            Self::A5 => (5.83, 8.27),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Margins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PdfOptions {
    pub landscape: bool,
    pub print_background: bool,
    pub scale: Option<f64>,
    pub paper: PaperSize,
    pub margins: Option<Margins>,
    pub page_ranges: Option<String>,
}

impl Session {
    pub async fn set_viewport(&mut self, viewport: Viewport) -> Result<(), Error> {
        self.emulate_cdp(
            "Emulation.setDeviceMetricsOverride",
            json!({
                "width": viewport.width,
                "height": viewport.height,
                "deviceScaleFactor": viewport.device_scale_factor,
                "mobile": viewport.mobile,
            }),
        )
        .await
    }

    pub async fn clear_viewport(&mut self) -> Result<(), Error> {
        self.emulate_cdp("Emulation.clearDeviceMetricsOverride", json!({}))
            .await
    }

    pub async fn emulate(&mut self, device: Device) -> Result<(), Error> {
        let spec = device.spec();
        self.set_viewport(spec.viewport).await?;
        self.emulate_cdp(
            "Emulation.setTouchEmulationEnabled",
            json!({ "enabled": spec.touch, "maxTouchPoints": if spec.touch { 5 } else { 1 } }),
        )
        .await?;
        self.set_user_agent(spec.user_agent).await
    }

    pub async fn set_user_agent(&mut self, user_agent: &str) -> Result<(), Error> {
        self.emulate_cdp(
            "Emulation.setUserAgentOverride",
            json!({ "userAgent": user_agent }),
        )
        .await
    }

    pub async fn set_locale(&mut self, locale: &str) -> Result<(), Error> {
        self.emulate_cdp("Emulation.setLocaleOverride", json!({ "locale": locale }))
            .await
    }

    pub async fn set_timezone(&mut self, timezone: &str) -> Result<(), Error> {
        self.emulate_cdp(
            "Emulation.setTimezoneOverride",
            json!({ "timezoneId": timezone }),
        )
        .await
    }

    pub async fn set_geolocation(&mut self, location: Option<Geolocation>) -> Result<(), Error> {
        let Some(location) = location else {
            return self
                .emulate_cdp("Emulation.clearGeolocationOverride", json!({}))
                .await;
        };
        let origin = self.evaluate("location.origin").await?;
        let mut grant = json!({ "permissions": ["geolocation"] });
        if let Some(origin) = origin.as_str().filter(|o| *o != "null") {
            grant["origin"] = json!(origin);
        }
        drop(self.browser_send("Browser.grantPermissions", grant).await?);
        self.emulate_cdp(
            "Emulation.setGeolocationOverride",
            json!({
                "latitude": location.latitude,
                "longitude": location.longitude,
                "accuracy": location.accuracy,
            }),
        )
        .await
    }

    pub async fn set_color_scheme(&mut self, scheme: ColorScheme) -> Result<(), Error> {
        self.emulate_cdp(
            "Emulation.setEmulatedMedia",
            json!({ "features": [{ "name": "prefers-color-scheme", "value": scheme.name() }] }),
        )
        .await
    }

    pub async fn set_offline(&mut self, offline: bool) -> Result<(), Error> {
        self.emulation_network(offline, Throttle::None).await
    }

    pub async fn set_throttle(&mut self, throttle: Throttle) -> Result<(), Error> {
        self.emulation_network(false, throttle).await
    }

    pub async fn set_extra_headers(
        &mut self,
        headers: BTreeMap<String, String>,
    ) -> Result<(), Error> {
        self.emulate_cdp("Network.setExtraHTTPHeaders", json!({ "headers": headers }))
            .await
    }

    pub async fn set_javascript_enabled(&mut self, enabled: bool) -> Result<(), Error> {
        self.emulate_cdp(
            "Emulation.setScriptExecutionDisabled",
            json!({ "value": !enabled }),
        )
        .await
    }

    pub async fn pdf(&mut self, options: PdfOptions) -> Result<Vec<u8>, Error> {
        let (width, height) = options.paper.inches();
        let mut params = json!({
            "landscape": options.landscape,
            "printBackground": options.print_background,
            "paperWidth": width,
            "paperHeight": height,
        });
        if let Some(scale) = options.scale {
            params["scale"] = json!(scale);
        }
        if let Some(margins) = options.margins {
            params["marginTop"] = json!(margins.top);
            params["marginRight"] = json!(margins.right);
            params["marginBottom"] = json!(margins.bottom);
            params["marginLeft"] = json!(margins.left);
        }
        if let Some(ranges) = options.page_ranges {
            params["pageRanges"] = json!(ranges);
        }
        let printed = self.send("Page.printToPDF", params).await?;
        let data = printed
            .get("data")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::Protocol {
                method: "Page.printToPDF".into(),
                code: None,
                message: "missing data".into(),
            })?;
        crate::network::decode_base64(data, "Page.printToPDF")
    }

    async fn emulation_network(&mut self, offline: bool, throttle: Throttle) -> Result<(), Error> {
        let conditions = throttle.conditions();
        self.emulate_cdp(
            "Network.emulateNetworkConditions",
            json!({
                "offline": offline,
                "latency": conditions.latency_ms,
                "downloadThroughput": conditions.download_bytes_per_sec,
                "uploadThroughput": conditions.upload_bytes_per_sec,
            }),
        )
        .await
    }

    async fn emulate_cdp(&mut self, method: &str, params: Value) -> Result<(), Error> {
        drop(self.send(method, params).await?);
        Ok(())
    }
}
