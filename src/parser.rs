pub mod assumption_token;
pub mod destructuring_array_token;
pub mod list_token;
pub mod statement_token;
pub mod update_token;
pub mod conditional_token;
pub mod data_token;
pub mod defered_relation_token;
pub mod expresion_token;
pub mod inmediate_relation_token;
pub mod line_token;

pub mod error;

use crate::engine::RelId;
use crate::lexer;
use crate::parser::error::FailureExplanation;

use self::error::ParserError;
use self::line_token::*;

pub trait HasRelId {
    fn get_rel_id(&self) -> RelId;
}

pub fn parse(
    lexograms: &Vec<lexer::Lexogram>,
) -> Result<Vec<Line>, ParserError> {
    let mut ret = vec![];
    let mut cursor = 0;

    for (i, _) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match read_line(&lexograms, i, "".into())? {
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
