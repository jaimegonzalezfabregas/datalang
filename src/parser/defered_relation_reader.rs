use std::vec;

use crate::engine::table::truth::Truth;
use crate::engine::var_context::VarContext;
use crate::engine::RelId;
use crate::lexer::LexogramType::*;
use crate::parser::assumption_reader::read_assumption;
use crate::{lexer, parser::list_reader::read_list};

use super::assumption_reader::Assumption;
use super::error::ParserError;
use super::{FailureExplanation, Relation};
use crate::parser::expresion_reader::Expresion;

#[derive(Debug, Clone)]
pub struct DeferedRelation {
    pub negated: bool,
    pub assumptions: Vec<Assumption>,
    pub rel_name: String,
    pub args: Vec<Expresion>,
}

impl Relation for DeferedRelation {
    fn get_rel_id(&self) -> RelId {
        return RelId {
            identifier: self.rel_name.clone(),
            column_count: self.args.len(),
        };
    }
}

impl DeferedRelation {
    pub fn apply(&self, context: &VarContext) -> Result<Truth, String> {
        let mut literal_vec = vec![];
        for exp in &self.args {
            literal_vec.push(exp.literalize(context)?)
        }
        Ok(Truth::from(literal_vec))
    }
}

pub fn read_defered_relation(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    check_querry: bool,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(DeferedRelation, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum RelationParserStates {
        SpectingStatementIdentifierOrNegation,
        SpectingStatementIdentifier,
        SpectingAssuming,
        SpectingStatementIdentifierOrassumptionOrNegation,
        Spectingassumption,
        SpectingComaBetweenassumptionsOrEndOfassumptions,
        SpectingStatementList,
        SpectingQuery,
    }
    use RelationParserStates::*;

    if debug_print {
        println!("{debug_margin}read_defered_relation at {start_cursor}");
    }

    let mut cursor = start_cursor;
    let mut negated = false;
    let mut op_rel_name = None;
    let mut args = vec![];
    let mut assumptions = vec![];
    let mut state = SpectingStatementIdentifierOrassumptionOrNegation;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.to_owned(), state) {
            (
                OpNot,
                SpectingStatementIdentifierOrassumptionOrNegation
                | SpectingStatementIdentifierOrNegation,
            ) => {
                negated = true;
                state = SpectingStatementIdentifier;
            }
            (_, Spectingassumption) => {
                match read_assumption(lexograms, i, debug_margin.clone() + "   ", debug_print)? {
                    Ok((assumption, jump_to)) => {
                        cursor = jump_to;
                        assumptions.push(assumption);
                    }
                    Err(err) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "defered relation".into(),
                            failed_because: format!("specting assumption").into(),
                            parent_failure: vec![err],
                        }))
                    }
                }
                state = SpectingComaBetweenassumptionsOrEndOfassumptions
            }
            (LeftKey, SpectingStatementIdentifierOrassumptionOrNegation) => {
                state = Spectingassumption;
            }
            (RightKey, SpectingComaBetweenassumptionsOrEndOfassumptions) => {
                state = SpectingAssuming;
            }
            (Coma, SpectingComaBetweenassumptionsOrEndOfassumptions) => {
                state = Spectingassumption;
            }
            (Assuming, SpectingAssuming) => state = SpectingStatementIdentifierOrNegation,
            (
                Identifier(str),
                SpectingStatementIdentifier | SpectingStatementIdentifierOrassumptionOrNegation,
            ) => {
                op_rel_name = Some(str);
                state = SpectingStatementList;
            }
            (_, SpectingStatementList) => {
                match read_list(
                    lexograms,
                    i,
                    false,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "defered relation".into(),
                            failed_because: "specting list".into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                    Ok((v, jump_to)) => {
                        cursor = jump_to;
                        args = v;
                        if check_querry {
                            state = SpectingQuery;
                        } else {
                            if let Some(rel_name) = op_rel_name {
                                return Ok(Ok((
                                    DeferedRelation {
                                        negated,
                                        assumptions,
                                        rel_name,
                                        args,
                                    },
                                    jump_to,
                                )));
                            } else {
                                unreachable!()
                            }
                        }
                    }
                }
            }
            (Query, SpectingQuery) => {
                if let Some(rel_name) = op_rel_name {
                    return Ok(Ok((
                        DeferedRelation {
                            negated,
                            assumptions,
                            rel_name,
                            args,
                        },
                        i + 1,
                    )));
                } else {
                    unreachable!()
                }
            }
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "defered relation".into(),
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: vec![],
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "defered relation".into(),
        failed_because: "file ended".into(),
        parent_failure: vec![],
    }))
}
