use std::collections::HashMap;

mod conditional_truth;
mod truth;

use crate::parser::{
    conditional_reader::Conditional, data_reader::Data, defered_relation_reader::DeferedRelation,
    expresion_reader::Expresion, inmediate_relation_reader::InmediateRelation, Relation,
};

#[derive(Debug, Clone)]
enum Command {
    IsTrueThat(Truth),
    IsTrueWhen(ConditionalTruth),
}
use Command::*;

use self::{conditional_truth::ConditionalTruth, truth::Truth};

use super::{Engine, RelId};

#[derive(Debug, Clone)]
pub struct Table {
    rel_id: RelId,
    history: Vec<Command>,
}

impl Table {
    fn check_relation<T: Relation>(&self, rule: &&T) -> Result<(), String> {
        if self.rel_id == rule.get_rel_id() {
            Ok(())
        } else {
            Err("Cant process a row for a diferent table".into())
        }
    }

    pub fn new(rel_id: &RelId) -> Self {
        Self {
            history: vec![],
            rel_id: rel_id.to_owned(),
        }
    }

    pub fn add_rule(&mut self, rule: InmediateRelation) -> Result<(), String> {
        self.check_relation(&&rule)?;

        match rule.negated {
            true => self.history.push(IsTrueThat(Truth::from(rule))),
            false => {
                let what_we_want_to_remove = &rule.args.to_owned();
                self.history = self
                    .history
                    .iter()
                    .filter(|command| match command {
                        IsTrueThat(truth) => truth.afirms(what_we_want_to_remove),
                        IsTrueWhen(_) => true,
                    })
                    .map(|e| e.to_owned())
                    .collect()
            } // TODO borrar es mortalmente costoso para acelerar las consultas
        };
        Ok(())
    }

    pub(crate) fn add_conditional(&mut self, cond: Conditional) -> Result<(), String> {
        self.check_relation(&&cond)?;

        self.history.push(IsTrueWhen(ConditionalTruth::from(cond)));
        Ok(())
    }

    pub fn get_all_contents(
        self: &Table,
        max_depth: usize,
        caller_depth_map: Option<HashMap<RelId, usize>>,
        engine: &Engine,
    ) -> Result<Vec<Vec<Data>>, String> {
        let mut depth_map = caller_depth_map.unwrap_or(HashMap::new()).clone();
        let go_deeper = if let Some(depth_count) = depth_map.get_mut(&self.rel_id) {
            *depth_count += 1;
            depth_count.to_owned() < max_depth
        } else {
            depth_map.insert(self.rel_id.to_owned(), 0);
            0 < max_depth
        };
        let mut ret = vec![];

        for command in &self.history {
            match (command, go_deeper) {
                (IsTrueThat(truth), _) => ret.push(truth.get_data().clone()),
                (Command::IsTrueWhen(conditional), true) => {
                    ret.extend(conditional.get_data(engine, &depth_map))
                }
                _ => (),
            }
        }

        Ok(ret)
    }

    pub fn get_contents(
        self: &Table,
        filter: &DeferedRelation,
        engine: &Engine,
    ) -> Result<Vec<Vec<Data>>, String> {
        self.check_relation(&filter)?;

        let all_truths = self.get_all_contents(10, None, engine)?;

        let mut matched_truths = vec![];
        for truth in all_truths {
            let mut discard = false;
            let mut context: HashMap<String, Data> = HashMap::new();

            for check in truth.iter().zip(filter.to_owned().args) {
                discard = match check {
                    (truth_data, filter_expresion @ Expresion::Arithmetic(_, _, _)) => {
                        let solution = filter_expresion.solve(&truth_data, &context);
                        match solution {
                            Ok(Some(new_context)) => {
                                context = new_context;
                                true
                            }
                            Ok(None) => true,
                            Err(_) => false,
                        }
                    }
                    _ => false,
                };
                if discard {
                    break;
                }
            }

            if !discard {
                matched_truths.push(truth.to_owned());
            }
        }

        Ok(matched_truths)
    }
}
