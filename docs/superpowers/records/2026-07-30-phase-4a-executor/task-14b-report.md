STATUS: DONE

# Task 14b report -- closing criterion 1's coverage gap

Scope, per the team lead's dispatch, reading
`.superpowers/sdd/2026-07-30-phase-4a-executor/criterion-1-coverage-gap.md`
(STATUS: DONE, someone else's analysis, treated as a starting point to
re-verify rather than finished work):

1. Remove `num/digits_rounding.rex`, `num/exponential.rex`,
   `num/operators.rex` from `rust/corpus/phase-4a.txt` (subset-membership
   change only -- the files stay in `corpus/num/`, they are valid programs
   4b/4c will use once `ExprKind::List` exists). All three construct `List`
   from a `SAY a, b` comma, which 4a fails loudly on where the oracle prints
   two lines and exits 0 -- a correctness blocker for criterion 1's
   zero-divergence requirement, not a coverage question.
2. Write three new programs (the gap report's Programs A, B, C, exact names
   given) covering the nineteen unconstructed variants: `InstructionKind`
   (`Loop`, `Label`), `ExprKind` (`DotVariable`, `Logical`), `LoopKind`
   (`Over`, non-stem target only -- `DO OVER` on a stem is the excluded
   traversal-order deviation and can never pass), `PrefixOp` (`Plus`,
   `Not`), `EndStyle` (`Select` without `OTHERWISE`, `LabeledOtherwise`),
   `Trace` (`Default`, `Value`, `Skip` -- `Skip` is `Error 24.901`, a
   raiser, not a no-op, so its program must end there), and the ten
   remaining `Operator` variants.

File scope: `rust/corpus/phase-4a.txt`, new files under `rust/corpus/lang/`,
and their `rust/crates/rexx-parse/tests/sourceline_oracle/*.txt`
expectations (new files need one from scratch; any existing file I edit
needs it regenerated -- the line-indexed coupling I documented in
`rust/corpus/README.md` last round, now biting on my own new work if I'm
not careful). Not touching `rexx-exec` -- live work from another agent.

Filling this in as I go.

## Plan

1. Read the gap report in full, and independently re-verify its two load-
   bearing claims before acting on them: that the three `List` programs are
   genuinely redundant for coverage (re-run the analysis logic myself in
   spirit, or at least spot-check), and that each of the three draft
   programs really does construct what it claims -- "the analysis was
   deliberately done by someone else so the gap list and the programs
   closing it get separate pairs of eyes," which only means something if I
   actually look with my own eyes rather than transcribe.
2. Edit `rust/corpus/phase-4a.txt`: remove the three `List`-constructing
   lines.
3. Write the three new programs, using the report's draft text as a
   starting point, and independently verify each against the oracle:
   correct exit code, and two runs producing byte-identical stdout and
   stderr, wrapped as `( ulimit -v 1048576; build/bin/rexx FILE )`.
4. Add all three to `phase-4a.txt`.
5. Regenerate `sourceline_oracle` expectations for the three new files (and
   for anything else I touch, if anything).
6. Run `cargo test -p rexx-parse` to confirm the sourceline oracle test and
   everything else downstream is still green.
7. Write up findings here, stage only my paths, commit once.

## Re-verifying the analysis before acting on it

* **The three `List` programs' comma-in-`SAY` claim: confirmed by reading
  all three files directly.** `digits_rounding.rex` line 9
  (`say 0.5 + 0 , 1.5 + 0 , 2.5 + 0 , -0.5 + 0`), `exponential.rex` lines 2
  and 7, and every non-trivial line of `operators.rex` all use a bare comma
  inside a `SAY` expression. Not disputed.
* **The "removing them doesn't change the gap list" claim: not
  independently re-run** (I don't have the scratch AST-walking analyser the
  report describes, and building an equivalent is Task 16's job, out of my
  scope), but it is a narrow, falsifiable claim already stated as a ruling
  the team lead acted on, and my own read of the three files found nothing
  in them beyond ordinary `NUMERIC`/`SAY`/arithmetic/`DO...END` already
  covered by the surviving 23-program subset (`arith_digits.rex` alone
  already covers `NUMERIC DIGITS`/`FORM`, division, `**`, `//`, `%`,
  exponential formatting). Consistent with the claim, not an independent
  re-derivation of it.
* **Every draft program was re-verified against the oracle from scratch,
  not trusted from the report's transcripts** -- see the per-program
  sections below. All three matched the report's predicted output and exit
  code exactly on first measurement, both before adding this file's header
  comment and after (which only shifts the trace-numeric-request's clause
  echo line number, unimportant since both interpreters run the same
  file).

## Program A -- `corpus/lang/prefix_dotvar_logical_over_label.rex`

Closes `InstructionKind::Loop`, `InstructionKind::Label`,
`ExprKind::DotVariable`, `ExprKind::Logical`, `LoopKind::Over` (on a
string, not a stem -- `do i over 'abc'`, confirmed it iterates exactly
once, yielding `abc`), `PrefixOp::Plus`, `PrefixOp::Not`,
`EndStyle::Select` (a `SELECT` whose one `WHEN` matches, so its `END` has
no `OTHERWISE`), `Trace::Default` (bare `trace`), `Trace::Value`
(`trace value 'N'`).

Oracle exit code: 0 (both runs). Two runs: byte-identical stdout (`5` /
`0` / `The NIL object` / `both` / `abc` / `sel` / `lp` / `lp`), empty
stderr both times -- confirms neither `TRACE` form here emits trace
output, matching the report's claim that adding them doesn't disturb the
expectation.

## Program B -- `corpus/lang/comparison_operators_remaining.rex`

Closes the ten `Operator` variants no other corpus program constructs
(`BackslashGreaterThan` `\>`, `BackslashLessThan` `\<`,
`StrictBackslashGreaterThan` `\>>`, `StrictBackslashLessThan` `\<<`,
`StrictGreaterThanEqual` `>>=`, `StrictLessThanEqual` `<<=`,
`LessThanGreaterThan` `<>`, `GreaterThanLessThan` `><`, `Or` `|`, `Xor`
`&&`) and `EndStyle::LabeledOtherwise` (`select label s` with an
`OTHERWISE`).

Oracle exit code: 0 (both runs). Two runs: byte-identical stdout
(`1 0 1 0 1 1 1 1 1 1`, one value per line, then `lo`), empty stderr both
times.

## Program C -- `corpus/lang/trace_numeric_request.rex`

Closes `Trace::Skip` (`trace 5`, a raiser rather than a no-op --
**Error 24.901**, "Numeric TRACE requests are valid only from interactive
debugging."). Lives alone because it terminates the program; nothing
follows it.

Oracle exit code: **232** (both runs, matching `256 - 24`). Two runs:
byte-identical stdout (`0`) and byte-identical stderr (the clause echo
`N *-* trace 5` -- `N` is this file's own line number for that clause, 10,
since it comes after an 8-line header comment plus `say 0`; unimportant,
both interpreters see the same file -- then the two `Error 24`/`24.901`
lines).

## A wrinkle in generating this file's `sourceline_oracle` expectation, not covered by the existing driver

`sourceline_oracle.rs`'s documented driver does `a = .Package~new(f)~source`
inside `srclines.rex`, then reports `a`'s lines. `.Package~new` **executes**
the target file's prolog as a side effect of constructing it (the same fact
the project's global constraints already warn about for probing). For every
other corpus program this is silent scaffolding, but `trace_numeric_request.rex`'s
whole *point* is that its prolog raises an uncaught `Error 24.901` -- so
`.Package~new` itself never returns, the crash happens **inside** the
assignment to `a`, and the driver dies with it before printing anything
useful. Confirmed: running the stock driver against this file produces
`count ` (empty) and nothing else.

Fix, verified before relying on it: wrap the assignment in
`SIGNAL ON SYNTAX`, and on the trapped condition, fall back to reading the
file as plain text with `LINEIN()` in a loop instead of asking
`~source` for it. This is a safe substitute *specifically for this file*
because it has no CRLF terminators, no `CTRL-Z`, and a normal trailing
newline -- none of the edge cases `no_trailing_newline.rex` exists to
distinguish `~source` from a naive line reader on. Verified the fallback
driver still takes the primary (`~source`) path for ordinary files by
running it against the other two new programs and an existing one
(`drop_stem_tail.rex`): all three still matched `wc -l` exactly, so the
`SIGNAL ON SYNTAX` wrapper doesn't change behaviour for anything that
doesn't crash. Generated all three new files' expectations with this
driver; `wc -l` matches every one of the three (20, 21, 10).

**Not committed anywhere**, since `sourceline_oracle.rs`'s module doc
comment (where the stock driver lives) is outside this task's file scope
(I touched `rust/corpus/phase-4a.txt`, new files under `rust/corpus/lang/`,
and their `.txt` expectations only). Flagging this to the team lead in my
completion message so whoever owns that file can decide whether to fold
the `SIGNAL ON SYNTAX` fallback into the documented driver -- the next
program that needs to construct `Trace::Skip`, or any other variant whose
whole point is to crash its own prolog, will hit the same wall.

## Verification run

`cd rust && cargo test -p rexx-parse --no-fail-fast`: every test binary
passed, including `sourceline_matches_the_interpreter_for_every_corpus_program`,
`every_corpus_lang_program_parses`, `every_corpus_program_tiles`, and
`every_variant_is_constructed_by_the_corpus_and_samples` -- the last of
these is presumably an early cut of (or overlaps with) Task 16's coverage
gate, and its passing is independent confirmation, from a test I did not
write, that the three new programs plus the trimmed subset now construct
what criterion 1 needs. (The three test binaries the deep-expression
stack overflow was hitting before -- `program.rs`, `tiling.rs`,
`variants.rs` -- all pass clean now too; that defect is tracked elsewhere
and not something this task touched.)

## Files changed

* `rust/corpus/phase-4a.txt`: removed `num/digits_rounding.rex`,
  `num/exponential.rex`, `num/operators.rex`; added the three new
  programs; both changes documented inline with the reasoning, matching
  the file's own established comment style.
* `rust/corpus/lang/prefix_dotvar_logical_over_label.rex`,
  `comparison_operators_remaining.rex`, `trace_numeric_request.rex`: new,
  each with a header comment naming exactly what it closes.
* `rust/crates/rexx-parse/tests/sourceline_oracle/{prefix_dotvar_logical_over_label,comparison_operators_remaining,trace_numeric_request}.txt`:
  new expectations, all three verified against `wc -l`.

Not changed, deliberately: `rust/corpus/README.md` (no explicit ask this
round, unlike Task 14a's; noting the "Phase 4a additions" table and the
`num/` table are now slightly behind the subset list, as a suggestion for
whoever picks that up next, not something I did unbidden) and
`rust/crates/rexx-parse/tests/sourceline_oracle.rs` (the `SIGNAL ON
SYNTAX` fallback finding above, out of scope, flagged to the team lead
instead).
