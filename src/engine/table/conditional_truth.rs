use core::fmt;

use crate::{
    engine::{
        recursion_tally::RecursionTally, var_context::VarContext,
        var_context_universe::VarContextUniverse, Engine,
    },
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
        recursion_tally: &RecursionTally,
        debug_margin: String,
        debug_print: bool,
    ) -> Vec<Truth> {
        if debug_print {
            println!("{debug_margin}getting truths of {self}");
        }

        let mut base_context = VarContext::new();

        for (filter, template) in filter.args.iter().zip(self.template.args.to_owned()) {
            match filter.literalize(&base_context) {
                Ok(data) => match template.solve(
                    &data,
                    &base_context,
                    debug_margin.to_owned() + "|  ",
                    debug_print,
                ) {
                    Ok(new_context) => base_context = new_context,
                    Err(_) => (),
                },
                Err(_) => (),
            }
        }

        if debug_print {
            println!("{debug_margin}base context is {base_context}");
        }

        let mut results = VarContextUniverse::new_restricting();
        results.insert(base_context);
        let mut last_results_len = 0;

        let mut simplified_statement = self.condition.to_owned();

        while (results.len() == last_results_len) {
            last_results_len = results.len();

            (results, simplified_statement) = simplified_statement.get_posible_contexts(
                engine,
                recursion_tally,
                &results,
                debug_margin.to_owned() + "|  ",
                debug_print,
            );
        }

        let ret = results
            .iter()
            // .map(|e| {
            //     println!("contexts: {e:?}");
            //     e
            // })
            .map(|c| self.template.to_truth(&c))
            .filter(|e| match e {
                Ok(_) => true,
                Err(err) => {
                    println!("{err:?}");
                    false
                }
            })
            .map(|e| match e {
                Ok(res) => res,
                Err(_) => unreachable!(),
            })
            .collect();

        if debug_print {
            println!("{debug_margin}* truths of {self} are {ret:?}");
        }

        ret
    }
    pub fn from(c: Conditional) -> Self {
        ConditionalTruth {
            condition: c.conditional,
            template: c.relation,
        }
    }
}
