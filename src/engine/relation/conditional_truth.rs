use core::fmt;

use crate::{
    engine::{
        recursion_tally::RecursionTally, truth_list::TruthList, var_context::VarContext,
        var_context_universe::VarContextUniverse, Engine,
    },
    parser::{
        conditional_reader::Conditional, data_reader::Data,
        defered_relation_reader::DeferedRelation, statement_reader::Statement,
    },
};

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
    pub fn get_deductions(
        &mut self,
        filter: &DeferedRelation,
        engine: &Engine,
        recursion_tally: &RecursionTally,
        debug_margin: String,
        debug_print: bool,
    ) -> Result<TruthList, String> {
        if debug_print {
            println!("{debug_margin}getting deductions of {self}");
        }

        let mut base_context = VarContext::new();

        for (filter, template) in filter.args.iter().zip(self.template.args.to_owned()) {
            match filter.literalize(&base_context) {
                Ok(Data::Any) => (),
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

        // if debug_print {
        //     println!("{debug_margin}base context to respect {filter} is {base_context}");
        // }

        let mut posible_contexts = VarContextUniverse::new();
        posible_contexts.insert(base_context);

        // if debug_print {
        //     println!("{debug_margin}base universe is {posible_contexts}");
        // }

        posible_contexts = self.condition.memo_get_posible_contexts(
            engine,
            recursion_tally,
            &posible_contexts,
            debug_margin.to_owned() + "|  ",
            debug_print,
        )?;

        if debug_print {
            println!("{debug_margin}* universe of {self} is {posible_contexts}");
        }
        let mut ret = TruthList::new();
        for context in posible_contexts.iter() {
            match self.template.to_truth(&context) {
                Ok(truth) => ret.add(truth),
                Err(_) => (),
            };
        }

        if debug_print {
            println!("{debug_margin}* truths of {self} filtered by {filter} are {ret}");
        }

        Ok(ret)
    }
    pub fn from(c: Conditional) -> Self {
        ConditionalTruth {
            condition: c.conditional,
            template: c.relation,
        }
    }
}
