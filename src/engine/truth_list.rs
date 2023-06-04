use std::collections::HashSet;

use super::relation::truth::Truth;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Completeness {
    FullKnoliedge,
    PartialKnoliedge,
}

#[derive(Clone, Debug)]
pub struct TruthList {
    truths: HashSet<Truth>,
    completeness: Completeness,
}

impl TruthList {
    pub fn new() -> Self {
        Self {
            truths: HashSet::new(),
            completeness: Completeness::PartialKnoliedge,
        }
    }

    pub fn to_vector(&self) -> Vec<Truth> {
        self.into_iter().collect()
    }

    pub fn into_iter(&self) -> impl Iterator<Item = Truth> {
        self.truths.clone().into_iter()
    }

    pub fn add(&mut self, truth: Truth) {
        self.truths.insert(truth);
    }

    pub(crate) fn set_completeness(&mut self, c: Completeness)  {
        self.completeness = c
    }

    pub fn get_completeness(&self) -> Completeness{
        self.completeness.to_owned()
    }
}
