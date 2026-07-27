//! Prints `input|canonical` for each line of a file, for differential testing
//! against the C++ interpreter's `value + 0`.
use rexx_num::Number;

fn main() {
    let path = std::env::args().nth(1).expect("usage: canon <file>");
    let text = std::fs::read_to_string(&path).expect("readable input file");
    for line in text.lines() {
        let out = match Number::parse(line) {
            Some(n) => n.format(9),
            None => "<SYNTAX>".to_string(),
        };
        println!("{line}|{out}");
    }
}
