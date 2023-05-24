pub mod assumption_reader;
pub mod conditional_reader;
pub mod data_reader;
pub mod defered_relation_reader;
pub mod destructuring_array_reader;
pub mod expresion_reader;
pub mod inmediate_relation_reader;
pub mod line_reader;
pub mod list_reader;
pub mod statement_reader;
pub mod update_reader;

pub mod error;

use crate::engine::RelId;
use crate::lexer;
use crate::parser::error::FailureExplanation;

use self::error::ParserError;
use self::line_reader::*;

pub trait Relation {
    fn get_rel_id(&self) -> RelId;
}

pub fn parse(
    lexograms: &Vec<lexer::Lexogram>,
    debug_print: bool,
) -> Result<Vec<Line>, ParserError> {
    let mut ret = vec![];
    let mut cursor = 0;

    for (i, _) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match read_line(&lexograms, i, "".into(), debug_print)? {
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
