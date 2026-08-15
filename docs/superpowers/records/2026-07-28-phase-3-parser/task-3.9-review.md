# Task 3.9 review: `TRACE` source lines (`*-*` only)

Reviewed: commit `6614ac34` (implementation) plus `ad1c708f` (plan-text correction).
Verdicts: spec compliance PASS.
Task quality: approved, with minor findings below.

## Method

Everything the coordinator already verified (probe C both LF and CRLF spellings, probe A, probe B, probe D, probe F, the 557/0 test run, clippy clean) was taken as given and not re-checked.
Effort went into: `join_span`'s edge cases (verified against the live code with a throwaway harness, not just by reading), the borrowed/owned contract, whether the tests could pass vacuously (a live defeat experiment), `assert_traced`'s honesty, and the coverage gaps the report admits (`ELSE`/`OTHERWISE`, multi-label).

A scratch Cargo project (`probe-harness`, in the session scratchpad) was built with a path dependency on `rexx-parse` to call `join_span`/`span_bytes`/`line_span` directly and print results.
This never touched the repository.
The one repository edit was the required defeat experiment in `rust/crates/rexx-parse/src/source.rs`, reverted with `git checkout -- rust/crates/rexx-parse/src/source.rs`; `git status --short` is clean.

## `join_span` edge cases

Constructed and ran against the actual code (not just read):

* Span ending mid-CRLF terminator (line "ab", CRLF, line "cd"; span `0..3`, cutting between `\r` and `\n`): returns owned `"ab"`.
  Right: the `\r` is already outside the line's terminator-excluded content range, so nothing partial leaks in.
* Span exactly covering a CRLF terminator (`2..4`, nothing else): returns owned empty.
  Right under the stated contract (drop every terminator byte inside the span); such a span could never come from a real clause boundary.
* Empty span sitting exactly on the terminator's first byte (`2..2`): returns **borrowed** empty.
* LF-CR (two terminators, empty line between, `"a\n\rb"`): `join_span(0..4)` returns owned `"ab"`, dropping both terminators and contributing nothing from the empty middle line.
  Right.
* Source of only terminators (`"\n\n\n"`, 3 empty lines): `join_span(0..3)` returns owned empty.
  Right.
* Span past the end (`0..999` on a 1-byte source): `None`, matching `span_bytes`.
* Backwards span (`3..1`): `None`, matching `span_bytes` (`<[u8]>::get` on a backwards range is `None`, so this falls out of the first line for free).
* Ctrl-Z truncated source (`"say 1\n\x1amore stuff"`): line count is 1 (truncation happens in `new`, before line scanning, exactly as documented); `join_span(0..5)` borrows `"say 1"`; `join_span(0..20)` (past the truncation point) is `None`.
  All right.
* Zero-line (empty) `Program` source, `join_span(0..0)`: returns **owned** empty, where a one-line source's equivalent boundary case (`join_span(1..1)` on `"x"`) returns **borrowed** empty.
  This is the exact inconsistency the report's "What went wrong" section discloses.
  I confirmed it's real and confirmed it's unreachable from any real clause span: an empty `Program` source has zero instructions, so no `clause_span` ever points into it, and an `Interpret` source always has exactly one line even when empty (per `new`'s documented rule), so it never hits the empty-`lines` branch either.
  Disclosed correctly, provably unreachable, not a defect.

No edge case in the brief's list produced a wrong answer or a panic.

## Borrowed/owned contract

The fast path (`self.line_span(self.line_of(span.start))` bounds-checked against `span`) never returns borrowed for a span that truly crosses a terminator: the check requires `span.end <= line.end`, and `line.end` excludes the terminator by construction, so any span reaching into or past a terminator falls through to the owned path.
I could not construct a false-borrow.
The one false-owned case is the zero-line/empty-span corner above, already covered.

## Whether the tests could pass vacuously — decisive experiment run

Replaced `join_span`'s body with `self.span_bytes(span).map(Cow::Borrowed)` (i.e., defeated the join back to a raw slice) and ran `cargo test -p rexx-parse --test sourceline`:

* **4 of 8 new tests went red**: `a_continued_clause_joins_without_its_terminator`, `a_continued_clause_span_contains_the_terminator_it_joins_out`, `a_crlf_continuation_drops_both_terminator_bytes`, `join_span_borrows_on_one_line_and_agrees_with_span_bytes_about_none`.
* The other 4 (`trace_output_rex_reconstructs_every_traced_line`, `probe_a_...`, `probe_b_...`, `interpret_fragment_clauses_reconstruct_too`) stayed green — correctly, since none of those probes contains a continuation (explicitly true of `trace_output.rex` and probes A/B/F per the brief), so a defeated join is byte-identical to the real one for every span they exercise.

This is real, and it's better than the report's own characterization: the report names only "two tests" as the pinning mechanism (the ones asserting the raw span still contains the terminator), but the defeat experiment shows two *more* tests independently catch the same regression through their normal assertions.
The mitigation is not vacuous.

## `assert_traced` honesty

Reviewed the helper (`tests/sourceline.rs:205`) directly: per-clause byte comparison plus a `line_of(span.start) == line` check, no sequence/count assertion, exactly as documented and exactly as the brief says is correct for a re-traced loop body.

Two things checked:

1. **A wrong instruction index would be caught by content, not just line number.** Every `(line, index, expected)` triple in the four `assert_traced`-based tests names a clause whose text is textually distinct from every other clause reachable at a nearby index within the same probe (`nop;` vs `do i = 1 to 2;` vs `say i;` vs `end`, etc.), so an accidental index swap fails on the byte comparison even where the line number would coincidentally match.
2. **An incomplete expectation list is a real, unexploited gap.** Nothing in `assert_traced` verifies that the caller's list is exhaustive against the oracle transcript — an empty list, or a list missing some of the oracle's `*-*` lines, would still "pass," because the function only asserts what it's given. This is the harness weakness the task asked me to check for. It exists in principle. I cross-checked all four `assert_traced` calls against the oracle transcripts reproduced in the task-3.9-report.md line by line: `trace_output.rex` (6 lines vs 6 entries), probe A (9 vs 9), probe B (4 vs 4), probe E (2 vs 2) — every oracle-observed `*-*` line is present in the corresponding test, so the gap is not exploited in this diff. It is worth flagging as a latent risk for whoever writes the next `assert_traced`-based test, since nothing enforces completeness mechanically.

## Coverage the implementer admits is missing — measured against the oracle

The report says `ELSE`/`OTHERWISE` end bytes and multi-label clauses got no new multi-line evidence.
I constructed continuation cases for both and ran them against `build/bin/rexx`, then checked the *unmodified* Rust code (via the scratch harness) against that oracle output:

* `if 1 = 2 then nop\nelse say 1,\n    2` — oracle: `3 *-* else` / `3 *-*     say 1,    2`.
  Rust `join_span` on the arm clause's span produces `"say 1,    2"`, byte-identical.
* `select\n  when 1 = 2 then nop\n  otherwise say 1,\n    2\nend` — oracle: `4 *-* otherwise` / `4 *-*     say 1,    2`.
  Rust produces `"say 1,    2"`, byte-identical.
* `a: b: nop` (multi-label, single line, not itself a continuation case but explicitly named as untested) — oracle: `a:` / `b:` / `nop`, each its own `*-*` line.
  Rust reproduces all three, byte-identical.
* A three-fragment continuation (`say 1,\n  2,\n    3`), which none of the eight shipped tests exercise (they only cover two-fragment joins) — oracle: `say 1,  2,    3`.
  Rust's loop-based join (not hardcoded to two lines) reproduces this byte-identical.

**Finding: these all work correctly, but none of them is covered by the shipped test suite.** The report is honest that this evidence is missing; I'm confirming the missing evidence would have passed had it been written, using the exact spans `finish_split`/`split_before` already produce.
This is a real coverage gap, not a functional defect — `ELSE`/`OTHERWISE` continuation is reachable, and the join handles it because it's span-generic and doesn't special-case the keyword clauses at all.

## Fabrication check

The doc comment's citation — `ProgramSource::extract` at `ProgramSource.cpp:153`, whose multi-line branch concatenates `getStringLine` results which are terminator-excluded — is accurate.
`RexxString *ProgramSource::extract(SourceLocation &location)` is defined at exactly line 153 in `interpreter/parser/ProgramSource.cpp`; its multi-line branch (`getStringLine(location.getLineNumber(), location.getOffset())`, a loop of whole-line `getStringLine(counter)` calls, then `getStringLine(location.getEndLine(), 0, location.getEndOffset())`) is structurally identical to the Rust port's "first-partial, middle-whole, last-partial" line walk, and `getStringLine`'s underlying `getLine` returns content only, matching the "terminators excluded" claim.
Not fabricated.

## Project constraints

* Bytes not `str`: confirmed throughout `join_span` and the new tests. The only `str` usage is `String::from_utf8_lossy` inside `assert_eq!` failure-message formatting, which is display-only and does not affect the comparisons.
* No `unsafe` added.
* No depth field added; `clause.rs`/`ast.rs`/`instruction.rs` are untouched, consistent with the report's claim that no span repair was needed — a claim I independently corroborated via the `ELSE`/`OTHERWISE`/multi-label probes above, which all matched the oracle using the existing, unmodified span-producing code.
* No value-marker creep: the implementation only ever produces `*-*` text; the report's oracle transcripts retain value-marker lines purely as captured evidence, not as something the code interprets.
* One test comment enumerates markers seen in one specific capture (`>L>, >V>, >O>, >>>, >=>`, in `trace_output_rex_reconstructs_every_traced_line`'s comment). I checked it against the report's own transcript of that exact file: those are exactly the five markers present, no others. It is not offered as the complete 19-marker table and is factually correct for what it describes, but it's the kind of list the brief specifically warns has been wrong three times before, so it's flagged for a second pair of eyes rather than waved through silently.
* Comment style — **two structuring semicolons found** in the Rust-file portion of the diff, against the user's "no structuring semicolons in comments" rule:
  * `src/source.rs` (`join_span` doc comment): "Borrowed when the span sits on one line, which every uncontinued clause does; owned only when there is a terminator to drop."
  * `tests/sourceline.rs` (`trace_output_rex_reconstructs_every_traced_line`): "...also carries value-marker lines (>L>, >V>, >O>, >>>, >=>) and the program's own output; those need an executor and are Phase 4's, so only the six *-* lines appear here."
  No em-dashes appear in the Rust-file portion of the diff (only in the pre-existing, unrelated plan-doc prose, which already used them before this diff and is not Rust source).
* Plan-doc commit `ad1c708f` touches only `docs/superpowers/plans/2026-07-28-phase-3-parser.md`, is a text-only correction (confirmed via `git show --stat` and `git show -- rust/`), and its content matches what it claims to fix (the `say "x","y"` vs "four leading blanks" contradiction, and the `trace i`-vs-`trace r` framing).

## Verdicts

* **Spec compliance: PASS.** The terminator-stripping join is implemented correctly, matches the C++ oracle's own multi-line `extract` logic, handles every edge case tried (including several beyond the shipped tests), respects the forbidden-accessor and no-depth-field constraints, and stays out of value-marker territory.
* **Task quality: approved**, with two minor comment-style fixes needed (structuring semicolons) and a real-but-unexploited completeness gap in `assert_traced` worth naming for future test authors, plus a recommendation to add regression coverage for continued `ELSE`/`OTHERWISE` clauses and a 3+-line continuation, since I had to build a throwaway harness to confirm those paths are actually exercised by the existing code rather than merely plausible.
