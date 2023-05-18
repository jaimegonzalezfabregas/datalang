use super::{
    conditional_reader::Conditional,
    defered_relation_reader::{read_defered_relation, DeferedRelation},
    error::{FailureExplanation, ParserError},
};
use crate::{
    lexer::{self},
    parser::conditional_reader::read_conditional,
};

#[derive(Debug, Clone)]
pub enum Asumption {
    Relation(DeferedRelation),
    Conditional(Conditional),
}

pub fn read_asumption(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Asumption, usize), FailureExplanation>, ParserError> {
    let a;
    let b;
    match read_conditional(
        lexograms,
        start_cursor,
        debug_margin.clone() + "   ",
        debug_print,
    )? {
        Ok((ret, jump_to)) => return Ok(Ok((Asumption::Conditional(ret), jump_to))),
        Err(e) => a = e,
    }

    match read_defered_relation(
        lexograms,
        start_cursor,
        true,
        debug_margin.clone() + "   ",
        debug_print,
    )? {
        Ok((ret, jump_to)) => return Ok(Ok((Asumption::Relation(ret), jump_to))),
        Err(e) => b = e,
    }

    Ok(Err(FailureExplanation {
        lex_pos: start_cursor,
        if_it_was: "line".into(),
        failed_because: "wasnt neither an extensional nor an intensional statement".into(),
        parent_failure: vec![a, b],
    }))
}
