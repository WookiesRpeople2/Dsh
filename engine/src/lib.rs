pub mod ast;
pub mod history;
pub mod lexer;
pub mod parser;
pub mod readline;
pub mod state;

#[macro_export]
macro_rules! field_default {
    () => {
        None
    };
    ($default:expr) => {
        Some($default.to_string())
    };
}

#[macro_export]
macro_rules! define_builtins {
    (
        $(
            $cmd_str:literal => $variant:ident $({ $( $field:ident $(= $default:expr)? ),* $(,)? })?
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone)]
        pub enum BuiltinCommand {
            $(
                $variant $( { $( $field: String ),* } )?,
            )*
        }

        use $crate::parser::Parser;

        impl BuiltinCommand {
            pub fn try_parse(program: &str, args: &[String]) -> Option<CommandType> {
                match program {
                    $(
                        $cmd_str => {
                            $(
                                #[allow(unused_mut)]
                                let mut idx = 0usize;
                                $(
                                    let $field = Parser::get_arg(
                                        args,
                                        idx,
                                        $crate::field_default!($($default)?),
                                        program,
                                    );
                                    idx += 1;
                                )*
                                let _ = idx;
                            )?
                            Some(CommandType::Builtin(BuiltinCommand::$variant $( { $( $field ),* } )?))
                        }
                    )*
                    _ => None,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! tok {
    ($token:expr) => {
        ($token, None)
    };
    ($token:expr, $next:expr => $alt:expr) => {
        ($token, Some(($next, $alt)))
    };
}
