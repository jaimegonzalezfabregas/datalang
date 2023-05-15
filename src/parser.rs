use crate::lexer;
use crate::syntax::*;
use lexer::LexogramType::*;

#[derive(Debug)]
pub enum ParserError {
    PrintfError(sprintf::PrintfError),
    Custom(String),
    SyntaxError(FailureExplanation),
}

#[derive(Debug)]
pub struct FailureExplanation {
    lex_pos: usize,
    if_it_was: String,
    failed_because: String,
    parent_failure: Option<Vec<FailureExplanation>>,
}

impl From<sprintf::PrintfError> for ParserError {
    fn from(e: sprintf::PrintfError) -> Self {
        Self::PrintfError(e)
    }
}

impl From<String> for ParserError {
    fn from(e: String) -> Self {
        Self::Custom(e)
    }
}

fn add_expresions(a: Expresion, b: Expresion) -> Result<Expresion, String> {
    let op1 = match &a {
        Expresion::Arithmetic(_, _, _) => a.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("primer argumento no literalizable".into()),
    };
    let op2 = match &b {
        Expresion::Arithmetic(_, _, _) => b.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("segundo argumento no literalizable".into()),
    };

    match &(op1, op2) {
        (_, VarLiteral::FullSet) => Err("cant operate on any".into()),
        (VarLiteral::FullSet, _) => Err("cant operate on any".into()),

        (VarLiteral::Set(set_a), VarLiteral::Set(set_b)) => {
            let mut ret = vec![];
            for it_a in set_a {
                for it_b in set_b {
                    ret.push(match (it_a, it_b) {
                        (Data::Number(x), Data::Number(y)) => Data::Number(x + y),
                        (Data::String(x), Data::String(y)) => Data::String(x.clone() + y),
                        (Data::Array(x), Data::Array(y)) => {
                            Data::Array(x.iter().chain(y.iter()).map(|e| e.clone()).collect())
                        }
                        _ => return Err("cant operate on diferently typed literals".into()),
                    })
                }
            }
            Ok(Expresion::Literal(VarLiteral::Set(ret)))
        }

        _ => Err("cant operate on non literals".into()),
    }
}

fn sub_expresions(a: Expresion, b: Expresion) -> Result<Expresion, String> {
    todo!()
}

fn mul_expresions(a: Expresion, b: Expresion) -> Result<Expresion, String> {
    todo!()
}

fn div_expresions(a: Expresion, b: Expresion) -> Result<Expresion, String> {
    todo!()
}

fn eq_expresions(a: Expresion, b: Expresion) -> Result<bool, String> {
    todo!()
}

fn lt_expresions(a: Expresion, b: Expresion) -> Result<bool, String> {
    todo!()
}

fn gt_expresions(a: Expresion, b: Expresion) -> Result<bool, String> {
    todo!()
}

fn lte_expresions(a: Expresion, b: Expresion) -> Result<bool, String> {
    todo!()
}

fn gte_expresions(a: Expresion, b: Expresion) -> Result<bool, String> {
    todo!()
}

fn read_expresion_item(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    only_literals: bool,
    debug_margin: String,
) -> Result<Result<(Expresion, usize), FailureExplanation>, ParserError> {
    println!("{}read_item at {}", debug_margin, start_cursor);

    match (lexograms[start_cursor].l_type.clone(), only_literals) {
        (Any, _) => Ok(Ok((
            Expresion::Literal(VarLiteral::FullSet),
            start_cursor + 1,
        ))),

        (OpLT | OpNot, _) => match read_set(lexograms, start_cursor, debug_margin.clone() + "   ")?
        {
            Ok(ret) => Ok(Ok(ret)),
            Err(explanation) => Ok(Err(FailureExplanation {
                lex_pos: start_cursor,
                if_it_was: "item".into(),
                failed_because: "was not an array".into(),
                parent_failure: Some(vec![explanation]),
            })),
        },
        (Identifier(str), false) => {
            Ok(Ok((Expresion::Var(VarName::Direct(str)), start_cursor + 1)))
        }
        (LeftBracket, false) => {
            match read_data(lexograms, start_cursor, debug_margin.clone() + "   ")? {
                Ok((ret, jump_to)) => Ok(Ok((
                    Expresion::Literal(VarLiteral::Set(vec![ret])),
                    jump_to,
                ))),
                Err(a) => match read_destructuring_array(
                    lexograms,
                    start_cursor,
                    debug_margin.clone() + "   ",
                )? {
                    Ok(ret) => Ok(Ok(ret)),

                    Err(b) => Ok(Err(FailureExplanation {
                        lex_pos: start_cursor,
                        if_it_was: "item".into(),
                        failed_because: "specting some array".into(),
                        parent_failure: Some(vec![a, b]),
                    })),
                },
            }
        }

        (_, _) => match read_data(lexograms, start_cursor, debug_margin.clone() + "   ")? {
            Ok((ret, jump_to)) => Ok(Ok((
                Expresion::Literal(VarLiteral::Set(vec![ret])),
                jump_to,
            ))),
            Err(e) => Ok(Err(FailureExplanation {
                lex_pos: start_cursor,
                if_it_was: "item".into(),
                failed_because: "specting data".into(),
                parent_failure: Some(vec![e]),
            })),
        },
    }
}

fn read_data(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
) -> Result<Result<(Data, usize), FailureExplanation>, ParserError> {
    println!("{}read_data at {}", debug_margin, start_cursor);

    match lexograms[start_cursor].l_type.clone() {
        Number(n) => Ok(Ok((Data::Number(n), start_cursor + 1))),
        Word(n) => Ok(Ok((Data::String(n), start_cursor + 1))),
        LeftBracket => {
            match read_data_array(lexograms, start_cursor, debug_margin.clone() + "   ")? {
                Ok((ret, jump_to)) => Ok(Ok((Data::Array(ret), jump_to))),
                Err(explanation) => Ok(Err(FailureExplanation {
                    lex_pos: start_cursor,
                    if_it_was: "item".into(),
                    failed_because: "was not an array".into(),
                    parent_failure: Some(vec![explanation]),
                })),
            }
        }

        _ => Ok(Err(FailureExplanation {
            lex_pos: start_cursor,
            if_it_was: "item".into(),
            failed_because: "pattern missmatch trying to read item".into(),
            parent_failure: None,
        })),
    }
}

fn read_destructuring_array(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
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

    println!(
        "{}read_destructuring_array at {}",
        debug_margin, start_cursor
    );

    let mut cursor = start_cursor;

    let mut ret = vec![];
    let mut state = SpectingStart;

    for (i, lex) in lexograms.iter().enumerate() {
        // println!("state: {:?}",state);
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state) {
            (LeftBracket, SpectingStart) => {
                state = SpectingItemOrEnd;
            }
            (DotDotDot, SpectingItemOrDotDotDot) => {
                state = SpectingIdentifierAfterDotDotDot;
            }
            (Identifier(str), SpectingIdentifierAfterDotDotDot) => {
                ret.push(Expresion::RestOfList(VarName::Direct(str)));
                state = SpectingEnd;
            }
            (Coma, SpectingComaOrEnd) => state = SpectingItemOrDotDotDot,
            (RightBracket, SpectingComaOrEnd | SpectingEnd | SpectingItemOrEnd) => {
                println!("{debug_margin}end of destructuring array at {}", i + 1);
                return Ok(Ok((Expresion::Var(VarName::DestructuredArray(ret)), i + 1)));
            }
            (_, SpectingItemOrEnd | SpectingItemOrDotDotDot) => {
                match read_expresion(lexograms, i, false, debug_margin.clone() + "   ")? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "destructuring_array".into(),
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
                    if_it_was: "destructuring_array".into(),
                    failed_because: format!("pattern missmatch on {:?} state", state).into(),
                    parent_failure: None,
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "destructuring_array".into(),
        failed_because: "file ended".into(),
        parent_failure: None,
    }))
}

fn read_data_array(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
) -> Result<Result<(Vec<Data>, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum ArrayParserStates {
        SpectingItemOrEnd,
        SpectingItem,
        SpectingComaOrEnd,
        SpectingStart,
    }
    use ArrayParserStates::*;

    println!("{}read_varLiteral_array at {}", debug_margin, start_cursor);

    let mut cursor = start_cursor;

    let mut ret = vec![];
    let mut state = SpectingStart;

    for (i, lex) in lexograms.iter().enumerate() {
        // println!("state: {:?}",state);
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state) {
            (LeftBracket, SpectingStart) => {
                state = SpectingItemOrEnd;
            }

            (Coma, SpectingComaOrEnd) => state = SpectingItem,
            (RightBracket, SpectingComaOrEnd | SpectingItemOrEnd) => {
                println!("{debug_margin}end of varLiteral_array at {}", i + 1);
                return Ok(Ok((ret, i + 1)));
            }
            (_, SpectingItemOrEnd | SpectingItem) => {
                match read_expresion(lexograms, i, true, debug_margin.clone() + "   ")? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "varLiteral_array".into(),
                            failed_because: "specting item".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                    Ok((expresion, jump_to)) => {
                        ret.push(expresion.literalize()?.get_element_if_singleton()?);
                        cursor = jump_to;
                    }
                }

                state = SpectingComaOrEnd;
            }
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "varLiteral_array".into(),
                    failed_because: format!("pattern missmatch on {:?} state", state).into(),
                    parent_failure: None,
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "data_array".into(),
        failed_because: "file ended".into(),
        parent_failure: None,
    }))
}

fn read_set(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
) -> Result<Result<(Expresion, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum ArrayParserStates {
        SpectingItemOrEnd,
        SpectingItem,
        SpectingComaOrEnd,
        SpectingStartOrNegation,
        SpectingStart,
    }
    use ArrayParserStates::*;

    println!("{}read_set at {}", debug_margin, start_cursor);

    let mut cursor = start_cursor;

    let mut negated = false;

    let mut ret = vec![];
    let mut state = SpectingStartOrNegation;

    for (i, lex) in lexograms.iter().enumerate() {
        // println!("state: {:?}",state);
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state) {
            (OpNot, SpectingStartOrNegation) => {
                negated = true;
                state = SpectingStart
            }
            (OpLT, SpectingStart | SpectingStartOrNegation) => {
                state = SpectingItemOrEnd;
            }
            (Coma, SpectingComaOrEnd) => state = SpectingItem,
            (OpGT, SpectingComaOrEnd | SpectingItemOrEnd) => {
                println!("{debug_margin}end of set at {}", i + 1);
                return if negated {
                    Ok(Ok((Expresion::Literal(VarLiteral::AntiSet(ret)), i + 1)))
                } else {
                    Ok(Ok((Expresion::Literal(VarLiteral::Set(ret)), i + 1)))
                };
            }
            (_, SpectingItemOrEnd | SpectingItem) => {
                match read_expresion(lexograms, i, true, debug_margin.clone() + "   ")? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "array".into(),
                            failed_because: "specting item".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                    Ok((expresion, jump_to)) => {
                        ret.push(expresion.literalize()?.get_element_if_singleton()?);
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
                match read_expresion_item(
                    lexograms,
                    i,
                    only_literals,
                    debug_margin.clone() + "   ",
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
                            failed_because: format!("pattern missmatch on {:?} state", state)
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
        SpectingItem,
        SpectingComaOrClosingParenthesis,
        SpectingOpenParenthesis,
    }

    println!("{}read_list at {}", debug_margin, start_cursor);

    use ListParserStates::*;
    let mut cursor = start_cursor;

    let mut ret = vec![];
    let mut state = SpectingOpenParenthesis;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state, only_literals) {
            (LeftParenthesis, SpectingOpenParenthesis, _) => {
                state = SpectingItem;
            }
            (RightParenthesis, SpectingComaOrClosingParenthesis, _) => {
                return Ok(Ok((ret, i + 1)));
            }
            (Coma, SpectingComaOrClosingParenthesis, _) => {
                state = SpectingItem;
            }
            (_, SpectingItem, _) => {
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
                state = SpectingComaOrClosingParenthesis;
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

fn read_literal_relation(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
) -> Result<Result<(Line, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum RelationParserStates {
        SpectingStatementIdentifierOrNot,
        SpectingStatementIdentifier,
        SpectingStatementList,
    }
    use RelationParserStates::*;

    println!("{debug_margin}read_literal_relation at {start_cursor}");

    let mut cursor = start_cursor;
    let mut r_name = RelName("default_relation_name".into());
    let mut state = SpectingStatementIdentifierOrNot;

    let mut forget = false;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state) {
            (OpNot, SpectingStatementIdentifierOrNot) => {
                forget = true;
                state = SpectingStatementIdentifier
            }
            (Identifier(str), SpectingStatementIdentifier | SpectingStatementIdentifierOrNot) => {
                r_name = RelName(str);
                state = SpectingStatementList;
            }
            (_, SpectingStatementList) => {
                return match read_list(lexograms, i, true, debug_margin.clone() + "   ")? {
                    Err(e) => Ok(Err(FailureExplanation {
                        lex_pos: i,
                        if_it_was: "literal relation".into(),
                        failed_because: "specting list".into(),
                        parent_failure: Some(vec![e]),
                    })),
                    Ok((v, new_cursor)) => {
                        let mut literal_vec = vec![];

                        for exp in v {
                            literal_vec.push(exp.literalize()?);
                        }

                        if forget {
                            Ok(Ok((Line::ForgetRelation(r_name, literal_vec), new_cursor)))
                        } else {
                            Ok(Ok((Line::CreateRelation(r_name, literal_vec), new_cursor)))
                        }
                    }
                }
            }
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "literal relation".into(),
                    failed_because: format!("pattern missmatch on {:?} state", state).into(),
                    parent_failure: None,
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "literal relation".into(),
        failed_because: "file ended".into(),
        parent_failure: None,
    }))
}

fn read_querring_relation(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
) -> Result<Result<(RelName, Vec<Expresion>, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum RelationParserStates {
        SpectingStatementIdentifier,
        SpectingStatementList,
        SpectingQuery,
    }
    use RelationParserStates::*;

    println!("{debug_margin}read_querring_relation at {start_cursor}");

    let mut cursor = start_cursor;
    let mut r_name = RelName("default_relation_name".into());
    let mut args = vec![];
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
                match read_list(lexograms, i, false, debug_margin.clone() + "   ")? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "querring relation".into(),
                            failed_because: "specting list".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                    Ok((v, jump_to)) => {
                        cursor = jump_to;
                        args = v;
                        state = SpectingQuery;
                    }
                }
            }
            (Query, SpectingQuery) => return Ok(Ok((r_name, args, i + 1))),
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "querring relation".into(),
                    failed_because: format!("pattern missmatch on {:?} state", state).into(),
                    parent_failure: None,
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "querring relation".into(),
        failed_because: "file ended".into(),
        parent_failure: None,
    }))
}

fn read_intensional(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
) -> Result<Result<(Line, usize), FailureExplanation>, ParserError> {
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
                match read_querring_relation(lexograms, i, debug_margin.clone() + "   ")? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "intensional".into(),
                            failed_because: "specting relation".into(),
                            parent_failure: Some(vec![e]),
                        }))
                    }
                    Ok((r, args, jump_to)) => {
                        cursor = jump_to;
                        base_relation = Statement::Relation(r, args);
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
                            Line::TrueWhen(Box::new(base_relation), Box::new(cond)),
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
                match read_querring_relation(lexograms, i, debug_margin.clone() + "   ")? {
                    Ok((rel_name, args, jump_to)) => {
                        return Ok(Ok((Statement::Relation(rel_name, args), jump_to)))
                    }

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
) -> Result<Result<(Line, usize), FailureExplanation>, ParserError> {
    let a;
    let b;
    let c;
    match read_literal_relation(lexograms, start_cursor, debug_margin.clone() + "   ")? {
        Ok(ret) => return Ok(Ok(ret)),
        Err(e) => a = e,
    }
    match read_intensional(lexograms, start_cursor, debug_margin.clone() + "   ")? {
        Ok(ret) => return Ok(Ok(ret)),
        Err(e) => b = e,
    }
    match read_querring_relation(lexograms, start_cursor, debug_margin.clone() + "   ")? {
        Ok((rel_name, args, jump_to)) => return Ok(Ok((Line::Query(rel_name, args), jump_to))),
        Err(e) => c = e,
    }
    Ok(Err(FailureExplanation {
        lex_pos: start_cursor,
        if_it_was: "line".into(),
        failed_because: "wasnt neither an extensional nor an intensional statement".into(),
        parent_failure: Some(vec![a, b,c]),
    }))
}

pub fn parse(lexograms: Vec<lexer::Lexogram>) -> Result<Vec<Line>, ParserError> {
    let mut ret = vec![];
    let mut cursor = 0;

    for (i, _) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match read_line(&lexograms, i, "".into())? {
            Ok((statement, jump_to)) => {
                ret.push(statement);
                cursor = jump_to;
            }
            Err(e) => {
                return Err(ParserError::SyntaxError(e));
            }
        }
    }

    Ok(ret)
}
