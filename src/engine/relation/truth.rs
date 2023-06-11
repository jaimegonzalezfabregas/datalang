use std::fmt;

use crate::{
    engine::{var_context::VarContext, RelId},
    parser::{
        data_token::Data, defered_relation_token::DeferedRelation,
        inmediate_relation_token::InmediateRelation, HasRelId,
    },
};
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct Truth {
    rel_id: RelId,
    data: Vec<Data>,
}

impl fmt::Display for Truth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = String::new();
        args += &"(";
        for (i, d) in self.data.iter().enumerate() {
            args += &format!("{d}");
            if i != self.data.len() - 1 {
                args += &",";
            }
        }
        args += &")";

        write!(f, "{}{args}", self.rel_id.identifier)
    }
}

impl Ord for Truth {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self
            .data
            .iter()
            .zip(other.data.iter())
            .skip_while(|(a, b)| a == b)
            .next()
        {
            None => std::cmp::Ordering::Equal,
            Some((a, b)) => a.cmp(b),
        }
    }
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
        debug_margin: String,
        debug_print: bool,
    ) -> Result<Truth, String> {
        let mut ret = self.clone();

        let mut context = caller_context;
        let mut pinned = vec![false; self.get_width()];

        while !pinned.iter().all(|&e| e) {
            let starting_pinned_count = pinned.iter().filter(|e| **e).count();
            for (i, (goal, filter_expresion)) in
                self.data.iter().zip(filter.to_owned().args).enumerate()
            {
                if !pinned[i] {
                    let solution = filter_expresion.solve(
                        &goal,
                        &context,
                        debug_margin.to_owned() + "|  ",
                        debug_print,
                    );
                    match solution {
                        Ok(new_context) => {
                            context = new_context;
                            pinned[i] = true;
                            match filter_expresion.literalize(&context) {
                                Ok(Data::Any) => (),
                                Ok(data) => ret.data[i] = data,
                                Err(_) => unimplemented!(),
                            }
                        }

                        Err(_) => (),
                    }
                }
            }
            let ending_pinned_count = pinned.iter().filter(|e| **e).count();
            if starting_pinned_count == ending_pinned_count {
                return Err("unsolveable".into());
            }
        }
        Ok(ret)
    }
}

impl From<&InmediateRelation> for Truth {
    fn from(value: &InmediateRelation) -> Self {
        Self {
            data: value.args.to_owned(),
            rel_id: value.get_rel_id(),
        }
    }
}

impl From<&(Vec<Data>, RelId)> for Truth {
    fn from(args: &(Vec<Data>, RelId)) -> Self {
        Self {
            data: args.0.to_owned(),
            rel_id: args.1.to_owned(),
        }
    }
}
