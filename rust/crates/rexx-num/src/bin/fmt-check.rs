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

/// An empty field means the argument was omitted. Anything else must parse:
/// silently treating a malformed field as "omitted" would turn a typo in a
/// case file into a case that quietly tests something other than what it
/// says, and it would still be compared against the oracle's answer for the
/// case as written.
fn arg(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    Some(s.parse().unwrap_or_else(|_| panic!("malformed argument field {s:?}")))
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
        let digits: u64 = digits_field.parse().unwrap();
        let out = match Number::parse(f[2]) {
            // A value that is not a number reaches FORMAT and TRUNC as a bad
            // *argument*, which is error 93 -- not the 41 that a bad numeric
            // literal in source text raises. Both builtins agree, and both
            // report 93 for `abc` and for an out-of-range exponent alike.
            // `Number::parse` returning None is right either way; picking the
            // error number is the caller's job, so the harness has to model
            // the calling context rather than the parse failure.
            None => "<E93>".to_string(),
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
