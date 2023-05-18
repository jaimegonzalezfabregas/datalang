use crate::{
    lexer::{self, LexogramType::*},
    parser::list_reader::read_list,
    syntax::{Line, RelName},
};

use super::{
    error::{FailureExplanation, ParserError},
    var_literal_reader::VarLiteral,
};

#[derive(Debug, Clone, PartialEq)]
pub struct InmediateRelation {
    negated: bool,
    rel_name: RelName,
    args: Vec<VarLiteral>,
}

pub fn read_inmediate_relation(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(InmediateRelation, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum RelationParserStates {
        SpectingStatementIdentifierOrNegation,
        SpectingStatementIdentifier,
        SpectingStatementList,
    }
    use RelationParserStates::*;

    if debug_print {
        println!("{debug_margin}read_literal_relation at {start_cursor}");
    }
    let cursor = start_cursor;
    let mut r_name = RelName("default_relation_name".into());
    let mut state = SpectingStatementIdentifierOrNegation;

    let mut forget = false;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state) {
            (OpNot, SpectingStatementIdentifierOrNegation) => {
                forget = true;
                state = SpectingStatementIdentifier
            }
            (
                Identifier(str),
                SpectingStatementIdentifier | SpectingStatementIdentifierOrNegation,
            ) => {
                r_name = RelName(str);
                state = SpectingStatementList;
            }
            (_, SpectingStatementList) => {
                return match read_list(
                    lexograms,
                    i,
                    true,
                    debug_margin.clone() + "   ",
                    debug_print,
                )? {
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
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
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
