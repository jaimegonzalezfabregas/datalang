use std::fmt;

use crate::engine::RelId;
use crate::lexer::LexogramType::*;
use crate::parser::statement_reader::read_statement;
use crate::{
    lexer,
    parser::{defered_relation_reader::read_defered_relation, error::FailureExplanation},
};

use super::defered_relation_reader::DeferedRelation;
use super::error::ParserError;
use super::statement_reader::Statement;
use super::Relation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    pub conditional: Statement,
    pub relation: DeferedRelation,
}

impl fmt::Display for Conditional {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} :- {}", self.relation, self.conditional)
    }
}

impl Relation for Conditional {
    fn get_rel_id(&self) -> RelId {
        self.relation.get_rel_id()
    }
}

pub fn read_conditional(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Conditional, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum IntensionalParserStates {
        SpectingDeferedRelation,
        SpectingTrueWhen,
        SpectingCondition,
    }
    use IntensionalParserStates::*;

    if debug_print {
        println!("{debug_margin}read_intensional at {start_cursor}");
    }
    let mut cursor = start_cursor;
    let mut base_relation = None;
    let mut state = SpectingDeferedRelation;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.to_owned(), state) {
            (_, SpectingDeferedRelation) => {
                match read_defered_relation(
                    lexograms,
                    i,
                    false,
                    debug_margin.to_owned() + "|  ",
                    debug_print,
                )? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "conditional".into(),
                            failed_because: "specting relation".into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                    Ok((r, jump_to)) => {
                        cursor = jump_to;
                        base_relation = Some(r);
                        state = SpectingTrueWhen;
                    }
                }
            }
            (TrueWhen, SpectingTrueWhen) => state = SpectingCondition,
            (_, SpectingCondition) => {
                match (
                    read_statement(lexograms, i, debug_margin.to_owned() + "|  ", debug_print)?,
                    base_relation,
                ) {
                    (Err(e), _) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "conditional".into(),
                            failed_because: "specting statement".into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                    (Ok((cond, jump_to)), Some(def_rel)) => {
                        return Ok(Ok((
                            Conditional {
                                relation: def_rel,
                                conditional: cond,
                            },
                            jump_to,
                        )))
                    }
                    _ => unreachable!(),
                }
            }

            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "conditional".into(),
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: vec![],
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len() - 1,
        if_it_was: "conditional".into(),
        failed_because: "file ended".into(),
        parent_failure: vec![],
    }))
}
