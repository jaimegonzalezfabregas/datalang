pub mod engine;
mod lexer;
mod parser;
mod syntax;
mod utils;
use std::{fs::read_to_string, io};

use crate::engine::Engine;

#[derive(Debug)]
enum DLErr {
    LexerError(lexer::LexerError),
    IOError(std::io::Error),
    ParserError(parser::ParserError),
    RuntimeError(engine::RuntimeError),
}

impl From<lexer::LexerError> for DLErr {
    fn from(e: lexer::LexerError) -> Self {
        Self::LexerError(e)
    }
}

impl From<parser::ParserError> for DLErr {
    fn from(e: parser::ParserError) -> Self {
        Self::ParserError(e)
    }
}

impl From<engine::RuntimeError> for DLErr {
    fn from(e: engine::RuntimeError) -> Self {
        Self::RuntimeError(e)
    }
}

impl From<std::io::Error> for DLErr {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

fn main() -> Result<(), DLErr> {
    let initializing_commands = read_to_string("example.dl")?;

    let mut engine = Engine::new();
    engine.input(initializing_commands);

    let stdin = io::stdin();

    loop {
        let mut buffer = String::new();

        print!(">");
        io::Write::flush(&mut io::stdout())
            .ok()
            .expect("Could not flush stdout");
        stdin.read_line(&mut buffer)?;

        if buffer.eq("/exit\r\n") || buffer.eq("/exit\n") {
            break;
        }

        engine.input(buffer);
    }

    Ok(())
}
