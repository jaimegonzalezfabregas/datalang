use crate::syntax::VarLiteral;

#[derive(Debug, Clone)]
enum Command {
    IsTrueThat(Vec<VarLiteral>),
    IsFalseThat(Vec<VarLiteral>),
}
use Command::*;

#[derive(Debug, Clone)]
pub struct Table {
    width: usize,
    history: Vec<Command>,
}

impl Table {
    pub fn new(width: &usize) -> Self {
        Self {
            width: *width,
            history: vec![],
        }
    }

    pub fn add(self: &mut Table, row: Vec<VarLiteral>) -> Result<(), String> {
        if row.len() != self.width {
            Err("Cant add to a table a row with mismatching number of columns".into())
        } else {
            self.history.push(IsTrueThat(row));
            Ok(())
        }
    }

    pub fn remove(self: &mut Table, row: Vec<VarLiteral>) -> Result<(), String> {
        if row.len() != self.width {
            Err("Cant remove to a table a row with mismatching number of columns".into())
        } else {
            self.history.push(IsFalseThat(row));
            Ok(())
        }
    }

    pub fn contains_eq(self: &Table, row: Vec<VarLiteral>) -> Result<bool, String> {
        if row.len() != self.width {
            Err("Cant compare a row with mismatching number of columns".into())
        } else {
            for command in self.history.iter().rev() {
                let (inner, ret) = match command {
                    IsFalseThat(inner) => (inner, false),
                    IsTrueThat(inner) => (inner, true),
                };

                let mut c = 0;
                for (a, b) in row.iter().zip(inner) {
                    if a.set_eq(b) {
                        c += 1;
                    }
                }
                if c == row.len() {
                    return Ok(ret);
                }
            }
            Ok(false)
        }
    }

    pub fn contains_subset_of(self: &Table, superSet: Vec<VarLiteral>) -> Result<bool, String> {
        if superSet.len() != self.width {
            Err("Cant compare a row with mismatching number of columns".into())
        } else {
            for command in self.history.iter().rev() {
                let (inner, ret) = match command {
                    IsFalseThat(inner) => (inner, false),
                    IsTrueThat(inner) => (inner, true),
                };

                let mut c = 0;
                for (a, b) in superSet.iter().zip(inner) {
                    if a.contains_set(b) {
                        c += 1;
                    }
                }
                if c == superSet.len() {
                    return Ok(ret);
                }
            }
            Ok(false)
        }
    }

    pub fn contains_superset_of(self: &Table, subSet: Vec<VarLiteral>) -> Result<bool, String> {
        if subSet.len() != self.width {
            Err("Cant compare a row with mismatching number of columns".into())
        } else {
            for command in self.history.iter().rev() {
                let (inner, ret) = match command {
                    IsFalseThat(inner) => (inner, false),
                    IsTrueThat(inner) => (inner, true),
                };

                let mut c = 0;
                for (a, b) in subSet.iter().zip(inner) {
                    if b.contains_set(a) {
                        c += 1;
                    }
                }
                if c == subSet.len() {
                    return Ok(ret);
                }
            }
            Ok(false)
        }
    }

    fn set_of_nth_column(self: &Table, n: usize) -> VarLiteral {
        let mut ret = VarLiteral::EmptySet;
        for command in &self.history {
            match command {
                IsTrueThat(v) => match v[n] {
                    VarLiteral::FullSet => ret = VarLiteral::FullSet,
                    VarLiteral::Set(_) => todo!(),
                    VarLiteral::AntiSet(_) => todo!(),
                    VarLiteral::EmptySet => (),
                },
                IsFalseThat(v) => match v[n] {
                    VarLiteral::FullSet => ret = VarLiteral::EmptySet,
                    VarLiteral::Set(_) => todo!(),
                    VarLiteral::AntiSet(_) => todo!(),
                    VarLiteral::EmptySet => (),
                },
            }
        }
        ret
    }
}
