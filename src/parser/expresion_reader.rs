use crate::engine::var_context::VarContext;
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
    pub fn literalize(self: &Expresion, context: &VarContext) -> Result<Data, String> {
        let ret = match self.to_owned() {
            Expresion::Arithmetic(a, b, f) => {
                Ok((f.forward)(a.literalize(context)?, b.literalize(context)?)?)
            }
            Expresion::Literal(e) => Ok(e),
            Expresion::Var(VarName::Direct(str)) => match context.get(&str) {
                Some(value) => Ok(value.to_owned()),
                None => Err(format!("var {str} not defined on context")),
            },
            _ => Err(format!("no se ha podido literalizar: {self:?}")),
        };

        ret
    }

    pub fn solve(
        self: &Expresion,
        goal: &Data,
        caller_context: &VarContext,
    ) -> Result<VarContext, String> {
        // return Ok significa que goal y self han podido ser evaluadas a lo mismo

        // println!("\n------ call to solve with goal:{goal:?} at {self:?} at {caller_context:?}");

        match self.literalize(&caller_context) {
            Ok(d) => {
                if d == goal.to_owned() {
                    Ok(caller_context.to_owned())
                } else {
                    Err("La literalizacion y el goal no coinciden".into())
                }
            }
            Err(_) => match self {
                Expresion::Arithmetic(a, b, func) => {
                    let literalize_a = a.literalize(&caller_context);
                    let literalize_b = b.literalize(&caller_context);

                    match (literalize_a, literalize_b) {
                        (Ok(_), Ok(_)) => {
                            Err("parece que se intentan operar dos datos incompatibles".into())
                        }
                        (Ok(op_1), Err(_)) => {
                            let new_goal = (func.reverse_op2)(op_1, goal.to_owned())?;
                            b.solve(&new_goal, caller_context)
                        }
                        (Err(_), Ok(op_2)) => {
                            let new_goal = (func.reverse_op1)(op_2, goal.to_owned())?;
                            a.solve(&new_goal, caller_context)
                        }
                        (Err(_), Err(_)) => {
                            Err("parece que esta expresión contiene varias incognitas".into())
                        }
                    }
                }
                Expresion::Literal(_) => unreachable!(),
                Expresion::Var(VarName::Direct(name)) => {
                    let mut new_context = caller_context.to_owned();
                    new_context.set(name.to_owned(), goal.to_owned());
                    return Ok(new_context);
                }
                Expresion::Var(VarName::Anonimus) => {
                    return Ok(caller_context.to_owned());
                }
                Expresion::Var(_) => {
                    return Err("using vartype not suported for solving".into());
                }
            },
        }
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

        match (lex.l_type.to_owned(), state, only_literals.to_owned()) {
            (OpAdd, SpectingOperatorOrEnd, _) => {
                append_mode = Some(Operation {
                    forward: add_direct,
                    reverse_op1: add_reverse_op1,
                    reverse_op2: add_reverse_op2,
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (OpSub, SpectingOperatorOrEnd, _) => {
                append_mode = Some(Operation {
                    forward: substract_direct,
                    reverse_op1: substract_reverse_op1,
                    reverse_op2: substract_reverse_op2,
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (OpMul, SpectingOperatorOrEnd, _) => {
                append_mode = Some(Operation {
                    forward: multiply_direct,
                    reverse_op1: multiply_reverse_op1,
                    reverse_op2: multiply_reverse_op2,
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (OpDiv, SpectingOperatorOrEnd, _) => {
                append_mode = Some(Operation {
                    forward: divide_direct,
                    reverse_op1: divide_reverse_op1,
                    reverse_op2: divide_reverse_op2,
                });
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
                        op_ret = Some(match (&append_mode, op_ret) {
                            (Some(op), Some(ret)) => {
                                Expresion::Arithmetic(Box::new(ret), Box::new(e), op.clone())
                            }
                            _ => e,
                        });
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
                        op_ret = Some(match (&append_mode, op_ret) {
                            (Some(op), Some(ret)) => {
                                Expresion::Arithmetic(Box::new(ret), Box::new(e), op.clone())
                            }
                            _ => e,
                        });
                        append_mode = None
                    }
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "expresion".into(),
                            failed_because: format!("Specting expresion item").into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                }
                state = SpectingOperatorOrEnd
            }

            (_, SpectingOperatorOrEnd, _) => {
                return Ok(Ok((op_ret.unwrap_or_else(|| unreachable!()), i)))
            }
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
