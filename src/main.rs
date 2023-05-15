pub mod engine;
mod lexer;
mod parser;
mod syntax;
use std::{
    fs::{read, read_to_string, File},
    io,
};

use crate::{engine::Engine, lexer::Lexogram};

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

fn get_ASTs_from_chars(commands: String) -> Result<Vec<syntax::Line>, DLErr> {
    let lexic = lexer::lex(commands)?;
    println!(
        "lexografic analisis: {:?}\n",
        lexic
            .iter()
            .enumerate()
            .collect::<Vec<(usize, &Lexogram)>>()
    );

    let ast_vec = parser::parse(lexic)?;
    println!("sintaxis analisis: {:?}\n", ast_vec);

    Ok(ast_vec)
}

fn main() -> Result<(), DLErr> {
    let initializing_commands = read_to_string("example.dl")?;

    let mut engine = Engine::new();
    engine.ingest(get_ASTs_from_chars(initializing_commands)?)?;

    let mut buffer = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.

    while buffer != "exit" {
        stdin.read_line(&mut buffer)?;
        let ast = get_ASTs_from_chars(buffer)?;
        engine.ingest(ast)?;
        buffer = String::new();
    }

    println!("engine instrinsics: {:?}\n", engine);

    Ok(())
}
