use std::io::Empty;

use crate::lexer;
use lexer::LexogramType::*;
#[derive(PartialEq)]
struct RelName(String);
enum VarName {
    DestructuredList(Vec<Expresion>),
    String(String),
}
enum VarLiteral {
    Number(f64),
    String(String),
}
enum Statement {
    // resolvable to a bolean
    Hypothetical(Box<Statement>, Box<Statement>),
    And(Box<Statement>, Box<Statement>),
    Or(Box<Statement>, Box<Statement>),
    Eq(RelName, RelName),
    Not(Box<Statement>),
    Arithmetic(Expresion, Expresion, fn(Expresion, Expresion) -> bool),
    Extensional(RelName, Vec<Expresion>),
    Intensional(RelName, Vec<Expresion>, Box<Statement>),
    Empty,
}

enum Expresion {
    // resolvable to a value
    Arithmetic(
        Box<Expresion>,
        Box<Expresion>,
        fn(Expresion, Expresion) -> Expresion,
    ),
    Array(Box<Vec<Expresion>>),
    Literal(VarLiteral),
    Var(VarName),
}

enum ParserError {
    PrintfError(sprintf::PrintfError),
    Custom(String),
}

impl From<sprintf::PrintfError> for ParserError {
    fn from(e: sprintf::PrintfError) -> Self {
        Self::PrintfError(e)
    }
}

fn try_read_array(
    lexograms: &Vec<lexer::Lexogram>,
    cursor: usize,
) -> Result<(Option<Vec<Expresion>>, usize), ParserError> {
    todo!()
}

enum AttrListParserStates {
    SpectingAttrListItemOrEnd,
    SpectingAttrListItem,
    SpectingAttrListComaOrEnd,
    SpectingAttrListStart,
}

fn try_read_attr_list(
    lexograms: &Vec<lexer::Lexogram>,
    cursor: usize,
) -> Result<(Option<Vec<Expresion>>, usize), ParserError> {
    use AttrListParserStates::*;

    let mut i = 0;
    let mut ret = vec![];
    let mut state = SpectingAttrListStart;

    let jump_to = cursor;

    for lex in lexograms {
        i += 1;
        if jump_to > i {
            continue;
        }
        match (lex.l_type, state) {
            (LeftParenthesis, SpectingAttrListStart) => {
                state = SpectingAttrListItemOrEnd;
            }
            (RightParenthesis, SpectingAttrListItemOrEnd | SpectingAttrListComaOrEnd) => {
                return Ok((Some(ret), i))
            }
            (Coma, SpectingAttrListComaOrEnd) => return Ok((Some(ret), i)),
            (Word(str), SpectingAttrListItem | SpectingAttrListItemOrEnd) => {
                ret.push(Expresion::Literal(VarLiteral::String(str)));
                state = SpectingAttrListComaOrEnd;
            }
            (Number(n), SpectingAttrListItem | SpectingAttrListItemOrEnd) => {
                ret.push(Expresion::Literal(VarLiteral::Number(n)));
                state = SpectingAttrListComaOrEnd;
            }
            (LeftBracket, SpectingAttrListItem | SpectingAttrListItemOrEnd) => {
                match try_read_array(&lexograms, i) {
                    Ok((Some(ExpresionVec), jump)) => {
                        ret.push(Expresion::Array(Box::new(ExpresionVec)));
                        jump_to = jump;
                    }
                    Ok((None, jumpto)) => return Ok((None, cursor)),
                    Err(e) => return e,
                }
                state = SpectingAttrListComaOrEnd;
            }
            (_, _) => {
                return Ok((None, cursor));
            }
        }
    }
    return Ok((None, cursor));
}
enum ExtensionalParserStates {
    SpectingStatement,
    SpectingStatementOrTrueWhen,
}

fn try_read_extensional(
    lexograms: &Vec<lexer::Lexogram>,
    cursor: usize,
) -> Result<(Option<Statement>, usize), ParserError> {
    let ret = vec![];
    let mut wip_statement = Statement::Empty;

    let state = ExtensionalParserStates::SpectingStatement;

    for lex in lexograms {
        match (lex.l_type, state, wip_statement) {
            (Identifier(str), SpectingStatement | SpectingStatementOrTrueWhen, _) => {
                ret.push(wip_statement);
                wip_statement = Statement::Extensional(RelName(str), vec![]);
                state = ExtensionalParserStates::SpectingAttrList;
            }
            (TrueWhen, SpectingStatementOrTrueWhen, Statement::Extensional(rName, attrList)) => {
                wip_statement = Statement::Intensional(rName, attrList, Box::new(Statement::Empty));
            }
            (LeftParenthesis, SpectingAttrList, _) => {
                state = ExtensionalParserStates::SpectingAttrListItemOrEnd;
            }
            (RightParenthesis, SpectingAttrListItemOrEnd | SpectingAttrListComaOrEnd, _) => {
                state = ExtensionalParserStates::SpectingStatementOrTrueWhen;
            }
            (Word(str), SpectingAttrListItem | SpectingAttrListItemOrEnd, _) => {
                wip_statement = state = SpectingAttrListComaOrEnd;
            }
            (Number(str), SpectingAttrListItem | SpectingAttrListItemOrEnd, _) => {
                state = SpectingAttrListComaOrEnd;
            }
            (LeftBracket, SpectingAttrListItem | SpectingAttrListItemOrEnd, _) => {
                state = SpectingAttrListComaOrEnd;
            }
        }
    }
    Ok(ret)
}

pub fn parse(lexograms: Vec<lexer::Lexogram>) -> Result<Vec<Statement>, ParserError> {
    try_read_extensional(lexograms);
    todo!()
}
