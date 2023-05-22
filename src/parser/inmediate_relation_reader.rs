use crate::{
    engine::RelId,
    lexer::{self, LexogramType::*},
    parser::list_reader::read_list,
};

use super::{
    data_reader::Data,
    error::{FailureExplanation, ParserError},
    Relation,
};

#[derive(Debug, Clone, PartialEq)]
pub struct InmediateRelation {
    pub negated: bool,
    pub rel_name: String,
    pub args: Vec<Data>,
}

impl Relation for InmediateRelation {
    fn get_rel_id(&self) -> RelId {
        return RelId {
            identifier: self.rel_name.clone(),
            column_count: self.args.len(),
        };
    }
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
        match (lex.l_type.to_owned(), state) {
            (OpNot, SpectingStatementIdentifierOrNegation) => {
                negated = true;
                state = SpectingStatementIdentifier
            }
            (
                Identifier(str),
                SpectingStatementIdentifier | SpectingStatementIdentifierOrNegation,
            ) => {
                op_rel_name = Some(str);
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
                            literal_vec.push(exp.literalize(None)?);
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
                    _ => unreachable!(),
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
