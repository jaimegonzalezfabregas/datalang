use crate::parser::{
    conditional_reader::Conditional,
    data_reader::Data,
    expresion_reader::{Expresion, VarName},
    inmediate_relation_reader::InmediateRelation,
    statement_reader::Statement,
    var_literal_reader::VarLiteral,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
enum Command {
    IsTrueThat(Vec<VarLiteral>),
    IsFalseThat(Vec<VarLiteral>),
    IsTrueWhen(Conditional),
}
use Command::*;

use super::RelId;

#[derive(Debug, Clone)]
pub struct Table {
    width: usize,
    history: Vec<Command>,
}

impl Table {
    pub fn new(rel_id: &RelId) -> Self {
        Self {
            width: rel_id.column_count,
            history: vec![],
        }
    }

    pub fn add_rule(self: &mut Table, rule: InmediateRelation) -> Result<(), String> {
        if self.width != rule.get_rel_id().column_count {
            Err("Cant add to a table a row with mismatching number of columns".into())
        } else {
            self.history.push(match rule.negated {
                true => IsTrueThat(rule.args),
                false => IsFalseThat(rule.args),
            });
            Ok(())
        }
    }

    pub(crate) fn add_conditional(&self, cond: Conditional) -> Result<(), String> {
        if self.width != cond.get_rel_id().column_count {
            Err("Cant add to a table a row with mismatching number of columns".into())
        } else {
            self.history.push(IsTrueWhen(cond));
            Ok(())
        }
    }

    pub fn contains_superset_of(self: &Table, sub_set: &Vec<VarLiteral>) -> Result<bool, String> {
        if sub_set.len() != self.width {
            Err("Cant compare a row with mismatching number of columns".into())
        } else {
            for command in self.history.iter().rev() {
                let (inner, ret) = match command {
                    IsFalseThat(inner) => (inner, false),
                    IsTrueThat(inner) => (inner, true),
                    IsTrueWhen(_) => todo!(),
                    IsFalseWhen(_) => todo!(),
                };

                let mut c = 0;
                for (a, b) in sub_set.iter().zip(inner) {
                    if b.contains_set(a) {
                        c += 1;
                    }
                }
                if c == sub_set.len() {
                    return Ok(ret);
                }
            }
            Ok(false)
        }
    }

    fn universe_of_nth_column(self: &Table, n: usize) -> Result<VarLiteral, String> {
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
                        ret = VarLiteral::EmptySet;
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
                    VarLiteral::EmptySet => (),
                },
                IsTrueWhen(cond) => match cond.relation.args[n] {
                    Expresion::Literal(v) => match v {
                        VarLiteral::FullSet => {
                            ret = VarLiteral::EmptySet;
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
                        VarLiteral::EmptySet => (),
                    },
                    Expresion::Var(VarName::Direct(str)) => return universe_of_engine(),
                    _ => panic!("unespected item at conditional"),
                },
            };
        }
        Ok(ret)
    }

    pub fn get_contents(
        self: &Table,
        constraints: &Vec<Expresion>,
        backtrack_context: &HashMap<String, Data>,
    ) -> Result<Vec<Vec<Data>>, String> {
        let first_non_singleton = constraints.iter().position(|exp| match exp.literalize() {
            Ok(l) => match l {
                VarLiteral::FullSet => true,
                VarLiteral::Set(s) => s.len() != 1,
                _ => false,
            },
            Err(_) => true,
        });

        if let Some(backtrack_pos) = first_non_singleton {
            let column_universe = self.universe_of_nth_column(backtrack_pos)?;

            let backtrack_universe = match column_universe {
                VarLiteral::EmptySet => HashSet::new(),
                VarLiteral::FullSet => self.universe_of_table()?,
                VarLiteral::Set(set) => set,
                VarLiteral::AntiSet(anti_set) => self
                    .universe_of_table()?
                    .difference(&anti_set)
                    .map(|e| e.to_owned())
                    .collect(),
            };

            match &constraints[backtrack_pos] {
                var @ Expresion::Var(VarName::Direct(_)) => {
                    let mut ret = vec![];
                    for value in backtrack_universe {
                        let new_constraints =
                            vector_find_replace(&constraints, var, &Expresion::singleton(&value));

                        let partial_results = self.get_contents(&new_constraints)?;

                        ret = ret
                            .iter()
                            .chain(partial_results.iter())
                            .map(|e| e.clone())
                            .collect()
                    }
                    Ok(ret)
                }
                Expresion::Literal(VarLiteral::Set(constraint_values)) => {
                    let mut ret = vec![];

                    for value in constraint_values {
                        if backtrack_universe.contains(value) {
                            let mut new_constraints = constraints.clone();
                            new_constraints[backtrack_pos] = Expresion::singleton(value);

                            let partial_results = self.get_contents(&new_constraints)?;

                            ret = ret
                                .iter()
                                .chain(partial_results.iter())
                                .map(|e| e.clone())
                                .collect()
                        }
                    }

                    Ok(ret)
                }
                Expresion::Literal(VarLiteral::FullSet) => {
                    let mut ret = vec![];

                    for value in backtrack_universe {
                        let mut new_constraints = constraints.clone();
                        new_constraints[backtrack_pos] = Expresion::singleton(&value);

                        let partial_results = self.get_contents(&new_constraints)?;

                        ret = ret
                            .iter()
                            .chain(partial_results.iter())
                            .map(|e| e.clone())
                            .collect()
                    }

                    Ok(ret)
                }
                _ => Err(format!(
                    "unespected_expresion at argument at pos {backtrack_pos}: {:?}",
                    constraints[backtrack_pos]
                )),
            }
        } else {
            let mut var_literal_result = vec![];

            for exp in constraints {
                var_literal_result.push(match exp {
                    Expresion::Literal(VarLiteral::FullSet) => VarLiteral::FullSet,
                    _ => VarLiteral::singleton(&exp.literalize()?.get_element_if_singleton()?),
                });
            }

            Ok(if self.contains_superset_of(&var_literal_result)? {
                let mut ret = vec![];
                for data in var_literal_result {
                    ret.push(data.get_element_if_singleton()?);
                }

                vec![ret]
            } else {
                vec![]
            })
        }
    }

    fn universe_of_table(&self) -> Result<HashSet<Data>, String> {
        let mut ret = HashSet::new();
        for comm in self.history.iter() {
            let vec = match comm {
                IsTrueThat(e) => e,
                IsFalseThat(e) => e,
                IsTrueWhen(_) => todo!(),
                IsFalseWhen(_) => todo!(),
            };

            let mut values = HashSet::new();

            vec.iter().for_each(|e| match e {
                VarLiteral::Set(s) => values.extend(s.iter().map(|e| e.to_owned())),
                _ => (),
            });

            ret.extend(values);
        }

        Ok(ret)
    }
}

fn vector_find_replace<T: 'static>(v: &Vec<T>, find: &T, replace: &T) -> Vec<T>
where
    T: PartialEq<T>,
    T: Clone,
{
    v.iter()
        .map(|original_value| {
            if original_value.clone() == find.clone() {
                replace.clone()
            } else {
                original_value.clone()
            }
        })
        .collect::<Vec<T>>()
}
