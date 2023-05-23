use std::collections::HashMap;

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
impl ConditionalTruth {
    pub fn get_truths(&self, engine: &Engine, depth_map: &HashMap<RelId, usize>) -> Vec<Truth> {
        self.condition
            .get_posible_contexts(
                engine,
                depth_map,
                self.condition.get_context_universe(engine, depth_map),
            )
            .iter()
            .map(|c| self.result_template.apply(c))
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
