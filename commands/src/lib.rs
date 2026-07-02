pub mod theme;
use engine::{ast::BuiltinCommand, state::ShellState};

use crate::theme::color;

pub fn execute_builtin(command: BuiltinCommand, state: &mut ShellState) -> Result<(), String> {
    match command {
        BuiltinCommand::Cd { path } => {
            std::env::set_current_dir(path).map_err(|e| e.to_string())?
        }

        BuiltinCommand::Export { key, value } => unsafe {
            std::env::set_var(key, value);
        },

        BuiltinCommand::Alias { name, command } => println!("alias {}={}", name, command),

        BuiltinCommand::Exit => std::process::exit(0),
        BuiltinCommand::Theme => color(state),
    }

    Ok(())
}
