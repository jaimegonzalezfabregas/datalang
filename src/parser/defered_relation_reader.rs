use crate::engine::RelId;
use crate::lexer::LexogramType::*;
use crate::parser::asumption_reader::read_asumption;
use crate::{lexer, parser::list_reader::read_list};

use super::asumption_reader::Asumption;
use super::error::ParserError;
use super::FailureExplanation;
use crate::parser::expresion_reader::Expresion;

#[derive(Debug, Clone)]
pub struct DeferedRelation {
    pub asumptions: Vec<Asumption>,
    pub rel_name: String,
    pub args: Vec<Expresion>,
}

impl DeferedRelation {
    pub fn get_rel_id(&self) -> RelId {
        return RelId {
            identifier: self.rel_name.clone(),
            column_count: self.args.len(),
        };
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
        SpectingStatementIdentifier,
        SpectingAssuming,
        SpectingStatementIdentifierOrAsumption,
        SpectingAsumption,
        SpectingComaBetweenAsumptionsOrEndOfAsumptions,
        SpectingStatementList,
        SpectingQuery,
    }
    use RelationParserStates::*;

    if debug_print {
        println!("{debug_margin}read_defered_relation at {start_cursor}");
    }

    let mut cursor = start_cursor;
    let mut op_rel_name = None;
    let mut args = vec![];
    let mut asumptions = vec![];
    let mut state = SpectingStatementIdentifierOrAsumption;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.to_owned(), state) {
            (_, SpectingAsumption) => {
                match read_asumption(lexograms, i, debug_margin.clone() + "   ", debug_print)? {
                    Ok((asumption, jump_to)) => {
                        cursor = jump_to;
                        asumptions.push(asumption);
                    }
                    Err(err) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "defered relation".into(),
                            failed_because: format!("specting asumption").into(),
                            parent_failure: vec![err],
                        }))
                    }
                }
                state = SpectingComaBetweenAsumptionsOrEndOfAsumptions
            }
            (LeftKey, SpectingStatementIdentifierOrAsumption) => {
                state = SpectingAsumption;
            }
            (RightKey, SpectingComaBetweenAsumptionsOrEndOfAsumptions) => {
                state = SpectingAssuming;
            }
            (Coma, SpectingComaBetweenAsumptionsOrEndOfAsumptions) => {
                state = SpectingAsumption;
            }
            (Assuming, SpectingAssuming) => state = SpectingStatementIdentifier,
            (
                Identifier(str),
                SpectingStatementIdentifier | SpectingStatementIdentifierOrAsumption,
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
                                        asumptions,
                                        rel_name,
                                        args,
                                    },
                                    jump_to,
                                )));
                            } else {
                                panic!("unreacheable state");
                            }
                        }
                    }
                }
            }
            (Query, SpectingQuery) => {
                if let Some(rel_name) = op_rel_name {
                    return Ok(Ok((
                        DeferedRelation {
                            asumptions,
                            rel_name,
                            args,
                        },
                        i + 1,
                    )));
                } else {
                    panic!("unreacheable state");
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
