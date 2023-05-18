use std::collections::HashSet;

use crate::parser::{expresion_reader::Expresion, data_reader::Data, var_literal_reader::VarLiteral};

pub fn add_expresions(a: Expresion, b: Expresion) -> Result<Expresion, String> {
    let op1 = match &a {
        Expresion::Arithmetic(_, _, _) => a.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("primer argumento no literalizable".into()),
    };
    let op2 = match &b {
        Expresion::Arithmetic(_, _, _) => b.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("segundo argumento no literalizable".into()),
    };

    match &(op1, op2) {
        (VarLiteral::Set(set_a), VarLiteral::Set(set_b)) => {
            let mut ret = HashSet::new();
            for it_a in set_a {
                for it_b in set_b {
                    ret.insert(match (it_a, it_b) {
                        (Data::Number(x), Data::Number(y)) => Data::Number(x + y),
                        (Data::String(x), Data::String(y)) => Data::String(x.clone() + y),
                        (Data::Array(x), Data::Array(y)) => {
                            Data::Array(x.iter().chain(y.iter()).map(|e| e.clone()).collect())
                        }
                        _ => return Err("cant operate on diferently typed literals".into()),
                    });
                }
            }
            Ok(Expresion::Literal(VarLiteral::Set(ret)))
        }

        _ => Err("cant operate on non literals".into()),
    }
}

pub fn sub_expresions(a: Expresion, b: Expresion) -> Result<Expresion, String> {
    let op1 = match &a {
        Expresion::Arithmetic(_, _, _) => a.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("primer argumento no literalizable".into()),
    };
    let op2 = match &b {
        Expresion::Arithmetic(_, _, _) => b.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("segundo argumento no literalizable".into()),
    };

    match &(op1, op2) {
        (VarLiteral::Set(set_a), VarLiteral::Set(set_b)) => {
            let mut ret = HashSet::new();
            for it_a in set_a {
                for it_b in set_b {
                    ret.insert(match (it_a, it_b) {
                        (Data::Number(x), Data::Number(y)) => Data::Number(x - y),
                        (Data::String(_), Data::String(_)) => {
                            return Err("cant substract strings".into())
                        }
                        (Data::Array(_), Data::Array(_)) => {
                            return Err("cant substract arrays".into())
                        }
                        _ => return Err("cant operate on diferently typed literals".into()),
                    });
                }
            }
            Ok(Expresion::Literal(VarLiteral::Set(ret)))
        }

        _ => Err("cant operate on non literals".into()),
    }
}

pub fn mul_expresions(a: Expresion, b: Expresion) -> Result<Expresion, String> {
    let op1 = match &a {
        Expresion::Arithmetic(_, _, _) => a.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("primer argumento no literalizable".into()),
    };
    let op2 = match &b {
        Expresion::Arithmetic(_, _, _) => b.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("segundo argumento no literalizable".into()),
    };

    match &(op1, op2) {
        (VarLiteral::Set(set_a), VarLiteral::Set(set_b)) => {
            let mut ret = HashSet::new();
            for it_a in set_a {
                for it_b in set_b {
                    ret.insert(match (it_a, it_b) {
                        (Data::Number(x), Data::Number(y)) => Data::Number(x * y),
                        (Data::String(_), Data::String(_)) => {
                            return Err("cant multiply strings".into())
                        }
                        (Data::Array(_), Data::Array(_)) => {
                            return Err("cant multiply arrays".into())
                        }
                        _ => return Err("cant operate on diferently typed literals".into()),
                    });
                }
            }
            Ok(Expresion::Literal(VarLiteral::Set(ret)))
        }

        _ => Err("cant operate on non literals".into()),
    }
}

pub fn div_expresions(a: Expresion, b: Expresion) -> Result<Expresion, String> {
    let op1 = match &a {
        Expresion::Arithmetic(_, _, _) => a.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("primer argumento no literalizable".into()),
    };
    let op2 = match &b {
        Expresion::Arithmetic(_, _, _) => b.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("segundo argumento no literalizable".into()),
    };

    match &(op1, op2) {
        (VarLiteral::Set(set_a), VarLiteral::Set(set_b)) => {
            let mut ret = HashSet::new();
            for it_a in set_a {
                for it_b in set_b {
                    ret.insert(match (it_a, it_b) {
                        (Data::Number(x), Data::Number(y)) => Data::Number(x / y),
                        (Data::String(_), Data::String(_)) => {
                            return Err("cant divide strings".into())
                        }
                        (Data::Array(_), Data::Array(_)) => return Err("cant divide arrays".into()),
                        _ => return Err("cant operate on diferently typed literals".into()),
                    });
                }
            }
            Ok(Expresion::Literal(VarLiteral::Set(ret)))
        }

        _ => Err("cant operate on non literals".into()),
    }
}

pub fn eq_expresions(a: Expresion, b: Expresion) -> Result<bool, String> {
    let op1 = match &a {
        Expresion::Arithmetic(_, _, _) => a.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("primer argumento no literalizable".into()),
    };
    let op2 = match &b {
        Expresion::Arithmetic(_, _, _) => b.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("segundo argumento no literalizable".into()),
    };

    Ok(op1.eq(&op2) && op2.eq(&op1)) // hashset eq is a subset check
}

pub fn lt_expresions(a: Expresion, b: Expresion) -> Result<bool, String> {
    let op1 = match &a {
        Expresion::Arithmetic(_, _, _) => a.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("primer argumento no literalizable".into()),
    };
    let op2 = match &b {
        Expresion::Arithmetic(_, _, _) => b.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("segundo argumento no literalizable".into()),
    };

    match (
        op1.get_element_if_singleton()?,
        op2.get_element_if_singleton()?,
    ) {
        (Data::Number(x), Data::Number(y)) => Ok(x < y),
        (Data::String(x), Data::String(y)) => Ok(x < y),
        (Data::Array(_), Data::Array(_)) => Err("cant compare arrays".into()),
        _ => Err("cant operate on diferently typed literals".into()),
    }
}

pub fn gt_expresions(a: Expresion, b: Expresion) -> Result<bool, String> {
    let op1 = match &a {
        Expresion::Arithmetic(_, _, _) => a.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("primer argumento no literalizable".into()),
    };
    let op2 = match &b {
        Expresion::Arithmetic(_, _, _) => b.literalize()?,
        Expresion::Literal(l) => l.clone(),
        _ => return Err("segundo argumento no literalizable".into()),
    };

    match (
        op1.get_element_if_singleton()?,
        op2.get_element_if_singleton()?,
    ) {
        (Data::Number(x), Data::Number(y)) => Ok(x > y),
        (Data::String(x), Data::String(y)) => Ok(x > y),
        (Data::Array(_), Data::Array(_)) => Err("cant compare arrays".into()),
        _ => Err("cant operate on diferently typed literals".into()),
    }
}

pub fn lte_expresions(a: Expresion, b: Expresion) -> Result<bool, String> {
    Ok(!gt_expresions(a, b)?)
}

pub fn gte_expresions(a: Expresion, b: Expresion) -> Result<bool, String> {
    Ok(!lt_expresions(a, b)?)
}
