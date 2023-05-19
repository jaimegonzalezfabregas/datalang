use crate::parser::{data_reader::Data, expresion_reader::Expresion};

pub fn add_direct(op1: Data, op2: Data) -> Result<Data, String> {
    Ok(match (op1, op2) {
        (Data::Number(x), Data::Number(y)) => Data::Number(x + y),
        (Data::String(x), Data::String(y)) => Data::String(x.clone() + &y),
        (Data::Array(x), Data::Array(y)) => {
            Data::Array(x.iter().chain(y.iter()).map(|e| e.clone()).collect())
        }
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn add_reverse_op1(op2: Data, res: Data) -> Result<Data, String> {
    Ok(match (op2, res) {
        (Data::Number(x), Data::Number(r)) => Data::Number(r - x),
        (Data::String(x), Data::String(r)) => todo!(),
        (Data::Array(x), Data::Array(r)) => todo!(),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn add_reverse_op2(op1: Data, res: Data) -> Result<Data, String> {
    Ok(match (op1, res) {
        (Data::Number(x), Data::Number(r)) => Data::Number(r - x),
        (Data::String(x), Data::String(r)) => todo!(),
        (Data::Array(x), Data::Array(r)) => todo!(),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn substract_direct(op1: Data, op2: Data) -> Result<Data, String> {
    Ok(match (op1, op2) {
        (Data::Number(x), Data::Number(y)) => Data::Number(x - y),
        (Data::String(_), Data::String(_)) => return Err("cant substract strings".into()),
        (Data::Array(_), Data::Array(_)) => return Err("cant substract arrays".into()),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn substract_reverse_op1(op2: Data, res: Data) -> Result<Data, String> {
    Ok(match (op2, res) {
        (Data::Number(x), Data::Number(r)) => Data::Number(x - r),
        (Data::String(_), Data::String(_)) => return Err("cant substract strings".into()),
        (Data::Array(_), Data::Array(_)) => return Err("cant substract arrays".into()),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn substract_reverse_op2(op1: Data, res: Data) -> Result<Data, String> {
    Ok(match (op1, res) {
        (Data::Number(x), Data::Number(r)) => Data::Number(x + r),
        (Data::String(_), Data::String(_)) => return Err("cant substract strings".into()),
        (Data::Array(_), Data::Array(_)) => return Err("cant substract arrays".into()),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn multiply_direct(op1: Data, op2: Data) -> Result<Data, String> {
    Ok(match (op1, op2) {
        (Data::Number(x), Data::Number(y)) => Data::Number(x * y),
        (Data::String(_), Data::String(_)) => return Err("cant multiply strings".into()),
        (Data::Array(_), Data::Array(_)) => return Err("cant multiply arrays".into()),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn multiply_reverse_op1(op2: Data, res: Data) -> Result<Data, String> {
    Ok(match (op2, res) {
        (Data::Number(x), Data::Number(r)) => Data::Number(x / r),
        (Data::String(_), Data::String(_)) => return Err("cant multiply strings".into()),
        (Data::Array(_), Data::Array(_)) => return Err("cant multiply arrays".into()),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn multiply_reverse_op2(op1: Data, res: Data) -> Result<Data, String> {
    Ok(match (op1, res) {
        (Data::Number(x), Data::Number(r)) => Data::Number(x / r),
        (Data::String(_), Data::String(_)) => return Err("cant multiply strings".into()),
        (Data::Array(_), Data::Array(_)) => return Err("cant multiply arrays".into()),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn divide_direct(op1: Data, op2: Data) -> Result<Data, String> {
    Ok(match (op1, op2) {
        (Data::Number(x), Data::Number(y)) => Data::Number(x / y),
        (Data::String(_), Data::String(_)) => return Err("cant divide strings".into()),
        (Data::Array(_), Data::Array(_)) => return Err("cant divide arrays".into()),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn divide_reverse_op1(op2: Data, res: Data) -> Result<Data, String> {
    Ok(match (op2, res) {
        (Data::Number(x), Data::Number(r)) => Data::Number(x * r),
        (Data::String(_), Data::String(_)) => return Err("cant divide strings".into()),
        (Data::Array(_), Data::Array(_)) => return Err("cant divide arrays".into()),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
}

pub fn divide_reverse_op2(op1: Data, res: Data) -> Result<Data, String> {
    Ok(match (op1, res) {
        (Data::Number(x), Data::Number(r)) => Data::Number(x / r),
        (Data::String(_), Data::String(_)) => return Err("cant divide strings".into()),
        (Data::Array(_), Data::Array(_)) => return Err("cant divide arrays".into()),
        _ => return Err("cant operate on diferently typed literals".into()),
    })
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

    match (op1, op2) {
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

    match (op1, op2) {
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
