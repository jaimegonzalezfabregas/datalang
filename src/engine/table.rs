use std::collections::HashMap;

use crate::parser::{
    conditional_reader::Conditional,
    data_reader::Data,
    expresion_reader::{Expresion, VarName},
    inmediate_relation_reader::InmediateRelation,
};

#[derive(Debug, Clone)]
enum Command {
    IsTrueThat(Vec<Data>),
    IsFalseThat(Vec<Data>),
    Conditional(Conditional),
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

    pub fn add_rule(&mut self, rule: InmediateRelation) -> Result<(), String> {
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

    pub(crate) fn add_conditional(&mut self, cond: Conditional) -> Result<(), String> {
        if self.width != cond.get_rel_id().column_count {
            Err("Cant add to a table a row with mismatching number of columns".into())
        } else {
            self.history.push(Conditional(cond));
            Ok(())
        }
    }

    pub fn get_all_contents(self: &Table) -> Result<Vec<Vec<Data>>, String> {
        todo!()
    }

    pub fn get_contents(self: &Table, filter: Vec<Expresion>) -> Result<Vec<Vec<Data>>, String> {
        let all_truths = self.get_all_contents()?;
        let mut context: HashMap<String, Data> = HashMap::new();

        let mut matched_truths = vec![];
        for truth in all_truths {
            let mut discard = false;
            for check in truth.iter().zip(filter) {
                discard = match check {
                    (d, Expresion::Literal(f)) => f != d.to_owned(),
                    (d, Expresion::Var(VarName::Direct(name))) => match context.get(&name) {
                        Some(prev_val) => prev_val.to_owned() != d.to_owned(),
                        None => {
                            context.insert(name, d.to_owned());
                            false
                        }
                    },
                    _ => false,
                };
                if discard {
                    break;
                }
            }

            if !discard {
                matched_truths.push(truth);
            }
        }

        let mut expresion_matched_truths = vec![];
        for truth in all_truths {
            let mut discard = false;
            for check in truth.iter().zip(filter) {
                discard = match check {
                    (d, e @ Expresion::Arithmetic(_, _, _)) => {
                        (e.literalize(None)?) == d.to_owned()
                    }
                    _ => false,
                };
                if discard {
                    break;
                }
            }

            if !discard {
                expresion_matched_truths.push(truth);
            }
        }

        Ok(expresion_matched_truths)
    }
}
