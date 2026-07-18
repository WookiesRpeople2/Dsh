use crate::color::Color;
use crate::key::{KeyCode, KeyEvent, Modifiers, read_key};
use crate::style;
use crate::terminal::{self, RawMode};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorResult {
    Submit(String),
    Cancel,
    Eof,
}

#[derive(Debug, Clone)]
pub struct EditorConfig {
    pub prompt: String,
    pub prompt_color: Color,
}

impl EditorConfig {
    pub fn new(prompt: impl Into<String>, prompt_color: Color) -> Self {
        Self {
            prompt: prompt.into(),
            prompt_color,
        }
    }
}

pub trait HistoryNavigation {
    fn previous(&mut self, current: &str) -> Option<String>;
    fn next(&mut self) -> Option<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorAction {
    Continue,
    Submit,
    Cancel,
}

#[derive(Clone, Copy)]
enum EditorOp {
    Submit,
    Cancel,
    CancelIfEmpty,
    Backspace,
    HistoryPrev,
    HistoryNext,
    MoveLeft,
    MoveRight,
}

static KEY_BINDINGS: LazyLock<HashMap<KeyEvent, EditorOp>> = LazyLock::new(|| {
    HashMap::from([
        (
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: Modifiers::empty(),
            },
            EditorOp::Submit,
        ),
        (
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: Modifiers::empty(),
            },
            EditorOp::Backspace,
        ),
        (
            KeyEvent {
                code: KeyCode::Up,
                modifiers: Modifiers::empty(),
            },
            EditorOp::HistoryPrev,
        ),
        (
            KeyEvent {
                code: KeyCode::Down,
                modifiers: Modifiers::empty(),
            },
            EditorOp::HistoryNext,
        ),
        (
            KeyEvent {
                code: KeyCode::Left,
                modifiers: Modifiers::empty(),
            },
            EditorOp::MoveLeft,
        ),
        (
            KeyEvent {
                code: KeyCode::Right,
                modifiers: Modifiers::empty(),
            },
            EditorOp::MoveRight,
        ),
        (
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: Modifiers::CONTROL,
            },
            EditorOp::Cancel,
        ),
        (
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: Modifiers::CONTROL,
            },
            EditorOp::CancelIfEmpty,
        ),
    ])
});

pub struct Editor {
    prompt_display: String,
    prompt_width: u16,
    buffer: String,
    cursor: usize,
}

impl Editor {
    pub fn new(config: EditorConfig) -> Self {
        let prompt_display = style::styled(&config.prompt, config.prompt_color);
        let prompt_width = config.prompt.chars().count() as u16;
        Self {
            prompt_display,
            prompt_width,
            buffer: String::new(),
            cursor: 0,
        }
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_buffer(&mut self, text: impl Into<String>) {
        let text = text.into();
        self.buffer = text.trim_end_matches(['\r', '\n']).to_owned();
        self.cursor = self.buffer.chars().count();
    }

    pub fn read<H: HistoryNavigation>(&mut self, history: &mut H) -> io::Result<EditorResult> {
        let _raw = RawMode::enable()?;
        let mut out = io::stdout();

        write!(out, "\r{}", self.prompt_display)?;
        out.flush()?;

        let result = loop {
            let key = {
                let mut input = io::stdin().lock();
                match read_key(&mut input) {
                    Ok(key) => key,
                    Err(_) => break EditorResult::Eof,
                }
            };

            match self.handle_key(history, key, &mut out)? {
                EditorAction::Continue => {}
                EditorAction::Submit => break EditorResult::Submit(self.buffer.clone()),
                EditorAction::Cancel => break EditorResult::Cancel,
            }
        };

        if matches!(result, EditorResult::Submit(_)) {
            finish_line(&mut out);
        }

        Ok(result)
    }

    pub fn handle_key<H: HistoryNavigation>(
        &mut self,
        history: &mut H,
        key: KeyEvent,
        out: &mut io::Stdout,
    ) -> io::Result<EditorAction> {
        if let Some(op) = KEY_BINDINGS.get(&key).copied() {
            return self.apply_op(history, op, out);
        }

        if let KeyCode::Char(ch) = key.code
            && !key.modifiers.contains(Modifiers::CONTROL)
        {
            let at = char_byte_idx(&self.buffer, self.cursor);
            self.buffer.insert(at, ch);
            self.cursor += 1;
            self.redraw(out)?;
        }

        Ok(EditorAction::Continue)
    }

    fn apply_op<H: HistoryNavigation>(
        &mut self,
        history: &mut H,
        op: EditorOp,
        out: &mut io::Stdout,
    ) -> io::Result<EditorAction> {
        match op {
            EditorOp::Submit => self.op_submit(history, out),
            EditorOp::Cancel => self.op_cancel(history, out),
            EditorOp::CancelIfEmpty => self.op_cancel_if_empty(history, out),
            EditorOp::Backspace => self.op_backspace(history, out),
            EditorOp::HistoryPrev => self.op_history_prev(history, out),
            EditorOp::HistoryNext => self.op_history_next(history, out),
            EditorOp::MoveLeft => self.op_move_left(history, out),
            EditorOp::MoveRight => self.op_move_right(history, out),
        }
    }

    fn op_submit<H: HistoryNavigation>(
        &mut self,
        _history: &mut H,
        _out: &mut io::Stdout,
    ) -> io::Result<EditorAction> {
        Ok(EditorAction::Submit)
    }

    fn op_cancel<H: HistoryNavigation>(
        &mut self,
        _history: &mut H,
        _out: &mut io::Stdout,
    ) -> io::Result<EditorAction> {
        Ok(EditorAction::Cancel)
    }

    fn op_cancel_if_empty<H: HistoryNavigation>(
        &mut self,
        _history: &mut H,
        _out: &mut io::Stdout,
    ) -> io::Result<EditorAction> {
        if self.buffer.is_empty() {
            Ok(EditorAction::Cancel)
        } else {
            Ok(EditorAction::Continue)
        }
    }

    fn op_backspace<H: HistoryNavigation>(
        &mut self,
        _history: &mut H,
        out: &mut io::Stdout,
    ) -> io::Result<EditorAction> {
        if self.cursor > 0 {
            let start = char_byte_idx(&self.buffer, self.cursor - 1);
            let end = char_byte_idx(&self.buffer, self.cursor);
            self.buffer.replace_range(start..end, "");
            self.cursor -= 1;
            self.redraw(out)?;
        }
        Ok(EditorAction::Continue)
    }

    fn op_history_prev<H: HistoryNavigation>(
        &mut self,
        history: &mut H,
        out: &mut io::Stdout,
    ) -> io::Result<EditorAction> {
        if let Some(entry) = history.previous(&self.buffer) {
            self.set_buffer(entry);
            self.redraw(out)?;
        }
        Ok(EditorAction::Continue)
    }

    fn op_history_next<H: HistoryNavigation>(
        &mut self,
        history: &mut H,
        out: &mut io::Stdout,
    ) -> io::Result<EditorAction> {
        if let Some(entry) = history.next() {
            self.set_buffer(entry);
            self.redraw(out)?;
        }
        Ok(EditorAction::Continue)
    }

    fn op_move_left<H: HistoryNavigation>(
        &mut self,
        _history: &mut H,
        out: &mut io::Stdout,
    ) -> io::Result<EditorAction> {
        if self.cursor > 0 {
            self.cursor -= 1;
            terminal::move_left(out, 1)?;
            out.flush()?;
        }
        Ok(EditorAction::Continue)
    }

    fn op_move_right<H: HistoryNavigation>(
        &mut self,
        _history: &mut H,
        out: &mut io::Stdout,
    ) -> io::Result<EditorAction> {
        if self.cursor < self.buffer.chars().count() {
            self.cursor += 1;
            terminal::move_right(out, 1)?;
            out.flush()?;
        }
        Ok(EditorAction::Continue)
    }

    pub fn redraw<W: Write>(&self, out: &mut W) -> io::Result<()> {
        write!(out, "\r")?;
        terminal::clear_line(out)?;
        write!(out, "{}{}", self.prompt_display, self.buffer)?;
        terminal::move_to_column(out, self.prompt_width + self.cursor as u16)?;
        out.flush()?;
        Ok(())
    }
}

fn finish_line<W: Write>(out: &mut W) {
    let _ = write!(out, "\r\n");
    let _ = out.flush();
}

fn char_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
