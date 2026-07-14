use std::io::{Stdout, Write};

use crate::color::Color;
use crate::key::{KeyCode, read_key};
use crate::terminal::{self, RawMode};

pub struct MenuItem {
    pub label: String,
    pub color: Option<Color>,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: None,
        }
    }

    pub fn colored(label: impl Into<String>, color: Color) -> Self {
        Self {
            label: label.into(),
            color: Some(color),
        }
    }
}

pub fn show_menu(items: &[MenuItem]) -> Option<usize> {
    if items.is_empty() {
        return None;
    }

    let _raw = RawMode::enable().ok()?;
    let mut out = std::io::stdout();
    terminal::hide_cursor(&mut out).ok()?;

    let (_, mut start_row) = terminal::cursor_position().unwrap_or((0, 0));
    let (_, height) = terminal::size().unwrap_or((80, 24));

    let needed_rows = items.len() as u16;
    if start_row + needed_rows >= height {
        let scroll = start_row + needed_rows - height + 1;
        terminal::scroll_up(&mut out, scroll).ok()?;
        start_row = start_row.saturating_sub(scroll);
    }

    let mut selected = 0usize;
    let result = loop {
        draw_menu(&mut out, start_row, items, selected);

        let key = {
            let mut input = std::io::stdin().lock();
            match read_key(&mut input) {
                Ok(key) => key,
                Err(_) => break None,
            }
        };

        match key.code {
            KeyCode::Up => {
                selected = if selected == 0 {
                    items.len() - 1
                } else {
                    selected - 1
                };
            }
            KeyCode::Down => {
                selected = (selected + 1) % items.len();
            }
            KeyCode::Enter => break Some(selected),
            KeyCode::Esc | KeyCode::Char('q') => break None,
            _ => {}
        }
    };

    cleanup(&mut out, start_row, items.len());
    result
}

fn draw_menu(out: &mut Stdout, start_row: u16, items: &[MenuItem], selected: usize) {
    for (i, item) in items.iter().enumerate() {
        terminal::move_to(out, 0, start_row + i as u16).ok();
        terminal::clear_line(out).ok();

        let marker = if i == selected { ">" } else { " " };
        if let Some(color) = item.color {
            terminal::set_foreground(out, color).ok();
        }
        write!(out, "{marker} {}", item.label).ok();
        if item.color.is_some() {
            terminal::reset_style(out).ok();
        }
    }
    out.flush().ok();
}

fn cleanup(out: &mut Stdout, start_row: u16, count: usize) {
    for i in 0..count {
        terminal::move_to(out, 0, start_row + i as u16).ok();
        terminal::clear_line(out).ok();
    }
    terminal::move_to(out, 0, start_row).ok();
    terminal::show_cursor(out).ok();
    out.flush().ok();
}
