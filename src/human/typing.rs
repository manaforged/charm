use std::time::Duration;

use super::rng::{Rng, from_ms, ms};

const MS_PER_MIN: f64 = 60_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Space,
    Enter,
    Tab,
    Backspace,
}

impl Key {
    fn of(ch: char) -> Self {
        match ch {
            ' ' => Key::Space,
            '\n' => Key::Enter,
            '\t' => Key::Tab,
            other => Key::Char(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keystroke {
    pub key: Key,
    pub before: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingOptions {
    pub wpm: f64,
    pub chars_per_word: f64,
    pub error_rate: f64,
    pub swap_error_rate: f64,
    pub error_detection_rate: f64,
    pub bigram_boost: f64,
    pub burst_rate: f64,
    pub burst_factor: f64,
    pub burst_min_len: u64,
    pub burst_max_len: u64,
    pub close_keys_boost: f64,
    pub close_keys_distance: usize,
    pub far_keys_penalty: f64,
    pub far_keys_distance: usize,
    pub first_char_factor: f64,
    pub fatigue_factor: f64,
    pub fatigue_cap: f64,
    pub uppercase_penalty: Duration,
    pub jitter_sd: f64,
    pub min_char_delay: Duration,
    pub max_char_delay: Duration,
    pub word_pause: Duration,
    pub word_pause_variance: Duration,
    pub min_word_pause: Duration,
    pub word_pause_extra: Duration,
    pub thinking_pause_rate: f64,
    pub thinking_pause_min: Duration,
    pub thinking_pause_extra: Duration,
    pub reaction_time: Duration,
    pub reaction_scale_min: f64,
    pub reaction_scale_extra: f64,
    pub backspace_delay: Duration,
    pub backspace_scale_min: f64,
    pub backspace_scale_extra: f64,
    pub drift_volatility: f64,
    pub drift_momentum: f64,
    pub drift_range: f64,
    pub keyboard_rows: Vec<&'static str>,
    pub fast_bigrams: Vec<&'static str>,
}

impl Default for TypingOptions {
    fn default() -> Self {
        Self {
            wpm: 45.0,
            chars_per_word: 5.0,
            error_rate: 0.015,
            swap_error_rate: 0.008,
            error_detection_rate: 0.92,
            bigram_boost: 0.82,
            burst_rate: 0.12,
            burst_factor: 0.6,
            burst_min_len: 3,
            burst_max_len: 7,
            close_keys_boost: 0.92,
            close_keys_distance: 1,
            far_keys_penalty: 1.12,
            far_keys_distance: 4,
            first_char_factor: 1.1,
            fatigue_factor: 1.0008,
            fatigue_cap: 2.5,
            uppercase_penalty: Duration::from_millis(55),
            jitter_sd: 0.25,
            min_char_delay: Duration::from_millis(20),
            max_char_delay: Duration::from_millis(800),
            word_pause: Duration::from_millis(90),
            word_pause_variance: Duration::from_millis(45),
            min_word_pause: Duration::from_millis(50),
            word_pause_extra: Duration::from_millis(120),
            thinking_pause_rate: 0.05,
            thinking_pause_min: Duration::from_millis(600),
            thinking_pause_extra: Duration::from_millis(1400),
            reaction_time: Duration::from_millis(240),
            reaction_scale_min: 0.6,
            reaction_scale_extra: 0.8,
            backspace_delay: Duration::from_millis(110),
            backspace_scale_min: 0.8,
            backspace_scale_extra: 0.4,
            drift_volatility: 0.04,
            drift_momentum: -0.25,
            drift_range: 0.35,
            keyboard_rows: vec!["qwertyuiop", "asdfghjkl", "zxcvbnm"],
            fast_bigrams: vec![
                "th", "he", "in", "er", "an", "re", "on", "at", "en", "es", "st", "nd", "ng", "ed",
                "ou", "ion", "ti", "te", "or",
            ],
        }
    }
}

impl TypingOptions {
    #[must_use]
    pub fn typo_free() -> Self {
        Self {
            error_rate: 0.0,
            swap_error_rate: 0.0,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn ensure_correction(self) -> Self {
        Self {
            error_detection_rate: 1.0,
            ..self
        }
    }

    fn key_pos(&self, ch: char) -> Option<(usize, usize)> {
        let c = ch.to_lowercase().next()?;
        self.keyboard_rows
            .iter()
            .enumerate()
            .find_map(|(r, row)| row.chars().position(|k| k == c).map(|i| (r, i)))
    }

    fn key_dist(&self, a: char, b: char) -> Option<usize> {
        let (ra, ia) = self.key_pos(a)?;
        let (rb, ib) = self.key_pos(b)?;
        Some(ra.abs_diff(rb) + ia.abs_diff(ib))
    }

    fn row_char(&self, r: usize, i: usize) -> Option<char> {
        self.keyboard_rows.get(r).and_then(|row| row.chars().nth(i))
    }

    fn neighbor(&self, ch: char, rng: &mut Rng) -> Option<char> {
        let (r, i) = self.key_pos(ch)?;
        let mut opts = Vec::new();
        if i > 0 {
            opts.extend(self.row_char(r, i - 1));
        }
        opts.extend(self.row_char(r, i + 1));
        if r > 0 {
            opts.extend(self.row_char(r - 1, i));
        }
        opts.extend(self.row_char(r + 1, i));
        if opts.is_empty() {
            return None;
        }
        let pick = ((rng.next_f64() * opts.len() as f64) as usize).min(opts.len() - 1);
        let n = opts[pick];
        Some(if ch.is_uppercase() {
            n.to_uppercase().next().unwrap_or(n)
        } else {
            n
        })
    }

    fn fast_bigram(&self, prev: char, ch: char) -> bool {
        let pair: String = [prev, ch].iter().flat_map(|c| c.to_lowercase()).collect();
        self.fast_bigrams.iter().any(|b| *b == pair)
    }
}

struct Typist<'a> {
    o: &'a TypingOptions,
    base: f64,
    drift: f64,
    burst: u64,
    count: i32,
    prev: Option<char>,
    pending: f64,
    out: Vec<Keystroke>,
}

impl Typist<'_> {
    fn char_delay(&mut self, ch: char, first: bool, rng: &mut Rng) -> f64 {
        let o = self.o;
        let mut d = self.base * self.drift;
        if self.burst > 0 {
            d *= o.burst_factor;
            self.burst -= 1;
        }
        if let Some(prev) = self.prev {
            if o.fast_bigram(prev, ch) {
                d *= o.bigram_boost;
            }
            match o.key_dist(prev, ch) {
                Some(dist) if dist <= o.close_keys_distance => d *= o.close_keys_boost,
                Some(dist) if dist >= o.far_keys_distance => d *= o.far_keys_penalty,
                _ => {}
            }
        }
        if first {
            d *= o.first_char_factor;
        }
        d *= o.fatigue_factor.powi(self.count).min(o.fatigue_cap);
        if ch.is_uppercase() {
            d += ms(o.uppercase_penalty);
        }
        d += rng.gauss() * d * o.jitter_sd;
        self.count = self.count.saturating_add(1);
        d.clamp(ms(o.min_char_delay), ms(o.max_char_delay))
    }

    fn press(&mut self, key: Key, delay: f64) {
        let before = from_ms(self.pending + delay);
        self.pending = 0.0;
        self.out.push(Keystroke { key, before });
    }

    fn type_char(&mut self, ch: char, first: bool, rng: &mut Rng) {
        let d = self.char_delay(ch, first, rng);
        self.press(Key::of(ch), d);
        self.prev = Some(ch);
    }

    fn word_start(&mut self, rng: &mut Rng) {
        let o = self.o;
        self.pending += if rng.chance(o.thinking_pause_rate) {
            ms(o.thinking_pause_min) + rng.next_f64() * ms(o.thinking_pause_extra)
        } else {
            let g = ms(o.word_pause) + rng.gauss() * ms(o.word_pause_variance);
            g.max(ms(o.min_word_pause)) + rng.next_f64() * ms(o.word_pause_extra)
        };
    }

    fn word(&mut self, chars: &[char], pause: bool, rng: &mut Rng) {
        let o = self.o;
        if pause {
            self.word_start(rng);
        }
        self.drift += o.drift_momentum * (self.drift - 1.0) + rng.gauss() * o.drift_volatility;
        self.drift = self.drift.clamp(1.0 - o.drift_range, 1.0 + o.drift_range);
        if self.burst == 0 && rng.chance(o.burst_rate) {
            self.burst = rng.range_int(o.burst_min_len, o.burst_max_len);
        }
        for (i, &ch) in chars.iter().enumerate() {
            let visible = !ch.is_whitespace();
            let first = i == 0 && visible;
            if visible
                && rng.chance(o.error_rate)
                && let Some(wrong) = o.neighbor(ch, rng)
            {
                self.typo(ch, wrong, first, rng);
                continue;
            }
            self.type_char(ch, first, rng);
        }
    }

    fn typo(&mut self, ch: char, wrong: char, first: bool, rng: &mut Rng) {
        let o = self.o;
        self.type_char(wrong, first, rng);
        if rng.chance(o.error_detection_rate) {
            let react = ms(o.reaction_time)
                * (o.reaction_scale_min + rng.next_f64() * o.reaction_scale_extra);
            let back = ms(o.backspace_delay)
                * (o.backspace_scale_min + rng.next_f64() * o.backspace_scale_extra);
            self.press(Key::Backspace, react + back);
            self.type_char(ch, first, rng);
        }
    }
}

fn split_words(text: &str) -> Vec<Vec<char>> {
    let mut words: Vec<Vec<char>> = Vec::new();
    let mut current = Vec::new();
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            words.push(vec![ch]);
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

pub fn keystrokes(text: &str, options: &TypingOptions, rng: &mut Rng) -> Vec<Keystroke> {
    let mut typist = Typist {
        o: options,
        base: MS_PER_MIN / (options.wpm * options.chars_per_word),
        drift: 1.0,
        burst: 0,
        count: 0,
        prev: None,
        pending: 0.0,
        out: Vec::new(),
    };
    for (w, chars) in split_words(text).iter().enumerate() {
        let pause = w > 0 && chars.iter().any(|c| !c.is_whitespace());
        typist.word(chars, pause, rng);
    }
    typist.out
}
