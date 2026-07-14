use crate::color::Color;
use crate::key::{KeyCode, KeyEvent, Modifiers};
use crate::style;
use crate::terminal::{self, RawMode};
use std::io::{self, Read, Write};

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
        self.buffer = text.into();
        self.cursor = self.buffer.chars().count();
    }

    pub fn read<H: HistoryNavigation>(
        &mut self,
        history: &mut H,
    ) -> io::Result<EditorResult> {
        let _raw = RawMode::enable()?;
        let mut out = io::stdout();

        write!(out, "{}", self.prompt_display)?;
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
        match key.code {
            KeyCode::Enter => Ok(EditorAction::Submit),
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let start = char_byte_idx(&self.buffer, self.cursor - 1);
                    let end = char_byte_idx(&self.buffer, self.cursor);
                    self.buffer.replace_range(start..end, "");
                    self.cursor -= 1;
                    self.redraw(out)?;
                }
                Ok(EditorAction::Continue)
            }
            KeyCode::Up => {
                if let Some(entry) = history.previous(&self.buffer) {
                    self.set_buffer(entry);
                    self.redraw(out)?;
                }
                Ok(EditorAction::Continue)
            }
            KeyCode::Down => {
                if let Some(entry) = history.next() {
                    self.set_buffer(entry);
                    self.redraw(out)?;
                }
                Ok(EditorAction::Continue)
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    terminal::move_left(out, 1)?;
                    out.flush()?;
                }
                Ok(EditorAction::Continue)
            }
            KeyCode::Right => {
                if self.cursor < self.buffer.chars().count() {
                    self.cursor += 1;
                    terminal::move_right(out, 1)?;
                    out.flush()?;
                }
                Ok(EditorAction::Continue)
            }
            KeyCode::Char('c') if key.modifiers.contains(Modifiers::CONTROL) => {
                Ok(EditorAction::Cancel)
            }
            KeyCode::Char('d')
                if key.modifiers.contains(Modifiers::CONTROL) && self.buffer.is_empty() =>
            {
                Ok(EditorAction::Cancel)
            }
            KeyCode::Char(ch) => {
                let at = char_byte_idx(&self.buffer, self.cursor);
                self.buffer.insert(at, ch);
                self.cursor += 1;
                self.redraw(out)?;
                Ok(EditorAction::Continue)
            }
            _ => Ok(EditorAction::Continue),
        }
    }

    pub fn redraw<W: Write>(&self, out: &mut W) -> io::Result<()> {
        terminal::move_to_column(out, 0)?;
        terminal::clear_line(out)?;
        write!(out, "{}{}", self.prompt_display, self.buffer)?;
        terminal::move_to_column(out, self.prompt_width + self.cursor as u16)?;
        out.flush()?;
        Ok(())
    }
}

fn finish_line<W: Write>(out: &mut W) {
    let _ = writeln!(out);
    let _ = out.flush();
}

fn char_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn read_key<R: Read>(input: &mut R) -> io::Result<KeyEvent> {
    crate::key::read_key(input)
}
