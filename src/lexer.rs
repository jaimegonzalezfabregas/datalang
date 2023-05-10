use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::slice;

#[derive(Debug, Clone)]
pub enum LexogramType {
    RightParenthesis,
    LeftParenthesis,
    RightBracket,
    LeftBracket,
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
        (")", LexogramType::RightParenthesis),
        ("(", LexogramType::LeftParenthesis),
        ("]", LexogramType::RightBracket),
        ("[", LexogramType::LeftBracket),
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
        (":-", LexogramType::TrueWhen),
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

pub fn lex(f: File) -> Result<Vec<Lexogram>, LexerError> {
    let simple = simple_lexogram_analisis(f)?;
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
                    pos_f,
                    pos_s: _,
                    l_type: LexogramType::CharEq,
                }, Lexogram {
                    pos_f: _,
                    pos_s,
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

fn read_next_char(f: &mut File) -> Option<char> {
    let mut c: u8 = 0;

    match f.read(slice::from_mut(&mut c)) {
        Ok(0) => None,
        Ok(_) => Some(c as char),
        Err(_) => None,
    }
}

fn simple_lexogram_analisis(mut f: File) -> Result<Vec<Lexogram>, LexerError> {
    let mut ret: Vec<Lexogram> = vec![];
    let mut tail = String::new();

    let mut c = '\0';
    let mut last_tail_reset = 0;

    let mut char_i = 0;
    let mut repeat = true;
    match read_next_char(&mut f) {
        Some(char) => c = char,
        None => repeat = false,
    }

    while repeat {
        char_i += 1;

        if c == '"' && tail.len() == 0 {
            let mut inside_a_string = true;
            let mut scaping = false;
            while inside_a_string {
                match (read_next_char(&mut f), scaping) {
                    (None, _) => {
                        return Err(LexerError {
                            pos_s: last_tail_reset,
                            pos_f: char_i,
                            msg: LexerErrorMsg::Custom("Specting matching \" found EOF".into()),
                        })
                    }
                    (Some('"'), false) => inside_a_string = false,
                    (Some('\\'), false) => scaping = true,
                    (Some(c), _) => {
                        char_i += 1;
                        tail.push(c);
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

        match read_next_char(&mut f) {
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
