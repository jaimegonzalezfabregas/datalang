use regex::Regex;
use std::collections::HashMap;
use std::io;

use crate::utils::*;

#[derive(Debug, Clone)]
pub enum LexogramType {
    RightParenthesis,
    LeftParenthesis,
    RightBracket,
    LeftBracket,
    RightKey,
    LeftKey,
    DotDotDot,
    Coma,
    Identifier(String),
    Number(f64),
    Word(String),
    OpLT,
    OpLTE,
    OpGT,
    OpGTE,
    OpMul,
    OpDiv,
    OpAdd,
    OpSub,
    TrueWhen,
    OpEq,
    OpNot,
    OpAnd,
    OpOr,
    Assuming,
    CharNewLine,
    CharCarriageReturn,
    WhiteSpace,
    CharEq,
    CharColon,
    Any,
    Query,
}
#[derive(Debug, Clone)]
pub struct Lexogram {
    pub pos_s: usize,
    pub pos_f: usize,
    pub l_type: LexogramType,
}

#[derive(Debug)]
pub struct LexerError {
    pos_s: usize,
    pos_f: usize,
    msg: LexerErrorMsg,
}

impl LexerError {
    pub fn print(&self, original_string: &String) -> String {
        let ret = format!(
            "Lexer error breakdown: \n\"{:?}\" at: {}",
            self.msg,
            print_hilighted(original_string, self.pos_s, self.pos_f, "".into())
        );

        ret
    }
}

#[derive(Debug)]
pub enum LexerErrorMsg {
    IO(io::Error),
    Parse(std::num::ParseFloatError),
    Custom(String),
}

impl From<io::Error> for LexerErrorMsg {
    fn from(e: io::Error) -> Self {
        LexerErrorMsg::IO(e)
    }
}

impl From<std::num::ParseFloatError> for LexerErrorMsg {
    fn from(e: std::num::ParseFloatError) -> Self {
        LexerErrorMsg::Parse(e)
    }
}

fn parse(w: String) -> Result<Option<LexogramType>, LexerErrorMsg> {
    let re = Regex::new(r"^[+-]?([0-9]*[.])?[0-9]+$").unwrap();
    if w.len() == 0 {
        Ok(None)
    } else {
        if re.is_match(&w) {
            Ok(Some(LexogramType::Number(w.parse::<f64>()?)))
        } else {
            Ok(Some(LexogramType::Identifier(w)))
        }
    }
}

fn check_tail(pos_s: usize, tail: &str) -> Result<Option<Vec<Lexogram>>, LexerErrorMsg> {
    let reserved_lexograms = HashMap::from([
        (":", LexogramType::CharColon),
        (")", LexogramType::RightParenthesis),
        ("(", LexogramType::LeftParenthesis),
        ("]", LexogramType::RightBracket),
        ("[", LexogramType::LeftBracket),
        ("}", LexogramType::RightKey),
        ("{", LexogramType::LeftKey),
        ("...", LexogramType::DotDotDot),
        (",", LexogramType::Coma),
        ("<", LexogramType::OpLT),
        ("<=", LexogramType::OpLTE),
        (">", LexogramType::OpGT),
        (">=", LexogramType::OpGTE),
        ("*", LexogramType::OpMul),
        ("/", LexogramType::OpDiv),
        ("+", LexogramType::OpAdd),
        ("-", LexogramType::OpSub),
        ("=", LexogramType::CharEq),
        ("\r", LexogramType::CharCarriageReturn),
        ("\n", LexogramType::CharNewLine),
        (" ", LexogramType::WhiteSpace),
        ("!", LexogramType::OpNot),
        ("&&", LexogramType::OpAnd),
        ("||", LexogramType::OpOr),
        ("_", LexogramType::Any),
        ("?", LexogramType::Query),
    ]);

    let mut ret: Vec<Lexogram> = vec![];
    for (chars, token) in &reserved_lexograms {
        if tail.ends_with(chars) {
            let unparsed_size = tail.len() - chars.len();
            let finished_word = String::from(&tail[..unparsed_size]);

            match parse(finished_word) {
                Ok(Some(t)) => ret.push(Lexogram {
                    pos_s: pos_s,
                    pos_f: pos_s + unparsed_size,
                    l_type: t,
                }),
                Ok(None) => (),
                Err(e) => return Err(e),
            }

            ret.push(Lexogram {
                pos_s: pos_s + unparsed_size,
                pos_f: pos_s + tail.len(),
                l_type: token.clone(),
            });
            return Ok(Some(ret));
        }
    }
    Ok(None)
}

pub fn lex(str: &String) -> Result<Vec<Lexogram>, LexerError> {
    let simple = simple_lexogram_analisis(str)?;
    compound_lexogram_analisis(simple)
}

fn compound_lexogram_analisis(simple: Vec<Lexogram>) -> Result<Vec<Lexogram>, LexerError> {
    let mut ret = vec![];

    let mut queue = vec![];
    for l in simple {
        queue.push(l);

        let mut repeat_scan = true;

        while repeat_scan {
            repeat_scan = false;

            match &queue[..] {
                [Lexogram {
                    pos_f: _,
                    pos_s: _,
                    l_type: LexogramType::CharCarriageReturn,
                }]
                | [Lexogram {
                    pos_f: _,
                    pos_s: _,
                    l_type: LexogramType::CharNewLine,
                }]
                | [Lexogram {
                    pos_f: _,
                    pos_s: _,
                    l_type: LexogramType::WhiteSpace,
                }] => queue = vec![],

                [Lexogram {
                    pos_f: _,
                    pos_s: _,
                    l_type: LexogramType::CharEq,
                }] => (),

                [Lexogram {
                    pos_f: _,
                    pos_s,
                    l_type: LexogramType::CharEq,
                }, Lexogram {
                    pos_f,
                    pos_s: _,
                    l_type: LexogramType::OpGT,
                }] => {
                    ret.push(Lexogram {
                        pos_f: *pos_f,
                        pos_s: *pos_s,
                        l_type: LexogramType::Assuming,
                    });
                    queue = vec![];
                }

                [Lexogram {
                    pos_f,
                    pos_s,
                    l_type: LexogramType::CharEq,
                }, next] => {
                    ret.push(Lexogram {
                        pos_f: *pos_f,
                        pos_s: *pos_s,
                        l_type: LexogramType::OpEq,
                    });
                    repeat_scan = true;
                    queue = vec![next.clone()];
                }

                [Lexogram {
                    pos_f: _,
                    pos_s: _,
                    l_type: LexogramType::CharColon,
                }] => (),

                [Lexogram {
                    pos_f,
                    pos_s: _,
                    l_type: LexogramType::CharColon,
                }, Lexogram {
                    pos_f: _,
                    pos_s,
                    l_type: LexogramType::OpSub,
                }] => {
                    ret.push(Lexogram {
                        pos_f: *pos_f,
                        pos_s: *pos_s,
                        l_type: LexogramType::TrueWhen,
                    });
                    repeat_scan = true;
                    queue = vec![];
                }

                [Lexogram {
                    pos_f,
                    pos_s,
                    l_type: LexogramType::CharColon,
                }, _] => {
                    return Err(LexerError {
                        pos_s: *pos_s,
                        pos_f: *pos_f,
                        msg: LexerErrorMsg::Custom(
                            "compound lexogram analisis found a stray colon".into(),
                        ),
                    })
                }

                [any_lex] => {
                    ret.push(any_lex.clone());
                    queue = vec![];
                }

                [] => (),

                [first, .., last] => {
                    return Err(LexerError {
                        pos_s: first.pos_s,
                        pos_f: last.pos_f,
                        msg: LexerErrorMsg::Custom("compound lexogram analisis got stuck".into()),
                    })
                }
            }
        }
    }

    Ok(ret)
}

fn simple_lexogram_analisis(str: &String) -> Result<Vec<Lexogram>, LexerError> {
    let mut ret: Vec<Lexogram> = vec![];
    let mut tail = String::new();

    let mut c: u8 = 0;
    let mut last_tail_reset = 0;

    let mut char_i = 0;
    let mut repeat = true;

    match str.bytes().nth(char_i) {
        Some(char) => c = char,
        None => repeat = false,
    }

    while repeat {
        char_i += 1;

        if c == 34 /* " */ && tail.len() == 0 {
            let mut inside_a_string = true;
            let mut scaping = false;
            while inside_a_string {
                match (str.bytes().nth(char_i), scaping) {
                    (None, _) => {
                        return Err(LexerError {
                            pos_s: last_tail_reset,
                            pos_f: char_i,
                            msg: LexerErrorMsg::Custom("Specting matching \" found EOF".into()),
                        })
                    }
                    (Some(34 /* " */), false) => {
                        char_i += 1;
                        inside_a_string = false
                    }
                    (Some(92 /* \ */), false) => {
                        char_i += 1;
                        scaping = true
                    }
                    (Some(c), _) => {
                        char_i += 1;
                        tail.push(c as char);
                        scaping = false;
                    }
                }
            }

            ret.push(Lexogram {
                pos_s: last_tail_reset,
                pos_f: char_i,
                l_type: LexogramType::Word(tail),
            });
            tail = String::new();
        } else {
            tail.push(c as char);
        }

        match check_tail(last_tail_reset, tail.as_str()) {
            Ok(Some(mut lexograms)) => {
                ret.append(&mut lexograms);
                tail = String::new();
                last_tail_reset = char_i;
            }
            Ok(None) => (),
            Err(e) => {
                return Err(LexerError {
                    pos_s: last_tail_reset,
                    pos_f: char_i,
                    msg: e,
                })
            }
        }

        match str.bytes().nth(char_i) {
            Some(char) => c = char,
            None => repeat = false,
        }
    }

    match parse(tail.clone()) {
        Ok(Some(token_t)) => {
            ret.push(Lexogram {
                pos_s: last_tail_reset,
                pos_f: char_i,
                l_type: token_t,
            });
            tail = String::new();
            last_tail_reset = char_i;
        }
        Ok(None) => (),
        Err(e) => {
            return Err(LexerError {
                pos_s: last_tail_reset,
                pos_f: char_i,
                msg: e,
            })
        }
    }

    if tail.len() == 0 {
        Ok(ret)
    } else {
        Err(LexerError {
            pos_s: last_tail_reset,
            pos_f: char_i,
            msg: LexerErrorMsg::Custom("unprocesable ending chars".into()),
        })
    }
}
