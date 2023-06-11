use std::{collections::HashSet, fmt};

use super::relation::truth::Truth;

#[derive(Clone, Debug)]
pub struct TruthList {
    truths: HashSet<Truth>,
}

impl fmt::Display for TruthList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::new();

        for truth in &self.truths {
            ret += &format!("{truth},");
        }

        write!(f, "{{{}}}", ret)
    }
}

impl TruthList {
    pub fn new() -> Self {
        Self {
            truths: HashSet::new(),
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
}
