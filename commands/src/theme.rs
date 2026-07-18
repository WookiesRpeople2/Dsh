use constants::{PROMPT_SECTION, PROMPT_SECTION_PROMPT_COLOR_KEY};
use engine::state::ShellState;
use helpers::io::write_config;
use readline::{MenuItem, MenuStyle, NAMED_COLORS, show_menu_with};

pub fn color(state: &mut ShellState) {
    let items: Vec<MenuItem<&str>> = NAMED_COLORS
        .iter()
        .map(|(name, color)| MenuItem::new(*name, *name).color(*color))
        .collect();

    match show_menu_with(&items, &MenuStyle::titled("Prompt color")) {
        Some(name) => {
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
