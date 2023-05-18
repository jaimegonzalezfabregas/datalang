use crate::engine::operations::*;
use crate::lexer::LexogramType::*;

use crate::parser::defered_relation_reader::read_defered_relation;
use crate::parser::expresion_reader::read_expresion;
use crate::syntax::{Expresion, Statement};

use crate::lexer::{self};

use super::error::{FailureExplanation, ParserError};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // resolvable to a bolean
    Hypothetical(Vec<Line>, Box<Statement>), // TODO
    And(Box<Statement>, Box<Statement>),
    Or(Box<Statement>, Box<Statement>),
    Not(Box<Statement>),
    Arithmetic(
        Expresion,
        Expresion,
        fn(Expresion, Expresion) -> Result<bool, String>,
    ),
    Relation(DeferedRelation),
    Empty,
}

pub fn read_statement(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Statement, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum StatementParserStates {
        SpectingFirstExpresionOrRelation,
        SpectingExrpresionComparisonOperator,
        SpectingSecondExpresion,
    }
    use StatementParserStates::*;

    if debug_print {
        println!("{}read_statement at {}", debug_margin, start_cursor);
    }

    let mut cursor = start_cursor;
    let mut state = SpectingFirstExpresionOrRelation;

    let mut first_expresion = Expresion::Empty;
    let mut append_mode = OpEq;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }

        match (lex.l_type.clone(), state) {
            (_, SpectingFirstExpresionOrRelation) => {
                match read_defered_relation(
                    lexograms,
                    i,
                    false,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Ok((rel_name, args, jump_to)) => {
                        return Ok(Ok((Statement::Relation(rel_name, args), jump_to)))
                    }

                    Err(e1) => {
                        match read_expresion(
                            lexograms,
                            i,
                            false,
                            debug_margin.clone() + "   ",
                            debug_print,
                        )? {
                            Ok((e, jump_to)) => {
                                first_expresion = e;
                                println!("{debug_margin} expresion ended at {jump_to} ");
                                cursor = jump_to;
                                state = SpectingExrpresionComparisonOperator;
                            }
                            Err(e2) => {
                                return Ok(Err(FailureExplanation {
                                    lex_pos: i,
                                    if_it_was: "statement".into(),
                                    failed_because:
                                        "was neither a relation nor a expresion comparation".into(),
                                    parent_failure: Some(vec![e1, e2]),
                                }))
                            }
                        }
                    }
                }
            }
            (op @ (OpEq | OpGT | OpLT | OpGTE | OpLTE), SpectingExrpresionComparisonOperator) => {
                append_mode = op;
                state = SpectingSecondExpresion;
            }
            (_, SpectingSecondExpresion) => {
                match read_expresion(
                    lexograms,
                    i,
                    false,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Ok((second_expresion, jump_to)) => {
                        return Ok(Ok((
                            match append_mode {
                                OpEq => Statement::Arithmetic(
                                    first_expresion,
                                    second_expresion,
                                    eq_expresions,
                                ),
                                OpLT => Statement::Arithmetic(
                                    first_expresion,
                                    second_expresion,
                                    lt_expresions,
                                ),
                                OpLTE => Statement::Arithmetic(
                                    first_expresion,
                                    second_expresion,
                                    lte_expresions,
                                ),
                                OpGT => Statement::Arithmetic(
                                    first_expresion,
                                    second_expresion,
                                    gt_expresions,
                                ),
                                OpGTE => Statement::Arithmetic(
                                    first_expresion,
                                    second_expresion,
                                    gte_expresions,
                                ),
                                _ => {
                                    return Ok(Err(FailureExplanation {
                                        lex_pos: i,
                                        if_it_was: "statement".into(),
                                        failed_because: "corrupted operator".into(),
                                        parent_failure: None,
                                    }))
                                }
                            },
                            jump_to,
                        )))
                    }
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "statement".into(),
                            failed_because: "specting second statement after operator".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                }
            }

            (lex, state) => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "statement".into(),
                    failed_because: format!(
                        "pattern missmatch on {:#?} state reading lex {:#?}",
                        state, lex
                    )
                    .into(),
                    parent_failure: None,
                }))
            }
        }
    }

    todo!();
}
