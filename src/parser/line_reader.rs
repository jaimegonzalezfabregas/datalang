use core::fmt;

use super::{
    assumption_reader::{read_assumption, Assumption},
    defered_relation_reader::{read_defered_relation, DeferedRelation},
    error::*,
};
use crate::lexer;

#[derive(Debug, Clone)]
pub enum Line {
    Assumption(Assumption),
    Query(DeferedRelation),
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Assumption(ass) => write!(f, "{ass}"),
            Line::Query(que) => write!(f, "{que}"),
        }
    }
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
        debug_margin.to_owned() + "|  ",
        debug_print,
    )? {
        Ok((defered_rel, jump_to)) => return Ok(Ok((Line::Query(defered_rel), jump_to))),
        Err(e) => a = e,
    }
    match read_assumption(lexograms, start_cursor, debug_margin, debug_print)? {
        Ok((defered_rel, jump_to)) => return Ok(Ok((Line::Assumption(defered_rel), jump_to))),
        Err(e) => b = e,
    }

    Ok(Err(FailureExplanation {
        lex_pos: start_cursor,
        if_it_was: "line".into(),
        failed_because: "wasnt neither an extensional nor an intensional statement".into(),
        parent_failure: (vec![a, b]),
    }))
}
