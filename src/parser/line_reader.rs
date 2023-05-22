use super::{
    asumption_reader::{read_asumption, Asumption},
    defered_relation_reader::{read_defered_relation, DeferedRelation},
    error::*,
};
use crate::lexer;

#[derive(Debug, Clone)]
pub enum Line {
    Asumption(Asumption),
    Query(DeferedRelation),
}

pub fn read_line(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Line, usize), FailureExplanation>, ParserError> {
    let a;
    let b;

    match read_defered_relation(
        lexograms,
        start_cursor,
        true,
        debug_margin.clone() + "   ",
        debug_print,
    )? {
        Ok((defered_rel, jump_to)) => return Ok(Ok((Line::Query(defered_rel), jump_to))),
        Err(e) => a = e,
    }
    match read_asumption(lexograms, start_cursor, debug_margin, debug_print)? {
        Ok((defered_rel, jump_to)) => return Ok(Ok((Line::Asumption(defered_rel), jump_to))),
        Err(e) => b = e,
    }

    Ok(Err(FailureExplanation {
        lex_pos: start_cursor,
        if_it_was: "line".into(),
        failed_because: "wasnt neither an extensional nor an intensional statement".into(),
        parent_failure: (vec![a, b]),
    }))
}
