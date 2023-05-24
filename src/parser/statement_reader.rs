use std::collections::{HashMap, HashSet};
use std::vec;

use crate::engine::var_context::VarContext;
use crate::engine::{Engine, RelId};
use crate::lexer::LexogramType::*;

use crate::parser::defered_relation_reader::read_defered_relation;
use crate::parser::expresion_reader::read_expresion;

use crate::lexer::{self};

use super::defered_relation_reader::DeferedRelation;
use super::error::{FailureExplanation, ParserError};
use super::expresion_reader::Expresion;
use super::Relation;

#[derive(Clone, Copy)]
enum AppendModes {
    None,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum Comparison {
    Eq,
    Lt,
    Gt,
    Gte,
    Lte,
}

#[derive(Debug, Clone)]
pub enum Statement {
    // resolvable to a bolean
    And(Box<Statement>, Box<Statement>),
    Or(Box<Statement>, Box<Statement>),
    Not(Box<Statement>),
    ExpresionComparison(Expresion, Expresion, Comparison),
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

        match (lex.l_type.to_owned(), state, op_ret.clone()) {
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
                    if_it_was: "statement".into(),
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
            if_it_was: "statement".into(),
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

    let mut op_first_expresion = None;
    let mut op_append_mode = None;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }

        match (
            lex.l_type.to_owned(),
            state,
            op_first_expresion.to_owned(),
            op_append_mode.to_owned(),
        ) {
            (_, SpectingFirstExpresionOrRelation, _, _) => {
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
                        op_first_expresion = Some(e);
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
            (
                op @ (OpEq | OpGT | OpLT | OpGTE | OpLTE),
                SpectingExrpresionComparisonOperator,
                _,
                _,
            ) => {
                op_append_mode = Some(op);
                state = SpectingSecondExpresion;
            }
            (_, SpectingSecondExpresion, Some(first_expresion), Some(append_mode)) => {
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
                                    Comparison::Eq,
                                ),
                                OpLT => Statement::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    Comparison::Lt,
                                ),
                                OpLTE => Statement::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    Comparison::Lte,
                                ),
                                OpGT => Statement::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    Comparison::Gt,
                                ),
                                OpGTE => Statement::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    Comparison::Gte,
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

            (lex, state, a, b) => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "statement".into(),
                    failed_because: format!(
                        "pattern missmatch on {:#?} state ({a:?} {b:?}) reading lex {:#?}",
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
        (Some(_), AppendModes::None, _) => unreachable!(),
    })
}

impl Statement {
    pub fn get_context_universe(
        &self,
        engine: &Engine,
        caller_depth_map: &HashMap<RelId, usize>,
    ) -> HashSet<VarContext> {
        match self {
            Statement::Or(statement_a, statement_b) | Statement::And(statement_a, statement_b) => {
                let deep_universe_a = statement_a.get_context_universe(engine, caller_depth_map);
                let deep_universe_b = statement_b.get_context_universe(engine, caller_depth_map);

                let mut full_deep_universe = HashSet::new();
                for a_context in &deep_universe_a {
                    for b_context in &deep_universe_b {
                        full_deep_universe.insert(a_context.extend(b_context));
                    }
                }
                full_deep_universe
            }
            Statement::Relation(rel) => match engine.get_table(rel.get_rel_id()) {
                Some(table) => match table.get_all_contents(Some(caller_depth_map), engine) {
                    Ok(table_truths) => {
                        let mut ret = HashSet::new();
                        for truth in table_truths {
                            for (col_data, col_exp) in truth.get_data().iter().zip(&rel.args) {
                                match col_exp.solve(col_data, &VarContext::new()) {
                                    Ok(new_context) => {
                                        ret.insert(new_context);
                                    }
                                    Err(_) => (),
                                }
                            }
                        }
                        ret
                    }
                    Err(_) => HashSet::new(),
                },
                None => HashSet::new(),
            },
            _ => HashSet::new(),
        }
    }

    pub fn get_posible_contexts(
        &self,
        engine: &Engine,
        caller_depth_map: &HashMap<RelId, usize>,
        universe: &HashSet<VarContext>,
    ) -> HashSet<VarContext> {
        let ret = match self {
            Statement::And(statement_a, statement_b) => {
                let contexts_a =
                    statement_a.get_posible_contexts(engine, caller_depth_map, universe);
                let contexts_b =
                    statement_b.get_posible_contexts(engine, caller_depth_map, universe);

                // println!("\nAND: \n{contexts_a:?}\n{contexts_b:?}");

                contexts_a
                    .intersection(&contexts_b)
                    .map(|e| e.to_owned())
                    .collect::<HashSet<VarContext>>()
            }
            Statement::Or(statement_a, statement_b) => {
                let mut contexts_a =
                    statement_a.get_posible_contexts(engine, caller_depth_map, universe);
                let contexts_b =
                    statement_b.get_posible_contexts(engine, caller_depth_map, universe);

                // println!("\nOR: \n{contexts_a:?}\n{contexts_b:?}");

                contexts_a.extend(contexts_b);
                contexts_a
            }
            Statement::Not(statement) => {
                let contexts = statement.get_posible_contexts(engine, caller_depth_map, universe);

                // println!("\nNOT: \n{contexts:?}");

                universe
                    .difference(&contexts)
                    .map(|e| e.to_owned())
                    .collect()
            }
            Statement::ExpresionComparison(exp_a, exp_b, Comparison::Eq) => universe
                .iter()
                .flat_map(|context| {
                    let a = exp_a.literalize(context);
                    let b = exp_b.literalize(context);
                    match (exp_a, exp_b, a, b) {
                        (_, _, Ok(data_a), Ok(data_b)) => {
                            if data_a == data_b {
                                vec![context.to_owned()]
                            } else {
                                vec![]
                            }
                        }
                        (_, exp, Ok(goal), Err(_)) | (exp, _, Err(_), Ok(goal)) => {
                            match exp.solve(&goal, context) {
                                Ok(new_context) => vec![new_context],
                                Err(_) => vec![],
                            }
                        }
                        (_, _, Err(_), Err(_)) => vec![],
                    }
                })
                .map(|e| e.to_owned())
                .collect(),

            Statement::ExpresionComparison(exp_a, exp_b, comp) => universe
                .iter()
                .filter(|context| {
                    let a = exp_a.literalize(context);
                    let b = exp_b.literalize(context);
                    match (a, b) {
                        (Ok(data_a), Ok(data_b)) => match comp {
                            Comparison::Lt => data_a < data_b,
                            Comparison::Gt => data_a > data_b,
                            Comparison::Gte => data_a <= data_b,
                            Comparison::Lte => data_a >= data_b,
                            Comparison::Eq => unreachable!(),
                        },
                        _ => false,
                    }
                })
                .map(|e| e.to_owned())
                .collect(),

            Statement::Relation(rel) => universe
                .iter()
                .filter(|context| match engine.query(rel, context) {
                    Ok(vec) => {
                        println!("\n{rel:?}\n{vec:?}\n{context:?}");
                        vec.len() != 0
                    }
                    Err(_) => false,
                })
                .map(|e| e.to_owned())
                .collect(),
        };

        ret
    }
}
