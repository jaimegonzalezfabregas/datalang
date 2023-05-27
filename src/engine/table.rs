use std::{
    collections::{HashMap, HashSet},
    fmt, vec,
};
mod conditional_truth;
pub mod truth;

use crate::parser::{
    conditional_reader::Conditional,
    defered_relation_reader::DeferedRelation,
    expresion_reader::{Expresion, VarName},
    inmediate_relation_reader::InmediateRelation,
    Relation,
};

use self::{conditional_truth::ConditionalTruth, truth::Truth};

use super::{var_context::VarContext, Engine, RelId};

#[derive(Debug, Clone)]
pub struct Table {
    rel_id: RelId,
    truths: HashSet<Truth>,
    conditions: HashSet<ConditionalTruth>,
}

pub struct ContentIterator {
    filter: DeferedRelation,
    condition_vec: Vec<ConditionalTruth>,
    curent_returning_queue: Vec<Truth>,
    creator_depth_map: HashMap<RelId, usize>,
    engine: Engine,
    debug_margin: String,
    debug_print: bool,
}

impl Iterator for ContentIterator {
    type Item = Truth;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ret) = self.curent_returning_queue.pop() {
            Some(ret)
        } else if let Some(cond) = self.condition_vec.pop() {
            self.curent_returning_queue = cond.get_truths(
                &self.filter,
                &self.engine,
                &self.creator_depth_map,
                self.debug_margin.to_owned() + "|  ",
                self.debug_print,
            );

            self.next()
        } else {
            None
        }
    }
}

impl Table {
    fn open_filter(&self) -> DeferedRelation {
        DeferedRelation {
            negated: false,
            assumptions: vec![],
            rel_name: self.rel_id.identifier.to_owned(),
            args: vec![Expresion::Var(VarName::Anonimus); self.rel_id.column_count],
        }
    }

    fn check_relation<T: Relation>(&self, rule: &&T) -> Result<(), String> {
        if self.rel_id == rule.get_rel_id() {
            Ok(())
        } else {
            Err("Cant process a row for a diferent table".into())
        }
    }

    pub fn new(rel_id: &RelId) -> Self {
        Self {
            rel_id: rel_id.to_owned(),
            truths: HashSet::new(),
            conditions: HashSet::new(),
        }
    }

    pub fn add_truth(&mut self, rule: InmediateRelation) -> Result<(), String> {
        self.check_relation(&&rule)?;
        match rule.negated {
            false => {
                self.truths.insert(Truth::from(&rule));
            }
            true => {
                let what_we_want_to_remove = &rule.args.to_owned();
                self.truths
                    .retain(|elm| !elm.afirms(what_we_want_to_remove));
            }
        };
        Ok(())
    }

    pub(crate) fn add_conditional(&mut self, cond: Conditional) -> Result<(), String> {
        self.check_relation(&&cond)?;

        self.conditions.insert(ConditionalTruth::from(cond));
        Ok(())
    }

    pub fn get_content_iter(
        self: &Table,
        filter: &DeferedRelation,
        caller_depth_map: &HashMap<RelId, usize>,
        engine: &Engine,
        debug_margin: String,
        debug_print: bool,
    ) -> ContentIterator {
        if debug_print {
            println!(
                "{debug_margin}get all contents of {} with dm: {caller_depth_map:?}",
                self.rel_id.identifier
            )
        }

        let mut depth_map = caller_depth_map.to_owned();
        const MAX_DEPTH: usize = 7;

        let go_deeper = if let Some(depth_count) = depth_map.get_mut(&self.rel_id) {
            *depth_count -= 1;
            depth_count.to_owned() > 0
        } else {
            depth_map.insert(self.rel_id.to_owned(), MAX_DEPTH);
            true
        };

        ContentIterator {
            filter: filter.to_owned(),
            condition_vec: if go_deeper {
                self.conditions.to_owned().into_iter().collect()
            } else {
                vec![]
            },
            curent_returning_queue: self.truths.to_owned().into_iter().collect(),
            creator_depth_map: depth_map,
            engine: engine.to_owned(),
            debug_margin,
            debug_print,
        }
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
                "{debug_margin}get filtered truths of {} with filter {filter}",
                self.rel_id.identifier
            );
        }

        self.check_relation(&filter)?;

        let all_truths = self.get_content_iter(
            filter,
            caller_depth_map,
            engine,
            debug_margin.to_owned() + "|  ",
            debug_print,
        );

        let mut matched_truths = vec![];
        for truth in all_truths {
            if let Ok(_) = truth.fits_filter(filter, VarContext::new()) {
                matched_truths.push(truth.to_owned());
            }
        }

        Ok(matched_truths)
    }

    pub fn contains(
        self: &Table,
        filter: &DeferedRelation,
        engine: &Engine,
        caller_depth_map: &HashMap<RelId, usize>,
        debug_margin: String,
        debug_print: bool,
    ) -> Result<bool, String> {
        if debug_print {
            println!(
                "{debug_margin}check contains in {} filter {filter}",
                self.rel_id.identifier
            );
        }

        self.check_relation(&filter)?;

        let all_truths = self.get_content_iter(
            &self.open_filter(),
            caller_depth_map,
            engine,
            debug_margin.to_owned() + "|  ",
            debug_print,
        );

        for truth in all_truths {
            if let Ok(_) = truth.fits_filter(filter, VarContext::new()) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::new();
        for truth in self.truths.iter() {
            ret += &format!("{truth}");
        }
        for condition in self.conditions.iter() {
            ret += &format!("{condition}");
        }

        write!(f, "{}", ret)
    }
}
