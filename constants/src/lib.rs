use crossterm::style::Color as CtColor;

pub const WELCOME_MESSAGE: &str = r#"             (``',
            / `''/
           /    /
        O\/    /
        \,    /
        (    /
       /x`''7/
      (x   //`\
     / `''7'`\ \
    /    /   /()\
   (    /   `|~~|`
    `'''     |  |
             |  |
             |  |
             |  |
             |  |
           /`    `\
 ,-------'`        `'-------,
`~~~~~~~~~~~~~~~~~~~~~~~~~~~~`"#;

pub const COLORS: &[(&str, CtColor)] = &[
    ("Red", CtColor::Red),
    ("Green", CtColor::Green),
    ("Yellow", CtColor::Yellow),
    ("Blue", CtColor::Blue),
    ("Magenta", CtColor::Magenta),
    ("Cyan", CtColor::Cyan),
    ("White", CtColor::White),
    ("Black", CtColor::Black),
];

pub const CONFIG_PATH: &str = ".config/dsh/config.toml";
pub const PROMPT_SECTION: &str = "Prompt";
pub const PROMPT_SECTION_PROMPT_COLOR_KEY: &str = "prompt_color";
pub const PROMPT_SECTION_PROMPT: &str = "prompt";
pub const PROMPT_DEFAULT: &str = "⇥ ";
pub const ESC: char = '\x1b';
