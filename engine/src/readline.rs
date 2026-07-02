use constants::COLORS;
use crossterm::{
    cursor::{MoveLeft, MoveToColumn, position},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::{Color as CtColor, Stylize},
    terminal::{self, Clear, ClearType},
};
use std::io::{Write, stdout};

use crate::{history::History, state::ShellState};

pub struct ReadParams<'a> {
    pub out: &'a mut std::io::Stdout,
    pub prompt_str: String,
    pub prompt_width: u16,
    pub buffer: String,
    pub history: History,
    pub cursor: usize,
}

pub fn read_line(state: &mut ShellState) -> Option<String> {
    let mut out = stdout();
    let prompt = prompt(state);
    terminal::enable_raw_mode().ok()?;

    write!(out, "{}", prompt).ok();
    out.flush().ok();

    let mut read_params = ReadParams {
        out: &mut out,
        prompt_str: prompt,
        prompt_width: state.prompt.chars().count() as u16,
        buffer: String::new(),
        history: state.history.clone(),
        cursor: 0,
    };

    let line = read_key_code(&mut read_params);
    terminal::disable_raw_mode().ok()?;

    if let Some(ref l) = line {
        state.history = read_params.history;
        state.history.push(l);
    }

    line
}

fn read_key_code(read_params: &mut ReadParams) -> Option<String> {
    loop {
        match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter => {
                    handle_enter(read_params.out);
                    read_params.out.flush().ok();
                    break Some(read_params.buffer.to_string());
                }

                KeyCode::Backspace => {
                    if read_params.cursor > 0 {
                        let start = char_byte_idx(&read_params.buffer, read_params.cursor - 1);
                        let end = char_byte_idx(&read_params.buffer, read_params.cursor);
                        read_params.buffer.replace_range(start..end, "");
                        read_params.cursor -= 1;
                        redraw(read_params);
                    }
                }

                KeyCode::Up => {
                    if let Some(entry) = read_params.history.up(&read_params.buffer.to_string()) {
                        read_params.buffer = entry.to_string();
                        read_params.cursor = read_params.buffer.to_string().chars().count();
                        redraw(read_params);
                    }
                }

                KeyCode::Down => {
                    if let Some(entry) = read_params.history.down() {
                        read_params.buffer = entry.to_string();
                        read_params.cursor = read_params.buffer.to_string().chars().count();
                        redraw(read_params);
                    }
                }

                KeyCode::Left => {
                    if read_params.cursor > 0 {
                        read_params.cursor -= 1;
                        execute!(read_params.out, MoveLeft(1)).ok();
                        read_params.out.flush().ok();
                    }
                }

                KeyCode::Right => {
                    if read_params.cursor < read_params.buffer.chars().count() {
                        read_params.cursor += 1;
                        execute!(read_params.out, crossterm::cursor::MoveRight(1)).ok();
                        read_params.out.flush().ok();
                    }
                }

                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    break None;
                }

                KeyCode::Char('d')
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && read_params.buffer.is_empty() =>
                {
                    break None;
                }

                KeyCode::Char(c) => {
                    let at = char_byte_idx(&read_params.buffer, read_params.cursor);
                    read_params.buffer.insert(at, c);
                    read_params.cursor += 1;
                    redraw(read_params);
                }

                _ => {}
            },
            Ok(_) => {}
            Err(_) => break None,
        }
    }
}

fn redraw(params: &mut ReadParams) {
    execute!(params.out, MoveToColumn(0), Clear(ClearType::CurrentLine)).ok();
    write!(params.out, "{}{}", params.prompt_str, params.buffer).ok();
    execute!(
        params.out,
        MoveToColumn(params.prompt_width + params.cursor as u16)
    )
    .ok();
    params.out.flush().ok();
}

fn handle_enter(out: &mut std::io::Stdout) {
    let (_, row) = position().unwrap_or((0, 0));
    let rows = terminal::size().map(|(_, h)| h).unwrap_or(0);
    if row + 1 >= rows {
        execute!(out, terminal::ScrollUp(1), MoveToColumn(0)).ok();
    } else {
        execute!(out, crossterm::cursor::MoveToNextLine(1)).ok();
    }
}

fn char_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn prompt(state: &ShellState) -> String {
    state
        .clone()
        .prompt
        .with(
            COLORS
                .iter()
                .find(|(n, _)| *n == state.prompt_color)
                .map(|(_, c)| *c)
                .unwrap_or(CtColor::White),
        )
        .to_string()
}
