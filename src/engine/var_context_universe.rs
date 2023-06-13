use std::{collections::HashSet, fmt};

use print_macros::*;

use super::var_context::VarContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarContextUniverse {
    pub contents: HashSet<VarContext>,
}
use std::hash::Hash;
impl Hash for VarContextUniverse {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.contents
            .iter()
            .cloned()
            .collect::<Vec<VarContext>>()
            .hash(state);
    }
}

impl fmt::Display for VarContextUniverse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::new();

        for context in &self.contents {
            ret += &format!("{context},");
        }

        write!(f, "[{}]", ret)
    }
}

impl VarContextUniverse {
    pub fn new() -> Self {
        Self {
            contents: HashSet::new(),
        }
    }

    pub fn or(self, other: Self) -> Self {
        let mut ret = Self::new();

        for context in &self.contents {
            ret.insert(context.to_owned());
        }

        for context in &other.contents {
            ret.insert(context.to_owned());
        }
        printprocessop!("{} or {} = {}", self, other, ret); // TODO Do I need this print??

        ret
    }

    pub fn iter(&self) -> impl Iterator<Item = VarContext> {
        self.contents.to_owned().into_iter()
    }

    pub fn insert(&mut self, context: VarContext) {
        self.contents.insert(context);
    }

    pub fn difference(&self, contexts_to_remove: &Self) -> Self {
        let mut ret = HashSet::new();

        for context in self.iter() {
            let copy_of_context = context.clone();
            if contexts_to_remove
                .iter()
                .find(move |e| e == &copy_of_context)
                .is_none()
            {
                ret.insert(context);
            }
        }

        Self { contents: ret }
    }

    pub fn len(&self) -> usize {
        self.contents.len()
    }
}
