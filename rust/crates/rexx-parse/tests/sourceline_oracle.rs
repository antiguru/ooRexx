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
//! **Constructing the package runs the file's prolog** -- this is also the
//! project's standing "never instantiate `.Package~new` on a repository
//! file" rule, and this driver is the one place that already breaks it. It
//! has been safe only because every corpus prolog used to be trivial: `SAY`
//! and assignment, nothing that raises. `trace_numeric_request.rex` is the
//! first one where that stopped being true (its whole point is a prolog
//! that raises `Error 24.901`), and it will not be the last, since this
//! phase now writes witness programs whose entire purpose is a specific
//! failure. A driver that only works on programs that succeed is not a
//! driver for a corpus that deliberately contains failures -- so the driver
//! below traps the construction failing and falls back to reading the file
//! as plain text instead, rather than assuming every prolog behaves.
//!
//! The fallback is `LINEIN()` in a loop, not another call into `.Package`,
//! specifically so a crashing prolog cannot run a second time. It is a
//! faithful substitute for `~source` only for a file carrying none of the
//! shapes a naive line reader and `~source` read differently: a CRLF
//! terminator, an embedded `CTRL-Z`, or a missing final newline
//! (`no_trailing_newline.rex`'s entire reason to exist).
//!
//! **Which programs take the fallback is not written down here, because it is
//! not a fact about this module.** A file takes it when constructing its
//! package raises, and constructing a package runs the file's prolog and
//! installs its directives, so every witness program whose whole purpose is a
//! failure takes it -- measured with a copy of this driver instrumented to
//! report the path it took, a large fraction of `corpus/lang` does,
//! `trace_numeric_request.rex` among them. What has to
//! hold instead is the condition above, and the test below asserts it over
//! every corpus program: no CR byte, no `CTRL-Z`, and a final terminator on
//! every file except the one whose name says it has none. A future witness
//! program that broke any of those would redden that assertion rather than
//! quietly gain an expectation this driver captured wrongly.
//!
//! `no_trailing_newline.rex` is therefore the file that must never be the one
//! whose prolog is made to crash: measured, it takes the primary path and is
//! 7 lines under `~source`, not 6, and the `SIGNAL ON SYNTAX` wrapper does not
//! change what any non-crashing file's expectation looks like.
//!
//! To regenerate, put this driver in a scratch directory as `srclines.rex`:
//!
//! ```rexx
//! parse arg f
//! signal on syntax name fallback
//! a = .Package~new(f)~source
//! call report a
//! exit 0
//!
//! fallback:
//!   lines = .array~new
//!   do while lines(f) \== 0
//!     lines~append(linein(f))
//!   end
//!   call stream f, "c", "close"
//!   call report lines
//!   exit 0
//!
//! report: procedure
//!   use arg a
//!   say "%SRCG%COUNT" a~items
//!   do i = 1 to a~items
//!     say "%SRCG%L" || a[i]
//!   end
//!   return
//! ```
//!
//! and run, from the repository root (the `ulimit` guards against the
//! interpreter requesting unbounded memory, and the `</dev/null` against the
//! prolog reading the console -- `pull_queue.rex`'s prolog does, and without
//! that redirect this driver blocks forever waiting for a line that a
//! terminal will never send):
//!
//! ```bash
//! for f in rust/corpus/lang/*.rex; do
//!   name=$(basename "$f" .rex)
//!   out=$( ( ulimit -v 1048576; build/bin/rexx SCRATCH/srclines.rex "$f" \
//!             </dev/null ) 2>/dev/null | grep '^%SRCG%' )
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
    let mut unterminated: Vec<String> = Vec::new();
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
        if text.last() != Some(&b'\n') {
            unterminated.push(name.to_owned());
        }
        // The CR and `CTRL-Z` shapes the module comment names as making its
        // regeneration driver's fallback unfaithful, asserted here rather
        // than written down, for the reason that comment gives. A missing
        // final terminator is the shape the walk collects above and the tail
        // below pins to the one file that is allowed it.
        assert!(
            !text.contains(&b'\r'),
            "{name}: a CR byte, which the regeneration driver's LINEIN \
             fallback does not read the way `~source` does"
        );
        assert!(
            !text.contains(&0x1a),
            "{name}: an embedded CTRL-Z, which the regeneration driver's \
             LINEIN fallback does not read the way `~source` does"
        );
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
    unterminated.sort();
    assert_eq!(
        unterminated,
        ["no_trailing_newline"],
        "the set of corpus programs whose last line has no terminator moved. \
         Losing no_trailing_newline.rex leaves the criterion's edge case \
         untested, which is why that file exists; gaining another leaves a \
         file this module's regeneration driver cannot capture faithfully if \
         its prolog ever raises"
    );
}
