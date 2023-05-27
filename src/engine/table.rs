use std::{collections::HashMap, fmt};

mod conditional_truth;
pub mod truth;

use crate::parser::{
    conditional_reader::Conditional, defered_relation_reader::DeferedRelation,
    inmediate_relation_reader::InmediateRelation, Relation,
};

#[derive(Debug, Clone)]
enum Command {
    IsTrueThat(Truth),
    IsTrueWhen(ConditionalTruth),
}
use Command::*;

use self::{conditional_truth::ConditionalTruth, truth::Truth};

use super::{var_context::VarContext, Engine, RelId};

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
            false => self.history.push(IsTrueThat(Truth::from(&rule))),
            true => {
                let what_we_want_to_remove = &rule.args.to_owned();
                self.history = self
                    .history
                    .iter()
                    .filter(|command| match command {
                        IsTrueThat(truth) => !truth.afirms(what_we_want_to_remove),
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
        caller_depth_map: &HashMap<RelId, usize>,
        engine: &Engine,
        debug_margin: String,
        debug_print: bool,
    ) -> Result<Vec<Truth>, String> {
        if debug_print {
            println!(
                "{debug_margin} get all contents of {}",
                self.rel_id.identifier
            )
        }

        let mut depth_map = caller_depth_map.to_owned();
        const MAX_DEPTH: usize = 5;

        let go_deeper = if let Some(depth_count) = depth_map.get_mut(&self.rel_id) {
            *depth_count -= 1;
            depth_count.to_owned() > 0
        } else {
            depth_map.insert(self.rel_id.to_owned(), MAX_DEPTH);
            true
        };
        let mut ret = vec![];

        for command in &self.history {
            match (command, go_deeper) {
                (IsTrueThat(truth), _) => ret.push(truth.to_owned()),
                (Command::IsTrueWhen(conditional), true) => ret.extend(conditional.get_truths(
                    engine,
                    &depth_map,
                    debug_margin.to_owned() + "|   ",
                    debug_print,
                )),
                _ => (),
            }
        }

        if debug_print {
            println!(
                "{debug_margin} got all contents of {}",
                self.rel_id.identifier
            )
        }

        Ok(ret)
    }

    pub fn get_filtered_truths(
        self: &Table,
        filter: &DeferedRelation,
        engine: &Engine,
        caller_depth_map: &HashMap<RelId, usize>,
        debug_margin: String,
        debug_print: bool,
    ) -> Result<Vec<Truth>, String> {
        if debug_print {
            println!(
                "{debug_margin} get filtered truths of {} with filter {filter}",
                self.rel_id.identifier
            );
        }

        self.check_relation(&filter)?;

        let all_truths = self.get_all_contents(
            caller_depth_map,
            engine,
            debug_margin.to_owned() + "   ",
            debug_print,
        )?;

        let mut matched_truths = vec![];
        for truth in all_truths {
            if let Ok(_) = truth.fits_filter(filter, VarContext::new()) {
                matched_truths.push(truth.to_owned());
            }
        }

        Ok(matched_truths)
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::new();

        for command in self.history.iter() {
            ret += &format!("{command}");
        }

        write!(f, "{}", ret)
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsTrueThat(truth) => write!(f, "{truth}\n"),
            IsTrueWhen(conditional_truth) => write!(f, "{conditional_truth}\n"),
        }
    }
}
