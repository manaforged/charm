use crate::error::Error;
use crate::session::Session;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum MouseButton {
    #[default]
    Left,
    Right,
    Middle,
}

impl MouseButton {
    fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Middle => "middle",
        }
    }

    fn bit(self) -> u8 {
        match self {
            Self::Left => 1,
            Self::Right => 2,
            Self::Middle => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const ALT: Self = Self(1);
    pub const CONTROL: Self = Self(2);
    pub const META: Self = Self(4);
    pub const SHIFT: Self = Self(8);

    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    const fn suppresses_text(self) -> bool {
        self.0 & SUPPRESS_TEXT != 0
    }

    fn parse(name: &str) -> Option<Self> {
        MODIFIER_KEYS
            .iter()
            .find(|entry| entry.1 == name)
            .map(|entry| entry.0)
    }

    fn held(self) -> impl DoubleEndedIterator<Item = (Self, Key)> {
        MODIFIER_KEYS
            .iter()
            .filter(move |entry| self.contains(entry.0))
            .map(|&(flag, name, code, vk)| (flag, Key::new(name, code, vk, None, false)))
    }
}

const MODIFIER_KEYS: [(Modifiers, &str, &str, u32); 4] = [
    (Modifiers::ALT, "Alt", "AltLeft", 18),
    (Modifiers::CONTROL, "Control", "ControlLeft", 17),
    (Modifiers::META, "Meta", "MetaLeft", 91),
    (Modifiers::SHIFT, "Shift", "ShiftLeft", 16),
];

impl std::ops::BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ClickOptions {
    pub button: MouseButton,
    pub count: u8,
    pub modifiers: Modifiers,
    pub position: Option<(f64, f64)>,
}

const SUPPRESS_TEXT: u8 = 1 | 2 | 4;

const NAMED: [(&str, &str, &str, u32, Option<&str>); 12] = [
    ("Enter", "Enter", "Enter", 13, None),
    ("Tab", "Tab", "Tab", 9, None),
    ("Backspace", "Backspace", "Backspace", 8, None),
    ("Delete", "Delete", "Delete", 46, None),
    ("Escape", "Escape", "Escape", 27, None),
    ("Space", "Space", " ", 32, Some(" ")),
    ("ArrowLeft", "ArrowLeft", "ArrowLeft", 37, None),
    ("ArrowRight", "ArrowRight", "ArrowRight", 39, None),
    ("ArrowUp", "ArrowUp", "ArrowUp", 38, None),
    ("ArrowDown", "ArrowDown", "ArrowDown", 40, None),
    ("Home", "Home", "Home", 36, None),
    ("End", "End", "End", 35, None),
];

const SHIFTED: &str = "!1@2#3$4%5^6&7*8(9)0_-+={[}]|\\:;\"'<,>.?/~`";

fn unshift(ch: char) -> Option<char> {
    let glyphs: Vec<char> = SHIFTED.chars().collect();
    glyphs
        .chunks(2)
        .find(|pair| pair.first() == Some(&ch))
        .and_then(|pair| pair.get(1).copied())
}

const PUNCT: [(char, &str, u32); 11] = [
    ('-', "Minus", 189),
    ('=', "Equal", 187),
    ('[', "BracketLeft", 219),
    (']', "BracketRight", 221),
    ('\\', "Backslash", 220),
    (';', "Semicolon", 186),
    ('\'', "Quote", 222),
    (',', "Comma", 188),
    ('.', "Period", 190),
    ('/', "Slash", 191),
    ('`', "Backquote", 192),
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Key {
    key: String,
    code: String,
    vk: u32,
    text: Option<String>,
    shift: bool,
}

impl Key {
    fn new(key: &str, code: &str, vk: u32, text: Option<&str>, shift: bool) -> Self {
        Self {
            key: key.to_owned(),
            code: code.to_owned(),
            vk,
            text: text.map(str::to_owned),
            shift,
        }
    }

    fn resolve(name: &str) -> Self {
        if let Some((_, code, key, vk, text)) = NAMED
            .iter()
            .find(|entry| entry.0 == name || entry.2 == name)
        {
            return Self::new(key, code, *vk, *text, false);
        }
        if let Some((_, key)) = Modifiers::parse(name).and_then(|m| m.held().next()) {
            return key;
        }
        let mut chars = name.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => {
                Self::glyph(ch).unwrap_or_else(|| Self::new(name, name, 0, None, false))
            }
            _ => Self::new(name, name, 0, None, false),
        }
    }

    fn glyph(ch: char) -> Option<Self> {
        let text = ch.to_string();
        let upper = ch.to_ascii_uppercase();
        let (code, vk, shift) = if ch.is_ascii_alphabetic() {
            (
                format!("Key{upper}"),
                u32::from(upper),
                ch.is_ascii_uppercase(),
            )
        } else if ch.is_ascii_digit() {
            (format!("Digit{ch}"), u32::from(ch), false)
        } else if let Some(base) = unshift(ch) {
            let key = Self::glyph(base)?;
            (key.code, key.vk, true)
        } else {
            return Self::punct(ch);
        };
        Some(Self::new(&text, &code, vk, Some(&text), shift))
    }

    fn punct(ch: char) -> Option<Self> {
        let text = ch.to_string();
        PUNCT
            .iter()
            .find(|(glyph, _, _)| *glyph == ch)
            .map(|(_, code, vk)| Self::new(&text, code, *vk, Some(&text), false))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pointer {
    Move,
    Drag,
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Press {
    pub(crate) button: MouseButton,
    pub(crate) count: u8,
    pub(crate) modifiers: Modifiers,
}

impl Press {
    pub(crate) fn new(button: MouseButton, modifiers: Modifiers) -> Self {
        Self {
            button,
            count: 1,
            modifiers,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyPhase {
    Down,
    Up,
}

impl Session {
    async fn mouse_event(&mut self, params: Value) -> Result<(), Error> {
        drop(self.send("Input.dispatchMouseEvent", params).await?);
        Ok(())
    }

    pub async fn mouse_move(&mut self, x: f64, y: f64) -> Result<(), Error> {
        let press = Press::new(MouseButton::Left, Modifiers::NONE);
        self.pointer(Pointer::Move, x, y, press).await
    }

    pub(crate) async fn pointer(
        &mut self,
        phase: Pointer,
        x: f64,
        y: f64,
        press: Press,
    ) -> Result<(), Error> {
        let (kind, name, buttons) = match phase {
            Pointer::Move => ("mouseMoved", "none", 0),
            Pointer::Drag => ("mouseMoved", press.button.name(), press.button.bit()),
            Pointer::Down => ("mousePressed", press.button.name(), press.button.bit()),
            Pointer::Up => ("mouseReleased", press.button.name(), 0),
        };
        self.mouse_event(json!({
            "type": kind,
            "x": x.round(),
            "y": y.round(),
            "button": name,
            "buttons": buttons,
            "clickCount": press.count,
            "modifiers": press.modifiers.bits(),
        }))
        .await
    }

    pub async fn mouse_click(&mut self, x: f64, y: f64, button: MouseButton) -> Result<(), Error> {
        let options = ClickOptions {
            button,
            ..ClickOptions::default()
        };
        self.click_at(x, y, options).await
    }

    pub async fn click_at(&mut self, x: f64, y: f64, options: ClickOptions) -> Result<(), Error> {
        let mut press = Press::new(options.button, options.modifiers);
        self.pointer(Pointer::Move, x, y, press).await?;
        for count in 1..=options.count.max(1) {
            press.count = count;
            self.pointer(Pointer::Down, x, y, press).await?;
            self.pointer(Pointer::Up, x, y, press).await?;
        }
        Ok(())
    }

    pub async fn wheel(&mut self, delta_x: f64, delta_y: f64) -> Result<(), Error> {
        let center = self
            .evaluate_in(
                None,
                crate::session::World::Isolated,
                "[innerWidth/2,innerHeight/2]",
            )
            .await?;
        let at = |i: usize| center.get(i).and_then(Value::as_f64).unwrap_or(0.0);
        self.mouse_event(json!({
            "type": "mouseWheel",
            "x": at(0).round(),
            "y": at(1).round(),
            "buttons": 0,
            "deltaX": delta_x,
            "deltaY": delta_y,
        }))
        .await
    }

    pub async fn scroll(&mut self, delta_y: f64) -> Result<(), Error> {
        self.wheel(0.0, delta_y).await
    }

    async fn key_event(
        &mut self,
        phase: KeyPhase,
        key: &Key,
        modifiers: Modifiers,
    ) -> Result<(), Error> {
        let typed = key.text.is_some() && !modifiers.suppresses_text();
        let kind = match phase {
            KeyPhase::Down if typed => "keyDown",
            KeyPhase::Down => "rawKeyDown",
            KeyPhase::Up => "keyUp",
        };
        let mut params = json!({
            "type": kind,
            "modifiers": modifiers.bits(),
            "key": key.key,
            "code": key.code,
            "windowsVirtualKeyCode": key.vk,
        });
        if let (KeyPhase::Down, true, Some(text), Some(map)) =
            (phase, typed, &key.text, params.as_object_mut())
        {
            map.insert("text".into(), Value::String(text.clone()));
        }
        drop(self.send("Input.dispatchKeyEvent", params).await?);
        Ok(())
    }

    async fn tap(&mut self, key: &Key, held: Modifiers) -> Result<(), Error> {
        let modifiers = if key.shift && held == Modifiers::NONE {
            Modifiers::SHIFT
        } else {
            held
        };
        self.key_event(KeyPhase::Down, key, modifiers).await?;
        self.key_event(KeyPhase::Up, key, modifiers).await
    }

    pub async fn press(&mut self, combo: &str) -> Result<(), Error> {
        let (held, key) = parse_combo(combo);
        let mut down = Modifiers::NONE;
        for (flag, modifier) in held.held() {
            down = down | flag;
            self.key_event(KeyPhase::Down, &modifier, down).await?;
        }
        self.tap(&key, held).await?;
        for (flag, modifier) in held.held().rev() {
            down = Modifiers(down.0 & !flag.0);
            self.key_event(KeyPhase::Up, &modifier, down).await?;
        }
        Ok(())
    }

    pub(crate) async fn type_chars(&mut self, text: &str) -> Result<(), Error> {
        for ch in text.chars() {
            if ch.is_ascii() && !ch.is_ascii_control() {
                self.tap(&Key::resolve(&ch.to_string()), Modifiers::NONE)
                    .await?;
            } else {
                let params = json!({ "text": ch.to_string() });
                drop(self.send("Input.insertText", params).await?);
            }
        }
        Ok(())
    }
}

fn parse_combo(combo: &str) -> (Modifiers, Key) {
    let (mods, name) = match combo.strip_suffix('+') {
        Some(rest) if rest.is_empty() || rest.ends_with('+') => {
            (rest.strip_suffix('+').unwrap_or(rest), "+")
        }
        _ => combo.rsplit_once('+').unwrap_or(("", combo)),
    };
    let held = mods
        .split('+')
        .filter_map(Modifiers::parse)
        .fold(Modifiers::NONE, |acc, m| acc | m);
    (held, Key::resolve(name))
}
