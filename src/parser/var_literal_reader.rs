use std::collections::HashSet;

use crate::lexer::LexogramType::*;
use crate::parser::data_reader::read_data;
use crate::{
    lexer,
    parser::{error::FailureExplanation, expresion_reader::read_expresion},
};

use super::data_reader::Data;
use super::error::ParserError;

#[derive(Debug, Clone, PartialEq)]
pub enum VarLiteral {
    EmptySet,
    FullSet,
    Set(HashSet<Data>),
    AntiSet(HashSet<Data>),
}

impl VarLiteral {
    pub fn add(self: &mut VarLiteral, d: Data) -> Result<(), String> {
        match self {
            VarLiteral::EmptySet => *self = VarLiteral::singleton(&d),
            VarLiteral::FullSet => (),
            VarLiteral::Set(v) => {
                v.insert(d);
            }
            VarLiteral::AntiSet(v) => {
                v.retain(|e| !d.eq(e));
            }
        }

        Ok(())
    }
    pub fn remove(self: &mut VarLiteral, d: Data) -> Result<(), String> {
        match self {
            VarLiteral::EmptySet => (),
            VarLiteral::FullSet => *self = VarLiteral::AntiSet(HashSet::from([d])),
            VarLiteral::AntiSet(v) => {
                v.insert(d);
            }
            VarLiteral::Set(v) => {
                v.retain(|e| !d.eq(e));
            }
        }

        Ok(())
    }

    pub fn singleton(value: &Data) -> VarLiteral {
        return VarLiteral::Set(HashSet::from([value.to_owned()]));
    }

    pub fn get_element_if_singleton(&self) -> Result<Data, String> {
        match self {
            VarLiteral::FullSet | VarLiteral::EmptySet | VarLiteral::AntiSet(_) => {
                Err("Not a singleton".into())
            }
            VarLiteral::Set(e) => {
                if e.len() == 1 {
                    return Ok(e.iter().take(1).collect::<Vec<&Data>>()[0].to_owned());
                } else {
                    Err("Not a singleton".into())
                }
            }
        }
    }

    pub fn set_eq(&self, other: &VarLiteral) -> bool {
        match (self, other) {
            (VarLiteral::FullSet, VarLiteral::FullSet) => true,
            (VarLiteral::EmptySet, VarLiteral::EmptySet) => true,
            (VarLiteral::Set(a), VarLiteral::Set(b)) => {
                for (a_it, b_it) in a.iter().zip(b) {
                    if !a_it.eq(b_it) {
                        return false;
                    }
                }
                true
            }
            (VarLiteral::AntiSet(a), VarLiteral::AntiSet(b)) => {
                for (a_it, b_it) in a.iter().zip(b) {
                    if !a_it.eq(b_it) {
                        return false;
                    }
                }
                true
            }

            (_, _) => false,
        }
    }

    pub fn contains_set(&self, contained_set: &VarLiteral) -> bool {
        match (contained_set, self) {
            (_, VarLiteral::FullSet) => true,
            (_, VarLiteral::EmptySet) => false,
            (VarLiteral::EmptySet, _) => true,
            (VarLiteral::FullSet, _) => false,

            (VarLiteral::Set(contained), VarLiteral::Set(container)) => {
                contained.is_subset(container)
            }

            (VarLiteral::AntiSet(_), VarLiteral::Set(_)) => false,
            (VarLiteral::AntiSet(not_in_contained), VarLiteral::AntiSet(not_in_container)) => {
                not_in_contained.is_superset(not_in_container)
            }

            (VarLiteral::Set(contained), VarLiteral::AntiSet(not_in_container)) => {
                not_in_container
                    .symmetric_difference(contained)
                    .map(|_| 0)
                    .collect::<Vec<i32>>()
                    .len()
                    == not_in_container
                        .union(contained)
                        .map(|_| 0)
                        .collect::<Vec<i32>>()
                        .len()
            }
        }
    }

    fn contains_element(&self, data: &Data) -> bool {
        match self {
            VarLiteral::FullSet => true,
            VarLiteral::EmptySet => false,
            VarLiteral::Set(set) => set.contains(data),
            VarLiteral::AntiSet(set) => !set.contains(data),
        }
    }
}

pub fn read_var_literal(
    lexograms: &Vec<lexer::Lexogram>,
    start_cursor: usize,
    debug_margin: String,
    debug_print: bool,
) -> Result<Result<(VarLiteral, usize), FailureExplanation>, ParserError> {
    #[derive(Debug, Clone, Copy)]
    enum ArrayParserStates {
        SpectingDataOrEnd,
        SpectingData,
        SpectingComaOrEnd,
        SpectingLTOrNegationOrAnyOrData,
        SpectingLT,
    }
    use ArrayParserStates::*;

    if debug_print {
        println!("{}read_var_literal at {}", debug_margin, start_cursor);
    }
    let mut cursor = start_cursor;

    let mut negated = false;

    let mut ret = HashSet::new();
    let mut state = SpectingLTOrNegationOrAnyOrData;

    for (i, lex) in lexograms.iter().enumerate() {
        // println!("state: {:#?}",state);
        if cursor > i {
            continue;
        }
        match (lex.l_type.clone(), state) {
            (OpNot, SpectingLTOrNegationOrAnyOrData) => {
                negated = true;
                state = SpectingLT
            }
            (OpLT, SpectingLT | SpectingLTOrNegationOrAnyOrData) => {
                state = SpectingDataOrEnd;
            }
            (Any, SpectingLTOrNegationOrAnyOrData) => return Ok(Ok((VarLiteral::FullSet, i + 1))),
            (Coma, SpectingComaOrEnd) => state = SpectingData,
            (OpGT, SpectingComaOrEnd | SpectingDataOrEnd) => {
                println!("{debug_margin}end of set at {}", i + 1);
                return if negated {
                    Ok(Ok((VarLiteral::AntiSet(ret), i + 1)))
                } else {
                    Ok(Ok((VarLiteral::Set(ret), i + 1)))
                };
            }
            (_, SpectingDataOrEnd | SpectingData) => {
                match read_data(lexograms, i, debug_margin.clone() + "   ", debug_print)? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "array".into(),
                            failed_because: "specting item".into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                    Ok((expresion, jump_to)) => {
                        ret.insert(expresion);
                        cursor = jump_to;
                    }
                }

                state = SpectingComaOrEnd;
            }
            (_, SpectingLTOrNegationOrAnyOrData) => {
                match read_data(lexograms, i, debug_margin.clone() + "   ", debug_print)? {
                    Err(e) => {
                        return Ok(Err(FailureExplanation {
                            lex_pos: i,
                            if_it_was: "array".into(),
                            failed_because: "specting item".into(),
                            parent_failure: (vec![e]),
                        }))
                    }
                    Ok((data, jump_to)) => return Ok(Ok((VarLiteral::singleton(&data), jump_to))),
                }
            }

            _ => {
                return Ok(Err(FailureExplanation {
                    lex_pos: i,
                    if_it_was: "array".into(),
                    failed_because: format!("pattern missmatch on {:#?} state", state).into(),
                    parent_failure: vec![],
                }))
            }
        }
    }
    Ok(Err(FailureExplanation {
        lex_pos: lexograms.len(),
        if_it_was: "array".into(),
        failed_because: "file ended".into(),
        parent_failure: vec![],
    }))
}
