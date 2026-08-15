# Task 3.4 review: Clause splitting

Base `cdda308e`, commit `5eec1fcb`.

## Verdict 1: Spec compliance

**PASS.**
`split_clauses` implements rules 1-3 exactly as the (implementer-corrected) brief states them, and correctly implements none of rule 4.
`Clause` carries `tokens`, `span` and `label` with the field types and semantics the brief specifies.
The one deviation from the brief's text is `Clause`/`split_clauses` being `pub` rather than `pub(crate)`, which is forced by a genuine contradiction inside the brief itself (see adjudication 1) and not a shortcut.

## Verdict 2: Code quality

**PASS.**
The implementation is short, correct in every case I probed (17 unit tests plus my own independent oracle probes below, all agreeing), and the report's numeric claims about its own testing (mutation-failure counts, corpus size) check out exactly when I reran them myself.
Two Minor comment-style nits (structuring semicolons) are the only blemishes found.

## Independent verification performed

* Re-ran the 3 mutations the report describes against the actual committed `clause.rs`, restoring the file after each and confirming a clean tree (`git status --short` / `git diff --stat` empty) before and after:
  * span end taken from the token before the terminator instead of the terminator's own end &rarr; **5 of 17** tests fail (report: 5-6). Exact match.
  * label recognition disabled (`labelled = false && …`) &rarr; **6 of 17** fail. Exact match.
  * label span stops at the name token instead of the colon &rarr; **6 of 17** fail. Exact match.
* Independently probed cases the report did not cover: CRLF line endings, a bare-CR-only file, and a `\n\r` pair (two terminators, empty line between). Wrote a throwaway integration test calling `scan`/`split_clauses` directly and compared the resulting `span` text against `build/bin/rexx trace r` output on the same three files; all matched byte-for-byte. Deleted the test afterward; tree confirmed clean.
* Probed `a:b` (no `say`, no `call`) at top level: oracle traces `a:` then `b` (the `b` clause then fails as a shell command, `/bin/sh: B: not found`), confirming the label rule fires unconditionally at clause start even where it produces a nonsensical program — matches the implementation's unconditional check on `tokens[start]`/`tokens[start+1]`.
* Probed `call a:b`: oracle does not split it (colon is a namespace qualifier mid-clause) and later fails with error 98.987 "Namespace A not found" — matches the existing `a_colon_after_the_first_token_is_not_a_label` test's model.
* Probed two labels with nothing else on their line (`a: b:` then `say 1` on the next line): oracle produces exactly two label clauses and no spurious empty third clause before `say 1` — matches by direct trace of the loop logic (the inner `while start < limit` exits without pushing anything when nothing remains between the last label and the terminator).
* Confirmed against the actual C++ (`Token.hpp:580`, `InstructionParser.cpp:150-176`, `InstructionParser.cpp:2792-2811`) that `isSymbolOrLiteral()` is exactly `TOKEN_LITERAL || TOKEN_SYMBOL`, that the interpret-label check (`Error_Unexpected_label_interpret`, i.e. 47.1) fires at the point a label is recognized regardless of whether tokens follow the colon, and that `labelNew` sets the instruction's end to the colon's end unconditionally. All three exactly as the report describes.
* Confirmed `rust/crates/rexx-parse/src/lib.rs` and `token.rs` at base `cdda308e` already declared `ParseCtx`/`TokenCursor` `pub` and re-exported them, and that `tests/tokens.rs` already exercises `TokenCursor` as an integration test — the pre-existing debt the report cites is real and pre-dates this task.
* Counted 12,990 `.rex`/`.orx`/`.cls`/`.testGroup`/`.testUnit` files in the repo, matching the report's corpus-sweep file count exactly.

No Critical or Important findings. See severity list below.

## The five adjudications

1. **`pub` instead of `pub(crate)` for `Clause`/`split_clauses`: accept as-is, record.** The brief self-contradicts (declares `pub(crate)` three lines from naming an integration-test file that cannot see a `pub(crate)` item), and the implementer's resolution matches the crate's existing convention and precedent (`ParseCtx`/`TokenCursor` were already `pub` for the identical reason before this task, and Task 3.3's review already flagged that debt as owed to Task 3.5). This task adds two more items to that pile, so Task 3.5 now owes narrowing 4 items (`ParseCtx`, `TokenCursor`, `Clause`, `split_clauses`) and moving 3 test files' worth of coverage into `#[cfg(test)]` modules, not 2. Worth flagging to whoever scopes Task 3.5, not worth blocking this task on.

2. **Rule 3 implementation matches the corrected rule in every case checked, including both named edge cases.** A label at end of line with nothing following (`here:`, no trailing clause) and a label followed immediately by `;` (`here: ; nop`) both trace correctly through the implementation by direct trace of the code and both are independently confirmed against the oracle (report's p4/r4, and my own reruns).

3. **Task 3.9's characterisation is accurate and actionable.** A continued clause's `span` genuinely contains the line terminator bytes (confirmed: `say 1,\n  + 2` gives `span == 0..12`, which includes the `\n`), while `trace r` prints the fragments with the terminator dropped rather than the raw slice. Task 3.9 therefore needs a per-line join that strips terminators between fragments, not `text[span]`; the report's q4 probe is the right starting fixture for that.

4. **Error 47.1 belongs in Task 3.6; `split_clauses` should stay `&[Token]`.** The C++ raises it at exactly the point a label is recognized (`InstructionParser.cpp:156`), before it even checks whether tokens follow the colon, using `isInterpret()` on the parser's own state. Task 3.6 already receives `ParseCtx` (which carries `source: &ProgramSource`, and `source.kind()` answers the same question) and already owns the label-adjacent rule-4 logic, so it can raise 47.1 for `clause.label.is_some() && ctx.source.kind() == SourceKind::Interpret` without any change to this task's signature.

5. **Keeping the always-`Ok` `Result<Vec<Clause>, ParseError>`: accept as-is.** It is the brief's specified shared interface across the pipeline, costs nothing today (clippy `-D warnings` stays clean; no `unnecessary_wraps`-style lint fires under the lints this workspace runs), and is a one-line change with no external callers if a later task finds it genuinely dead.

## Findings

### Critical
None.

### Important
None.

### Minor

* **M1 — two structuring semicolons in `clause.rs`'s doc comments**, against the house rule ("no em-dashes or structuring semicolons in comments"). `src/clause.rs:45-46`: "For an explicit `;` that puts the semicolon inside the span; for an end of line it stops at the last byte of the line's content…" joins two independent statements with a semicolon instead of a period. `src/clause.rs:62`: "…all three of which reach here as an `Eoc`; and a label's `:` ends a clause too." does the same. Cosmetic; no behavioural effect. Fix: split each into two sentences.
* **M2 — the `terminator == None` fallback (`src/clause.rs:85-87`) is untested.** It exists to handle a token slice that ends without a trailing `Eoc`, which the module's own doc comment says `scan` never produces, and no test constructs such a slice by hand to exercise it. The logic is simple enough to be correct by inspection (treats the last token's own end as the clause's end, matching the "ends the final clause at its last token" contract stated just above it), but it is currently reachable only by violating a stated precondition, so nothing pins its behaviour.

## Unverifiable from the diff

None. Everything the brief and report claimed was either directly inspectable in the diff, reproducible against `build/bin/rexx`, or reproducible by rerunning the implementer's own mutation tests against the committed code.
