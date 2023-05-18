use crate::{syntax::Statement, parser::{error::FailureExplanation, statement_reader::read_statement}, lexer};
use crate::ParserError;
use crate::lexer::LexogramType::*;

pub fn read_logical_statement_concatenation(
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

    let mut ret = Statement::Empty;

    #[derive(Clone, Copy)]
    enum AppendModes {
        None,
        And,
        Or,
    }

    let mut append_mode = AppendModes::None;

    let mut negate_next_statement = false;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }

        match (lex.l_type.clone(), state) {
            (OpAnd, SpectingOperatorOrEnd) => {
                append_mode = AppendModes::And;
                state = SpectingStatementOrOpenParenthesis;
            }

            (OpOr, SpectingOperatorOrEnd) => {
                append_mode = AppendModes::Or;
                state = SpectingStatementOrOpenParenthesis;
            }

            (OpNot, SpectingStatementOrNegationOrOpenParenthesis) => {
                negate_next_statement = true;
                state = SpectingStatementOrOpenParenthesis
            }

            (
                LeftParenthesis,
                SpectingStatementOrOpenParenthesis | SpectingStatementOrNegationOrOpenParenthesis,
            ) => {
                match read_logical_statement_concatenation(
                    lexograms,
                    i + 1,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Ok((e, jump_to)) => {
                        cursor = jump_to;
                        ret = match (append_mode, negate_next_statement) {
                            (AppendModes::And, false) => Statement::And(Box::new(ret), Box::new(e)),
                            (AppendModes::Or, false) => Statement::Or(Box::new(ret), Box::new(e)),
                            (AppendModes::And, true) => {
                                Statement::And(Box::new(ret), Box::new(Statement::Not(Box::new(e))))
                            }
                            (AppendModes::Or, true) => {
                                Statement::Or(Box::new(ret), Box::new(Statement::Not(Box::new(e))))
                            }
                            (AppendModes::None, _) => e,
                        };
                        negate_next_statement = false;
                    }
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "statement concatenation".into(),
                            failed_because: "specting nested statement concatenation".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                }

                state = SpectingClosingParenthesis
            }

            (RightParenthesis, SpectingClosingParenthesis) => state = SpectingOperatorOrEnd,

            (
                _,
                SpectingStatementOrOpenParenthesis | SpectingStatementOrNegationOrOpenParenthesis,
            ) => {
                match read_statement(lexograms, i, debug_margin.clone() + "   ", debug_print)? {
                    Ok((e, jump_to)) => {
                        cursor = jump_to;
                        ret = match (append_mode, negate_next_statement) {
                            (AppendModes::And, false) => Statement::And(Box::new(ret), Box::new(e)),
                            (AppendModes::Or, false) => Statement::Or(Box::new(ret), Box::new(e)),
                            (AppendModes::And, true) => {
                                Statement::And(Box::new(ret), Box::new(Statement::Not(Box::new(e))))
                            }
                            (AppendModes::Or, true) => {
                                Statement::Or(Box::new(ret), Box::new(Statement::Not(Box::new(e))))
                            }
                            (AppendModes::None, _) => e,
                        };
                        negate_next_statement = false;
                    }
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "statement concatenation".into(),
                            failed_because: "specting nested statement concatenation".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                }
                state = SpectingOperatorOrEnd
            }

            (_, SpectingOperatorOrEnd) => return Ok(Ok((ret, i))),
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "expresion".into(),
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: None,
                }))
            }
        }
    }
    match state {
        SpectingOperatorOrEnd => Ok(Ok((ret, lexograms.len()))),
        _ => Ok(Err(FailureExplanation {
            lex_pos: lexograms.len(),
            if_it_was: "expresion".into(),
            failed_because: "file ended".into(),
            parent_failure: None,
        })),
    }
}
