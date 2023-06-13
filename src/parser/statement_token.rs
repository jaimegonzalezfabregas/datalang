use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashSet};
use std::hash::{Hash, Hasher};
use std::{fmt, vec};

use crate::engine::recursion_tally::RecursionTally;
use crate::engine::var_context::VarContext;
use crate::engine::var_context_universe::VarContextUniverse;
use crate::engine::Engine;
use crate::lexer::LexogramType::*;

use crate::parser::defered_relation_token::read_defered_relation;
use crate::parser::expresion_token::read_expresion;

use crate::lexer::{self};
use crate::printdev;

use super::data_token::Data;
use super::defered_relation_token::DeferedRelation;
use super::error::{FailureExplanation, ParserError};
use super::expresion_token::Expresion;

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
pub enum StatementSemantics {
    // resolvable to a bolean
    True,
    And(Box<Statement>, Box<Statement>),
    Or(Box<Statement>, Box<Statement>),
    Not(Box<Statement>),
    ExpresionComparison(Expresion, Expresion, Comparison),
    Relation(DeferedRelation),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Statement {
    memoizer: BTreeMap<u64, Result<VarContextUniverse, String>>,
    semantics: StatementSemantics,
}

impl From<StatementSemantics> for Statement {
    fn from(value: StatementSemantics) -> Self {
        Self {
            memoizer: BTreeMap::new(),
            semantics: value,
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.semantics {
            StatementSemantics::And(sta, stb) => write!(f, "({sta} && {stb})"),
            StatementSemantics::Or(sta, stb) => write!(f, "({sta} || {stb})"),
            StatementSemantics::Not(st) => write!(f, "!({st})"),
            StatementSemantics::ExpresionComparison(sta, stb, Comparison::Eq) => {
                write!(f, "({sta}={stb})")
            }
            StatementSemantics::ExpresionComparison(sta, stb, Comparison::Lt) => {
                write!(f, "({sta}<{stb})")
            }
            StatementSemantics::ExpresionComparison(sta, stb, Comparison::Gt) => {
                write!(f, "({sta}>{stb})")
            }
            StatementSemantics::ExpresionComparison(sta, stb, Comparison::Gte) => {
                write!(f, "({sta}>={stb})")
            }
            StatementSemantics::ExpresionComparison(sta, stb, Comparison::Lte) => {
                write!(f, "({sta}<={stb})")
            }
            StatementSemantics::Relation(rel) => write!(f, "{rel}"),
            StatementSemantics::True => write!(f, "true"),
        }
    }
}

pub fn read_statement(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    
) -> Result<Result<(Statement, usize), FailureExplanation>, ParserError> {
    printdev!(
            "{}read_logical_statement_concatenation at {}",
            debug_margin, start_cursor
        );
    

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
                return Ok(Ok((StatementSemantics::True.into(), i + 1)));
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
    
) -> Result<Result<(Statement, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum StatementParserStates {
        SpectingFirstExpresionOrRelation,
        SpectingExrpresionComparisonOperator,
        SpectingSecondExpresion,
    }
    use StatementParserStates::*;

    printdev!("{}read_statement at {}", debug_margin, start_cursor);
    

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
                    
                )? {
                    Ok((def_rel, jump_to)) => {
                        return Ok(Ok((StatementSemantics::Relation(def_rel).into(), jump_to)))
                    }

                    Err(err) => err1 = err,
                }
                match read_expresion(
                    lexograms,
                    i,
                    false,
                    debug_margin.to_owned() + "|  ",
                    
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
                    
                )? {
                    Ok((second_expresion, jump_to)) => {
                        return Ok(Ok((
                            match append_mode {
                                OpEq => StatementSemantics::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    Comparison::Eq,
                                ),
                                OpLT => StatementSemantics::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    Comparison::Lt,
                                ),
                                OpLTE => StatementSemantics::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    Comparison::Lte,
                                ),
                                OpGT => StatementSemantics::ExpresionComparison(
                                    first_expresion,
                                    second_expresion,
                                    Comparison::Gt,
                                ),
                                OpGTE => StatementSemantics::ExpresionComparison(
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
                            }
                            .into(),
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
            StatementSemantics::And(Box::new(prev_statement), Box::new(new_statement)).into()
        }
        (Some(prev_statement), AppendModes::Or, false) => {
            StatementSemantics::Or(Box::new(prev_statement), Box::new(new_statement)).into()
        }
        (Some(prev_statement), AppendModes::And, true) => StatementSemantics::And(
            Box::new(prev_statement),
            Box::new(StatementSemantics::Not(Box::new(new_statement)).into()),
        )
        .into(),
        (Some(prev_statement), AppendModes::Or, true) => StatementSemantics::Or(
            Box::new(prev_statement),
            Box::new(StatementSemantics::Not(Box::new(new_statement)).into()),
        )
        .into(),
        (Some(_), AppendModes::None, _) => unreachable!(),
    })
}

impl Statement {
    pub fn memo_get_posible_contexts(
        &mut self,
        engine: &Engine,
        recursion_tally: &RecursionTally,
        universe: &VarContextUniverse,
        debug_margin: String,
        
    ) -> Result<VarContextUniverse, String> {
        printdev!("{debug_margin}get posible contexts of {self} over universe:{universe}");
        

        let mut memo_hash = DefaultHasher::new();
        engine.hash(&mut memo_hash);
        universe.hash(&mut memo_hash);

        let hash = memo_hash.finish();

        let ret = if let Some(recall) = self.memoizer.get(&hash) {
            printdev!("{debug_margin}CACHE HIT");
            
            recall.to_owned()?
        } else {
            let ret = self.get_posible_contexts(
                engine,
                recursion_tally,
                universe,
                debug_margin.to_owned() + "|  ",
                
            );

            self.memoizer.insert(hash, ret.to_owned());

            ret?
        };
        printdev!("{debug_margin}* universe for {self} based on {universe} is {ret}");
        
        Ok(ret)
    }

    fn get_posible_contexts(
        &mut self,
        engine: &Engine,
        recursion_tally: &RecursionTally,
        universe: &VarContextUniverse,
        debug_margin: String,
        
    ) -> Result<VarContextUniverse, String> {
        let ret = match &mut self.semantics {
            StatementSemantics::Or(statement_a, statement_b) => {
                let deep_universe_a = statement_a.memo_get_posible_contexts(
                    engine,
                    recursion_tally,
                    universe,
                    debug_margin.to_owned() + "1  ",
                    
                )?;

                let deep_universe_b = statement_b.memo_get_posible_contexts(
                    engine,
                    recursion_tally,
                    universe,
                    debug_margin.to_owned() + "2  ",
                    
                )?;

                deep_universe_a.or(
                    deep_universe_b,
                    debug_margin.to_owned() + "|  ",
                    
                )
            }

            StatementSemantics::And(statement_a, statement_b) => {
                let mut ret = universe.to_owned();
                loop {
                    let first_universe_a = statement_a.memo_get_posible_contexts(
                        engine,
                        recursion_tally,
                        universe,
                        debug_margin.to_owned() + "1  ",
                        
                    )?;

                    let first_universe_b = statement_b.memo_get_posible_contexts(
                        engine,
                        recursion_tally,
                        universe,
                        debug_margin.to_owned() + "2  ",
                        
                    )?;

                    let universe_a = statement_a.memo_get_posible_contexts(
                        engine,
                        recursion_tally,
                        &first_universe_b,
                        debug_margin.to_owned() + "3  ",
                        
                    )?;

                    let universe_b = statement_b.memo_get_posible_contexts(
                        engine,
                        recursion_tally,
                        &first_universe_a,
                        debug_margin.to_owned() + "4  ",
                        
                    )?;

                    let new_ret =
                        universe_a.or(universe_b, debug_margin.to_owned() + "|  ");
                    if new_ret != ret {
                        ret = new_ret
                    } else {
                        break;
                    }
                }
                ret
            }
            StatementSemantics::Not(statement) => {
                let negated_contexts = statement.memo_get_posible_contexts(
                    engine,
                    recursion_tally,
                    universe,
                    debug_margin.to_owned() + "|  ",
                    
                )?;

                universe.difference(&negated_contexts) //TODO i dont think i can simplify a not, look into it
            }
            StatementSemantics::ExpresionComparison(exp_a, exp_b, Comparison::Eq) => {
                printdev!(
                        "{debug_margin}equality of {exp_a} and {exp_b} on universe {universe}"
                    );
                
                let mut fitting_contexts = HashSet::new();

                let owned_exp_a = exp_a.to_owned();
                let owned_exp_b = exp_b.to_owned();

                for context in universe.iter() {
                   
                    let a = owned_exp_a.literalize(&context);
                    let b = owned_exp_b.to_owned().literalize(&context);
                    match (&owned_exp_a, &owned_exp_b, a, b) {
                        (_, _, Ok(Data::Any), Ok(Data::Any)) => {
                            printdev!(
                                    "{debug_margin}{owned_exp_a} and {owned_exp_b} are not equal (value wise)"
                                );
                            
                        }
                        (_, exp, Ok(goal), Err(_) | Ok(Data::Any))
                        | (exp, _, Err(_) | Ok(Data::Any), Ok(goal)) => {
                           
                            match exp.solve(
                                &goal,
                                &context,
                                debug_margin.to_owned() + "|  ",
                                
                            ) {
                                Ok(new_context) => {
                                    fitting_contexts.insert(new_context);
                                }
                                Err(_) => (),
                            }
                        }
                        (_, _, Ok(data_a), Ok(data_b)) => {
                          
                            if data_a == data_b {
                                fitting_contexts.insert(context.to_owned());
                            }
                        }
                        (_, _, Err(_), Err(_)) => (),
                    }
                }

                VarContextUniverse {
                    contents: fitting_contexts,
                }
            }

            StatementSemantics::ExpresionComparison(exp_a, exp_b, comp) => {
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
                    .collect::<HashSet<VarContext>>();

                VarContextUniverse {
                    contents: fitting_contexts,
                }
            }
            StatementSemantics::Relation(rel) => {
                printdev!(
                        "{debug_margin}recursive relation querry for {rel} in each of: {universe}"
                    );
                

                let mut ret = VarContextUniverse::new();

                for base_context in universe.iter() {
                    let table_truths = engine.query(
                        &rel.clone_and_apply(&base_context),
                        &base_context,
                        recursion_tally,
                        debug_margin.to_owned() + "|   ",
                        
                    )?;

                    for truth in table_truths.into_iter() {
                        let mut unfiteable = false;
                        let mut context = base_context.clone();

                        for (col_data, col_exp) in truth.get_data().iter().zip(&rel.args) {
                            if !unfiteable {
                                match col_exp.solve(
                                    col_data,
                                    &context,
                                    debug_margin.to_owned() + "|  ",
                                    
                                ) {
                                    Ok(new_context) => {
                                       
                                        context = new_context
                                    }
                                    Err(_) => {
                                       
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

            StatementSemantics::True => universe.to_owned(),
        };

        Ok(ret)
    }
}
