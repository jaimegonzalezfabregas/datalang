pub mod operations;
mod table;

use crate::{
    lexer,
    parser::{self, data_reader::Data, line_reader::Line},
};
use std::{collections::HashMap, vec};

use self::table::Table;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct RelId {
    pub identifier: String,
    pub column_count: usize,
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
    tables: HashMap<RelId, Table>,
}

fn get_lines_from_chars(commands: String) -> Result<Vec<Line>, ()> {
    let lex_res = lexer::lex(&commands);

    println!("{lex_res:?}");

    match lex_res {
        Ok(lexic) => {
            let ast_res = parser::parse(&lexic);
            match ast_res {
                Ok(ast_vec) => Ok(ast_vec),
                Err(err) => {
                    err.print(&lexic, &commands);
                    Err(())
                }
            }
        }
        Err(e) => {
            e.print(&commands);
            Err(())
        }
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn input(self: &mut Engine, commands: String) {
        match get_lines_from_chars(String::from("\n") + &commands) {
            Ok(lines) => {
                for line in lines {
                    match self.ingest(line) {
                        Ok(()) => (),
                        Err(err) => {
                            println!("An error ocurred on the execution step: \n {err:?}");
                            break;
                        }
                    }
                }
            }
            Err(()) => (),
        }
    }

    pub fn ingest(self: &mut Engine, line: Line) -> Result<(), RuntimeError> {
        match line {
            Line::Relation(rel) => {
                let rel_id = rel.get_rel_id();
                let insertion_key = rel_id.clone();

                if let None = self.tables.get(&rel_id) {
                    self.tables.insert(insertion_key, Table::new(&rel_id));
                }

                if let Some(table) = self.tables.get_mut(&rel_id) {
                    table.add_rule(rel)?;
                }
            }
            Line::Query(rel) => {
                let rel_id = rel.get_rel_id();

                if let Some(table) = self.tables.get_mut(&rel_id) {
                    let query_res = table.get_contents(&rel.args)?;
                    draw_table(query_res);
                } else {
                    return Err(RuntimeError::RelationNotFound(rel_id));
                }
            }
            Line::TrueWhen(cond) => {
                let rel_id = cond.get_rel_id();
                let insertion_key = rel_id.clone();

                if let None = self.tables.get(&rel_id) {
                    self.tables.insert(insertion_key, Table::new(&rel_id));
                }

                if let Some(table) = self.tables.get_mut(&rel_id) {
                    table.add_conditional(cond);
                }
            }
        }

        Ok(())
    }
}

fn draw_table(matrix: Vec<Vec<Data>>) {
    if matrix.len() == 0 {
        println!("Empty Result");
        return;
    }

    let col_width = matrix.iter().fold(vec![0; matrix[0].len()], |acc, elm| {
        let mut ret = acc.clone();
        elm.iter().enumerate().for_each(|(i, e)| {
            let e_size = e.to_string().len();
            ret[i] = ret[i].max(e_size);
        });
        ret
    });

    for row in matrix {
        print!("(");
        for (i, elm) in row.iter().enumerate() {
            let representation = elm.to_string();
            print!("{representation}");

            for _ in 0..col_width[i] - representation.len() {
                print!(" ");
            }
            if i != row.len() - 1 {
                print!(", ")
            }
        }
        print!(")\n");
    }
}
