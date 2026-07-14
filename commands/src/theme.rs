use constants::{PROMPT_SECTION, PROMPT_SECTION_PROMPT_COLOR_KEY};
use engine::state::ShellState;
use helpers::io::write_config;
use readline::{MenuItem, NAMED_COLORS, show_menu};

pub fn color(state: &mut ShellState) {
    let items: Vec<MenuItem> = NAMED_COLORS
        .iter()
        .map(|(name, color)| MenuItem::colored(*name, *color))
        .collect();

    match show_menu(&items) {
        Some(i) => {
            let (name, _) = NAMED_COLORS[i];
            println!("Color set to {name}");
            write_config(
                PROMPT_SECTION.to_string(),
                PROMPT_SECTION_PROMPT_COLOR_KEY.to_string(),
                name.to_string(),
            );

            state.prompt_color = name.to_string();
        }
        None => println!("Cancelled, no color selected."),
    }
}
