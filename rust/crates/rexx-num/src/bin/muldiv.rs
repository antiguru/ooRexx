//! Differential harness for * / % and //.
use rexx_num::Number;

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
            (Some(x), Some(y)) => match op {
                "*" => x.mul(&y, digits).format(digits),
                _ => "<unimplemented>".to_string(),
            },
            _ => "<E41>".to_string(),
        };
        println!("{line}={out}");
    }
}
