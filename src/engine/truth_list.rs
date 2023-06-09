use std::{collections::HashSet, fmt};

use super::relation::truth::Truth;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Completeness {
    pub some_extra_info: bool,
    pub some_missing_info: bool,
}

impl fmt::Display for Completeness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{} {}],",
            if self.some_extra_info { "extra" } else { "" },
            if self.some_missing_info {
                "missing"
            } else {
                ""
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct TruthList {
    truths: HashSet<Truth>,
    completeness: Completeness,
}

impl Completeness {
    pub fn extra() -> Self {
        Completeness {
            some_extra_info: true,
            some_missing_info: false,
        }
    }
    pub fn missing() -> Self {
        Completeness {
            some_extra_info: false,
            some_missing_info: true,
        }
    }
    pub fn missing_extra() -> Self {
        Completeness {
            some_extra_info: true,
            some_missing_info: true,
        }
    }
    pub fn exact() -> Self {
        Completeness {
            some_extra_info: false,
            some_missing_info: false,
        }
    }
}

impl fmt::Display for TruthList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::new();

        for truth in &self.truths {
            ret += &format!("{truth},");
        }

        write!(f, "{{{}: {}}}", self.completeness, ret)
    }
}

impl TruthList {
    pub fn new(completeness: Completeness) -> Self {
        Self {
            truths: HashSet::new(),
            completeness,
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

    pub(crate) fn set_completeness(&mut self, c: Completeness) {
        self.completeness = c
    }

    pub fn get_completeness(&self) -> Completeness {
        self.completeness.to_owned()
    }
}
