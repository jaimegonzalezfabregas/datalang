use std::{collections::HashSet, fmt};

use super::{truth_list::Completeness, var_context::VarContext};

#[derive(Debug, Clone, PartialEq)]
pub struct VarContextUniverse {
    pub completeness: Completeness,
    pub contents: HashSet<VarContext>,
}

impl fmt::Display for VarContextUniverse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::new();

        for context in &self.contents {
            ret += &format!("{context},");
        }

        write!(f, "[{}:{}]", self.completeness, ret)
    }
}

impl VarContextUniverse {
    pub fn new(c: Completeness) -> Self {
        Self {
            contents: HashSet::new(),
            completeness: c,
        }
    }

    pub fn set_completeness(&mut self, c: Completeness) {
        self.completeness = c;
    }

    pub fn get_completeness(&self) -> Completeness {
        self.completeness.to_owned()
    }

    pub fn or(self, other: Self, debug_margin: String, debug_print: bool) -> Self {
        let res_completeness = Completeness {
            some_extra_info: self.completeness.some_extra_info
                || other.completeness.some_extra_info,
            some_missing_info: self.completeness.some_missing_info
                || other.completeness.some_missing_info,
        };
        let mut ret = Self::new(res_completeness);

        for context in &self.contents {
            ret.insert(context.to_owned());
        }

        for context in &other.contents {
            ret.insert(context.to_owned());
        }
        if debug_print {
            println!("{debug_margin}{self} or {other} = {ret}");
        }
        ret
    }

    pub fn and(self, other: Self, debug_margin: String, debug_print: bool) -> Self {
        if debug_print {
            print!("{debug_margin}{self} and {other} =");
        }
        let ret = match (
            self.completeness.some_missing_info,
            other.completeness.some_missing_info,
        ) {
            (true, true) => {
                let mut ret = self.or(other, "".into(), false);
                ret.completeness = Completeness {
                    some_extra_info: true,
                    some_missing_info: true,
                };
                ret
            }
            (true, false) => {
                let mut ret = other;
                ret.completeness.some_extra_info = true;
                ret.completeness.some_missing_info = true;
                ret
            }
            (false, true) => {
                let mut ret = self;
                ret.completeness.some_extra_info = true;
                ret.completeness.some_missing_info = true;
                ret
            }
            (false, false) => {
                let mut contents = HashSet::new();

                for context_a in &self.contents {
                    for content_b in &other.contents {
                        let op_merge = context_a.extend(&content_b);
                        match op_merge {
                            Some(merged) => {
                                contents.insert(merged);
                            }
                            None => (),
                        }
                    }
                }

                VarContextUniverse {
                    contents,
                    completeness: Completeness {
                        some_extra_info: self.completeness.some_extra_info
                            && self.completeness.some_extra_info,
                        some_missing_info: false,
                    },
                }
            }
        };

        if debug_print {
            println!(" {ret}");
        }
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

        Self {
            contents: ret,
            completeness: self.to_owned().completeness,
        }
    }

    pub fn len(&self) -> usize {
        self.contents.len()
    }
}
