use crate::lexer;
use lexer::LexogramType::*;
#[derive(Debug)]
struct RelName(String);

#[derive(Debug)]
pub enum VarName {
    DestructuredList(Vec<Expresion>),
    Direct(String),
}

#[derive(Debug)]
pub enum VarLiteral {
    Number(f64),
    String(String),
    Array(Box<Vec<Expresion>>),
}
#[derive(Debug)]
pub enum Statement {
    // resolvable to a bolean
    Hypothetical(Box<Statement>, Box<Statement>),
    And(Box<Statement>, Box<Statement>),
    Or(Box<Statement>, Box<Statement>),
    Eq(RelName, RelName),
    Not(Box<Statement>),
    Arithmetic(Expresion, Expresion, fn(Expresion, Expresion) -> bool),
    Relation(RelName, Vec<Expresion>),
    TrueWhen(Box<Statement>, Box<Statement>),
    Empty,
}

#[derive(Debug)]
struct FailureExplanation {
    lex_pos: usize,
    if_it_was: String,
    failed_because: String,
    parent_failure: Option<Vec<FailureExplanation>>,
}

#[derive(Debug)]
pub enum Expresion {
    // resolvable to a value
    Arithmetic(
        Box<Expresion>,
        Box<Expresion>,
        fn(Expresion, Expresion) -> Expresion,
    ),
    Literal(VarLiteral),
    RestOfList(VarName),
    Var(VarName),
    Empty,
}
#[derive(Debug)]
pub enum ParserError {
    PrintfError(sprintf::PrintfError),
    Custom(String),
}

impl From<sprintf::PrintfError> for ParserError {
    fn from(e: sprintf::PrintfError) -> Self {
        Self::PrintfError(e)
    }
}

fn add_expresions(a: Expresion, b: Expresion) -> Expresion {
    todo!()
}

fn sub_expresions(a: Expresion, b: Expresion) -> Expresion {
    todo!()
}

fn mul_expresions(a: Expresion, b: Expresion) -> Expresion {
    todo!()
}

fn div_expresions(a: Expresion, b: Expresion) -> Expresion {
    todo!()
}

fn eq_expresions(a: Expresion, b: Expresion) -> bool {
    todo!()
}

fn lt_expresions(a: Expresion, b: Expresion) -> bool {
    todo!()
}

fn gt_expresions(a: Expresion, b: Expresion) -> bool {
    todo!()
}

fn lte_expresions(a: Expresion, b: Expresion) -> bool {
    todo!()
}

fn gte_expresions(a: Expresion, b: Expresion) -> bool {
    todo!()
}

fn read_item(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    only_literals: bool,
    debug_margin: String,
) -> Result<Result<(Expresion, usize), FailureExplanation>, ParserError> {
    println!("{}read_item at {}", debug_margin, start_cursor);

    match (lexograms[start_cursor].l_type.clone(), only_literals) {
        (Number(n), _) => Ok(Ok((
            Expresion::Literal(VarLiteral::Number(n)),
            start_cursor + 1,
        ))),
        (Word(n), _) => Ok(Ok((
            Expresion::Literal(VarLiteral::String(n)),
            start_cursor + 1,
        ))),
        (LeftBracket, _) => {
            match read_array(
                lexograms,
                start_cursor,
                only_literals,
                debug_margin.clone() + "   ",
            )? {
                Ok(ret) => Ok(Ok(ret)),
                Err(explanation) => Ok(Err(FailureExplanation {
                    lex_pos: start_cursor,
                    if_it_was: "item".into(),
                    failed_because: "was not an array".into(),
                    parent_failure: Some(vec![explanation]),
                })),
            }
        }
        (Identifier(str), false) => {
            Ok(Ok((Expresion::Var(VarName::Direct(str)), start_cursor + 1)))
        }

        _ => Ok(Err(FailureExplanation {
            lex_pos: start_cursor,
            if_it_was: "item".into(),
            failed_because: "pattern missmatch trying to read item".into(),
            parent_failure: None,
        })),
    }
}

fn read_array(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    only_literals: bool,
    debug_margin: String,
) -> Result<Result<(Expresion, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum ArrayParserStates {
        SpectingItemOrEnd,
        SpectingIdentifierAfterDotDotDot,
        SpectingItemOrDotDotDot,
        SpectingComaOrEnd,
        SpectingEnd,
        SpectingStart,
    }
    use ArrayParserStates::*;

    println!("{}read_array at {}", debug_margin, start_cursor);

    let mut cursor = start_cursor;

    let mut ret = vec![];
    let mut state = SpectingStart;

    for (i, lex) in lexograms.iter().enumerate() {
        // println!("state: {:?}",state);
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state, only_literals) {
            (LeftBracket, SpectingStart, _) => {
                state = SpectingItemOrEnd;
            }
            (DotDotDot, SpectingItemOrDotDotDot, false) => {
                state = SpectingIdentifierAfterDotDotDot;
            }
            (Identifier(str), SpectingIdentifierAfterDotDotDot, false) => {
                ret.push(Expresion::RestOfList(VarName::Direct(str)));
                state = SpectingEnd;
            }
            (Coma, SpectingComaOrEnd, _) => state = SpectingItemOrDotDotDot,
            (RightBracket, SpectingComaOrEnd | SpectingEnd | SpectingItemOrEnd, _) => {
                println!("{debug_margin}end of array at {}", i + 1);
                if only_literals {
                    return Ok(Ok((
                        Expresion::Literal(VarLiteral::Array(Box::new(ret))),
                        i + 1,
                    )));
                } else {
                    return Ok(Ok((Expresion::Var(VarName::DestructuredList(ret)), i + 1)));
                }
            }
            (_, SpectingItemOrEnd | SpectingItemOrDotDotDot, _) => {
                match read_expresion(lexograms, i, only_literals, debug_margin.clone() + "   ")? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "array".into(),
                            failed_because: "specting item".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                    Ok((expresion, jump_to)) => {
                        ret.push(expresion);
                        cursor = jump_to;
                    }
                }

                state = SpectingComaOrEnd;
            }
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "array".into(),
                    failed_because: format!("pattern missmatch on {:?} state", state).into(),
                    parent_failure: None,
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "array".into(),
        failed_because: "file ended".into(),
        parent_failure: None,
    }))
}

fn read_expresion(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    only_literals: bool,
    debug_margin: String,
) -> Result<Result<(Expresion, usize), FailureExplanation>, ParserError> {
    println!("{}read_expresion at {}", debug_margin, start_cursor);

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
    let mut append_mode: Option<fn(Expresion, Expresion) -> Expresion> = None;

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
                match read_expresion(lexograms, i, only_literals, debug_margin.clone() + "   ")? {
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
                match read_item(lexograms, i, only_literals, debug_margin.clone() + "   ")? {
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
                            failed_because: format!("pattern missmatch on {:?} state", state)
                                .into(),
                            parent_failure: None,
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
                    failed_because: format!("pattern missmatch on {:?} state", state).into(),
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

fn read_list(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    only_literals: bool,
    debug_margin: String,
) -> Result<Result<(Vec<Expresion>, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum ListParserStates {
        SpectingItemOrEnd,
        SpectingItem,
        SpectingComaOrEnd,
        SpectingStart,
    }

    println!("{}read_list at {}", debug_margin, start_cursor);

    use ListParserStates::*;
    let mut cursor = start_cursor;

    let mut ret = vec![];
    let mut state = SpectingStart;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state, only_literals) {
            (LeftParenthesis, SpectingStart, _) => {
                state = SpectingItemOrEnd;
            }
            (RightParenthesis, SpectingItemOrEnd | SpectingComaOrEnd, _) => {
                return Ok(Ok((ret, i + 1)));
            }
            (Coma, SpectingComaOrEnd, _) => {
                state = SpectingItem;
            }
            (_, SpectingItem | SpectingItemOrEnd, _) => {
                match read_expresion(lexograms, i, only_literals, debug_margin.clone() + "   ")? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "list".into(),
                            failed_because: "Specting item".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                    Ok((e, i)) => {
                        ret.push(e);
                        cursor = i;
                    }
                }
                state = SpectingComaOrEnd;
            }
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "list".into(),
                    failed_because: format!("pattern missmatch on {:?} state", state).into(),
                    parent_failure: None,
                }));
            }
        }
    }
    return Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "list".into(),
        failed_because: "file ended".into(),
        parent_failure: None,
    }));
}

fn read_relation(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    only_literals: bool,
    debug_margin: String,
) -> Result<Result<(Statement, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum RelationParserStates {
        SpectingStatementIdentifier,
        SpectingStatementList,
    }
    use RelationParserStates::*;

    println!("{debug_margin}read_relation at {start_cursor}");

    let mut cursor = start_cursor;
    let mut r_name = RelName("default_relation_name".into());
    let mut state = SpectingStatementIdentifier;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state) {
            (Identifier(str), SpectingStatementIdentifier) => {
                r_name = RelName(str);
                state = SpectingStatementList;
            }
            (_, SpectingStatementList) => {
                match read_list(lexograms, i, only_literals, debug_margin.clone() + "   ")? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "relation".into(),
                            failed_because: "specting list".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                    Ok((v, new_cursor)) => {
                        return Ok(Ok((Statement::Relation(r_name, v), new_cursor)))
                    }
                }
            }
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "relation".into(),
                    failed_because: format!("pattern missmatch on {:?} state", state).into(),
                    parent_failure: None,
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "relation".into(),
        failed_because: "file ended".into(),
        parent_failure: None,
    }))
}

fn read_intensional(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
) -> Result<Result<(Statement, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum IntensionalParserStates {
        SpectingRelationDef,
        SpectingTrueWhen,
        SpectingCondition,
        SpectingExtensional,
    }
    use IntensionalParserStates::*;

    println!("{debug_margin}read_intensional at {start_cursor}");

    let mut cursor = start_cursor;
    let mut base_relation = Statement::Empty;
    let mut state = SpectingRelationDef;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state) {
            (_, SpectingRelationDef) => {
                match read_relation(lexograms, i, false, debug_margin.clone() + "   ")? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "intensional".into(),
                            failed_because: "specting relation".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                    Ok((r, jump_to)) => {
                        cursor = jump_to;
                        base_relation = r;
                        state = SpectingTrueWhen;
                    }
                }
            }
            (TrueWhen, SpectingTrueWhen) => state = SpectingCondition,
            (_, SpectingCondition) => {
                match read_logical_statement_concatenation(
                    lexograms,
                    i,
                    debug_margin.clone() + "   ",
                )? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "intensional".into(),
                            failed_because: "specting statement".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                    Ok((cond, jump_to)) => {
                        return Ok(Ok((
                            Statement::TrueWhen(Box::new(base_relation), Box::new(cond)),
                            jump_to,
                        )))
                    }
                }
            }

            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "intensional".into(),
                    failed_because: format!("pattern missmatch on {:?} state", state).into(),
                    parent_failure: None,
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "intensional".into(),
        failed_because: "file ended".into(),
        parent_failure: None,
    }))
}

fn read_statement(
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

    println!("{}read_statement at {}", debug_margin, start_cursor);

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
                match read_relation(lexograms, i, false, debug_margin.clone() + "   ")? {
                    Ok(ret) => return Ok(Ok(ret)),

                    Err(e1) => {
                        match read_expresion(lexograms, i, false, debug_margin.clone() + "   ")? {
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
                match read_expresion(lexograms, i, false, debug_margin.clone() + "   ")? {
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
                    Err(e) => {}
                }
            }

            (lex, state) => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "statement".into(),
                    failed_because: format!(
                        "pattern missmatch on {:?} state reading lex {:?}",
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

fn read_logical_statement_concatenation(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
) -> Result<Result<(Statement, usize), FailureExplanation>, ParserError> {
    println!(
        "{}read_logical_statement_concatenation at {}",
        debug_margin, start_cursor
    );

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

                            _ => ret,
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
                match read_statement(lexograms, i, debug_margin.clone() + "   ")? {
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

                            _ => ret,
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
                    failed_because: format!("pattern missmatch on {:?} state", state).into(),
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

fn read_line(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
) -> Result<Result<(Statement, usize), FailureExplanation>, ParserError> {
    let a;
    let b;
    match read_relation(lexograms, start_cursor, true, debug_margin.clone() + "   ")? {
        Ok(ret) => return Ok(Ok(ret)),
        Err(e) => a = e,
    }
    match read_intensional(lexograms, start_cursor, debug_margin.clone() + "   ")? {
        Ok(ret) => return Ok(Ok(ret)),
        Err(e) => b = e,
    }
    Ok(Err(FailureExplanation {
        lex_pos: start_cursor,
        if_it_was: "line".into(),
        failed_because: "wasnt neither an extensional nor an intensional statement".into(),
        parent_failure: Some(vec![a, b]),
    }))
}

pub fn parse(lexograms: Vec<lexer::Lexogram>) -> Result<Vec<Statement>, ParserError> {
    let test = read_line(&lexograms, 0, "".into())?;
    println!("\n{:?}\n", test);
    todo!()
}
