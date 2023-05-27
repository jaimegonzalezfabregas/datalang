pub mod operations;
pub mod table;
pub mod var_context;

use crate::{
    lexer,
    parser::{
        self, assumption_reader::Assumption, defered_relation_reader::DeferedRelation,
        inmediate_relation_reader::InmediateRelation, line_reader::Line, Relation,
    },
};
use std::{collections::HashMap, fmt, vec};

use self::{
    table::{truth::Truth, Table},
    var_context::VarContext,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd)]
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

fn get_lines_from_chars(raw_commands: String, debug_print: bool) -> Result<Vec<Line>, String> {
    let commands = String::from("\n") + &raw_commands;

    let lex_res = lexer::lex(&commands);

    if debug_print {
        println!("{lex_res:?}");
    }
    match lex_res {
        Ok(lexic) => {
            let ast_res = parser::parse(&lexic, debug_print);
            match ast_res {
                Ok(ast_vec) => {
                    if debug_print {
                        println!("{ast_vec:?}");
                    }
                    Ok(ast_vec)
                }
                Err(err) => Err(err.print(&lexic, &commands)),
            }
        }
        Err(e) => Err(e.print(&commands)),
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::new();
        for (_, table) in self.tables.iter() {
            ret += &format!("{table}");
        }

        write!(f, "{}", ret)
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn input(self: &mut Engine, commands: String, debug_print: bool) -> String {
        let mut ret = String::new();
        match get_lines_from_chars(commands, debug_print) {
            Ok(lines) => {
                for line in lines {
                    if debug_print {
                        println!("\nexecuting: {line}");
                    }
                    match self.ingest_line(line, String::new(), debug_print) {
                        Ok(Some(output)) => {
                            let mut sorted_output = output.clone();
                            sorted_output.sort();
                            if debug_print {
                                ret += &format!("{sorted_output:?}");
                            } else {
                                ret += &draw_table(sorted_output)
                            }
                        }
                        Ok(None) => (),
                        Err(err) => {
                            ret += &format!("An error ocurred on the execution step: \n {err:?}");
                            break;
                        }
                    }
                    if debug_print {
                        println!("\nengine_state: {self:?}");
                    }
                }
            }
            Err(err) => return err,
        }
        ret
    }

    pub fn query(
        &self,
        query: &DeferedRelation,
        context: &VarContext,
        caller_depth_map: &HashMap<RelId, usize>,
        debug_margin: String,
        debug_print: bool,
    ) -> Result<Vec<Truth>, RuntimeError> {
        if debug_print {
            println!("{debug_margin}query {query}, with context: {context:?}")
        }
        let rel_id = query.get_rel_id();
        let mut hypothetical_engine = self.clone();

        for assumption in &query.assumptions {
            hypothetical_engine.ingest_assumption(assumption, context)?;
        }

        if let Some(table) = hypothetical_engine.tables.get(&rel_id) {
            Ok(table.get_filtered_truths(
                &query.apply(context)?,
                &hypothetical_engine,
                caller_depth_map,
                debug_margin.to_owned() + "   ",
                debug_print,
            )?)
        } else {
            Err(RuntimeError::RelationNotFound(rel_id))
        }
    }

    fn ingest_assumption(
        self: &mut Engine,
        assumption: &Assumption,
        context: &VarContext,
    ) -> Result<(), RuntimeError> {
        match assumption {
            Assumption::Conditional(cond) => {
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
            Assumption::Update(_) => todo!(),
            Assumption::RelationInmediate(rel) => {
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
            Assumption::RelationDefered(d_rel) => {
                let mut data = vec![];
                for exp in &d_rel.args {
                    data.push(exp.literalize(context)?);
                }

                self.ingest_assumption(
                    &Assumption::RelationInmediate(InmediateRelation {
                        negated: false,
                        rel_name: d_rel.rel_name.to_owned(),
                        args: data,
                    }),
                    context,
                )
            }
        }
    }

    pub fn ingest_line(
        self: &mut Engine,
        line: Line,
        debug_margin: String,
        debug_print: bool,
    ) -> Result<Option<Vec<Truth>>, RuntimeError> {
        match line {
            Line::Query(q) => Ok(Some(self.query(
                &q,
                &VarContext::new(),
                &HashMap::new(),
                debug_margin.to_owned() + "   ",
                debug_print,
            )?)),
            Line::Assumption(assumption) => {
                self.ingest_assumption(&assumption, &VarContext::new())?;
                Ok(None)
            }
        }
    }

    pub fn get_table(&self, rel_id: RelId) -> Option<&Table> {
        self.tables.get(&rel_id)
    }
}

fn draw_table(matrix: Vec<Truth>) -> String {
    if matrix.len() == 0 {
        return "\nEmpty Result\n".into();
    }
    let mut ret = String::from("\n");
    let column_count = matrix[0].get_width();

    let col_width = matrix.iter().fold(vec![0; column_count], |acc, elm| {
        let mut ret = acc.clone();
        elm.get_data().iter().enumerate().for_each(|(i, e)| {
            let e_size = e.to_string().len();
            ret[i] = ret[i].max(e_size);
        });
        ret
    });

    for truth in matrix {
        ret += &format!("(");
        for (i, elm) in truth.get_data().iter().enumerate() {
            let representation = elm.to_string();
            ret += &format!("{representation}");

            for _ in 0..col_width[i] - representation.len() {
                ret += &format!(" ");
            }
            if i != column_count - 1 {
                ret += &format!(", ")
            }
        }
        ret += &format!(")\n");
    }
    ret
}
