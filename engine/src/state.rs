use constants::{
    PROMPT_DEFAULT, PROMPT_SECTION, PROMPT_SECTION_PROMPT, PROMPT_SECTION_PROMPT_COLOR_KEY,
};
use helpers::io::{get_value, read_config};

use crate::history::History;

#[derive(Debug, Clone)]
pub struct ShellState {
    pub prompt: String,
    pub prompt_color: String,
    pub history: History,
}

impl ShellState {
    pub fn new() -> Self {
        let config = read_config();
        let prompt = get_value(&config, PROMPT_SECTION, PROMPT_SECTION_PROMPT);
        let prompt_color = get_value(&config, PROMPT_SECTION, PROMPT_SECTION_PROMPT_COLOR_KEY);
        Self {
            prompt: prompt.unwrap_or(PROMPT_DEFAULT).to_string(),
            prompt_color: prompt_color.unwrap_or("White").to_string(),
            history: History::default(),
        }
    }
}
