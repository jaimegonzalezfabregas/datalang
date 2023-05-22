use super::{
    conditional_reader::read_conditional,
    defered_relation_reader::{read_defered_relation, DeferedRelation},
    error::*,
    inmediate_relation_reader::{read_inmediate_relation, InmediateRelation}, update_reader::Update,
};
use crate::lexer;
use crate::parser::conditional_reader::Conditional;

#[derive(Debug, Clone)]
pub enum Line {
    Relation(InmediateRelation),
    TrueWhen(Conditional),
    Query(DeferedRelation),
    Upadte(Update),
}

pub fn read_line(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Line, usize), FailureExplanation>, ParserError> {
    let a;
    let b;
    let c;
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
    match read_inmediate_relation(
        lexograms,
        start_cursor,
        debug_margin.clone() + "   ",
        debug_print,
    )? {
        Ok((i_rel, jump_to)) => return Ok(Ok((Line::Relation(i_rel), jump_to))),
        Err(e) => b = e,
    }
    match read_conditional(
        lexograms,
        start_cursor,
        debug_margin.clone() + "   ",
        debug_print,
    )? {
        Ok((ret, jump_to)) => return Ok(Ok((Line::TrueWhen(ret), jump_to))),
        Err(e) => c = e,
    }

    Ok(Err(FailureExplanation {
        lex_pos: start_cursor,
        if_it_was: "line".into(),
        failed_because: "wasnt neither an extensional nor an intensional statement".into(),
        parent_failure: (vec![a, b, c]),
    }))
}
