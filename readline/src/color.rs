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

impl Color {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Red" => Some(Self::Red),
            "Green" => Some(Self::Green),
            "Yellow" => Some(Self::Yellow),
            "Blue" => Some(Self::Blue),
            "Magenta" => Some(Self::Magenta),
            "Cyan" => Some(Self::Cyan),
            "White" => Some(Self::White),
            "Black" => Some(Self::Black),
            _ => None,
        }
    }

    pub fn ansi_fg(self) -> u8 {
        match self {
            Self::Black => 30,
            Self::Red => 31,
            Self::Green => 32,
            Self::Yellow => 33,
            Self::Blue => 34,
            Self::Magenta => 35,
            Self::Cyan => 36,
            Self::White => 37,
        }
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
