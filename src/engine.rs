pub mod operations;
pub mod recursion_tally;
pub mod relation;
pub mod truth_list;
pub mod var_context;
pub mod var_context_universe;

use print_macros::*;

use crate::{
    lexer,
    parser::{
        self, assumption_node::Assumption, defered_relation_node::DeferedRelation,
        inmediate_relation_node::InmediateRelation, line_node::Line, HasRelId,
    },
};
use std::{collections::BTreeMap, fmt, vec};

use self::{
    recursion_tally::RecursionTally,
    relation::{truth::Truth, Relation},
    truth_list::TruthList,
    var_context::VarContext,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct RelId {
    pub identifier: String,
    pub column_count: usize,
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
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
    recursion_limit: usize,
    tables: BTreeMap<RelId, Relation>,
}

use std::hash::Hash;
impl Hash for Engine {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.recursion_limit.hash(state);
        self.tables.hash(state);
    }
}

fn get_lines_from_chars(raw_commands: String) -> Result<Vec<Line>, String> {
    let commands = String::from("\n") + &raw_commands;

    let lex_res = lexer::lex(&commands);

    printparse!("{:?}", lex_res);

    match lex_res {
        Ok(lexic) => {
            let ast_res = parser::parse(&lexic);
            match ast_res {
                Ok(ast_vec) => {
                    printparse!("{:?}", ast_vec);
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
            recursion_limit: 5,
            tables: BTreeMap::new(),
        }
    }

    pub fn set_recursion_limit(&mut self, rl: usize) {
        self.recursion_limit = rl;
    }

    pub fn input(self: &mut Engine, commands: String) -> String {
        let mut ret = String::new();
        match get_lines_from_chars(commands) {
            Ok(lines) => {
                for line in lines {
                    printdev!("\nexecuting: {}", line);

                    match self.ingest_line(line) {
                        Ok(Some(output)) => {
                            let mut sorted_output = output.to_vector();
                            sorted_output.sort();
                            ret += &draw_table(sorted_output)
                        }
                        Ok(None) => (),
                        Err(err) => {
                            ret += &format!("An error ocurred on the execution step: \n {err:?}");
                            break;
                        }
                    }
                }
            }
            Err(err) => return err,
        }
        ret
    }

    pub fn get_relation(&mut self, rel_id: RelId) -> Relation {
        self.tables[&rel_id].to_owned()
    }

    pub fn query(
        &self,
        query: &DeferedRelation,
        context: &VarContext,
        recursion_tally: &RecursionTally,
    ) -> Result<TruthList, String> {
        printprocess!("query {}", query);

        let rel_id = query.get_rel_id();
        let mut hypothetical_engine = self.clone();

        for assumption in &query.assumptions {
            hypothetical_engine.ingest_assumption(assumption, context)?;
        }

        Ok(hypothetical_engine
            .get_relation(rel_id)
            .get_filtered_truths(&query, &hypothetical_engine, recursion_tally)?)
    }

    fn ingest_assumption(
        self: &mut Engine,
        assumption: &Assumption,
        context: &VarContext,
    ) -> Result<(), String> {
        match assumption {
            Assumption::Conditional(cond) => {
                let rel_id = cond.get_rel_id();
                let insertion_key = rel_id.clone();

                if let None = self.tables.get(&rel_id) {
                    self.tables.insert(insertion_key, Relation::new(&rel_id));
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
                    self.tables.insert(insertion_key, Relation::new(&rel_id));
                }

                if let Some(relation) = self.tables.get_mut(&rel_id) {
                    relation.add_truth(rel.to_owned())?;
                }
                Ok(())
            }
            Assumption::RelationDefered(d_rel) => {
                let mut datas = vec![];
                for exp in &d_rel.args {
                    match exp.literalize(context) {
                        Ok(data) => datas.push(data),
                        Err(msg) => {
                            return Err("Cant assume a relation with unliteralizable items: "
                                .to_string()
                                + &msg)
                        }
                    }
                }

                self.ingest_assumption(
                    &Assumption::RelationInmediate(InmediateRelation {
                        negated: false,
                        rel_name: d_rel.rel_name.to_owned(),
                        args: datas,
                    }),
                    context,
                )
            }
        }
    }

    pub fn ingest_line(self: &mut Engine, line: Line) -> Result<Option<TruthList>, RuntimeError> {
        match line {
            Line::Query(q) => Ok(Some(self.query(
                &q,
                &VarContext::new(),
                &RecursionTally::new(self.recursion_limit),
            )?)),
            Line::Assumption(assumption) => {
                self.ingest_assumption(&assumption, &VarContext::new())?;
                Ok(None)
            }
            Line::Comment(_) => Ok(None),
        }
    }

    pub fn get_table(&self, rel_id: RelId) -> Option<&Relation> {
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
