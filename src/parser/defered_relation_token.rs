use core::fmt;
use std::{hash, vec};

use print_macros::*;

use crate::engine::relation::truth::Truth;
use crate::engine::var_context::VarContext;
use crate::engine::RelId;
use crate::lexer::LexogramType::*;
use crate::parser::assumption_token::read_assumption;

use crate::{lexer, parser::list_token::read_list};

use super::assumption_token::Assumption;
use super::error::ParserError;
use super::{FailureExplanation, HasRelId};
use crate::parser::expresion_token::Expresion;

#[derive(Debug, Clone, Eq)]
pub struct DeferedRelation {
    pub negated: bool,
    pub assumptions: Vec<Assumption>,
    pub rel_name: String,
    pub args: Vec<Expresion>,
}

impl PartialEq for DeferedRelation {
    fn eq(&self, other: &Self) -> bool {
        self.negated == other.negated
            && self.assumptions.iter().eq(other.assumptions.iter())
            && self.rel_name == other.rel_name
            && self.args == other.args
    }
}

impl hash::Hash for DeferedRelation {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        format!("{self:?}").hash(state);
    }
}

impl HasRelId for DeferedRelation {
    fn get_rel_id(&self) -> RelId {
        return RelId {
            identifier: self.rel_name.clone(),
            column_count: self.args.len(),
        };
    }
}

impl DeferedRelation {
    pub fn to_truth(&self, context: &VarContext) -> Result<Truth, String> {
        let mut literal_vec = vec![];
        for exp in &self.args {
            literal_vec.push(exp.literalize(context)?)
        }
        Ok(Truth::from(&(literal_vec, self.get_rel_id())))
    }

    pub fn clone_and_apply(&self, context: &VarContext) -> DeferedRelation {
        let mut literalized_vec = vec![];
        for exp in &self.args {
            literalized_vec.push(match exp.literalize(context) {
                Ok(data) => Expresion::Literal(data),
                Err(_) => exp.clone(),
            })
        }
        DeferedRelation::from((&self.rel_name, literalized_vec))
    }
}

impl fmt::Display for DeferedRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = String::new();

        args += &"(";
        for (i, d) in self.args.iter().enumerate() {
            args += &format!("{d}");
            if i != self.args.len() - 1 {
                args += &",";
            }
        }
        args += &")";

        let mut assumptions = String::new();

        assumptions += &"(";
        for (i, d) in self.assumptions.iter().enumerate() {
            assumptions += &format!("{d}");
            if i != self.assumptions.len() - 1 {
                assumptions += &",";
            }
        }
        assumptions += &")";

        let asumption_prefix = if self.assumptions.len() == 0 {
            "".to_string()
        } else {
            format!("{{{assumptions}}}=>")
        };

        write!(
            f,
            "{}{}{}{args}",
            asumption_prefix,
            if self.negated { "!" } else { "" },
            self.rel_name
        )
    }
}

impl From<(&String, Vec<Expresion>)> for DeferedRelation {
    fn from(value: (&String, Vec<Expresion>)) -> Self {
        let (rel_name, args) = value;
        Self {
            negated: false,
            assumptions: vec![],
            rel_name: rel_name.to_owned(),
            args,
        }
    }
}

pub fn read_defered_relation(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    check_querry: bool,
) -> Result<Result<(DeferedRelation, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum RelationParserStates {
        SpectingStatementIdentifierOrNegation,
        SpectingStatementIdentifier,
        SpectingAssuming,
        SpectingStatementIdentifierOrassumptionOrNegation,
        Spectingassumption,
        SpectingComaBetweenassumptionsOrEndOfassumptions,
        SpectingStatementList,
        SpectingQuery,
    }
    use RelationParserStates::*;

    printdev!("read_defered_relation at {}", start_cursor);

    let mut cursor = start_cursor;
    let mut negated = false;
    let mut op_rel_name = None;
    let mut args = vec![];
    let mut assumptions = vec![];
    let mut state = SpectingStatementIdentifierOrassumptionOrNegation;

    for (i, lex) in lexograms.iter().enumerate() {
        if cursor > i {
            continue;
        }
        match (lex.l_type.to_owned(), state) {
            (
                OpNot,
                SpectingStatementIdentifierOrassumptionOrNegation
                | SpectingStatementIdentifierOrNegation,
            ) => {
                negated = true;
                state = SpectingStatementIdentifier;
            }
            (_, Spectingassumption) => {
                match read_assumption(lexograms, i)? {
                    Ok((assumption, jump_to)) => {
                        cursor = jump_to;
                        assumptions.push(assumption);
                    }
                    Err(err) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "defered relation".into(),
                            failed_because: format!("specting assumption").into(),
                            parent_failure: vec![err],
                        }))
                    }
                }
                state = SpectingComaBetweenassumptionsOrEndOfassumptions
            }
            (LeftKey, SpectingStatementIdentifierOrassumptionOrNegation) => {
                state = Spectingassumption;
            }
            (RightKey, SpectingComaBetweenassumptionsOrEndOfassumptions) => {
                state = SpectingAssuming;
            }
            (Coma, SpectingComaBetweenassumptionsOrEndOfassumptions) => {
                state = Spectingassumption;
            }
            (Assuming, SpectingAssuming) => state = SpectingStatementIdentifierOrNegation,
            (
                Identifier(str),
                SpectingStatementIdentifier
                | SpectingStatementIdentifierOrassumptionOrNegation
                | SpectingStatementIdentifierOrNegation,
            ) => {
                op_rel_name = Some(str);
                state = SpectingStatementList;
            }
            (_, SpectingStatementList) => match read_list(lexograms, i, false)? {
                Err(e) => {
                    return Ok(Err(FailureExplanation {
                        lex_pos: i,
                        if_it_was: "defered relation".into(),
                        failed_because: "specting list".into(),
                        parent_failure: (vec![e]),
                    }))
                }
                Ok((v, jump_to)) => {
                    cursor = jump_to;
                    args = v;
                    if check_querry {
                        state = SpectingQuery;
                    } else {
                        if let Some(rel_name) = op_rel_name {
                            return Ok(Ok((
                                DeferedRelation {
                                    negated,
                                    assumptions,
                                    rel_name,
                                    args,
                                },
                                jump_to,
                            )));
                        } else {
                            unreachable!()
                        }
                    }
                }
            },
            (Query, SpectingQuery) => {
                if let Some(rel_name) = op_rel_name {
                    return Ok(Ok((
                        DeferedRelation {
                            negated,
                            assumptions,
                            rel_name,
                            args,
                        },
                        i + 1,
                    )));
                } else {
                    unreachable!()
                }
            }
            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "defered relation".into(),
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: vec![],
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len() - 1,
        if_it_was: "defered relation".into(),
        failed_because: "file ended".into(),
        parent_failure: vec![],
    }))
}
