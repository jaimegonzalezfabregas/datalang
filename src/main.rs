pub mod engine;
mod lexer;
mod parser;
use std::fs::File;

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
    let f = File::open("example.dl")?;

    let lexic = lexer::lex(f)?;
    println!("lexografic analisis: {:?}\n", lexic);

    let ast = parser::parse(lexic)?;
    println!("sintaxis analisis: {:?}\n", ast);

    let mut engine = Engine::new();
    engine.ingest(ast)?;

    println!("engine instrinsics: {:?}\n", engine);

    Ok(())
}
