//! Differential harness for + - * / % **, and the comparison operators.
use rexx_num::{ArithError, CompareOp, DivOp, Number, compare};

/// Maps a case-file operator token to a comparison, or `None` if it is one
/// of the arithmetic operators instead. `\=`, `<>` and `><` all mean
/// `NotEqual` in the interpreter, so all three map here.
fn compare_op(op: &str) -> Option<CompareOp> {
    use CompareOp::*;
    Some(match op {
        "=" => Equal,
        "\\=" | "<>" | "><" => NotEqual,
        ">" => Greater,
        "<" => Less,
        ">=" => GreaterEqual,
        "<=" => LessEqual,
        "==" => StrictEqual,
        "\\==" => StrictNotEqual,
        ">>" => StrictGreater,
        "<<" => StrictLess,
        ">>=" => StrictGreaterEqual,
        "<<=" => StrictLessEqual,
        _ => return None,
    })
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: muldiv <file>");
    let text = std::fs::read_to_string(&path).expect("readable input file");
    for line in text.lines() {
        let mut parts = line.split('|');
        let digits: u64 = parts.next().unwrap().parse().unwrap();
        let a = parts.next().unwrap();
        let op = parts.next().unwrap();
        let b = parts.next().unwrap();
        let out = if let Some(cmp_op) = compare_op(op) {
            // The case-file format carries no FUZZ column, so this always
            // runs at FUZZ 0; NUMERIC FUZZ is exercised separately, since it
            // cannot be expressed in `digits|a|op|b`.
            match compare(a, b, digits, 0, cmp_op) {
                Ok(true) => "1".to_string(),
                Ok(false) => "0".to_string(),
                Err(e) => format!("<E{}>", ArithError::code(e)),
            }
        } else {
            match (Number::parse(a), Number::parse(b)) {
                (Some(x), Some(y)) => {
                    let r = match op {
                        "+" => x.add(&y, digits),
                        "-" => x.sub(&y, digits),
                        "*" => x.mul(&y, digits),
                        "/" => x.div(&y, digits, DivOp::Divide),
                        "%" => x.div(&y, digits, DivOp::IntegerDivide),
                        "//" => x.div(&y, digits, DivOp::Remainder),
                        "**" => x.pow(&y, digits),
                        other => panic!("unknown operator {other}"),
                    };
                    match r {
                        Ok(v) => v.format(digits),
                        Err(e) => format!("<E{}>", ArithError::code(e)),
                    }
                }
                _ => "<E41>".to_string(),
            }
        };
        println!("{line}={out}");
    }
}
