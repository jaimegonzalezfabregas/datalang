pub mod assumption_node;
pub mod conditional_node;
pub mod data_node;
pub mod defered_relation_node;
pub mod destructuring_array_node;
pub mod expresion_node;
pub mod inmediate_relation_node;
pub mod line_node;
pub mod list_node;
pub mod statement_node;
pub mod update_node;

pub mod error;

use crate::engine::RelId;
use crate::lexer;
use crate::parser::error::FailureExplanation;

use self::error::ParserError;
use self::line_node::*;

pub trait HasRelId {
    fn get_rel_id(&self) -> RelId;
}

pub fn parse(lexograms: &Vec<lexer::Lexogram>) -> Result<Vec<Line>, ParserError> {
    let mut ret = vec![];
    let mut cursor = 0;

    for (i, _) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match read_line(&lexograms, i)? {
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
