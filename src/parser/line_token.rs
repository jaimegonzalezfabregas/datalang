use core::fmt;

use super::{
    assumption_token::{read_assumption, Assumption},
    defered_relation_token::{read_defered_relation, DeferedRelation},
    error::*,
};
use crate::lexer::{self, LexogramType};

#[derive(Debug, Clone)]
pub enum Line {
    Assumption(Assumption),
    Query(DeferedRelation),
    Comment(Box<Line>),
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Assumption(ass) => write!(f, "{ass}"),
            Line::Query(que) => write!(f, "{que}"),
            Line::Comment(line) => write!(f, "#{line}"),
        }
    }
}

pub fn read_line(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Line, usize), FailureExplanation>, ParserError> {
    if let LexogramType::Comment = lexograms[start_cursor].l_type {
        match read_line(
            lexograms,
            start_cursor + 1,
            debug_margin.to_owned() + "|  ",
            debug_print,
        )? {
            Ok((line, jump_to)) => return Ok(Ok((Line::Comment(Box::new(line)), jump_to))),
            Err(e) => {
                return Ok(Err(FailureExplanation {
                    lex_pos: start_cursor,
                    if_it_was: "comment".into(),
                    failed_because: "a valid line wasnt found".into(),
                    parent_failure: vec![e],
                }))
            }
        }
    } else {
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
        match read_assumption(
            lexograms,
            start_cursor,
            debug_margin.to_owned() + "|  ",
            debug_print,
        )? {
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
}
