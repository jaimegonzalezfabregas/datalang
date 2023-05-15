use std::collections::HashSet;

use crate::syntax::{Expresion, VarLiteral, VarName};

#[derive(Debug, Clone)]
enum Command {
    IsTrueThat(Vec<VarLiteral>),
    IsFalseThat(Vec<VarLiteral>),
}
use Command::*;

#[derive(Debug, Clone)]
pub struct Table {
    width: usize,
    history: Vec<Command>,
}

impl Table {
    pub fn new(width: &usize) -> Self {
        Self {
            width: *width,
            history: vec![],
        }
    }

    pub fn add(self: &mut Table, row: Vec<VarLiteral>) -> Result<(), String> {
        if row.len() != self.width {
            Err("Cant add to a table a row with mismatching number of columns".into())
        } else {
            self.history.push(IsTrueThat(row));
            Ok(())
        }
    }

    pub fn remove(self: &mut Table, row: Vec<VarLiteral>) -> Result<(), String> {
        if row.len() != self.width {
            Err("Cant remove to a table a row with mismatching number of columns".into())
        } else {
            self.history.push(IsFalseThat(row));
            Ok(())
        }
    }

    pub fn contains_eq(self: &Table, row: Vec<VarLiteral>) -> Result<bool, String> {
        if row.len() != self.width {
            Err("Cant compare a row with mismatching number of columns".into())
        } else {
            for command in self.history.iter().rev() {
                let (inner, ret) = match command {
                    IsFalseThat(inner) => (inner, false),
                    IsTrueThat(inner) => (inner, true),
                };

                let mut c = 0;
                for (a, b) in row.iter().zip(inner) {
                    if a.set_eq(b) {
                        c += 1;
                    }
                }
                if c == row.len() {
                    return Ok(ret);
                }
            }
            Ok(false)
        }
    }

    pub fn contains_subset_of(self: &Table, superSet: Vec<VarLiteral>) -> Result<bool, String> {
        if superSet.len() != self.width {
            Err("Cant compare a row with mismatching number of columns".into())
        } else {
            for command in self.history.iter().rev() {
                let (inner, ret) = match command {
                    IsFalseThat(inner) => (inner, false),
                    IsTrueThat(inner) => (inner, true),
                };

                let mut c = 0;
                for (a, b) in superSet.iter().zip(inner) {
                    if a.contains_set(b) {
                        c += 1;
                    }
                }
                if c == superSet.len() {
                    return Ok(ret);
                }
            }
            Ok(false)
        }
    }

    pub fn contains_superset_of(self: &Table, subSet: Vec<VarLiteral>) -> Result<bool, String> {
        if subSet.len() != self.width {
            Err("Cant compare a row with mismatching number of columns".into())
        } else {
            for command in self.history.iter().rev() {
                let (inner, ret) = match command {
                    IsFalseThat(inner) => (inner, false),
                    IsTrueThat(inner) => (inner, true),
                };

                let mut c = 0;
                for (a, b) in subSet.iter().zip(inner) {
                    if b.contains_set(a) {
                        c += 1;
                    }
                }
                if c == subSet.len() {
                    return Ok(ret);
                }
            }
            Ok(false)
        }
    }

    fn set_of_nth_column(self: &Table, n: usize) -> Result<VarLiteral, String> {
        let mut ret = VarLiteral::EmptySet;
        for command in &self.history {
            match command {
                IsTrueThat(v) => match &v[n] {
                    VarLiteral::FullSet => {
                        ret = VarLiteral::FullSet;
                    }
                    VarLiteral::Set(vec) => {
                        for elm in vec {
                            ret.add(elm.to_owned())?;
                        }
                    }
                    VarLiteral::AntiSet(vec) => {
                        for elm in vec {
                            ret.remove(elm.to_owned())?;
                        }
                    }
                    VarLiteral::EmptySet => ret = VarLiteral::EmptySet,
                },
                IsFalseThat(v) => match &v[n] {
                    VarLiteral::FullSet => {
                        ret = VarLiteral::FullSet;
                    }
                    VarLiteral::Set(vec) => {
                        for elm in vec {
                            ret.remove(elm.to_owned())?;
                        }
                    }
                    VarLiteral::AntiSet(vec) => {
                        for elm in vec {
                            ret.add(elm.to_owned())?;
                        }
                    }
                    VarLiteral::EmptySet => ret = VarLiteral::EmptySet,
                },
            };
        }
        Ok(ret)
    }

    pub fn get_contents(
        self: &Table,
        constraitns: Vec<Expresion>,
    ) -> Result<Vec<Vec<VarLiteral>>, String> {
        let mut only_literals = true;
        let mut literalized_constrains = vec![];
        let mut first_non_literal = 0;

        for (i, exp) in constraitns.iter().enumerate() {
            match exp.literalize() {
                Ok(l) => literalized_constrains.push(l),
                Err(_) => {
                    first_non_literal = i;
                    only_literals = false;
                    break;
                }
            }
        }
        if only_literals {
            return Ok(
                if self.contains_superset_of(literalized_constrains.clone())? {
                    vec![literalized_constrains]
                } else {
                    vec![]
                },
            );
        }

        let column_universe = self.set_of_nth_column(first_non_literal)?;

        match column_universe {
            VarLiteral::EmptySet => return Ok(vec![]),
            VarLiteral::FullSet => {
                let new_constraints = constraitns
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        if i == first_non_literal {
                            Expresion::Literal(VarLiteral::FullSet)
                        } else {
                            e.clone()
                        }
                    })
                    .collect();
                return self.get_contents(new_constraints);
            }
            VarLiteral::Set(posible_values) => match &constraitns[first_non_literal] {
                Expresion::Var(VarName::Direct(var_name)) => {
                    let mut ret = vec![];
                    for value in posible_values {
                        let new_constraints = constraitns
                            .iter()
                            .map(|e| {
                                if let Expresion::Var(VarName::Direct(var_name_of_e)) = e {
                                    if var_name_of_e.to_owned() == var_name.to_owned() {
                                        Expresion::Literal(VarLiteral::Set(HashSet::from([
                                            value.to_owned()
                                        ])))
                                    } else {
                                        e.clone()
                                    }
                                } else {
                                    e.clone()
                                }
                            })
                            .collect();

                        let partial_results = self.get_contents(new_constraints)?;

                        ret = ret
                            .iter()
                            .chain(partial_results.iter())
                            .map(|e| e.clone())
                            .collect()
                    }
                    Ok(ret)
                }
                _ => Err(format!(
                    "unespected_expresion at argument at pos {first_non_literal}: {:#?}",
                    constraitns[first_non_literal]
                )),
            },
            VarLiteral::AntiSet(_) => Err(format!(
                "cant iterate over inifinite set at column {first_non_literal}"
            )),
        }
    }
}
