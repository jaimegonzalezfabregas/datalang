pub mod operations;
pub mod table;
pub mod var_context;

use crate::{
    lexer,
    parser::{
        self, asumption_reader::Asumption, defered_relation_reader::DeferedRelation,
        inmediate_relation_reader::InmediateRelation, line_reader::Line, Relation,
    },
};
use std::{collections::HashMap, vec};

use self::{
    table::{truth::Truth, Table},
    var_context::VarContext,
};

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
    NoContextWhenNeeded,
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

fn get_lines_from_chars(commands: String) -> Result<Vec<Line>, String> {
    let lex_res = lexer::lex(&commands);

    println!("{lex_res:?}");

    match lex_res {
        Ok(lexic) => {
            let ast_res = parser::parse(&lexic);
            match ast_res {
                Ok(ast_vec) => Ok(ast_vec),
                Err(err) => Err(err.print(&lexic, &commands)),
            }
        }
        Err(e) => Err(e.print(&commands)),
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn input(self: &mut Engine, commands: String) -> String {
        let ret = String::new();
        match get_lines_from_chars(String::from("\n") + &commands) {
            Ok(lines) => {
                for line in lines {
                    match self.ingest_line(line) {
                        Ok(Some(output)) => draw_table(output),
                        Ok(None) => (),
                        Err(err) => {
                            println!("An error ocurred on the execution step: \n {err:?}");
                            break;
                        }
                    }
                }
            }
            Err(err) => return err,
        }
        ret
    }

    pub fn query(
        &self,
        query: DeferedRelation,
        context: &VarContext,
    ) -> Result<Vec<Truth>, RuntimeError> {
        let rel_id = query.get_rel_id();
        let mut hypothetical_engine = self.clone();

        for assumption in &query.asumptions {
            hypothetical_engine.ingest_asumption(assumption, context)?;
        }

        if let Some(table) = hypothetical_engine.tables.get(&rel_id) {
            Ok(table.get_truths(&query, &hypothetical_engine)?)
        } else {
            Err(RuntimeError::RelationNotFound(rel_id))
        }
    }

    fn ingest_asumption(
        self: &mut Engine,
        asumption: &Asumption,
        context: &VarContext,
    ) -> Result<(), RuntimeError> {
        match asumption {
            Asumption::Conditional(cond) => {
                let rel_id = cond.get_rel_id();
                let insertion_key = rel_id.clone();

                if let None = self.tables.get(&rel_id) {
                    self.tables.insert(insertion_key, Table::new(&rel_id));
                }

                if let Some(table) = self.tables.get_mut(&rel_id) {
                    table.add_conditional(cond.to_owned())?;
                }
                Ok(())
            }
            Asumption::Update(_) => todo!(),
            Asumption::RelationInmediate(rel) => {
                let rel_id = rel.get_rel_id();
                let insertion_key = rel_id.clone();

                if let None = self.tables.get(&rel_id) {
                    self.tables.insert(insertion_key, Table::new(&rel_id));
                }

                if let Some(table) = self.tables.get_mut(&rel_id) {
                    table.add_rule(rel.to_owned())?;
                }
                Ok(())
            }
            Asumption::RelationDefered(d_rel) => {
                let mut data = vec![];
                for exp in &d_rel.args {
                    data.push(exp.literalize(context)?);
                }

                self.ingest_asumption(
                    &Asumption::RelationInmediate(InmediateRelation {
                        negated: false,
                        rel_name: d_rel.rel_name.to_owned(),
                        args: data,
                    }),
                    context,
                )
            }
        }
    }

    pub fn ingest_line(self: &mut Engine, line: Line) -> Result<Option<Vec<Truth>>, RuntimeError> {
        match line {
            Line::Query(q) => Ok(Some(self.query(q, &VarContext::new())?)),
            Line::Asumption(asumption) => {
                self.ingest_asumption(&asumption, &VarContext::new())?;
                Ok(None)
            }
        }
    }

    pub fn get_table(&self, rel_id: RelId) -> Option<&Table> {
        self.tables.get(&rel_id)
    }
}

fn draw_table(matrix: Vec<Truth>) {
    let column_count = matrix[0].get_width();
    if matrix.len() == 0 {
        println!("Empty Result");
        return;
    }

    let col_width = matrix.iter().fold(vec![0; column_count], |acc, elm| {
        let mut ret = acc.clone();
        elm.get_data().iter().enumerate().for_each(|(i, e)| {
            let e_size = e.to_string().len();
            ret[i] = ret[i].max(e_size);
        });
        ret
    });

    for truth in matrix {
        print!("(");
        for (i, elm) in truth.get_data().iter().enumerate() {
            let representation = elm.to_string();
            print!("{representation}");

            for _ in 0..col_width[i] - representation.len() {
                print!(" ");
            }
            if i != column_count - 1 {
                print!(", ")
            }
        }
        print!(")\n");
    }
}
