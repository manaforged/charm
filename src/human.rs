mod cursor;
mod rng;
mod typing;

pub use cursor::{CursorOptions, Point, Rect, Step, Target, path, target_point};
pub use rng::{Band, DelayBand, Rng};
pub use typing::{Key, Keystroke, TypingOptions, keystrokes};

#[cfg(test)]
#[path = "human_tests.rs"]
mod tests;
