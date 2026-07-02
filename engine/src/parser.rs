use errors::errors::ShellErrors;

use crate::ast::*;
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.position].clone();
        self.position += 1;
        token
    }

    pub fn parse(&mut self) -> Shell {
        let mut pipelines = Vec::new();

        while *self.peek() != Token::EOF {
            pipelines.push(self.parse_pipeline());
        }

        Shell { pipelines }
    }

    fn parse_pipeline(&mut self) -> Pipeline {
        let mut commands = Vec::new();

        loop {
            commands.push(self.parse_command());

            match self.peek() {
                Token::Pipe => {
                    self.advance();
                }

                _ => break,
            }
        }

        Pipeline { commands }
    }

    fn parse_command(&mut self) -> Command {
        let mut args = Vec::new();

        let mut program = String::new();

        let mut redirects = Vec::new();

        loop {
            match self.peek() {
                Token::Command(_) => {
                    let word = match self.advance() {
                        Token::Command(w) => w,
                        _ => unreachable!(),
                    };
                    if program.is_empty() {
                        program = word;
                    } else {
                        args.push(word);
                    }
                }

                Token::RedirectOut => {
                    self.advance();
                    let file = self.expect_word();
                    redirects.push(Redirect::Output { file });
                }

                Token::RedirectAppend => {
                    self.advance();
                    let file = self.expect_word();
                    redirects.push(Redirect::Append { file });
                }

                Token::RedirectIn => {
                    self.advance();
                    let file = self.expect_word();
                    redirects.push(Redirect::Input { file });
                }

                Token::HomeSymbol => {
                    self.advance();
                    let mut word = std::env::var("HOME").expect("HOME not set");
                    if let Token::Command(_) = self.peek() {
                        if let Token::Command(rest) = self.advance() {
                            word.push_str(&rest);
                        }
                    }
                    if program.is_empty() {
                        program = word;
                    } else {
                        args.push(word);
                    }
                }

                Token::Pipe | Token::EOF => {
                    break;
                }
            }
        }

        Command {
            command_type: self.make_command(program, args),

            redirects,
        }
    }

    fn expect_word(&mut self) -> String {
        match self.advance() {
            Token::Command(word) => word,

            other => panic!("{}", ShellErrors::InvalidCommand(format!("{:?}", other))),
        }
    }

    pub fn get_arg(
        args: &[String],
        index: usize,
        default: Option<String>,
        program: &str,
    ) -> String {
        match args.get(index) {
            Some(arg) => arg.clone(),

            None => match default {
                Some(value) => value,

                None => panic!("{}", ShellErrors::InvalidCommand(program.to_string())),
            },
        }
    }

    fn make_command(&self, program: String, args: Vec<String>) -> CommandType {
        BuiltinCommand::try_parse(&program, &args)
            .unwrap_or(CommandType::External { program, args })
    }
}
