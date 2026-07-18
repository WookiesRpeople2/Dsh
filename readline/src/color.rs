use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

static BY_NAME: LazyLock<HashMap<&'static str, Color>> = LazyLock::new(|| {
    HashMap::from([
        ("Black", Color::Black),
        ("Red", Color::Red),
        ("Green", Color::Green),
        ("Yellow", Color::Yellow),
        ("Blue", Color::Blue),
        ("Magenta", Color::Magenta),
        ("Cyan", Color::Cyan),
        ("White", Color::White),
    ])
});

static ANSI_FG: LazyLock<HashMap<Color, u8>> = LazyLock::new(|| {
    HashMap::from([
        (Color::Black, 30),
        (Color::Red, 31),
        (Color::Green, 32),
        (Color::Yellow, 33),
        (Color::Blue, 34),
        (Color::Magenta, 35),
        (Color::Cyan, 36),
        (Color::White, 37),
    ])
});

impl Color {
    pub fn from_name(name: &str) -> Option<Self> {
        BY_NAME.get(name).copied()
    }

    pub fn ansi_fg(self) -> u8 {
        ANSI_FG.get(&self).copied().unwrap_or(37)
    }
}

pub const NAMED_COLORS: &[(&str, Color)] = &[
    ("Red", Color::Red),
    ("Green", Color::Green),
    ("Yellow", Color::Yellow),
    ("Blue", Color::Blue),
    ("Magenta", Color::Magenta),
    ("Cyan", Color::Cyan),
    ("White", Color::White),
    ("Black", Color::Black),
];
