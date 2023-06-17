use conditional_compilation::*;

use crate::lexer::LexogramType::*;

use crate::{
    lexer,
    parser::{expresion_node::read_expresion, FailureExplanation},
};

use super::{error::ParserError, expresion_node::Expresion};

pub fn read_list(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    only_literals: bool,
) -> Result<Result<(Vec<Expresion>, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum ListParserStates {
        SpectingItem,
        SpectingComaOrClosingParenthesis,
        SpectingOpenParenthesis,
    }

    printparse!("read_list at {}", start_cursor);

    use ListParserStates::*;
    let mut cursor = start_cursor;

    let mut ret = vec![];
    let mut state = SpectingOpenParenthesis;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.to_owned(), state, only_literals) {
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
                match read_expresion(lexograms, i, only_literals)? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "list".into(),
                            failed_because: "Specting item".into(),
                            parent_failure: (vec![e]),
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
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: vec![],
                }));
            }
        }
    }
    return Ok(Err(FailureExplanation {
        lex_pos: lexograms.len() - 1,
        if_it_was: "list".into(),
        failed_because: "file ended".into(),
        parent_failure: vec![],
    }));
}
