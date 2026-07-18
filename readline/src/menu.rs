use std::collections::HashMap;
use std::io::{Stdout, Write};
use std::sync::LazyLock;

use crate::color::Color;
use crate::key::{KeyCode, KeyEvent, Modifiers, read_key};
use crate::style::{self, BOLD, DIM, RESET, REVERSE};
use crate::terminal::{self, RawMode};

#[derive(Debug, Clone)]
pub struct MenuItem<T> {
    pub label: String,
    pub value: T,
    pub color: Option<Color>,
}

impl<T> MenuItem<T> {
    pub fn new(label: impl Into<String>, value: T) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct MenuStyle {
    pub title: Option<String>,
}

impl MenuStyle {
    pub fn titled(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
        }
    }
}

#[derive(Clone, Copy)]
enum MenuOp {
    MoveUp,
    MoveDown,
    Confirm,
    Dismiss,
}

static MENU_BINDINGS: LazyLock<HashMap<KeyEvent, MenuOp>> = LazyLock::new(|| {
    HashMap::from([
        (
            KeyEvent {
                code: KeyCode::Up,
                modifiers: Modifiers::empty(),
            },
            MenuOp::MoveUp,
        ),
        (
            KeyEvent {
                code: KeyCode::Down,
                modifiers: Modifiers::empty(),
            },
            MenuOp::MoveDown,
        ),
        (
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: Modifiers::empty(),
            },
            MenuOp::Confirm,
        ),
        (
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: Modifiers::empty(),
            },
            MenuOp::Dismiss,
        ),
        (
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: Modifiers::empty(),
            },
            MenuOp::Dismiss,
        ),
    ])
});

pub fn show_menu<T: Clone>(items: &[MenuItem<T>]) -> Option<T> {
    show_menu_with(items, &MenuStyle::default())
}

pub fn show_menu_with<T: Clone>(items: &[MenuItem<T>], style: &MenuStyle) -> Option<T> {
    if items.is_empty() {
        return None;
    }

    let _raw = RawMode::enable().ok()?;
    let mut out = std::io::stdout();
    terminal::hide_cursor(&mut out).ok()?;

    let (_, mut start_row) = terminal::cursor_position().unwrap_or((0, 0));
    let (_, height) = terminal::size().unwrap_or((80, 24));

    let frame_rows = menu_frame_rows(style, items.len());
    if start_row + frame_rows >= height {
        let scroll = start_row + frame_rows - height + 1;
        terminal::scroll_up(&mut out, scroll).ok()?;
        start_row = start_row.saturating_sub(scroll);
    }

    let mut selected = 0usize;
    let result = loop {
        draw_menu(&mut out, start_row, items, style, selected);

        let key = {
            let mut input = std::io::stdin().lock();
            match read_key(&mut input) {
                Ok(key) => key,
                Err(_) => break None,
            }
        };

        let Some(op) = MENU_BINDINGS.get(&key).copied() else {
            continue;
        };

        match op {
            MenuOp::MoveUp => {
                selected = if selected == 0 {
                    items.len() - 1
                } else {
                    selected - 1
                };
            }
            MenuOp::MoveDown => {
                selected = (selected + 1) % items.len();
            }
            MenuOp::Confirm => break Some(items[selected].value.clone()),
            MenuOp::Dismiss => break None,
        }
    };

    cleanup(&mut out, start_row, frame_rows);
    result
}

fn menu_frame_rows(style: &MenuStyle, item_count: usize) -> u16 {
    let title_rows = u16::from(style.title.is_some());
    let spacer = u16::from(style.title.is_some());
    title_rows + spacer + item_count as u16 + 1 + 1
}

fn draw_menu<T>(
    out: &mut Stdout,
    start_row: u16,
    items: &[MenuItem<T>],
    style: &MenuStyle,
    selected: usize,
) {
    let mut row = start_row;

    if let Some(title) = &style.title {
        terminal::move_to(out, 0, row).ok();
        terminal::clear_line(out).ok();
        write!(out, "{BOLD}{title}{RESET}").ok();
        row += 1;

        terminal::move_to(out, 0, row).ok();
        terminal::clear_line(out).ok();
        row += 1;
    }

    for (i, item) in items.iter().enumerate() {
        terminal::move_to(out, 0, row).ok();
        terminal::clear_line(out).ok();
        draw_item(out, item, i == selected);
        row += 1;
    }

    terminal::move_to(out, 0, row).ok();
    terminal::clear_line(out).ok();
    row += 1;

    terminal::move_to(out, 0, row).ok();
    terminal::clear_line(out).ok();
    write!(out, "{DIM}↑↓ move   enter select   esc cancel{RESET}").ok();

    out.flush().ok();
}

fn draw_item<T>(out: &mut Stdout, item: &MenuItem<T>, selected: bool) {
    if selected {
        write!(out, "{REVERSE}{BOLD} › ").ok();
        write_label(out, item, true);
        write!(out, " {RESET}").ok();
        return;
    }

    write!(out, "{DIM}   {RESET}").ok();
    write_label(out, item, false);
}

fn write_label<T>(out: &mut Stdout, item: &MenuItem<T>, selected: bool) {
    if let Some(color) = item.color {
        if selected {
            write!(out, "{}", item.label).ok();
        } else {
            write!(out, "{}", style::styled(&item.label, color)).ok();
        }
    } else if selected {
        write!(out, "{}", item.label).ok();
    } else {
        write!(out, "{DIM}{}{RESET}", item.label).ok();
    }
}

fn cleanup(out: &mut Stdout, start_row: u16, frame_rows: u16) {
    for i in 0..frame_rows {
        terminal::move_to(out, 0, start_row + i).ok();
        terminal::clear_line(out).ok();
    }
    terminal::move_to(out, 0, start_row).ok();
    terminal::show_cursor(out).ok();
    out.flush().ok();
}
