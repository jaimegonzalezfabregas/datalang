use print_macros::*;

use super::error::{FailureExplanation, ParserError};
use crate::lexer::LexogramType::*;
use crate::parser::expresion_token::{read_expresion, Expresion, VarName};

use crate::lexer;

pub fn read_destructuring_array(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
) -> Result<Result<(VarName, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum ArrayParserStates {
        SpectingItemOrEnd,
        SpectingIdentifierAfterDotDotDot,
        SpectingItemOrDotDotDot,
        SpectingComaOrEnd,
        SpectingStart,
    }
    use ArrayParserStates::*;
    printparse!("read_destructuring_array at {}", start_cursor);

    let mut cursor = start_cursor;

    let mut ret = vec![];
    let mut state = SpectingStart;

    for (i, lex) in lexograms.iter().enumerate() {
        // println!("state: {:#?}",state);
        if cursor > i {
            continue;
        }
        match (lex.l_type.to_owned(), state) {
            (LeftBracket, SpectingStart) => {
                state = SpectingItemOrEnd;
            }
            (DotDotDot, SpectingItemOrDotDotDot) => {
                state = SpectingIdentifierAfterDotDotDot;
            }
            (Identifier(str), SpectingIdentifierAfterDotDotDot) => {
                ret.push(Expresion::Var(VarName::ExplodeArray(str)));
                state = SpectingItemOrEnd;
            }
            (Coma, SpectingComaOrEnd) => state = SpectingItemOrDotDotDot,
            (RightBracket, SpectingComaOrEnd | SpectingItemOrEnd) => {
                return Ok(Ok((VarName::DestructuredArray(ret), i + 1)));
            }
            (_, SpectingItemOrEnd | SpectingItemOrDotDotDot) => {
                match read_expresion(lexograms, i, false)? {
                    Err(err) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "destructuring_array".into(),
                            failed_because: "specting item".into(),
                            parent_failure: vec![err],
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
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: vec![],
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len() - 1,
        if_it_was: "destructuring_array".into(),
        failed_because: "file ended".into(),
        parent_failure: vec![],
    }))
}
