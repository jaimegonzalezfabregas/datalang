use std::hash;

use super::error::ParserError;
use crate::lexer::{self, LexogramType::*};
use crate::parser::{error::FailureExplanation, expresion_reader::read_expresion};

#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    Number(f64),
    String(String),
    Array(Vec<Data>),
}

impl Eq for Data {}

impl hash::Hash for Data {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        match self {
            Data::Number(n) => {
                if n.is_finite() {
                    n.to_bits().hash(state)
                } else if n.is_infinite() {
                    f64::INFINITY.to_bits().hash(state)
                } else {
                    f64::NAN.to_bits().hash(state)
                }
            }
            Data::String(str) => str.hash(state),
            Data::Array(array) => array.hash(state),
        }
    }
}

impl Data {
    pub fn to_string(&self) -> String {
        match self {
            Data::Number(n) => format!("{n}").into(),
            Data::String(s) => format!("\"{s}\"").into(),
            Data::Array(arr) => {
                "[".to_string()
                    + &arr
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                    + &"]".to_string()
            }
        }
    }
}

pub fn read_data(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Data, usize), FailureExplanation>, ParserError> {
    if debug_print {
        println!("{}read_data at {}", debug_margin, start_cursor);
    }

    match lexograms[start_cursor].l_type.clone() {
        Number(n) => Ok(Ok((Data::Number(n), start_cursor + 1))),
        Word(n) => Ok(Ok((Data::String(n), start_cursor + 1))),
        LeftBracket => {
            match read_data_array(
                lexograms,
                start_cursor,
                debug_margin.clone() + "   ",
                debug_print,
            )? {
                Ok((ret, jump_to)) => Ok(Ok((Data::Array(ret), jump_to))),
                Err(explanation) => Ok(Err(FailureExplanation {
                    lex_pos: start_cursor,
                    if_it_was: "data".into(),
                    failed_because: "was not an array".into(),
                    parent_failure: (vec![explanation]),
                })),
            }
        }

        _ => Ok(Err(FailureExplanation {
            lex_pos: start_cursor,
            if_it_was: "data".into(),
            failed_because: "pattern missmatch trying to read item".into(),
            parent_failure: vec![],
        })),
    }
}

pub fn read_data_array(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(Vec<Data>, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum ArrayParserStates {
        SpectingItemOrEnd,
        SpectingItem,
        SpectingComaOrEnd,
        SpectingStart,
    }
    use ArrayParserStates::*;

    if debug_print {
        println!("{}read_data_array at {}", debug_margin, start_cursor);
    }

    let mut cursor = start_cursor;

    let mut ret = vec![];
    let mut state = SpectingStart;

    for (i, lex) in lexograms.iter().enumerate() {
        // println!("state: {:#?}",state);
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state) {
            (LeftBracket, SpectingStart) => {
                state = SpectingItemOrEnd;
            }

            (Coma, SpectingComaOrEnd) => state = SpectingItem,
            (RightBracket, SpectingComaOrEnd | SpectingItemOrEnd) => {
                println!("{debug_margin}end of data_array at {}", i + 1);
                return Ok(Ok((ret, i + 1)));
            }
            (_, SpectingItemOrEnd | SpectingItem) => {
                match read_expresion(
                    lexograms,
                    i,
                    true,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "data_array".into(),
                            failed_because: "specting item".into(),
                            parent_failure: vec![e],
                        }))
                    }
                    Ok((expresion, jump_to)) => {
                        ret.push(match expresion.literalize() {
                            Ok(data) => data,
                            Err(err) => {
                                return Ok(Err(FailureExplanation {
                                    lex_pos: i,
                                    if_it_was: "data_array".into(),
                                    failed_because: format!("unliteralizable expresion: {err}")
                                        .into(),
                                    parent_failure: vec![],
                                }))
                            }
                        });
                        cursor = jump_to;
                    }
                }

                state = SpectingComaOrEnd;
            }
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "data_array".into(),
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: vec![],
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "data_array".into(),
        failed_because: "file ended".into(),
        parent_failure: vec![],
    }))
}
