use crate::parser::data_reader::Data;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarContext {
    map: HashMap<String, Data>,
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
            map: HashMap::new(),
        }
    }

    pub fn extend(&self, b_context: &VarContext) -> Option<VarContext> {
        let a = &self.map;
        let b = &b_context.map;

        let ret = if a
            .keys()
            .any(|a_key| b.contains_key(a_key) && b.get(a_key) != a.get(a_key))
        {
            None
        } else {
            let mut join = a.clone();
            join.extend(b.clone());
            Some(Self::from(join))
        };

        println!("extending {self} y {b_context} results in {ret:?}");

        ret
    }
}

impl From<HashMap<String, Data>> for VarContext {
    fn from(value: HashMap<String, Data>) -> Self {
        Self { map: value }
    }
}
