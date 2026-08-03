use crate::define_builtins;

#[derive(Debug, Clone)]
pub struct Shell {
    pub pipelines: Vec<Pipeline>,
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub command_type: CommandType,
    pub redirects: Vec<Redirect>,
}

#[derive(Debug, Clone)]
pub enum CommandType {
    Builtin(BuiltinCommand),
    External { program: String, args: Vec<String> },
}

define_builtins! {
    "cd"     => Cd     { path = "~" },
    "alias"  => Alias  { name, command },
    "export" => Export { key, value },
    "loc"    => Loc { path },
    "theme"  => Theme,
    "exit"   => Exit,
}

#[derive(Debug, Clone)]
pub enum Redirect {
    Input { file: String },
    Output { file: String },
    Append { file: String },
}
