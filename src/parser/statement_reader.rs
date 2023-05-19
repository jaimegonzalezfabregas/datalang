use crate::engine::operations::*;
use crate::lexer::LexogramType::*;

use crate::parser::defered_relation_reader::read_defered_relation;
use crate::parser::expresion_reader::read_expresion;

use crate::lexer::{self};

use super::defered_relation_reader::DeferedRelation;
use super::error::{FailureExplanation, ParserError};
use super::expresion_reader::Expresion;

#[derive(Clone, Copy)]
enum AppendModes {
    None,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum Statement {
    // resolvable to a bolean
    And(Box<Statement>, Box<Statement>),
    Or(Box<Statement>, Box<Statement>),
    Not(Box<Statement>),
    ExpresionComparison(
        Expresion,
        Expresion,
        fn(Expresion, Expresion) -> Result<bool, String>,
    ),
    Relation(DeferedRelation),
}

pub fn read_statement(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Statement, usize), FailureExplanation>, ParserError> {
    if debug_print {
        println!(
            "{}read_logical_statement_concatenation at {}",
            debug_margin, start_cursor
        );
    }

    #[derive(Debug, Clone, Copy)]
    enum StatementParserStates {
        SpectingStatementOrNegationOrOpenParenthesis,
        SpectingStatementOrOpenParenthesis,
        SpectingOperatorOrEnd,
        SpectingClosingParenthesis,
    }
    use StatementParserStates::*;
    let mut cursor = start_cursor;
    let mut state = SpectingStatementOrNegationOrOpenParenthesis;

    let mut op_ret = None;

    let mut append_mode = AppendModes::None;

    let mut negate_next_statement = false;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }

        match (lex.l_type.clone(), state, op_ret.clone()) {
            (OpAnd, SpectingOperatorOrEnd, _) => {
                append_mode = AppendModes::And;
                state = SpectingStatementOrOpenParenthesis;
            }

            (OpOr, SpectingOperatorOrEnd, _) => {
                append_mode = AppendModes::Or;
                state = SpectingStatementOrOpenParenthesis;
            }

            (OpNot, SpectingStatementOrNegationOrOpenParenthesis, _) => {
                negate_next_statement = true;
                state = SpectingStatementOrOpenParenthesis
            }

            (
                LeftParenthesis,
                SpectingStatementOrOpenParenthesis | SpectingStatementOrNegationOrOpenParenthesis,
                _,
            ) => {
                match read_statement(lexograms, i + 1, debug_margin.clone() + "   ", debug_print)? {
                    Ok((new_statement, jump_to)) => {
                        cursor = jump_to;

                        op_ret = merge_statements(
                            op_ret,
                            new_statement,
                            &append_mode,
                            &negate_next_statement,
                        );
                        negate_next_statement = false;
                    }
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "statement concatenation".into(),
                            failed_because: "specting nested statement concatenation".into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                }

                state = SpectingClosingParenthesis
            }

            (RightParenthesis, SpectingClosingParenthesis, _) => state = SpectingOperatorOrEnd,

            (
                _,
                SpectingStatementOrOpenParenthesis | SpectingStatementOrNegationOrOpenParenthesis,
                _,
            ) => {
                match read_statement_item(lexograms, i, debug_margin.clone() + "   ", debug_print)?
                {
                    Ok((new_statement, jump_to)) => {
                        cursor = jump_to;

                        op_ret = merge_statements(
                            op_ret,
                            new_statement,
                            &append_mode,
                            &negate_next_statement,
                        );

                        negate_next_statement = false;
                    }
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "statement concatenation".into(),
                            failed_because: "specting nested statement concatenation".into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                }
                state = SpectingOperatorOrEnd
            }

            (_, SpectingOperatorOrEnd, Some(ret)) => return Ok(Ok((ret, i))),
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "expresion".into(),
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: vec![],
                }))
            }
        }
    }
    match (state, op_ret) {
        (SpectingOperatorOrEnd, Some(ret)) => Ok(Ok((ret, lexograms.len()))),
        _ => Ok(Err(FailureExplanation {
            lex_pos: lexograms.len(),
            if_it_was: "expresion".into(),
            failed_because: "file ended".into(),
            parent_failure: vec![],
        })),
    }
}

pub fn read_statement_item(
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
                let err1;

                match read_defered_relation(
                    lexograms,
                    i,
                    false,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Ok((def_rel, jump_to)) => {
                        return Ok(Ok((Statement::Relation(def_rel), jump_to)))
                    }

                    Err(err) => err1 = err,
                }
                match read_expresion(
                    lexograms,
                    i,
                    false,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Ok((e, jump_to)) => {
                        first_expresion = e;
                        cursor = jump_to;
                        state = SpectingExrpresionComparisonOperator;
                    }
                    Err(err2) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "statement".into(),
                            failed_because: "was neither a relation nor a expresion comparation"
                                .into(),
                            parent_failure: (vec![err1, err2]),
                        }))
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
                                OpEq => Statement::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    eq_expresions,
                                ),
                                OpLT => Statement::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    lt_expresions,
                                ),
                                OpLTE => Statement::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    lte_expresions,
                                ),
                                OpGT => Statement::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    gt_expresions,
                                ),
                                OpGTE => Statement::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    gte_expresions,
                                ),
                                _ => {
                                    return Ok(Err(FailureExplanation {
                                        lex_pos: i,
                                        if_it_was: "statement".into(),
                                        failed_because: "corrupted operator".into(),
                                        parent_failure: vec![],
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
                            parent_failure: (vec![e]),
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
                    parent_failure: vec![],
                }))
            }
        }
    }

    todo!();
}

fn merge_statements(
    op_ret: Option<Statement>,
    new_statement: Statement,
    append_mode: &AppendModes,
    negate_next_statement: &bool,
) -> Option<Statement> {
    Some(match (op_ret, append_mode, negate_next_statement) {
        (None, _, _) => new_statement,
        (Some(prev_statement), AppendModes::And, false) => {
            Statement::And(Box::new(prev_statement), Box::new(new_statement))
        }
        (Some(prev_statement), AppendModes::Or, false) => {
            Statement::Or(Box::new(prev_statement), Box::new(new_statement))
        }
        (Some(prev_statement), AppendModes::And, true) => Statement::And(
            Box::new(prev_statement),
            Box::new(Statement::Not(Box::new(new_statement))),
        ),
        (Some(prev_statement), AppendModes::Or, true) => Statement::Or(
            Box::new(prev_statement),
            Box::new(Statement::Not(Box::new(new_statement))),
        ),
        (Some(_), AppendModes::None, _) => panic!("unreacheable state"),
    })
}
