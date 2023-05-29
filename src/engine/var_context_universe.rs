use std::{collections::HashSet, fmt};

use super::var_context::VarContext;

#[derive(Debug, Clone)]
pub struct VarContextUniverse {
    contents: Option<HashSet<VarContext>>,
}

impl From<Vec<VarContext>> for VarContextUniverse {
    fn from(value: Vec<VarContext>) -> Self {
        if value.len() > 0 {
            let mut ret = HashSet::new();
            ret.extend(value.to_owned().into_iter());
            Self {
                contents: Some(ret),
            }
        } else {
            Self { contents: None }
        }
    }
}

impl fmt::Display for VarContextUniverse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::new();

        match &self.contents {
            Some(contexts) => {
                ret += "[";
                for context in contexts {
                    ret += &format!("{context},");
                }
                ret += "]";
            }
            None => ret = "Unrestrictive".into(),
        }

        write!(f, "{}", ret)
    }
}

impl VarContextUniverse {
    pub fn new_unrestricting() -> Self {
        Self { contents: None }
    }

    pub fn is_restricting(&self) -> bool {
        self.contents.is_some()
    }

    pub fn or(self, other: Self, debug_margin: String, debug_print: bool) -> Self {
        let mut ret = Self { contents: None };

        match &self.contents {
            Some(contexts) => {
                for context in contexts {
                    ret.insert(context.to_owned());
                }
            }

            None => (),
        }
        match &other.contents {
            Some(contexts) => {
                for context in contexts {
                    ret.insert(context.to_owned());
                }
            }

            None => (),
        }
        if debug_print {
            println!("{debug_margin}{self} or {other} = {ret}");
        }
        ret
    }

    pub fn and(self, other: Self, debug_margin: String, debug_print: bool) -> Self {
        let ret = match (&self.contents, &other.contents) {
            (Some(a), Some(b)) => {
                let mut ret_set = HashSet::new();
                for context_a in a {
                    for context_b in b {
                        if let Some(merged) = context_a.extend(&context_b) {
                            ret_set.insert(merged);
                        }
                    }
                }
                Self {
                    contents: Some(ret_set),
                }
            }
            (None, Some(_)) => other.to_owned(),
            (Some(_), None) => self.to_owned(),
            (None, None) => Self::new_unrestricting(),
        };
        if debug_print {
            println!("{debug_margin}{self} and {other} = {ret}");
        }
        ret
    }

    pub fn iter(&self) -> impl Iterator<Item = VarContext> {
        return self.contents.to_owned().into_iter().flatten();
    }

    pub fn new_restricting() -> Self {
        Self {
            contents: Some(HashSet::new()),
        }
    }

    pub fn insert(&mut self, context: VarContext) {
        match &mut self.contents {
            Some(_) => (),
            None => {
                self.contents = Some(HashSet::new());
            }
        };
        match &mut self.contents {
            Some(set) => {
                set.insert(context);
            }
            None => {
                unreachable!();
            }
        }
    }

    pub(crate) fn difference(&self, contexts_to_remove: &Self) -> Self {
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

        if ret.len() == 0 {
            Self { contents: None }
        } else {
            Self {
                contents: Some(ret),
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self.contents {
            Some(set) => set.len(),
            None => 0,
        }
    }
}
