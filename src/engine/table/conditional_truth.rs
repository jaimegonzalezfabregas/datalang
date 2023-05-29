use core::fmt;

use crate::{
    engine::{recursion_tally::RecursionTally, var_context::VarContext, Engine},
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
                Ok(data) => match template.solve(&data, &base_context) {
                    Ok(new_context) => base_context = new_context,
                    Err(_) => (),
                },
                Err(_) => (),
            }
        }

        if debug_print {
            println!("{debug_margin}base context is {base_context}");
        }

        let univese_of_contexts = &self.condition.get_context_universe(
            filter,
            engine,
            &base_context,
            recursion_tally,
            debug_margin.to_owned() + "|  ",
            debug_print,
        );

        let ret = self
            .condition
            .get_posible_contexts(
                engine,
                recursion_tally,
                univese_of_contexts,
                debug_margin.to_owned() + "|  ",
                debug_print,
            )
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
