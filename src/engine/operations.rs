use crate::parser::data_reader::Data;

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
    match (op2, res) {
        (Data::Number(x), Data::Number(r)) => Ok(Data::Number(r - x)),
        (Data::String(x), Data::String(r)) => {
            if r.ends_with(&x) {
                Ok(Data::String(r[0..r.len() - x.len()].to_string()))
            } else {
                Err("cant operate on diferently typed literals".into())
            }
        }
        (Data::Array(x), Data::Array(r)) => todo!(),
        _ => Err("cant operate on diferently typed literals".into()),
    }
}

pub fn add_reverse_op2(op1: Data, res: Data) -> Result<Data, String> {
    match (op1, res) {
        (Data::Number(x), Data::Number(r)) => Ok(Data::Number(r - x)),
        (Data::String(x), Data::String(r)) => {
            if r.starts_with(&x) {
                Ok(Data::String(r[x.len()..].to_string()))
            } else {
                Err("cant operate on diferently typed literals".into())
            }
        }
        (Data::Array(x), Data::Array(r)) => todo!(),
        _ => Err("cant operate on diferently typed literals".into()),
    }
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
