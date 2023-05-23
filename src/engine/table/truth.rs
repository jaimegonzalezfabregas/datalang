
use crate::{
    engine::var_context::VarContext,
    parser::{
        data_reader::Data, defered_relation_reader::DeferedRelation,
        inmediate_relation_reader::InmediateRelation,
    },
};
#[derive(Debug, Clone, PartialEq)]
pub struct Truth {
    data: Vec<Data>,
}

impl Truth {
    pub fn afirms(&self, query: &Vec<Data>) -> bool {
        query == &self.data
    }
    pub fn get_data(&self) -> &Vec<Data> {
        &self.data
    }
    pub fn get_width(&self) -> usize {
        self.data.len()
    }
    pub fn fits_filter(
        &self,
        filter: &DeferedRelation,
        caller_context: VarContext,
    ) -> Result<VarContext, String> {
        let mut context = caller_context;
        let mut pinned = vec![false; self.get_width()];
        while !pinned.iter().all(|e| *e) {
            let starting_pinned_count = pinned.iter().filter(|e| **e).count();
            for (i, (goal, filter_expresion)) in
                self.data.iter().zip(filter.to_owned().args).enumerate()
            {
                let solution = filter_expresion.solve(&goal, &context);
                match solution {
                    Ok(new_context) => {
                        context = new_context;
                        pinned[i] = true;
                    }

                    Err(_) => (),
                }
            }
            let ending_pinned_count = pinned.iter().filter(|e| **e).count();
            if starting_pinned_count == ending_pinned_count {
                return Err("unsolveable".into());
            }
        }
        Ok(context)
    }
}

impl From<Vec<Data>> for Truth {
    fn from(value: Vec<Data>) -> Self {
        Self { data: value }
    }
}

impl From<InmediateRelation> for Truth {
    fn from(value: InmediateRelation) -> Self {
        Self { data: value.args }
    }
}
