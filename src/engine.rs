use core::panic;
use std::{collections::HashMap, vec};

use crate::parser::{RelName, Statement, VarLiteral};



#[derive(Debug, Clone)]
struct Table {
    data: Vec<Vec<VarLiteral>>,
}

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
pub struct Engine {
    extensional: HashMap<RelId, Table>,
    intensional: Vec<Deductions>,
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    RelationNotFound(RelId),
    UnmatchingLine(Statement),
}

impl Engine {
    pub fn new() -> Self {
        Self {
            extensional: HashMap::new(),
            intensional: vec![],
        }
    }

    pub fn ingest(self: &mut Engine, lines: Vec<Statement>) -> Result<(), RuntimeError> {
        for line in lines {
            println!("   ingesting line: {:?}\n", line);
            match line {
                Statement::ForgetRelation(RelName(str), data) => {
                    let rel_id = RelId {
                        column_count: data.len(),
                        identifier: str,
                    };
                    match self.extensional.get(&rel_id) {
                        None => {
                            return Err(RuntimeError::RelationNotFound(rel_id));
                        }
                        Some(table) => {}
                    }
                }
                Statement::CreateRelation(RelName(str), data) => {
                    let rel_id = RelId {
                        column_count: data.len(),
                        identifier: str,
                    };
                    match self.extensional.get_mut(&rel_id) {
                        None => {
                            self.extensional
                                .insert(rel_id.clone(), Table { data: vec![] });
                        }
                        _ => {}
                    }
                    match self.extensional.get_mut(&rel_id) {
                        None => (),
                        Some(table) => {
                            table.data.push(
                                data.iter()
                                    .map(|exp| {
                                        return match VarLiteral::literalize(exp.clone()) {
                                            Ok(l) => l,
                                            _ => {
                                                println!("el parser ha dejado pasar una expresion no literalizable ({:?}) en los argumentos de una relacion",exp)   ;
                                                panic!()
                                            }
                                        };
                                    })
                                    .collect(),
                            );
                        }
                    }
                }

                _ => return Err(RuntimeError::UnmatchingLine(line)),
            }

            println!("   engine intrinsics: {:?}\n", self);
        }

        Ok(())
    }
}
