use std::{collections::HashSet, fmt};
mod conditional_truth;
pub mod truth;

use crate::parser::{
    conditional_reader::Conditional,  defered_relation_reader::DeferedRelation,
     inmediate_relation_reader::InmediateRelation,
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

impl Relation {
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

    fn get_all_truths(
        self: &Relation,
        filter: &DeferedRelation,
        engine: &Engine,
        recursion_tally: &RecursionTally,
        debug_margin: String,
        debug_print: bool,
    ) -> Result<TruthList, String> {
        let mut ret = TruthList::new();
        ret.set_completeness(super::truth_list::Completeness::FullKnoliedge);

        for literal_truth in self.truths.to_owned() {
            ret.add(literal_truth);
        }

        for conditional in self.conditions.to_owned() {
            let sub_truth_list = conditional.get_deductions(
                filter,
                engine,
                recursion_tally,
                debug_margin.to_owned() + "|  ",
                debug_print,
            )?;
            if ret.get_completeness() == super::truth_list::Completeness::FullKnoliedge {
                ret.set_completeness(sub_truth_list.get_completeness())
            }
            for truth in sub_truth_list.into_iter() {
                ret.add(truth);
            }
        }

        Ok(ret)
    }

    pub fn get_filtered_truths(
        self: &Relation,
        filter: &DeferedRelation,
        engine: &Engine,
        recursion_tally: &RecursionTally,
        debug_margin: String,
        debug_print: bool,
    ) -> Result<TruthList, String> {
        if debug_print {
            println!(
                "{debug_margin}get filtered truths of {} with filter {filter}",
                self.rel_id.identifier
            );
        }

        let all_truths = self.get_all_truths(
            filter,
            engine,
            recursion_tally,
            debug_margin.to_owned() + "|  ",
            debug_print,
        )?;

        let mut matched_truths = TruthList::new();

        for truth in all_truths.into_iter() {
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
