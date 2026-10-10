use super::WaitFor;
use crate::input::{ClickOptions, Modifiers, MouseButton};
use crate::locator::SelectBy;
use crate::network::UrlMatch;
use crate::selector::{Selector, TextMatch};
use crate::wait::LoadState;
use serde::{Deserialize, Deserializer};
use std::time::Duration;

const BUTTONS: [MouseButton; 3] = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];
const MODIFIERS: [Modifiers; 4] = [
    Modifiers::ALT,
    Modifiers::CONTROL,
    Modifiers::META,
    Modifiers::SHIFT,
];
const LOADS: [LoadState; 3] = [
    LoadState::Load,
    LoadState::DomContentLoaded,
    LoadState::NetworkIdle,
];

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum WaitWire {
    Selector(String),
    Text(String),
    Url(String),
    UrlExact(String),
    UrlGlob(String),
    Load(LoadWire),
    Ms(u64),
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub(super) enum LoadWire {
    Load,
    DomContentLoaded,
    NetworkIdle,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ClickWire {
    button: ButtonWire,
    count: u8,
    modifiers: Vec<ModifierWire>,
    position: Option<(f64, f64)>,
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ButtonWire {
    #[default]
    Left,
    Right,
    Middle,
}

#[derive(Deserialize, Clone, Copy)]
enum ModifierWire {
    Alt,
    Control,
    Meta,
    Shift,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum SelectWire {
    Value(Vec<String>),
    Label(Vec<String>),
    Index(Vec<usize>),
}

impl From<WaitWire> for WaitFor {
    fn from(wire: WaitWire) -> Self {
        match wire {
            WaitWire::Selector(raw) => Self::Selector(Selector::parse_target(&raw)),
            WaitWire::Text(text) => Self::Text(TextMatch::Contains(text)),
            WaitWire::Url(part) => Self::Url(UrlMatch::Contains(part)),
            WaitWire::UrlExact(url) => Self::Url(UrlMatch::Exact(url)),
            WaitWire::UrlGlob(glob) => Self::Url(UrlMatch::Glob(glob)),
            WaitWire::Load(load) => {
                Self::LoadState(LOADS.get(load as usize).copied().unwrap_or_default())
            }
            WaitWire::Ms(ms) => Self::Duration(Duration::from_millis(ms)),
        }
    }
}

pub(super) fn selector<'de, D: Deserializer<'de>>(de: D) -> Result<Selector, D::Error> {
    Ok(Selector::parse_target(&String::deserialize(de)?))
}

pub(super) fn maybe_selector<'de, D: Deserializer<'de>>(
    de: D,
) -> Result<Option<Selector>, D::Error> {
    Ok(Option::<String>::deserialize(de)?.map(|raw| Selector::parse_target(&raw)))
}

pub(super) fn checked() -> bool {
    true
}

pub(super) fn select_by<'de, D: Deserializer<'de>>(de: D) -> Result<SelectBy, D::Error> {
    Ok(match SelectWire::deserialize(de)? {
        SelectWire::Value(values) => SelectBy::Value(values),
        SelectWire::Label(labels) => SelectBy::Label(labels),
        SelectWire::Index(indexes) => SelectBy::Index(indexes),
    })
}

pub(super) fn click<'de, D: Deserializer<'de>>(de: D) -> Result<ClickOptions, D::Error> {
    let wire = ClickWire::deserialize(de)?;
    let modifiers = wire.modifiers.iter().fold(Modifiers::NONE, |all, held| {
        all | MODIFIERS.get(*held as usize).copied().unwrap_or_default()
    });
    Ok(ClickOptions {
        button: BUTTONS
            .get(wire.button as usize)
            .copied()
            .unwrap_or_default(),
        count: wire.count,
        modifiers,
        position: wire.position,
    })
}
