use crate::lexer::LexogramType::*;
use crate::{lexer, syntax::Expresion};

use super::error::{FailureExplanation, ParserError};
use super::var_literal_reader::VarLiteral;
use crate::engine::operations::*;
use crate::parser::common::read_expresion_item;

#[derive(Debug, Clone, PartialEq)]
pub enum VarName {
    DestructuredArray(Vec<Expresion>),
    Direct(String),
}


#[derive(Debug, Clone, PartialEq)]
pub enum Expresion {
    // resolvable to a value
    Arithmetic(
        Box<Expresion>,
        Box<Expresion>,
        fn(Expresion, Expresion) -> Result<Expresion, String>,
    ),
    Literal(VarLiteral),
    RestOfList(VarName),
    Var(VarName),
    Empty,
}

impl Expresion {
    pub fn literalize(self: &Expresion) -> Result<VarLiteral, String> {
        let ret = match self.clone() {
            Expresion::Arithmetic(a, b, f) => {
                if let Expresion::Literal(VarLiteral::FullSet) = *a {
                    Ok(VarLiteral::FullSet)
                } else if let Expresion::Literal(VarLiteral::FullSet) = *a {
                    Ok(VarLiteral::FullSet)
                } else {
                    f(*a, *b)?.literalize()
                }
            }
            Expresion::Literal(e) => Ok(e),
            _ => Err(format!("no se ha podido literalizar: {:#?}", self)),
        };

        return ret;
    }

    pub fn singleton(value: &Data) -> Expresion {
        return Expresion::Literal(VarLiteral::singleton(value));
    }
}

pub fn read_expresion(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    only_literals: bool,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Expresion, usize), FailureExplanation>, ParserError> {
    if debug_print {
        println!("{}read_expresion at {}", debug_margin, start_cursor);
    }

    #[derive(Debug, Clone, Copy)]
    enum ExpressionParserStates {
        SpectingItemOrOpenParenthesis,
        SpectingOperatorOrEnd,
        SpectingClosingParenthesis,
    }
    use ExpressionParserStates::*;
    let mut cursor = start_cursor;
    let mut state = SpectingItemOrOpenParenthesis;

    let mut ret = Expresion::Empty;
    let mut append_mode: Option<fn(Expresion, Expresion) -> Result<Expresion, String>> = None;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }

        match (lex.l_type.clone(), state, only_literals) {
            (OpAdd, SpectingOperatorOrEnd, _) => {
                append_mode = Some(add_expresions);
                state = SpectingItemOrOpenParenthesis;
            }
            (OpSub, SpectingOperatorOrEnd, _) => {
                append_mode = Some(sub_expresions);
                state = SpectingItemOrOpenParenthesis;
            }
            (OpMul, SpectingOperatorOrEnd, _) => {
                append_mode = Some(mul_expresions);
                state = SpectingItemOrOpenParenthesis;
            }
            (OpDiv, SpectingOperatorOrEnd, _) => {
                append_mode = Some(div_expresions);
                state = SpectingItemOrOpenParenthesis;
            }
            (LeftParenthesis, SpectingItemOrOpenParenthesis, _) => {
                match read_expresion(
                    lexograms,
                    i,
                    only_literals,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Ok((e, jump_to)) => {
                        cursor = jump_to;
                        ret = match append_mode {
                            Some(append_mode_fn) => {
                                Expresion::Arithmetic(Box::new(ret), Box::new(e), append_mode_fn)
                            }
                            None => e,
                        }
                    }
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "expresion".into(),
                            failed_because: "specting nested expresion".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                }

                state = SpectingClosingParenthesis
            }

            (RightParenthesis, SpectingClosingParenthesis, _) => state = SpectingOperatorOrEnd,

            (_, SpectingItemOrOpenParenthesis, _) => {
                match read_expresion_item(
                    lexograms,
                    i,
                    only_literals,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Ok((e, jump_to)) => {
                        cursor = jump_to;
                        ret = match append_mode {
                            Some(append_mode_fn) => {
                                Expresion::Arithmetic(Box::new(ret), Box::new(e), append_mode_fn)
                            }
                            None => e,
                        }
                    }
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "expresion".into(),
                            failed_because: format!("pattern missmatch on {:#?} state", state)
                                .into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                }
                state = SpectingOperatorOrEnd
            }

            (_, SpectingOperatorOrEnd, _) => return Ok(Ok((ret, i))),
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


pub fn read_expresion_item(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    only_literals: bool,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Expresion, usize), FailureExplanation>, ParserError> {
    if debug_print {
        println!("{}read_item at {}", debug_margin, start_cursor);
    }

    match (lexograms[start_cursor].l_type.clone(), only_literals) {
        (Any, _) => Ok(Ok((
            Expresion::Literal(VarLiteral::FullSet),
            start_cursor + 1,
        ))),

        (OpLT | OpNot, _) => match read_var_literal(
            lexograms,
            start_cursor,
            debug_margin.clone() + "   ",
            debug_print,
        )? {
            Ok(ret) => Ok(Ok(ret)),
            Err(explanation) => Ok(Err(FailureExplanation {
                lex_pos: start_cursor,
                if_it_was: "expresion_item".into(),
                failed_because: "was not an array".into(),
                parent_failure: Some(vec![explanation]),
            })),
        },
        (Identifier(str), false) => {
            Ok(Ok((Expresion::Var(VarName::Direct(str)), start_cursor + 1)))
        }
        (LeftBracket, false) => {
            match read_data(
                lexograms,
                start_cursor,
                debug_margin.clone() + "   ",
                debug_print,
            )? {
                Ok((ret, jump_to)) => Ok(Ok((Expresion::singleton(&ret), jump_to))),
                Err(a) => match read_destructuring_array(
                    lexograms,
                    start_cursor,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Ok(ret) => Ok(Ok(ret)),

                    Err(b) => Ok(Err(FailureExplanation {
                        lex_pos: start_cursor,
                        if_it_was: "expresion_item".into(),
                        failed_because: "specting some array".into(),
                        parent_failure: Some(vec![a, b]),
                    })),
                },
            }
        }

        (_, _) => match read_data(
            lexograms,
            start_cursor,
            debug_margin.clone() + "   ",
            debug_print,
        )? {
            Ok((ret, jump_to)) => Ok(Ok((Expresion::singleton(&ret), jump_to))),
            Err(e) => Ok(Err(FailureExplanation {
                lex_pos: start_cursor,
                if_it_was: "expresion_item".into(),
                failed_because: "specting data".into(),
                parent_failure: Some(vec![e]),
            })),
        },
    }
}