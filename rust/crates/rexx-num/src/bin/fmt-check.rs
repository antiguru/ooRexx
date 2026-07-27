//! Differential harness for FORMAT and TRUNC.
//!
//! Reads `digits|func|number|a1|a2|a3|a4` lines, where an empty argument
//! field means the argument was omitted, and prints `line=result`, matching
//! the driver in `../../tests/data-format-oracle.rex` that produces the
//! interpreter's answers.
//!
//! A digits field ending in `E` -- `9E` -- means the case runs under
//! `NUMERIC FORM ENGINEERING`. Both sides parse it the same way, so the two
//! forms can share one case file.
use rexx_num::{Form, FormatError, Number};

fn arg(s: &str) -> Option<u32> {
    if s.is_empty() { None } else { s.parse().ok() }
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: fmt-check <file>");
    let text = std::fs::read_to_string(&path).expect("readable input file");
    for line in text.lines() {
        let f: Vec<&str> = line.split('|').collect();
        let (digits_field, form) = match f[0].strip_suffix('E') {
            Some(rest) => (rest, Form::Engineering),
            None => (f[0], Form::Scientific),
        };
        let digits: u32 = digits_field.parse().unwrap();
        let out = match Number::parse(f[2]) {
            None => "<E41>".to_string(),
            Some(n) => match f[1] {
                "TRUNC" => n.trunc(digits, arg(f[3]).unwrap_or(0)),
                "FORMAT" => {
                    match n.format_with(
                        digits,
                        form,
                        arg(f[3]),
                        arg(f[4]),
                        arg(f[5]),
                        arg(f[6]),
                    ) {
                        Ok(v) => v,
                        Err(e) => format!("<E{}>", FormatError::code(e)),
                    }
                }
                other => panic!("unknown function {other}"),
            },
        };
        println!("{line}={out}");
    }
}
