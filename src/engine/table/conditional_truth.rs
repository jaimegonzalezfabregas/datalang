use core::fmt;
use std::collections::HashMap;

use crate::{
    engine::{var_context::VarContext, Engine, RelId},
    parser::{
        conditional_reader::Conditional, defered_relation_reader::DeferedRelation,
        statement_reader::Statement,
    },
};

use super::truth::Truth;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConditionalTruth {
    condition: Statement,
    template: DeferedRelation,
}

impl fmt::Display for ConditionalTruth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} :- {}", self.template, self.condition)
    }
}

impl ConditionalTruth {
    pub fn get_truths(
        &self,
        filter: &DeferedRelation,
        engine: &Engine,
        depth_map: &HashMap<RelId, usize>,
        debug_margin: String,
        debug_print: bool,
    ) -> Vec<Truth> {
        if debug_print {
            println!("{debug_margin} getting truths of {self}");
        }

        let mut base_context = VarContext::new();

        for (filter, template) in filter.args.iter().zip(self.template.args.to_owned()) {
            match filter.literalize(&base_context) {
                Ok(data) => match template.solve(&data, &base_context) {
                    Ok(new_context) => base_context = new_context,
                    Err(_) => (),
                },
                Err(_) => (),
            }
        }

        let univese_of_contexts = &self.condition.get_context_universe(
            filter,
            engine,
            &base_context,
            depth_map,
            debug_margin.to_owned() + "|  ",
            debug_print,
        );

        self.condition
            .get_posible_contexts(
                engine,
                depth_map,
                univese_of_contexts,
                debug_margin.to_owned() + "|  ",
                debug_print,
            )
            .iter()
            // .map(|e| {
            //     println!("contexts: {e:?}");
            //     e
            // })
            .map(|c| self.template.to_truth(c))
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
            template: c.relation,
        }
    }
}
