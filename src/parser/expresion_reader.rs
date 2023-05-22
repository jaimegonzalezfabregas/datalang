use std::collections::HashMap;

use crate::lexer;
use crate::lexer::LexogramType::*;

use super::data_reader::{read_data, Data};
use super::error::{FailureExplanation, ParserError};
use crate::engine::operations::*;
use crate::parser::destructuring_array_reader::read_destructuring_array;

#[derive(Debug, Clone)]
pub enum VarName {
    DestructuredArray(Vec<Expresion>),
    Direct(String),
    RestOfArray(String),
    Anonimus,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Operation<Op, Res> {
    pub forward: fn(Op, Op) -> Result<Res, String>,
    pub reverse_op1: fn(Op, Res) -> Result<Res, String>,
    pub reverse_op2: fn(Op, Res) -> Result<Res, String>,
}

#[derive(Debug, Clone)]
pub enum Expresion {
    // resolvable to a value
    Arithmetic(Box<Expresion>, Box<Expresion>, Operation<Data, Data>),
    Literal(Data),
    Var(VarName),
}

impl Expresion {
    pub fn literalize(
        self: &Expresion,
        context: Option<&HashMap<String, Data>>,
    ) -> Result<Data, String> {
        let ret = match (self.to_owned(), context) {
            (Expresion::Arithmetic(a, b, f), _) => {
                Ok((f.forward)(a.literalize(context)?, b.literalize(context)?)?)
            }
            (Expresion::Literal(e), _) => Ok(e),
            (Expresion::Var(VarName::Direct(str)), Some(var_values)) => {
                match var_values.get(&str) {
                    Some(value) => Ok(value.to_owned()),
                    None => Err(format!("var {str} not defined on context")),
                }
            }
            _ => Err(format!("no se ha podido literalizar: {self:?}")),
        };

        ret
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

    let mut op_ret = None;
    let mut append_mode: Option<Operation<Data, Data>> = None;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }

        match (
            lex.l_type.to_owned(),
            state,
            only_literals.to_owned(),
            op_ret.to_owned(),
        ) {
            (OpAdd, SpectingOperatorOrEnd, _, _) => {
                append_mode = Some(Operation {
                    forward: add_direct,
                    reverse_op1: add_reverse_op1,
                    reverse_op2: add_reverse_op2,
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (OpSub, SpectingOperatorOrEnd, _, _) => {
                append_mode = Some(Operation {
                    forward: substract_direct,
                    reverse_op1: substract_reverse_op1,
                    reverse_op2: substract_reverse_op2,
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (OpMul, SpectingOperatorOrEnd, _, _) => {
                append_mode = Some(Operation {
                    forward: multiply_direct,
                    reverse_op1: multiply_reverse_op1,
                    reverse_op2: multiply_reverse_op2,
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (OpDiv, SpectingOperatorOrEnd, _, _) => {
                append_mode = Some(Operation {
                    forward: divide_direct,
                    reverse_op1: divide_reverse_op1,
                    reverse_op2: divide_reverse_op2,
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (LeftParenthesis, SpectingItemOrOpenParenthesis, _, Some(ret)) => {
                match read_expresion(
                    lexograms,
                    i,
                    only_literals,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Ok((e, jump_to)) => {
                        cursor = jump_to;
                        op_ret = match &append_mode {
                            Some(op) => Some(Expresion::Arithmetic(
                                Box::new(ret),
                                Box::new(e),
                                op.to_owned(),
                            )),
                            None => Some(e),
                        };
                        append_mode = None
                    }
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "expresion".into(),
                            failed_because: "specting nested expresion".into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                }

                state = SpectingClosingParenthesis
            }

            (RightParenthesis, SpectingClosingParenthesis, _, _) => state = SpectingOperatorOrEnd,

            (_, SpectingItemOrOpenParenthesis, _, Some(ret)) => {
                match read_expresion_item(
                    lexograms,
                    i,
                    only_literals,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Ok((e, jump_to)) => {
                        cursor = jump_to;
                        op_ret = match &append_mode {
                            Some(op) => Some(Expresion::Arithmetic(
                                Box::new(ret),
                                Box::new(e),
                                op.clone(),
                            )),
                            None => Some(e),
                        };
                        append_mode = None
                    }
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "expresion".into(),
                            failed_because: format!("pattern missmatch on {:#?} state", state)
                                .into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                }
                state = SpectingOperatorOrEnd
            }

            (_, SpectingOperatorOrEnd, _, Some(ret)) => return Ok(Ok((ret, i))),
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
                Ok((ret, jump_to)) => Ok(Ok((Expresion::Literal(ret), jump_to))),
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
                        parent_failure: vec![a, b],
                    })),
                },
            }
        }
        (Any, false) => Ok(Ok((Expresion::Var(VarName::Anonimus), start_cursor + 1))),

        (_, _) => match read_data(lexograms, start_cursor, debug_margin, debug_print)? {
            Ok((value, jump_to)) => Ok(Ok((Expresion::Literal(value), jump_to))),
            Err(err) => Ok(Err(FailureExplanation {
                lex_pos: start_cursor,
                if_it_was: "expresion_item".into(),
                failed_because: "specting some array".into(),
                parent_failure: vec![err],
            })),
        },
    }
}
