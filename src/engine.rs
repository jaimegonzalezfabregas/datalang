mod table;

use crate::syntax::*;
use std::{collections::HashMap, vec};

use self::table::Table;

#[derive(Debug, Clone)]
struct Deductions {
    identifier: String,
    condition: Statement,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct RelId {
    identifier: String,
    column_count: usize,
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    RelationNotFound(RelId),
    UnmatchingLine(Line),
    Explanation(String),
}

impl From<String> for RuntimeError {
    fn from(value: String) -> Self {
        Self::Explanation(value)
    }
}

#[derive(Debug, Clone)]
pub struct Engine {
    extensional: HashMap<RelId, Table>,
    intensional: Vec<Deductions>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            extensional: HashMap::new(),
            intensional: vec![],
        }
    }

    pub fn ingest(self: &mut Engine, lines: Vec<Line>) -> Result<(), RuntimeError> {
        for line in lines {
            match &line {
                action @ (Line::ForgetRelation(RelName(rel_name), literal_vec)
                | Line::CreateRelation(RelName(rel_name), literal_vec)) => {
                    let width = literal_vec.len();

                    let rel_id = RelId {
                        column_count: width,
                        identifier: rel_name.to_string(),
                    };
                    let insertion_key = rel_id.clone();

                    if let None = self.extensional.get(&rel_id) {
                        self.extensional.insert(insertion_key, Table::new(&width));
                    }

                    if let Some(table) = self.extensional.get_mut(&rel_id) {
                        if let Line::ForgetRelation(..) = action {
                            table.remove(literal_vec.clone())?;
                        } else {
                            table.add(literal_vec.clone())?;
                        }
                    }
                }
                Line::Query(RelName(rel_name), arg_vec) => {
                    let width = arg_vec.len();

                    let rel_id = RelId {
                        column_count: width,
                        identifier: rel_name.to_string(),
                    };

                    if let Some(table) = self.extensional.get_mut(&rel_id) {
                        let query_res = table.get_contents(arg_vec.to_owned())?;
                        println!("{:#?}", query_res);
                    } else {
                        return Err(RuntimeError::RelationNotFound(rel_id));
                    }
                }

                _ => return Err(RuntimeError::UnmatchingLine(line)),
            }
        }

        Ok(())
    }
}
