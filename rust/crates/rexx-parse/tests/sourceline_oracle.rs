/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! Phase 3 gate: `SOURCELINE(n)` matches the interpreter for every line of
//! every `rust/corpus/lang/` program, including the last line of a file
//! without a trailing newline (`no_trailing_newline.rex`, which the oracle
//! counts as a full line).
//!
//! The oracle's answers live in `tests/sourceline_oracle/<name>.txt`: line
//! one is `count N`, then the N source lines exactly as
//! `.Package~new(file)~source` returns them, terminators excluded. They were
//! captured by a driver rather than by editing the files under test, because
//! constructing the package runs the file's prolog and its output interleaves
//! with the driver's; the driver prefixes its own lines and the capture keeps
//! only those.
//!
//! To regenerate, put this driver in a scratch directory as `srclines.rex`:
//!
//! ```rexx
//! parse arg f
//! a = .Package~new(f)~source
//! say "%SRCG%COUNT" a~items
//! do i = 1 to a~items
//!   say "%SRCG%L" || a[i]
//! end
//! ```
//!
//! and run, from the repository root (the `ulimit` guards against the
//! interpreter requesting unbounded memory):
//!
//! ```bash
//! for f in rust/corpus/lang/*.rex; do
//!   name=$(basename "$f" .rex)
//!   out=$( ( ulimit -v 1048576; build/bin/rexx SCRATCH/srclines.rex "$f" ) \
//!           2>/dev/null | grep '^%SRCG%' )
//!   count=$(printf '%s\n' "$out" | sed -n 's/^%SRCG%COUNT //p')
//!   { echo "count $count"; printf '%s\n' "$out" | sed -n 's/^%SRCG%L//p'; } \
//!     > rust/crates/rexx-parse/tests/sourceline_oracle/$name.txt
//! done
//! ```

use std::path::Path;

use rexx_parse::{ProgramSource, SourceKind};

/// One parsed expectation file: the oracle's line count and its lines.
fn parse_expectation(bytes: &[u8], path: &str) -> (usize, Vec<Vec<u8>>) {
    let newline = bytes
        .iter()
        .position(|&b| b == b'\n')
        .unwrap_or_else(|| panic!("{path}: expectation file has no header line"));
    let header = &bytes[..newline];
    let count: usize = std::str::from_utf8(header)
        .ok()
        .and_then(|h| h.strip_prefix("count "))
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("{path}: header is not `count N`"));
    let body = &bytes[newline + 1..];
    // The capture writes one trailing newline per line, so splitting on `\n`
    // yields one final empty element that is not a line.
    let mut lines: Vec<Vec<u8>> = body.split(|&b| b == b'\n').map(<[u8]>::to_vec).collect();
    let trailer = lines.pop();
    assert_eq!(
        trailer,
        Some(Vec::new()),
        "{path}: expectation file does not end with a newline"
    );
    assert_eq!(
        lines.len(),
        count,
        "{path}: header count disagrees with the captured lines"
    );
    (count, lines)
}

#[test]
fn sourceline_matches_the_interpreter_for_every_corpus_program() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang");
    let oracle = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sourceline_oracle");

    let mut checked = 0;
    let mut saw_unterminated_final_line = false;
    for entry in std::fs::read_dir(&corpus).expect("corpus/lang exists") {
        let path = entry.expect("readable directory entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rex") {
            continue;
        }
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap();
        let expectation_path = oracle.join(format!("{name}.txt"));
        let expectation = std::fs::read(&expectation_path).unwrap_or_else(|e| {
            panic!(
                "{}: no oracle expectation for corpus program {name} ({e}); \
                 regenerate per the module comment",
                expectation_path.display()
            )
        });
        let (count, lines) = parse_expectation(&expectation, name);

        let text = std::fs::read(&path).expect("readable corpus file");
        if text.last() != Some(&b'\n') && text.last() != Some(&b'\r') {
            saw_unterminated_final_line = true;
        }
        let src = ProgramSource::new(text, SourceKind::Program);
        assert_eq!(
            src.line_count(),
            count,
            "{name}: SOURCELINE() disagrees with the oracle"
        );
        for (i, expected) in lines.iter().enumerate() {
            let n = i + 1;
            let actual = src
                .line(n)
                .unwrap_or_else(|| panic!("{name}: line {n} missing"));
            assert_eq!(
                actual,
                &expected[..],
                "{name}: SOURCELINE({n}) disagrees with the oracle"
            );
        }
        // Past the end must be an error, not an empty line: the oracle raises
        // 40.34 there, and `line` answering `None` is this crate's spelling.
        assert_eq!(src.line(count + 1), None, "{name}: line past the end");
        checked += 1;
    }

    // The walk found real work: the corpus is present, and the criterion's
    // named edge case, a file whose last line has no terminator, was among
    // the files rather than silently absent.
    assert!(checked >= 14, "corpus went missing: {checked}");
    assert!(
        saw_unterminated_final_line,
        "no corpus program lacks a trailing newline, so the criterion's edge \
         case is untested; no_trailing_newline.rex exists for this"
    );
}
