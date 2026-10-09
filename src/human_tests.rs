use super::{CursorOptions, Key, Point, Rect, Rng, Target, TypingOptions, keystrokes, path};

fn typed(keys: &[super::Keystroke]) -> String {
    let mut out = String::new();
    for stroke in keys {
        match stroke.key {
            Key::Char(c) => out.push(c),
            Key::Space => out.push(' '),
            Key::Enter => out.push('\n'),
            Key::Tab => out.push('\t'),
            Key::Backspace => {
                out.pop();
            }
        }
    }
    out
}

#[test]
fn the_same_seed_gives_the_same_path_and_keystrokes() {
    let options = CursorOptions::default();
    let to = Target::Point(Point { x: 640.0, y: 360.0 });
    let from = Point { x: 10.0, y: 10.0 };
    let first = path(from, to, &options, &mut Rng::new(7));
    let second = path(from, to, &options, &mut Rng::new(7));
    assert_eq!(format!("{first:?}"), format!("{second:?}"));
    let typing = TypingOptions::default();
    let a = keystrokes("hello world", &typing, &mut Rng::new(7));
    let b = keystrokes("hello world", &typing, &mut Rng::new(7));
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
}

#[test]
fn a_path_to_a_box_lands_inside_it() {
    let options = CursorOptions::default();
    let rect = Rect {
        x: 300.0,
        y: 200.0,
        width: 120.0,
        height: 40.0,
    };
    for seed in 0..200 {
        let steps = path(
            Point { x: 5.0, y: 5.0 },
            Target::Rect(rect),
            &options,
            &mut Rng::new(seed),
        );
        let last = steps.last().expect("a path has steps").point;
        assert!(
            last.x >= rect.x
                && last.x <= rect.x + rect.width
                && last.y >= rect.y
                && last.y <= rect.y + rect.height,
            "seed {seed} landed at {last:?}"
        );
    }
}

#[test]
fn corrected_typing_produces_the_text() {
    let text = "The quick brown fox, 42 times!";
    for seed in 0..100 {
        let clean = keystrokes(text, &TypingOptions::typo_free(), &mut Rng::new(seed));
        assert_eq!(typed(&clean), text);
        let corrected = keystrokes(
            text,
            &TypingOptions::default().ensure_correction(),
            &mut Rng::new(seed),
        );
        assert_eq!(typed(&corrected), text, "seed {seed}");
    }
}
