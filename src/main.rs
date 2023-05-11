mod lexer;
mod parser;
use std::fs::File;

#[derive(Debug)]
enum DLErr {
    LexerError(lexer::LexerError),
    IOError(std::io::Error),
    ParserError(parser::ParserError),
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

impl From<std::io::Error> for DLErr {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

fn main() -> Result<(), DLErr> {
    let f = File::open("example.dl")?;

    let lexic = lexer::lex(f)?;
    println!("lexografic analisis: {:?}", lexic);

    let _ast = parser::parse(lexic)?;

    Ok(())
}
