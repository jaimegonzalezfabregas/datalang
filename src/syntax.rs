#[derive(Debug, Clone)]
pub struct RelName(pub String);

#[derive(Debug, Clone)]
pub enum VarName {
    DestructuredArray(Vec<Expresion>),
    Direct(String),
}

#[derive(Debug, Clone)]
pub enum Data {
    Number(f64),
    String(String),
    Array(Vec<Data>),
}

#[derive(Debug, Clone)]
pub enum VarLiteral {
    EmptySet,
    FullSet,
    Set(Vec<Data>),
    AntiSet(Vec<Data>),
}

impl VarLiteral {
    fn add(self: &mut VarLiteral, d: Data) -> Result<(), String> {
        match self {
            VarLiteral::EmptySet => *self = VarLiteral::Set(vec![d]),
            VarLiteral::FullSet => (),
            VarLiteral::Set(v) => v.push(d),
            VarLiteral::AntiSet(v) => v.retain(|e| !d.eq(e)),
        }

        Ok(())
    }
    fn remove(self: &mut VarLiteral, d: Data) -> Result<(), String> {
        match self {
            VarLiteral::EmptySet => (),
            VarLiteral::FullSet => *self = VarLiteral::AntiSet(vec![d]),
            VarLiteral::AntiSet(v) => v.push(d),
            VarLiteral::Set(v) => v.retain(|e| !d.eq(e)),
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    // resolvable to a bolean
    Hypothetical(Vec<Line>, Box<Statement>), // TODO
    And(Box<Statement>, Box<Statement>),
    Or(Box<Statement>, Box<Statement>),
    Not(Box<Statement>),
    Arithmetic(
        Expresion,
        Expresion,
        fn(Expresion, Expresion) -> Result<bool, String>,
    ),
    Relation(RelName, Vec<Expresion>),
    Empty,
}

#[derive(Debug, Clone)]
pub enum Line {
    CreateRelation(RelName, Vec<VarLiteral>),
    ForgetRelation(RelName, Vec<VarLiteral>),
    TrueWhen(Box<Statement>, Box<Statement>),
    Query(RelName, Vec<Expresion>),
}

#[derive(Debug, Clone)]
pub enum Expresion {
    // resolvable to a value
    Arithmetic(
        Box<Expresion>,
        Box<Expresion>,
        fn(Expresion, Expresion) -> Result<Expresion, String>,
    ),
    Literal(VarLiteral),
    RestOfList(VarName),
    Var(VarName),
    Empty,
}

impl Expresion {
    pub fn literalize(self: &Expresion) -> Result<VarLiteral, String> {
        let ret = match self.clone() {
            Expresion::Arithmetic(a, b, f) => {
                if let Expresion::Literal(VarLiteral::FullSet) = *a {
                    Ok(VarLiteral::FullSet)
                } else if let Expresion::Literal(VarLiteral::FullSet) = *a {
                    Ok(VarLiteral::FullSet)
                } else {
                    f(*a, *b)?.literalize()
                }
            }
            Expresion::Literal(e) => Ok(e),
            _ => Err(format!("no se ha podido literalizar: {:?}", self)),
        };

        return ret;
    }
}

impl VarLiteral {
    pub fn get_element_if_singleton(&self) -> Result<Data, String> {
        match self {
            VarLiteral::FullSet | VarLiteral::EmptySet | VarLiteral::AntiSet(_) => {
                Err("Not a singleton".into())
            }
            VarLiteral::Set(e) => {
                if e.len() == 1 {
                    return Ok(e[0].clone());
                } else {
                    Err("Not a singleton".into())
                }
            }
        }
    }

    pub fn set_eq(&self, other: &VarLiteral) -> bool {
        match (self, other) {
            (VarLiteral::FullSet, VarLiteral::FullSet) => true,
            (VarLiteral::EmptySet, VarLiteral::EmptySet) => true,
            (VarLiteral::Set(a), VarLiteral::Set(b)) => {
                for (a_it, b_it) in a.iter().zip(b) {
                    if !a_it.eq(b_it) {
                        return false;
                    }
                }
                true
            }
            (VarLiteral::AntiSet(a), VarLiteral::AntiSet(b)) => {
                for (a_it, b_it) in a.iter().zip(b) {
                    if !a_it.eq(b_it) {
                        return false;
                    }
                }
                true
            }

            (_, _) => false,
        }
    }

    pub fn contains_set(&self, contained_set: &VarLiteral) -> bool {
        match (contained_set, self) {
            (_, VarLiteral::FullSet) => true,
            (_, VarLiteral::EmptySet) => false,
            (VarLiteral::EmptySet, _) => true,
            (VarLiteral::FullSet, _) => false,

            (VarLiteral::Set(contained), VarLiteral::Set(_) | VarLiteral::AntiSet(_)) => {
                for it_a in contained {
                    if !self.contains_element(it_a) {
                        return false;
                    }
                }
                true
            }
            (VarLiteral::AntiSet(_), VarLiteral::Set(_)) => false,
            (VarLiteral::AntiSet(not_in_contained), VarLiteral::AntiSet(not_in_container)) => {
                for it_a in not_in_contained {
                    let mut found = false;
                    for it_b in not_in_container {
                        if it_a.eq(it_b) {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return false;
                    }
                }
                true
            }
        }
    }

    fn contains_element(&self, data: &Data) -> bool {
        match self {
            VarLiteral::FullSet => true,
            VarLiteral::EmptySet => false,
            VarLiteral::Set(vec) => {
                for it in vec {
                    if it.eq(data) {
                        return true;
                    }
                }
                false
            }
            VarLiteral::AntiSet(vec) => {
                for it in vec {
                    if it.eq(data) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl Data {
    fn eq(&self, other: &Data) -> bool {
        match (self, other) {
            (Data::Number(a), Data::Number(b)) => a == b,
            (Data::String(a), Data::String(b)) => a == b,
            (Data::Array(a), Data::Array(b)) => {
                if a.len() != b.len() {
                    false
                } else {
                    let mut c = 0;
                    for (it_a, it_b) in a.iter().zip(b) {
                        if it_a.eq(it_b) {
                            return false;
                        } else {
                            return false;
                        }
                    }
                    true
                }
            }
            _ => false,
        }
    }
}
