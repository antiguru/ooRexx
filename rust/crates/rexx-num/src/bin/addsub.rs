//! Differential harness for + and -: reads `digits|a|op|b` lines and prints
//! `digits|a|op|b=result`, for diffing against the interpreter.
use rexx_num::Number;

fn main() {
    let path = std::env::args().nth(1).expect("usage: addsub <file>");
    let text = std::fs::read_to_string(&path).expect("readable input file");
    for line in text.lines() {
        let mut parts = line.split('|');
        let digits: u32 = parts.next().unwrap().parse().unwrap();
        let a = parts.next().unwrap();
        let op = parts.next().unwrap();
        let b = parts.next().unwrap();
        let out = match (Number::parse(a), Number::parse(b)) {
            (Some(x), Some(y)) => {
                let r = if op == "+" { x.add(&y, digits) } else { x.sub(&y, digits) };
                r.format(digits)
            }
            _ => "<E41>".to_string(),
        };
        println!("{line}={out}");
    }
}
