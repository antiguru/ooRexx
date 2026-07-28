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
                        println!(
                            "  {:?} {:?} {:?}",
                            token.span,
                            token.kind.tag(),
                            source.line_of(token.span.start)
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
