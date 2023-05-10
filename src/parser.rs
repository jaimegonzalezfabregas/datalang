struct RelName(String);

enum VarName {
    destructured_list(Vec<Expresion>),
    string(String),
}

enum VarLiteral {
    number(f64),
    string(String),
    list(Vec<Expresion>),
}

enum Statement {
    // resolvable to a bolean
    hypothetical(Statement, Statement),
    and(Statement, Statement),
    or(Statement, Statement),
    eq(RelName, RelName),
    not(Statement),
    arithmetic(Expresion, Expresion, fn(Expresion, Expresion) -> bool),
    extensional(RelName, Vec<VarLiteral>),
    intensional(RelName, Vec<RelName>, Statement),
}

enum Expresion {
    // resolvable to a value
    arithmetic(Expresion, Expresion, fn(Expresion, Expresion) -> Expresion),
    literal(Literal),
    var(VarName),
}
