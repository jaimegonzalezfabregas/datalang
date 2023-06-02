use std::{collections::HashSet, fmt, vec};
mod conditional_truth;
pub mod truth;

use crate::parser::{
    conditional_reader::Conditional, data_reader::Data, defered_relation_reader::DeferedRelation,
    expresion_reader::Expresion, inmediate_relation_reader::InmediateRelation, HasRelId,
};

use self::{conditional_truth::ConditionalTruth, truth::Truth};

use super::{
    recursion_tally::RecursionTally, truth_list::TruthList, var_context::VarContext, Engine, RelId,
};

#[derive(Debug, Clone)]
pub struct Relation {
    rel_id: RelId,
    truths: HashSet<Truth>,
    conditions: HashSet<ConditionalTruth>,
}

pub struct ContentIterator {
    filter: DeferedRelation,
    condition_vec: Vec<ConditionalTruth>,
    curent_returning_queue: Vec<Truth>,
    recursion_tally: RecursionTally,
    engine: Engine,
    debug_margin: String,
    debug_print: bool,
}

impl Iterator for ContentIterator {
    type Item = Result<Truth, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ret) = self.curent_returning_queue.pop() {
            Some(Ok(ret))
        } else if let Some(cond) = self.condition_vec.pop() {
            self.curent_returning_queue = cond.get_deductions(
                &self.filter,
                &self.engine,
                &self.recursion_tally,
                self.debug_margin.to_owned() + "|  ",
                self.debug_print,
            )?;

            self.next()
        } else {
            None
        }
    }
}

impl Relation {
    fn open_filter(&self) -> DeferedRelation {
        DeferedRelation {
            negated: false,
            assumptions: vec![],
            rel_name: self.rel_id.identifier.to_owned(),
            args: vec![Expresion::Literal(Data::Any); self.rel_id.column_count],
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
        self.conditions.insert(ConditionalTruth::from(cond));
        Ok(())
    }

    pub fn get_content_iter(
        self: &Relation,
        filter: DeferedRelation,
        mut recursion_tally: RecursionTally,
        engine: Engine,
        debug_margin: String,
        debug_print: bool,
    ) -> ContentIterator {
        if debug_print {
            println!(
                "{debug_margin}get all contents of {} with filter {filter}",
                self.rel_id.identifier
            )
        }
        recursion_tally.count_up(&self.rel_id);
        let go_deeper = recursion_tally.go_deeper(&self.rel_id);

        ContentIterator {
            filter,
            condition_vec: if go_deeper {
                self.conditions.to_owned().into_iter().collect()
            } else {
                vec![]
            },
            curent_returning_queue: self.truths.to_owned().into_iter().collect(),
            recursion_tally,
            engine,
            debug_margin,
            debug_print,
        }
    }

    pub fn get_filtered_truths(
        self: &Relation,
        filter: &DeferedRelation,
        engine: &Engine,
        recursion_tally: &RecursionTally,
        debug_margin: String,
        debug_print: bool,
    ) -> TruthList {
        if debug_print {
            println!(
                "{debug_margin}get filtered truths of {} with filter {filter}",
                self.rel_id.identifier
            );
        }

        let all_truths = self.get_content_iter(
            filter.to_owned(),
            recursion_tally.to_owned(),
            engine.to_owned(),
            debug_margin.to_owned() + "|  ",
            debug_print,
        );

        let mut matched_truths = TruthList::new();
        for truth in all_truths {
            if let Ok(fitted) = truth.fits_filter(
                filter,
                VarContext::new(),
                debug_margin.to_owned() + "|  ",
                debug_print,
            ) {
                matched_truths.add(fitted);
                println!("matched truths so far: {matched_truths:?}")
            }
        }

        Ok(matched_truths)
    }

    pub fn contains(
        self: &Relation,
        filter: &DeferedRelation,
        engine: &Engine,
        recursion_tally: &RecursionTally,
        debug_margin: String,
        debug_print: bool,
    ) -> Result<bool, String> {
        if debug_print {
            println!(
                "{debug_margin}check contains in {} filter {filter}",
                self.rel_id.identifier
            );
        }

        let all_truths = self.get_content_iter(
            self.open_filter(),
            recursion_tally.to_owned(),
            engine.to_owned(),
            debug_margin.to_owned() + "|  ",
            debug_print,
        );

        for truth in all_truths {
            if let Ok(_) = truth.fits_filter(
                filter,
                VarContext::new(),
                debug_margin.to_owned() + "|  ",
                debug_print,
            ) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl fmt::Display for Relation {
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
