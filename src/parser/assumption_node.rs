use core::fmt;

use super::{
    conditional_node::Conditional,
    defered_relation_node::{read_defered_relation, DeferedRelation},
    error::{FailureExplanation, ParserError},
    inmediate_relation_node::{read_inmediate_relation, InmediateRelation},
    update_node::{read_update, Update},
};
use crate::{
    lexer::{self},
    parser::conditional_node::read_conditional,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Assumption {
    RelationInmediate(InmediateRelation),
    RelationDefered(DeferedRelation),
    Conditional(Conditional),
    Update(Update),
}

impl fmt::Display for Assumption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Assumption::RelationInmediate(rel) => write!(f, "{rel}"),
            Assumption::RelationDefered(rel) => write!(f, "{rel}"),
            Assumption::Conditional(cond) => write!(f, "{cond}"),
            Assumption::Update(upd) => write!(f, "{upd}"),
        }
    }
}

pub fn read_assumption(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
) -> Result<Result<(Assumption, usize), FailureExplanation>, ParserError> {
    let a;
    let b;
    let c;
    let d;
    match read_inmediate_relation(lexograms, start_cursor)? {
        Ok((i_rel, jump_to)) => return Ok(Ok((Assumption::RelationInmediate(i_rel), jump_to))),
        Err(e) => a = e,
    }
    match read_conditional(lexograms, start_cursor)? {
        Ok((ret, jump_to)) => return Ok(Ok((Assumption::Conditional(ret), jump_to))),
        Err(e) => b = e,
    }
    match read_update(lexograms, start_cursor)? {
        Ok((ret, jump_to)) => return Ok(Ok((Assumption::Update(ret), jump_to))),
        Err(e) => c = e,
    }
    match read_defered_relation(lexograms, start_cursor, false)? {
        Ok((d_rel, jump_to)) => return Ok(Ok((Assumption::RelationDefered(d_rel), jump_to))),
        Err(e) => d = e,
    }

    Ok(Err(FailureExplanation {
        lex_pos: start_cursor,
        if_it_was: "assumption".into(),
        failed_because: "wasnt any type of assumption".into(),
        parent_failure: vec![a, b, c, d],
    }))
}
