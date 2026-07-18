use std::cell::Cell;
use std::collections::HashMap;
use std::io::{self, Read};
use std::sync::LazyLock;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum EscapeKind {
    Csi,
    Ss3,
}

static LINE_ENDINGS: LazyLock<HashMap<u8, LineEnding>> =
    LazyLock::new(|| HashMap::from([(b'\r', LineEnding::Cr), (b'\n', LineEnding::Lf)]));

static SINGLE_BYTE_KEYS: LazyLock<HashMap<u8, KeyEvent>> = LazyLock::new(|| {
    HashMap::from([
        (b'\x7f', key(KeyCode::Backspace, Modifiers::empty())),
        (b'\x08', key(KeyCode::Backspace, Modifiers::empty())),
        (b'\t', key(KeyCode::Tab, Modifiers::empty())),
        (0x03, key(KeyCode::Char('c'), Modifiers::CONTROL)),
        (0x04, key(KeyCode::Char('d'), Modifiers::CONTROL)),
    ])
});

static ESCAPE_PREFIXES: LazyLock<HashMap<u8, EscapeKind>> =
    LazyLock::new(|| HashMap::from([(b'[', EscapeKind::Csi), (b'O', EscapeKind::Ss3)]));

static ARROW_KEYS: LazyLock<HashMap<u8, KeyCode>> = LazyLock::new(|| {
    HashMap::from([
        (b'A', KeyCode::Up),
        (b'B', KeyCode::Down),
        (b'C', KeyCode::Right),
        (b'D', KeyCode::Left),
    ])
});

static UTF8_WIDTH: LazyLock<HashMap<u8, usize>> =
    LazyLock::new(|| HashMap::from([(0, 1), (1, 2), (2, 3), (3, 4)]));

#[derive(Clone, Copy)]
enum LineEnding {
    Cr,
    Lf,
}

pub fn read_key<R: Read>(input: &mut R) -> io::Result<KeyEvent> {
    let mut byte = [0u8; 1];
    input.read_exact(&mut byte)?;
    decode_key(input, byte[0])
}

fn decode_key<R: Read>(input: &mut R, byte: u8) -> io::Result<KeyEvent> {
    if let Some(ending) = LINE_ENDINGS.get(&byte).copied() {
        return decode_line_ending(input, ending);
    }

    if byte == b'\x1b' {
        return read_escape_sequence(input);
    }

    if let Some(event) = SINGLE_BYTE_KEYS.get(&byte).copied() {
        return Ok(event);
    }

    if byte <= 0x1f {
        return Ok(key(
            KeyCode::Char((byte + b'a' - 1) as char),
            Modifiers::CONTROL,
        ));
    }

    let ch = read_utf8_char(input, byte)?;
    Ok(key(KeyCode::Char(ch), Modifiers::empty()))
}

fn decode_line_ending<R: Read>(input: &mut R, ending: LineEnding) -> io::Result<KeyEvent> {
    match ending {
        LineEnding::Cr => {
            PENDING_LF.set(true);
            Ok(key(KeyCode::Enter, Modifiers::empty()))
        }
        LineEnding::Lf => {
            if PENDING_LF.get() {
                PENDING_LF.set(false);
                return read_key(input);
            }
            Ok(key(KeyCode::Enter, Modifiers::empty()))
        }
    }
}

fn read_escape_sequence<R: Read>(input: &mut R) -> io::Result<KeyEvent> {
    let mut next = [0u8; 1];
    input.read_exact(&mut next)?;

    let Some(kind) = ESCAPE_PREFIXES.get(&next[0]).copied() else {
        return Ok(key(KeyCode::Esc, Modifiers::empty()));
    };

    match kind {
        EscapeKind::Csi => read_csi(input),
        EscapeKind::Ss3 => read_ss3(input),
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

    let code = seq
        .bytes()
        .next()
        .and_then(|b| ARROW_KEYS.get(&b).copied())
        .unwrap_or(KeyCode::Esc);
    Ok(key(code, Modifiers::empty()))
}

fn read_ss3<R: Read>(input: &mut R) -> io::Result<KeyEvent> {
    let mut byte = [0u8; 1];
    input.read_exact(&mut byte)?;
    let code = ARROW_KEYS.get(&byte[0]).copied().unwrap_or(KeyCode::Esc);
    Ok(key(code, Modifiers::empty()))
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
    let class = utf8_width_key(first);
    UTF8_WIDTH.get(&class).copied().unwrap_or(4)
}

fn utf8_width_key(first: u8) -> u8 {
    if first & 0b1000_0000 == 0 {
        0
    } else if first & 0b1110_0000 == 0b1100_0000 {
        1
    } else if first & 0b1111_0000 == 0b1110_0000 {
        2
    } else {
        3
    }
}

fn key(code: KeyCode, modifiers: Modifiers) -> KeyEvent {
    KeyEvent { code, modifiers }
}
