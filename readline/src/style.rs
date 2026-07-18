use std::fmt;

use crate::color::Color;

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const REVERSE: &str = "\x1b[7m";

pub struct Styled<'a> {
    pub text: &'a str,
    pub color: Color,
}

impl fmt::Display for Styled<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[{}m{}{RESET}", self.color.ansi_fg(), self.text)
    }
}

pub fn styled(text: impl AsRef<str>, color: Color) -> String {
    Styled {
        text: text.as_ref(),
        color,
    }
    .to_string()
}
