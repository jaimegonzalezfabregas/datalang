mod defered_relation_reader;
mod inmediate_relation_reader;
mod line_reader;
mod var_literal_reader;
mod statement_reader;
mod conditional_reader;
mod list_reader;
mod expresion_reader;
mod logical_statement_concatenation_reader;
mod destructuring_array_reader;
mod asumption_reader;

pub mod error;
mod common;



use crate::lexer;
use crate::parser::error::FailureExplanation;

use self::error::ParserError;
use self::line_reader::*;



const DEBUG_PRINT: bool = false;

pub fn parse(lexograms: &Vec<lexer::Lexogram>) -> Result<Vec<Line>, ParserError> {
    let mut ret = vec![];
    let mut cursor = 0;

    for (i, _) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match read_line(&lexograms, i, "".into(), DEBUG_PRINT)? {
            Ok((statement, jump_to)) => {
                ret.push(statement);
                cursor = jump_to;
            }
            Err(e) => {
                return Err(ParserError::SyntaxError(e));
            }
        }
    }

    Ok(ret)
}
