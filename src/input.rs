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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modifier {
    Alt,
    Control,
    Meta,
    Shift,
}

impl Modifier {
    const ALL: [Self; 4] = [Self::Alt, Self::Control, Self::Meta, Self::Shift];

    fn bit(self) -> u8 {
        match self {
            Self::Alt => 1,
            Self::Control => 2,
            Self::Meta => 4,
            Self::Shift => 8,
        }
    }

    fn key(self) -> Key {
        let (name, code, vk) = match self {
            Self::Alt => ("Alt", "AltLeft", 18),
            Self::Control => ("Control", "ControlLeft", 17),
            Self::Meta => ("Meta", "MetaLeft", 91),
            Self::Shift => ("Shift", "ShiftLeft", 16),
        };
        Key::new(name, code, vk, None, false)
    }

    fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.key().key == name)
    }
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

const SHIFTED: [(char, char); 21] = [
    ('!', '1'),
    ('@', '2'),
    ('#', '3'),
    ('$', '4'),
    ('%', '5'),
    ('^', '6'),
    ('&', '7'),
    ('*', '8'),
    ('(', '9'),
    (')', '0'),
    ('_', '-'),
    ('+', '='),
    ('{', '['),
    ('}', ']'),
    ('|', '\\'),
    (':', ';'),
    ('"', '\''),
    ('<', ','),
    ('>', '.'),
    ('?', '/'),
    ('~', '`'),
];

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
        if let Some((_, code, key, vk, text)) = NAMED.iter().find(|entry| entry.0 == name) {
            return Self::new(key, code, *vk, *text, false);
        }
        if let Some(modifier) = Modifier::parse(name) {
            return modifier.key();
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
        if ch.is_ascii_lowercase() || ch.is_ascii_uppercase() {
            let code = format!("Key{upper}");
            return Some(Self::new(
                &text,
                &code,
                u32::from(upper),
                Some(&text),
                ch.is_ascii_uppercase(),
            ));
        }
        if ch.is_ascii_digit() {
            return Some(Self::new(
                &text,
                &format!("Digit{ch}"),
                u32::from(ch),
                Some(&text),
                false,
            ));
        }
        if let Some((_, base)) = SHIFTED.iter().find(|(glyph, _)| *glyph == ch) {
            let base_key = Self::punct(*base).or_else(|| {
                base.is_ascii_digit().then(|| {
                    Self::new(
                        &text,
                        &format!("Digit{base}"),
                        u32::from(*base),
                        None,
                        false,
                    )
                })
            })?;
            return Some(Self::new(
                &text,
                &base_key.code,
                base_key.vk,
                Some(&text),
                true,
            ));
        }
        Self::punct(ch)
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
        self.mouse_event(
            json!({ "type": "mouseMoved", "x": x.round(), "y": y.round(), "buttons": 0 }),
        )
        .await
    }

    pub async fn mouse_click(&mut self, x: f64, y: f64, button: MouseButton) -> Result<(), Error> {
        self.mouse_move(x, y).await?;
        for (kind, buttons) in [("mousePressed", button.bit()), ("mouseReleased", 0)] {
            self.mouse_event(json!({
                "type": kind,
                "x": x.round(),
                "y": y.round(),
                "button": button.name(),
                "buttons": buttons,
                "clickCount": 1,
            }))
            .await?;
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

    async fn key_event(&mut self, phase: KeyPhase, key: &Key, modifiers: u8) -> Result<(), Error> {
        let typed = key.text.is_some() && modifiers & SUPPRESS_TEXT == 0;
        let kind = match phase {
            KeyPhase::Down if typed => "keyDown",
            KeyPhase::Down => "rawKeyDown",
            KeyPhase::Up => "keyUp",
        };
        let mut params = json!({
            "type": kind,
            "modifiers": modifiers,
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

    pub async fn press(&mut self, combo: &str) -> Result<(), Error> {
        let (mods, name) = match combo.strip_suffix('+') {
            Some(rest) if rest.is_empty() || rest.ends_with('+') => {
                (rest.strip_suffix('+').unwrap_or(rest), "+")
            }
            _ => combo.rsplit_once('+').unwrap_or(("", combo)),
        };
        let held: Vec<Modifier> = mods.split('+').filter_map(Modifier::parse).collect();
        let mut modifiers = 0;
        for modifier in &held {
            modifiers |= modifier.bit();
            self.key_event(KeyPhase::Down, &modifier.key(), modifiers)
                .await?;
        }
        let key = Key::resolve(name);
        let shift = if key.shift && held.is_empty() {
            Modifier::Shift.bit()
        } else {
            0
        };
        self.key_event(KeyPhase::Down, &key, modifiers | shift)
            .await?;
        self.key_event(KeyPhase::Up, &key, modifiers | shift)
            .await?;
        for modifier in held.iter().rev() {
            modifiers &= !modifier.bit();
            self.key_event(KeyPhase::Up, &modifier.key(), modifiers)
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn type_chars(&mut self, text: &str) -> Result<(), Error> {
        for ch in text.chars() {
            if ch.is_ascii() && !ch.is_ascii_control() {
                let key =
                    Key::glyph(ch).unwrap_or_else(|| Key::new(" ", "Space", 32, Some(" "), false));
                let shift = if key.shift { Modifier::Shift.bit() } else { 0 };
                self.key_event(KeyPhase::Down, &key, shift).await?;
                self.key_event(KeyPhase::Up, &key, shift).await?;
            } else {
                let piece = ch.to_string();
                drop(
                    self.send("Input.insertText", json!({ "text": piece }))
                        .await?,
                );
            }
        }
        Ok(())
    }
}
