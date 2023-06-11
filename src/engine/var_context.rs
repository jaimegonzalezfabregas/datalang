use crate::parser::data_reader::Data;
use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone, Debug, Eq)]
pub struct VarContext {
    map: BTreeMap<String, Data>,
}

impl PartialEq for VarContext {
    fn eq(&self, other: &Self) -> bool {
        for (var_a, val_a) in &self.map {
            let mut found = false;
            for (var_b, val_b) in &other.map {
                if var_b == var_a {
                    match (val_a, val_b) {
                        (Data::Any, Data::Any) => found = true,
                        (a, b) => {
                            if a == b {
                                found = true
                            } else {
                                return false;
                            }
                        }
                    }
                }
            }
            if !found {
                return false;
            }
        }
        return true;
    }
}

impl Hash for VarContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        format!("{self:?}").hash(state);
    }
}

impl fmt::Display for VarContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::new();
        ret += "|";
        for (key, value) in self.map.iter() {
            ret += &format!("{key}:{value}|");
        }

        write!(f, "{}", ret)
    }
}

impl VarContext {
    pub fn get(&self, var_name: &String) -> Option<Data> {
        match self.map.get(var_name) {
            Some(ret) => Some(ret.to_owned()),
            None => None,
        }
    }

    pub fn set(&mut self, var_name: String, value: Data) {
        self.map.insert(var_name, value);
    }

    pub(crate) fn new() -> VarContext {
        VarContext {
            map: BTreeMap::new(),
        }
    }

    pub fn extend(&self, b_context: &VarContext) -> Option<VarContext> {
        let a = &self.map;
        let b = &b_context.map;

        let ret = if a.keys().any(|a_key| {
            let ret = b.contains_key(a_key)
                && b.get(a_key) != a.get(a_key)
                && a.get(a_key).cloned() != Some(Data::Any)
                && b.get(a_key).cloned() != Some(Data::Any);
            ret
        }) {
            None
        } else {
            let mut join = a.clone();
            for (var, val) in b.iter() {
                if val != &Data::Any {
                    join.insert(var.to_owned(), val.to_owned());
                }
            }
            Some(Self::from(join))
        };

        // println!("extending {self} y {b_context} results in {ret:?}");

        ret
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

impl From<BTreeMap<String, Data>> for VarContext {
    fn from(value: BTreeMap<String, Data>) -> Self {
        Self { map: value }
    }
}
