use std::fmt::{self};

use crate::engine::var_context::VarContext;
use crate::lexer;
use crate::lexer::LexogramType::*;

use super::data_token::{read_data, Data};
use super::error::{FailureExplanation, ParserError};
use crate::engine::operations::*;
use crate::parser::destructuring_array_token::read_destructuring_array;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VarName {
    DestructuredArray(Vec<Expresion>),
    Direct(String),
    ExplodeArray(String),
}

impl fmt::Display for VarName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarName::DestructuredArray(arr) => {
                let mut ret = String::new();

                ret += &"[";
                for (i, d) in arr.iter().enumerate() {
                    ret += &format!("{d}");
                    if i != arr.len() - 1 {
                        ret += &",";
                    }
                }
                ret += &"]";
                write!(f, "{ret}")
            }
            VarName::Direct(name) => write!(f, "{name}"),
            VarName::ExplodeArray(name) => write!(f, "...{name}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Operation<Op, Res> {
    pub forward: fn(Op, Op) -> Result<Res, String>,
    pub reverse_op1: fn(Op, Res) -> Result<Res, String>,
    pub reverse_op2: fn(Op, Res) -> Result<Res, String>,
    pub to_string: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Expresion {
    // resolvable to a value
    Arithmetic(Box<Expresion>, Box<Expresion>, Operation<Data, Data>),
    Literal(Data),
    Var(VarName),
}

impl fmt::Display for Expresion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expresion::Arithmetic(expa, expb, op) => write!(f, "{expa}{}{expb}", op.to_string),
            Expresion::Literal(l) => write!(f, "{l}"),
            Expresion::Var(v) => write!(f, "{v}"),
        }
    }
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
                None => Err(format!(
                    "literalize error: var {str} not defined on context {context}"
                )),
            },
            Expresion::Var(VarName::DestructuredArray(exp_vec)) => {
                let mut datas = vec![];
                for e in exp_vec.iter() {
                    match e {
                        Expresion::Var(VarName::ExplodeArray(var_name)) => {
                            let var_value = match context.get(var_name){
                                Some(ret) => ret,
                                None =>return Err(format!(
                                    "no se ha podido literalizar: {self} en el contexto {context} por ...{var_name}"
                                )),
                            };
                            match var_value {
                                Data::Array(arr) => datas.extend(arr),
                                _ =>     return Err(format!("no se ha podido literalizar: {self} en el contexto {context} por ...{var_name}"))
                            }
                        }
                        _ => datas.push(e.literalize(context)?),
                    }
                }

                Ok(Data::Array(datas))
            }
            _ => Err(format!(
                "no se ha podido literalizar: {self} en el contexto {context}"
            )),
        };

        ret
    }

    pub fn solve(
        self: &Expresion,
        goal: &Data,
        caller_context: &VarContext,
        debug_margin: String,
        debug_print: bool,
    ) -> Result<VarContext, String> {
        // return Ok significa que goal y self han podido ser evaluadas a lo mismo

        // if debug_print {
        //     println!("{debug_margin}call to solve with goal:[{goal}], expresion [{self}] and context {caller_context}");
        // }
        let ret = match self.literalize(&caller_context) {
            Ok(Data::Any) => match &self {
                Expresion::Var(VarName::Direct(name)) => {
                    let mut new_context = caller_context.to_owned();
                    new_context.set(name.to_owned(), goal.to_owned());
                    new_context
                }
                _ => caller_context.to_owned(),
            },

            Err(_) => match self {
                Expresion::Arithmetic(a, b, func) => {
                    let literalize_a = a.literalize(&caller_context);
                    let literalize_b = b.literalize(&caller_context);

                    match (literalize_a, literalize_b) {
                        (Err(_)|Ok(Data::Any),Err(_)|Ok(Data::Any) ) => {
                            return Err("parece que esta expresión contiene varias incognitas".into())
                        }
                        (Ok(op_1), Err(_)|Ok(Data::Any)) => {
                            let new_goal = (func.reverse_op2)(op_1, goal.to_owned())?;
                            b.solve(&new_goal, caller_context,debug_margin.to_owned() + "|  ",
                                    debug_print)?
                        }
                        (Err(_)|Ok(Data::Any), Ok(op_2)) => {
                            let new_goal = (func.reverse_op1)(op_2, goal.to_owned())?;
                            a.solve(&new_goal, caller_context,debug_margin.to_owned() + "|  ",
                                    debug_print)?
                        }
                        (Ok(_), Ok(_)) => {
                            return Err("parece que ambas ramas del arbol son literalizables, por lo que no hay nada que deducir".into())
                        }
                    }
                }
                Expresion::Literal(_) => unreachable!(),
                Expresion::Var(VarName::Direct(name)) => {
                    let mut new_context = caller_context.to_owned();
                    new_context.set(name.to_owned(), goal.to_owned());
                    new_context
                }
                Expresion::Var(VarName::DestructuredArray(template_arr)) => {
                    if let Data::Array(goal_arr) = goal {
                        if goal_arr.len() < template_arr.len() {
                            return Err("unmatchable arrays".into());
                        }

                        let mut new_context = caller_context.to_owned();
                        let mut last_i = 0;

                        for (i, array_position) in template_arr.iter().enumerate() {
                            last_i = i;
                            if let Expresion::Var(VarName::ExplodeArray(x)) = array_position {
                                match Expresion::Var(VarName::Direct(x.to_owned())).solve(
                                    &Data::Array(goal_arr[i..].to_vec()),
                                    &new_context,
                                    debug_margin.to_owned() + "|  ",
                                    debug_print,
                                ) {
                                    Ok(newer_context) => new_context = newer_context,
                                    Err(msg) => {
                                        return Err(format!("at array position {i} error: {msg}"))
                                    }
                                }
                                last_i = goal_arr.len() - 1;
                            } else {
                                match array_position.solve(
                                    &goal_arr[i],
                                    &new_context,
                                    debug_margin.to_owned() + "|  ",
                                    debug_print,
                                ) {
                                    Ok(newer_context) => new_context = newer_context,
                                    Err(msg) => {
                                        return Err(format!("at array position {i} error: {msg}"))
                                    }
                                }
                            }
                        }
                        if last_i == goal_arr.len() - 1 {
                            new_context
                        } else {
                            return Err("cant destructure an array with unmatching size".into());
                        }
                    } else {
                        return Err("cant destructure a non array goal to an array".into());
                    }
                }
                Expresion::Var(VarName::ExplodeArray(_)) => unreachable!(),
            },
            Ok(d) => {
                if d == goal.to_owned() {
                    caller_context.to_owned()
                } else if let Data::Any = goal {
                    caller_context.to_owned()
                } else {
                    return Err(format!("La literalizacion ({d}) y el goal ({goal}) no coinciden en el contexto: {caller_context}"));
                }
            }
        };

        // if debug_print {
        //     println!("{debug_margin}*solving de \"{self}\" con goal {goal} ha resultado en {ret}")
        // }

        Ok(ret)
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
                    to_string: "+".into(),
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (OpSub, SpectingOperatorOrEnd, _) => {
                append_mode = Some(Operation {
                    forward: substract_direct,
                    reverse_op1: substract_reverse_op1,
                    reverse_op2: substract_reverse_op2,
                    to_string: "-".into(),
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (OpMul, SpectingOperatorOrEnd, _) => {
                append_mode = Some(Operation {
                    forward: multiply_direct,
                    reverse_op1: multiply_reverse_op1,
                    reverse_op2: multiply_reverse_op2,
                    to_string: "*".into(),
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (OpDiv, SpectingOperatorOrEnd, _) => {
                append_mode = Some(Operation {
                    forward: divide_direct,
                    reverse_op1: divide_reverse_op1,
                    reverse_op2: divide_reverse_op2,
                    to_string: "/".into(),
                });
                state = SpectingItemOrOpenParenthesis;
            }
            (LeftParenthesis, SpectingItemOrOpenParenthesis, _) => {
                match read_expresion(
                    lexograms,
                    i,
                    only_literals,
                    debug_margin.to_owned() + "|  ",
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
                    debug_margin.to_owned() + "|  ",
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
            lex_pos: lexograms.len() - 1,
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
                debug_margin.to_owned() + "|  ",
                debug_print,
            )? {
                Ok((ret, jump_to)) => Ok(Ok((Expresion::Literal(ret), jump_to))),
                Err(a) => match read_destructuring_array(
                    lexograms,
                    start_cursor,
                    debug_margin.to_owned() + "|  ",
                    debug_print,
                )? {
                    Ok((ret, jump_to)) => Ok(Ok((Expresion::Var(ret), jump_to))),

                    Err(b) => Ok(Err(FailureExplanation {
                        lex_pos: start_cursor,
                        if_it_was: "expresion_item".into(),
                        failed_because: "specting some array".into(),
                        parent_failure: vec![a, b],
                    })),
                },
            }
        }

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
