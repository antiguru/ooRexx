//! Differential harness for the scanner: scans each file named on the command
//! line and prints one line per file, for diffing against `build/bin/rexxc`.
//!
//! `ok` corresponds to `rexxc` exiting 0 as far as scanning is concerned, and
//! `E<code>.<sub> line <n>` to the `Error <code>.<sub>` and reported line it
//! writes to stderr. `rexxc` also rejects programs this scanner accepts,
//! because it goes on to parse them; a difference is only a scanner difference
//! when the error number is one the scanner raises at all.
use rexx_parse::{ProgramSource, TokenKind, scan};

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    // With --tokens, also dump the token stream, which is what makes a
    // disagreement diagnosable rather than merely visible.
    let dump = args.peek().map(String::as_str) == Some("--tokens");
    if dump {
        args.next();
    }
    for path in args {
        let bytes = std::fs::read(&path).expect("readable input file");
        let source = ProgramSource::new(bytes);
        match scan(&source) {
            Ok(scanned) => {
                // Symbol occurrences against distinct symbols is the ratio
                // interning trades allocations for, so the harness reports it.
                let mut distinct = std::collections::BTreeSet::new();
                let mut occurrences = 0usize;
                for token in &scanned.tokens {
                    if let TokenKind::Symbol { id, .. } = token.kind {
                        occurrences += 1;
                        distinct.insert(id);
                    }
                }
                println!(
                    "{path}: ok {} tokens, {occurrences} symbol occurrences over {} distinct, \
                     {} interned, {} resources",
                    scanned.tokens.len(),
                    distinct.len(),
                    scanned.symbols.len(),
                    scanned.resources.len()
                );
                if dump {
                    for token in &scanned.tokens {
                        // A literal's value is printed in hex because it is
                        // bytes, not text, and because `c2x` is what the
                        // interpreter side of the comparison prints.
                        let payload = match &token.kind {
                            TokenKind::Symbol { id, class } => {
                                format!("{} {class:?}", scanned.symbols.name(*id))
                            }
                            TokenKind::Literal { value } => {
                                value.iter().map(|b| format!("{b:02X}")).collect()
                            }
                            TokenKind::Operator(op) | TokenKind::Assignment(op) => {
                                format!("{op:?}")
                            }
                            _ => String::new(),
                        };
                        println!(
                            "  line {} {:?} {:?} {payload}",
                            source.line_of(token.span.start),
                            token.span,
                            token.kind.tag(),
                        );
                    }
                    for body in &scanned.resources {
                        // The line count is what `~items` reports on the
                        // interpreter's side, so this is directly comparable.
                        println!(
                            "  resource at line {}: {} lines",
                            source.line_of(scanned.tokens[body.directive].span.start),
                            body.lines.len()
                        );
                    }
                }
            }
            Err(error) => println!(
                "{path}: E{}.{} line {}",
                error.code,
                error.sub,
                source.line_of(error.byte)
            ),
        }
    }
}
