use core::fmt;

use conditional_compilation::*;

use crate::lexer::LexogramType::*;

use crate::{
    lexer,
    parser::{defered_relation_node::read_defered_relation, error::FailureExplanation},
};

use super::defered_relation_node::DeferedRelation;
use super::error::ParserError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    pub filter: DeferedRelation,
    pub goal: DeferedRelation,
}

impl fmt::Display for Update {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.filter, self.goal)
    }
}

pub fn read_update(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
) -> Result<Result<(Update, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum IntensionalParserStates {
        SpectingDeferedRelationFilter,
        SpectingUpdate,
        SpectingDeferedRelationGoal,
    }
    use IntensionalParserStates::*;

    printparse!("read_intensional at {}", start_cursor);

    let mut cursor = start_cursor;
    let mut op_filter_rel = None;
    let mut state = SpectingDeferedRelationFilter;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.to_owned(), state) {
            (_, SpectingDeferedRelationFilter) => {
                match read_defered_relation(lexograms, i, false)? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "update".into(),
                            failed_because: "specting relation".into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                    Ok((r, jump_to)) => {
                        cursor = jump_to;
                        op_filter_rel = Some(r);
                        state = SpectingUpdate;
                    }
                }
            }
            (Update, SpectingUpdate) => state = SpectingDeferedRelationGoal,
            (_, SpectingDeferedRelationGoal) => {
                match (read_defered_relation(lexograms, i, false)?, op_filter_rel) {
                    (Err(e), _) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "update".into(),
                            failed_because: "specting relation".into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                    (Ok((r, jump_to)), Some(filter_rel)) => {
                        return Ok(Ok((
                            Update {
                                filter: filter_rel,
                                goal: r,
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
                    if_it_was: "update".into(),
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: vec![],
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len() - 1,
        if_it_was: "update".into(),
        failed_because: "file ended".into(),
        parent_failure: vec![],
    }))
}
