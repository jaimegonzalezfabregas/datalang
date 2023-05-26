use core::fmt;
use std::{backtrace, collections::HashMap};

use crate::{
    engine::{Engine, RelId},
    parser::{
        conditional_reader::Conditional, defered_relation_reader::DeferedRelation,
        statement_reader::Statement,
    },
};

use super::truth::Truth;
#[derive(Debug, Clone)]
pub struct ConditionalTruth {
    condition: Statement,
    result_template: DeferedRelation,
}

impl fmt::Display for ConditionalTruth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} :- {}", self.result_template, self.condition)
    }
}

impl ConditionalTruth {
    pub fn get_truths(
        &self,
        engine: &Engine,
        depth_map: &HashMap<RelId, usize>,
        debug_print: bool,
    ) -> Vec<Truth> {
        println!("{}", backtrace::Backtrace::capture());

        self.condition
            .get_posible_contexts(
                engine,
                depth_map,
                &self
                    .condition
                    .get_context_universe(engine, depth_map, debug_print),
                debug_print,
            )
            .iter()
            // .map(|e| {
            //     println!("contexts: {e:?}");
            //     e
            // })
            .map(|c| self.result_template.to_truth(c))
            .filter(|e| match e {
                Ok(_) => true,
                Err(_) => false,
            })
            .map(|e| match e {
                Ok(res) => res,
                Err(_) => unreachable!(),
            })
            .collect()
    }
    pub fn from(c: Conditional) -> Self {
        ConditionalTruth {
            condition: c.conditional,
            result_template: c.relation,
        }
    }
}
