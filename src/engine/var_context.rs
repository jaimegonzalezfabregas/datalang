use crate::parser::data_reader::Data;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarContext {
    op_map: Option<HashMap<String, Data>>,
}

impl Hash for VarContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        format!("{self:?}").hash(state);
    }
}

impl fmt::Display for VarContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.op_map {
            Some(map) => {
                let mut ret = String::new();
                ret += "|";
                for (key, value) in map.iter() {
                    ret += &format!("{key}:{value}|");
                }

                write!(f, "{}", ret)
            }
            None => write!(f, "no context"),
        }
    }
}

impl VarContext {
    pub fn get(&self, var_name: &String) -> Option<Data> {
        match &self.op_map {
            Some(map) => match map.get(var_name) {
                Some(ret) => Some(ret.to_owned()),
                None => None,
            },
            None => None,
        }
    }

    pub fn set(&mut self, var_name: String, value: Data) {
        match self.op_map {
            None => {
                self.op_map = Some(HashMap::new());
            }
            Some(_) => (),
        }
        match &mut self.op_map {
            None => {
                unreachable!()
            }
            Some(map) => {
                map.insert(var_name, value);
            }
        }
    }

    pub(crate) fn new() -> VarContext {
        VarContext { op_map: None }
    }

    pub fn extend(&self, b_context: &VarContext) -> VarContext {
        Self::from(match (&self.op_map, &b_context.op_map) {
            (None, None) => None,
            (None, Some(b)) => Some(b.to_owned()),
            (Some(a), None) => Some(a.to_owned()),
            (Some(a), Some(b)) => {
                if a.keys()
                    .any(|a_key| b.contains_key(a_key) && b.get(a_key) != a.get(a_key))
                {
                    None
                } else {
                    let mut join = a.clone();
                    join.extend(b.clone());
                    Some(join)
                }
            }
        })
    }
}

impl From<HashMap<String, Data>> for VarContext {
    fn from(value: HashMap<String, Data>) -> Self {
        Self {
            op_map: Some(value),
        }
    }
}

impl From<Option<HashMap<String, Data>>> for VarContext {
    fn from(value: Option<HashMap<String, Data>>) -> Self {
        Self { op_map: value }
    }
}
