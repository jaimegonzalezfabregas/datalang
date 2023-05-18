use crate::lexer::LexogramType::*;
use crate::{lexer, parser::list_reader::read_list};

use super::common::RelName;
use super::error::ParserError;
use super::FailureExplanation;
use crate::parser::expresion_reader::Expresion;

#[derive(Debug, Clone, PartialEq)]
pub struct DeferedRelation {
    rel_name: RelName,
    args: Vec<Expresion>,
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
        SpectingStatementIdentifierOrAsumption,
        SpectingAsumption,
        SpectingComaBetweenAsumptionsOrEndOfAsumptions,
        SpectingStatementList,
        SpectingQuery,
    }
    use RelationParserStates::*;

    if debug_print {
        println!("{debug_margin}read_querring_relation at {start_cursor}");
    }

    let mut cursor = start_cursor;
    let mut r_name = RelName("default_relation_name".into());
    let mut args = vec![];
    let mut state = SpectingStatementIdentifier;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.to_owned(), state) {
            (Identifier(str), SpectingStatementIdentifier) => {
                r_name = RelName(str);
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
                            if_it_was: "querring relation".into(),
                            failed_because: "specting list".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                    Ok((v, jump_to)) => {
                        cursor = jump_to;
                        args = v;
                        if check_querry {
                            state = SpectingQuery;
                        } else {
                            return Ok(Ok((
                                DeferedRelation {
                                    rel_name: r_name,
                                    args,
                                },
                                i + 1,
                            )));
                        }
                    }
                }
            }
            (Query, SpectingQuery) => {
                return Ok(Ok((
                    DeferedRelation {
                        rel_name: r_name,
                        args,
                    },
                    i + 1,
                )))
            }
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "querring relation".into(),
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: None,
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "querring relation".into(),
        failed_because: "file ended".into(),
        parent_failure: None,
    }))
}
