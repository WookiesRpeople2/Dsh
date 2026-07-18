use readline::{Color, Editor, EditorConfig, EditorResult};

use crate::state::ShellState;

pub fn read_line(state: &mut ShellState) -> Option<String> {
    let prompt_color = Color::from_name(&state.prompt_color).unwrap_or(Color::White);
    let config = EditorConfig::new(state.prompt.clone(), prompt_color);
    let mut editor = Editor::new(config);

    match editor.read(&mut state.history) {
        Ok(EditorResult::Submit(line)) => {
            state.history.push(&line);
            Some(line)
        }
        Ok(EditorResult::Cancel | EditorResult::Eof) => None,
        Err(err) => {
            eprintln!("input error: {err}");
            None
        }
    }
}
