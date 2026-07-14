use std::cell::Cell;
use std::io::{self, Read};

thread_local! {
    static PENDING_LF: Cell<bool> = const { Cell::new(false) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const CONTROL: Self = Self(0b0001);
    pub const ALT: Self = Self(0b0010);
    pub const SHIFT: Self = Self(0b0100);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Esc,
    Tab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

pub fn read_key<R: Read>(input: &mut R) -> io::Result<KeyEvent> {
    let mut byte = [0u8; 1];
    input.read_exact(&mut byte)?;
    decode_key(input, byte[0])
}

fn decode_key<R: Read>(input: &mut R, byte: u8) -> io::Result<KeyEvent> {
    match byte {
        b'\r' => {
            PENDING_LF.set(true);
            Ok(key(KeyCode::Enter, Modifiers::empty()))
        }
        b'\n' => {
            if PENDING_LF.get() {
                PENDING_LF.set(false);
                return read_key(input);
            }
            Ok(key(KeyCode::Enter, Modifiers::empty()))
        }
        b'\x7f' | b'\x08' => Ok(key(KeyCode::Backspace, Modifiers::empty())),
        b'\x1b' => read_escape_sequence(input),
        b'\t' => Ok(key(KeyCode::Tab, Modifiers::empty())),
        0x03 => Ok(key(KeyCode::Char('c'), Modifiers::CONTROL)),
        0x04 => Ok(key(KeyCode::Char('d'), Modifiers::CONTROL)),
        0x00..=0x1f => Ok(key(
            KeyCode::Char((byte + b'a' - 1) as char),
            Modifiers::CONTROL,
        )),
        _ => {
            let ch = read_utf8_char(input, byte)?;
            Ok(key(KeyCode::Char(ch), Modifiers::empty()))
        }
    }
}

fn read_escape_sequence<R: Read>(input: &mut R) -> io::Result<KeyEvent> {
    let mut next = [0u8; 1];
    input.read_exact(&mut next)?;

    match next[0] {
        b'[' => read_csi(input),
        b'O' => read_ss3(input),
        _ => Ok(key(KeyCode::Esc, Modifiers::empty())),
    }
}

fn read_csi<R: Read>(input: &mut R) -> io::Result<KeyEvent> {
    let mut seq = String::new();
    loop {
        let mut byte = [0u8; 1];
        input.read_exact(&mut byte)?;
        let ch = byte[0] as char;
        seq.push(ch);
        if ch.is_ascii_alphabetic() {
            break;
        }
    }

    match seq.as_str() {
        "A" => Ok(key(KeyCode::Up, Modifiers::empty())),
        "B" => Ok(key(KeyCode::Down, Modifiers::empty())),
        "C" => Ok(key(KeyCode::Right, Modifiers::empty())),
        "D" => Ok(key(KeyCode::Left, Modifiers::empty())),
        _ => Ok(key(KeyCode::Esc, Modifiers::empty())),
    }
}

fn read_ss3<R: Read>(input: &mut R) -> io::Result<KeyEvent> {
    let mut byte = [0u8; 1];
    input.read_exact(&mut byte)?;
    match byte[0] {
        b'A' => Ok(key(KeyCode::Up, Modifiers::empty())),
        b'B' => Ok(key(KeyCode::Down, Modifiers::empty())),
        b'C' => Ok(key(KeyCode::Right, Modifiers::empty())),
        b'D' => Ok(key(KeyCode::Left, Modifiers::empty())),
        _ => Ok(key(KeyCode::Esc, Modifiers::empty())),
    }
}

fn read_utf8_char<R: Read>(input: &mut R, first: u8) -> io::Result<char> {
    let width = utf8_width(first);
    let mut bytes = vec![first];
    for _ in 1..width {
        let mut next = [0u8; 1];
        input.read_exact(&mut next)?;
        bytes.push(next[0]);
    }

    let text = std::str::from_utf8(&bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8"))?;
    text.chars()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "empty utf-8 sequence"))
}

fn utf8_width(first: u8) -> usize {
    if first & 0b1000_0000 == 0 {
        1
    } else if first & 0b1110_0000 == 0b1100_0000 {
        2
    } else if first & 0b1111_0000 == 0b1110_0000 {
        3
    } else {
        4
    }
}

fn key(code: KeyCode, modifiers: Modifiers) -> KeyEvent {
    KeyEvent { code, modifiers }
}
