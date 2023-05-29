use std::{fmt, vec};

use crate::engine::recursion_tally::RecursionTally;
use crate::engine::var_context::VarContext;
use crate::engine::var_context_universe::VarContextUniverse;
use crate::engine::Engine;
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Comparison {
    Eq,
    Lt,
    Gt,
    Gte,
    Lte,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Statement {
    // resolvable to a bolean
    True,
    And(Box<Statement>, Box<Statement>),
    Or(Box<Statement>, Box<Statement>),
    Not(Box<Statement>),
    ExpresionComparison(Expresion, Expresion, Comparison),
    Relation(DeferedRelation),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::And(sta, stb) => write!(f, "({sta} && {stb})"),
            Statement::Or(sta, stb) => write!(f, "({sta} || {stb})"),
            Statement::Not(st) => write!(f, "!({st})"),
            Statement::ExpresionComparison(sta, stb, Comparison::Eq) => {
                write!(f, "({sta}={stb})")
            }
            Statement::ExpresionComparison(sta, stb, Comparison::Lt) => {
                write!(f, "({sta}<{stb})")
            }
            Statement::ExpresionComparison(sta, stb, Comparison::Gt) => {
                write!(f, "({sta}>{stb})")
            }
            Statement::ExpresionComparison(sta, stb, Comparison::Gte) => {
                write!(f, "({sta}>={stb})")
            }
            Statement::ExpresionComparison(sta, stb, Comparison::Lte) => {
                write!(f, "({sta}<={stb})")
            }
            Statement::Relation(rel) => write!(f, "{rel}"),
            Statement::True => write!(f, "true"),
        }
    }
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
        SpectingStatementOrNegationOrOpenParenthesisOrTrue,
        SpectingStatementOrOpenParenthesis,
        SpectingOperatorOrEnd,
        SpectingClosingParenthesis,
    }
    use StatementParserStates::*;
    let mut cursor = start_cursor;
    let mut state = SpectingStatementOrNegationOrOpenParenthesisOrTrue;

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

            (OpNot, SpectingStatementOrNegationOrOpenParenthesisOrTrue, _) => {
                negate_next_statement = true;
                state = SpectingStatementOrOpenParenthesis
            }

            (True, SpectingStatementOrNegationOrOpenParenthesisOrTrue, _) => {
                return Ok(Ok((Statement::True, i + 1)));
            }

            (
                LeftParenthesis,
                SpectingStatementOrOpenParenthesis
                | SpectingStatementOrNegationOrOpenParenthesisOrTrue,
                _,
            ) => {
                match read_statement(
                    lexograms,
                    i + 1,
                    debug_margin.to_owned() + "|  ",
                    debug_print,
                )? {
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
                SpectingStatementOrOpenParenthesis
                | SpectingStatementOrNegationOrOpenParenthesisOrTrue,
                _,
            ) => {
                match read_statement_item(
                    lexograms,
                    i,
                    debug_margin.to_owned() + "|  ",
                    debug_print,
                )? {
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
            lex_pos: lexograms.len() - 1,
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
                    debug_margin.to_owned() + "|  ",
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
                    debug_margin.to_owned() + "|  ",
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
                    debug_margin.to_owned() + "|  ",
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
    pub fn get_posible_contexts(
        &self,
        engine: &Engine,
        recursion_tally: &RecursionTally,
        universe: &VarContextUniverse,
        debug_margin: String,
        debug_print: bool,
    ) -> (VarContextUniverse, Statement) {
        if debug_print {
            println!("{debug_margin}get posible contexts of {self}");
        }
        let ret = match self {
            Statement::Or(statement_a, statement_b) => {
                let (deep_universe_a, simplified_statement_a) = statement_a.get_posible_contexts(
                    engine,
                    recursion_tally,
                    universe,
                    debug_margin.to_owned() + "|  ",
                    debug_print,
                );
                let (deep_universe_b, simplified_statement_b) = statement_b.get_posible_contexts(
                    engine,
                    recursion_tally,
                    universe,
                    debug_margin.to_owned() + "|  ",
                    debug_print,
                );

                (
                    deep_universe_a.or(
                        deep_universe_b,
                        debug_margin.to_owned() + "|  ",
                        debug_print,
                    ),
                    simplified_statement_a.or(&simplified_statement_b),
                )
            }

            Statement::And(statement_a, statement_b) => {
                let (deep_universe_a, simplified_statement_a) = statement_a.get_posible_contexts(
                    engine,
                    recursion_tally,
                    universe,
                    debug_margin.to_owned() + "|  ",
                    debug_print,
                );
                let (deep_universe_b, simplified_statement_b) = statement_b.get_posible_contexts(
                    engine,
                    recursion_tally,
                    universe,
                    debug_margin.to_owned() + "|  ",
                    debug_print,
                );

                (
                    deep_universe_a.and(
                        deep_universe_b,
                        debug_margin.to_owned() + "|  ",
                        debug_print,
                    ),
                    simplified_statement_a.and(&simplified_statement_b),
                )
            }
            Statement::Not(statement) => {
                let (contexts, _) = statement.get_posible_contexts(
                    engine,
                    recursion_tally,
                    universe,
                    debug_margin.to_owned() + "|  ",
                    debug_print,
                );

                // println!("\nNOT: \n{contexts:?}");

                (universe.difference(&contexts), self.to_owned()) //TODO i dont think i can simplify a not, look into it
            }
            Statement::ExpresionComparison(exp_a, exp_b, Comparison::Eq) => {
                if debug_print {
                    println!(
                        "{debug_margin}equality of {exp_a} and {exp_b} on universe {universe}"
                    );
                }
                let fitting_contexts = universe
                    .iter()
                    .flat_map(|context| {
                        if debug_print {
                            println!("{debug_margin}for context {context}");
                        }
                        let a = exp_a.literalize(&context);
                        let b = exp_b.literalize(&context);
                        if debug_print {
                            println!("{debug_margin}{exp_a}: {a:#?}, {exp_b}: {b:#?}");
                        }
                        match (exp_a, exp_b, a, b) {
                            (_, _, Ok(data_a), Ok(data_b)) => {
                                  if debug_print {
                                    println!("{debug_margin}{exp_a} was literalized to {data_a} and {exp_b} was literalized to {data_a}");
                                }
                                if data_a == data_b {
                                    vec![context.to_owned()]
                                } else {
                                    vec![]
                                }
                            }
                            (literalized_exp, exp, Ok(goal), Err(_)) | (exp, literalized_exp, Err(_), Ok(goal)) => {
                                if debug_print {
                                    println!("{debug_margin}{literalized_exp} was literalized to {goal}, trying to backwards solve");
                                }
                                match exp.solve(
                                    &goal,
                                    &context,
                                    debug_margin.to_owned() + "|  ",
                                    debug_print,
                                ) {
                                    Ok(new_context) => {
                                        vec![new_context]
                                    }
                                    Err(_) => vec![],
                                }
                            }
                            (_, _, Err(_), Err(_)) => vec![],
                        }
                    })
                    .collect::<Vec<VarContext>>();

                (VarContextUniverse::from(fitting_contexts), self.to_owned())
            }

            Statement::ExpresionComparison(exp_a, exp_b, comp) => {
                let fitting_contexts = universe
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
                    .collect::<Vec<VarContext>>();

                (VarContextUniverse::from(fitting_contexts), self.to_owned())
            }
            Statement::Relation(rel) => {
                let ret = match engine.get_table(rel.get_rel_id()) {
                    Some(table) => {
                        let mut ret = VarContextUniverse::new_restricting();

                        for base_context in universe.iter() {
                            let table_truths = table.get_content_iter(
                                rel.clone_n_apply(&base_context),
                                recursion_tally.to_owned(),
                                engine.to_owned(),
                                debug_margin.to_owned() + "|  ",
                                debug_print,
                            );
                            for truth in table_truths {
                                let mut unfiteable = false;
                                let mut context = base_context.clone();

                                for (col_data, col_exp) in truth.get_data().iter().zip(&rel.args) {
                                    if !unfiteable {
                                        match col_exp.solve(
                                            col_data,
                                            &context,
                                            debug_margin.to_owned() + "|  ",
                                            debug_print,
                                        ) {
                                            Ok(new_context) => {
                                                if debug_print {
                                                    println!("{debug_margin}fitting {col_data} to {col_exp} resulted on {new_context}");
                                                }
                                                context = new_context
                                            }
                                            Err(err) => {
                                                if debug_print {
                                                    println!("{debug_margin}fitting {col_data} to {col_exp} failed: {err}");
                                                }
                                                unfiteable = true;
                                            }
                                        }
                                    }
                                }
                                if !unfiteable {
                                    ret.insert(context);
                                }
                            }
                        }
                        ret
                    }
                    None => VarContextUniverse::new_restricting(),
                };

                (ret, Statement::True)
            }

            Statement::True => (universe.to_owned(), Statement::True),
        };
        if debug_print {
            println!("{debug_margin}* universe for {self} based on {universe} is {}, simplifing to {}",ret.0,ret.1);
        }
        ret
    }

    fn or(&self, other: &Statement) -> Statement {
        match (&self, &other) {
            (Statement::True, _) | (_, Statement::True) => Statement::True,
            _ => Statement::Or(Box::new(self.to_owned()), Box::new(other.to_owned())),
        }
    }
    fn and(&self, other: &Statement) -> Statement {
        match (&self, &other) {
            (Statement::True, res) | (res, Statement::True) => res.to_owned().to_owned(),
            _ => Statement::And(Box::new(self.to_owned()), Box::new(other.to_owned())),
        }
    }
}
