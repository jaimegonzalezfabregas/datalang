use std::collections::HashMap;

use crate::{
    engine::{Engine, RelId},
    parser::{
        conditional_reader::Conditional, data_reader::Data,
        defered_relation_reader::DeferedRelation, statement_reader::Statement,
    },
};
#[derive(Debug, Clone)]
pub struct ConditionalTruth {
    condition: Statement,
    result: DeferedRelation,
}
impl ConditionalTruth {
    pub fn get_data(&self, engine: &Engine, depth_map: &HashMap<RelId, usize>) -> Vec<Vec<Data>> {
        todo!()
    }
    pub fn from(c: Conditional) -> Self {
        ConditionalTruth {
            condition: c.conditional,
            result: c.relation,
        }
    }
}
