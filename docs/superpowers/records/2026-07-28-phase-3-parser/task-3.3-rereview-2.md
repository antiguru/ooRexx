# Task 3.3 re-review, round 2: the `ScanMode` -> `SourceKind` fix

Re-reviewed at `cdda308e` (fix commit `9d263b22`) against
`review-round2-code.diff` and `task-3.3-rereview.md`. Scope is exactly the
one Important the round-1 re-review found: `ScanMode` could not fix
`INTERPRET`'s CR/LF splitting or Ctrl-Z truncation because those happen in
`ProgramSource::new`, before any mode exists. No other part of the scanner
was re-reviewed.

## Verdict: fix confirmed, task complete

The fix moves the distinction from `scan`'s parameter to
`ProgramSource::new`'s second parameter (`SourceKind::{Program, Interpret}`),
so a source is built once, as one kind, and `scan(&ProgramSource)` reads
that kind off the source. This closes the exact gap the round-1 re-review
found, and independent probing against `build/bin/rexx` / `rexxc` (bytes
written directly to files, not the crate's own fixtures) confirms both the
new `Interpret` behaviour and the unchanged `Program` behaviour. No new
defect found.

## The five checks

**1. Over-correction (must reject LF/CR/CRLF/Ctrl-Z under `Interpret` without
breaking `;` and `interpret ""`) -- clean.** Wrote raw byte files (not the
crate's fixtures) for `say 1\nsay 2`, `say 1\rsay 2`, `say 1\r\nsay 2`,
`say 1\x1asay 2`, `say 1\x1a`, `say 1; say 2`, empty, and
`say c2x('\x1a')`, and ran each two ways: through `scan-check --interpret`
and through the real oracle by wrapping the identical bytes in an
`INTERPRET` argument and running `build/bin/rexx`. All eight agree exactly:
the five terminator/Ctrl-Z cases are `E13.1` in the scanner and
`Error 13.1` in the oracle; `;` still yields two clauses (`ok 8 tokens` /
oracle prints `1` then `2`); empty text yields `ok 0 tokens` / oracle runs
on to `after`; and the Ctrl-Z-in-a-literal case decodes to byte `1A` on both
sides. `interpret_text_is_one_line_so_a_line_terminator_in_it_is_an_invalid_character`
and `a_semicolon_still_separates_interpret_clauses` in the diff assert
exactly this and both are real tests, not just named as if they did.

**2. `Program`-path regression -- clean.** Built four raw-byte probe files
(shebang line 1, shebang line 2, CRLF, LF-then-CR, mid-line Ctrl-Z, `;`) and
ran each through both `build/bin/rexxc` and `scan-check` (no `--interpret`).
Every outcome matches: shebang on line 1 is skipped (scanner starts
tokenising at byte 20, oracle rc 0); shebang on line 2 is `E13.1 line 2` on
both; CRLF collapses to one terminator (token spans land on lines 1 and 2,
oracle rc 0); LF-then-CR produces an empty line 2 with the second clause on
line 3, matching on both sides; a mid-line Ctrl-Z truncates the file after
`say 1` (scanner sees 2 lines, 8 tokens, oracle rc 0 with nothing after the
Ctrl-Z parsed). Also spot-checked 300 random files from
`rust/corpus-l1` (12,059 files) through `scan-check` versus `rexxc`: one
mismatch, `LINES_test_stdin_count.rex`, and it is error 35.1 (`Invalid
expression`) at the parser level, a number the scanner never raises, so it
is exactly the accepted rc-mismatch class Step 5 names and unrelated to this
fix.

**3. Test-fixed-to-match-bug risk -- clean.** The commit message says the
corpus sweep "caught its own fixture bug": an external interpret-differential
fixture had a trailing newline and, before this fix, scanned as `ok`; after
the fix it is `13.1`, which is the newly-correct behaviour. The crate's own
committed test pins the fixed direction, not the old one: the sweep in
`interpret_text_is_one_line_so_a_line_terminator_in_it_is_an_invalid_character`
includes `"say 1\n"` (a trailing newline) in the list asserted to be
`Err((13, 1))`. That is tightening an assertion to the correct oracle
behaviour, not loosening one to match new code. No test in the diff weakens
an existing assertion; every changed assertion in `tests/scanner.rs` and
`tests/sourceline.rs` is either a mechanical signature update or a new,
stricter case.

**4. `line_of` / `line_span` / `span_bytes` / `line_count` under
`SourceKind::Interpret` -- clean, and total as claimed.** Wrote a standalone
probe binary (a separate scratch crate depending on `rexx-parse` by path,
not the crate's own tests) and exercised both an empty and a non-empty
`Interpret` source directly: `line_count()` is 1 in both cases (0 for the
same empty bytes under `Program`); `line(1)` returns the whole text (`[]`
when empty) and `line(2)` is `None`; `line_span(1)` is `0..len`;
`span_bytes` bounds-checks correctly (`0..len` is `Some`, `0..len+1` is
`None`); and `line_of` returns 1 for every byte offset tried, including 0,
mid-text, exactly `len`, `len+5`, and `usize::MAX` -- it is total, and the
one-line answer never changes regardless of where in (or past) the text the
offset falls. This matches `tests/sourceline.rs`'s own
`interpret_text_is_one_line_from_end_to_end`, independently reproduced.

**5. Silent re-introduction of a default -- clean.** `SourceKind` has no
`Default` impl (grepped for one; none exists). The crate's only
`unwrap_or` is the pre-existing Ctrl-Z position lookup in the `Program`
branch (`text.iter().position(...).unwrap_or(text.len())`), which is
unrelated to kind selection and was already there before this round.
Every `ProgramSource::new` call site in the workspace (all in
`rexx-parse`'s own `tests/`, `src/bin/scan-check.rs`, and one in
`token.rs`'s doctest-free code) passes `SourceKind` positionally and
explicitly; no other crate constructs a `ProgramSource` at all. `scan-check`
picks `SourceKind::Program` when `--interpret` is absent, but that is the
CLI tool's own explicit default for an unspecified flag, not a fallback
inside the library's constructor.

## New defects

None found. The fix closes the gap the round-1 re-review identified, the
`Program` path is unchanged in behaviour (only in how the kind is threaded
through), and the boundary accessors behave correctly and totally for the
one-line `Interpret` case.
