//! Differential harness for + - * / % and //.
use rexx_num::{ArithError, DivOp, Number};

fn main() {
    let path = std::env::args().nth(1).expect("usage: muldiv <file>");
    let text = std::fs::read_to_string(&path).expect("readable input file");
    for line in text.lines() {
        let mut parts = line.split('|');
        let digits: u32 = parts.next().unwrap().parse().unwrap();
        let a = parts.next().unwrap();
        let op = parts.next().unwrap();
        let b = parts.next().unwrap();
        let out = match (Number::parse(a), Number::parse(b)) {
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
        };
        println!("{line}={out}");
    }
}
