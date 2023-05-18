use crate::{
    lexer::{self, LexogramType::*},
    parser::{common::RelName, list_reader::read_list},
};

use super::{
    error::{FailureExplanation, ParserError},
    var_literal_reader::VarLiteral,
};

#[derive(Debug, Clone, PartialEq)]
pub struct InmediateRelation {
    pub negated: bool,
    pub rel_name: RelName,
    pub args: Vec<VarLiteral>,
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
    let mut op_rel_name = None;
    let mut state = SpectingStatementIdentifierOrNegation;

    let mut negated = false;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state) {
            (OpNot, SpectingStatementIdentifierOrNegation) => {
                negated = true;
                state = SpectingStatementIdentifier
            }
            (
                Identifier(str),
                SpectingStatementIdentifier | SpectingStatementIdentifierOrNegation,
            ) => {
                op_rel_name = Some(RelName(str));
                state = SpectingStatementList;
            }
            (_, SpectingStatementList) => {
                return match (
                    read_list(
                        lexograms,
                        i,
                        true,
                        debug_margin.clone() + "   ",
                        debug_print,
                    )?,
                    op_rel_name,
                ) {
                    (Err(e), _) => Ok(Err(FailureExplanation {
                        lex_pos: i,
                        if_it_was: "literal relation".into(),
                        failed_because: "specting list".into(),
                        parent_failure: (vec![e]),
                    })),
                    (Ok((args, new_cursor)), Some(rel_name)) => {
                        let mut literal_vec = vec![];

                        for exp in args {
                            literal_vec.push(exp.literalize()?);
                        }

                        Ok(Ok((
                            InmediateRelation {
                                args: literal_vec,
                                negated,
                                rel_name,
                            },
                            new_cursor,
                        )))
                    }
                    _ => panic!("unreacheable state"),
                }
            }
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "literal relation".into(),
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: vec![],
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "literal relation".into(),
        failed_because: "file ended".into(),
        parent_failure: vec![],
    }))
}
