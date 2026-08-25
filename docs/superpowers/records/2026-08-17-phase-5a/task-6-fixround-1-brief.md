# Task 6, fix round 1

The review found no Critical. It confirmed the mechanism on nine operator forms beyond your three,
confirmed the corpus arithmetic (109 = 106 + 3, nothing dropped), confirmed `coverage.rs`'s six lines
and the three `sourceline_oracle` fixtures are structurally mandatory with correct counts, confirmed
your corrected `corpus.rs` doc block is true and constraint-clean, and confirmed trapped conditions
clear `failure_site` so no stale frame survives. Five Important follow.

## 1. `is_stem_receiver` over-fires, and it is a new divergence

`eval.rs:1038-1042`, doc claim at `:1060-1061`. Measured by the reviewer: `s. = .nil` then
`say s. + 1` -- the oracle emits **no** frame and raises `97.1`, "does not understand message +";
head emits `       *-* Compiled method "+" with scope "String".` on both engines. The pinned
`rexx-run-15a1ffa98` emits no such line, so this arrived with your commit.

Your doc asserts the predicate "must be 'is the receiver a stem at all'". That is false. **The
oracle's frame needs the forwarded method to have actually run and raised.** A stem whose default
value understands no `+` never reaches one.

**Ruling: narrow the predicate to the default-value kinds for which a forwarded native method exists**
-- string and number -- so a `.nil` default emits no frame. That leaves our `41` against the oracle's
`97.1` on that program, which is **pre-existing** and not yours; the frame is yours and must go.

**A corpus program cannot hold this**, because the surrounding error already diverges and would swamp
it. The instrument is an in-crate test asserting that this shape emits no frame line. Do not add a
corpus program that pins the `41`.

## 2. The reason given for the `RAW_STDERR_COMPARISON` entries is false

`corpus.rs:308-312`, repeated in `corpus/phase-5a.txt`. You wrote that normalisation would erase the
difference. Measured: `normalize_line` (`tests/support/mod.rs:222-243`) copies the leading spaces
**and** the marker verbatim and collapses only the run *after* it; the frame line has exactly one
space there, od-verified, so **normalisation is a no-op on all three programs' stderr**.

The entries stay -- the brief mandates them. The reason must become true. The defensible one, if you
can confirm it: the frame line's leading whitespace is part of the bytes under test, and raw mode is
the only comparison that asserts that, whatever the normaliser happens to do today. State the
measurement beside it, so a reader knows the normaliser is currently a no-op here rather than
inferring that it is doing work.

## 3. The DO control expressions differ by exactly the frame line, and this task fixes them

`run.rs:7254`, `header_number`. Measured: `do i = b. to 5`, `do i = 1 to b.` and `do i = 1 by b. to 3`
each differ from the oracle in **exactly** the frame line, on both engines. Not a regression -- but
`eval.rs:1004-1007`'s own doc predicts it, and your report says nothing was left undone.

**Ruling: fix it here rather than record it.** The task's goal sentence is an error raised inside a
native method that an operator invoked, and a DO bound reaches the same forwarded method by the same
route; the reviewer measured all three differing in exactly one line, so with finding 1's predicate in
hand this is the same call at a second site. Add the three programs to the corpus alongside the
operator three, on the same terms.

Cost if this ruling is wrong: the task grows a second site and three probes, visible in the diff and
removable. If on inspection it is **not** the same call -- if `header_number` needs a different
predicate or reaches a different method -- stop, say so precisely, and record it as a named residual
with the three programs instead. That is a report I will accept; a silent omission is not.

## 4. Two constraint violations in the new prose

* `eval.rs:1055-1056` names a set's size: "one of the **two** object shapes".
* `dispatch.rs:861`: "`pub(crate)` **since Phase 5a Task 6**" is historical framing and fails the
  constraint's own strike test.

Also the Minor ones of the same kind: `eval.rs:1053` "The one predicate", `phase-5a.txt` "The three
vary", and `eval.rs:1030` "and unchanged here".

## 5. The performance staleness test

The constraint's literal test failed at your base and you reasoned about it, which it forbids. **That
was a plan defect, not yours, and I have fixed it**: the plan now phrases the test over the commits
that touch the crate source rather than over a cumulative diff, because the phase's own tasks change
`src/` and the diff form could never pass again from your task onward. Commit `5e4f84964`; the
extracted constraints file is re-extracted.

Re-run the **new** test and record its result. Note also that your narrowed pathspec omitted
`rexx-parse/src`, which the binary links -- the conclusion survives, because only
`instruction/tests.rs` changed there, but your check did not establish it.

## 6. The negative control's transcript, promoted from Minor

Two of the three programs' control output is paraphrased -- `...` and "(same shape)" -- so
"byte-exactly" rests on a summary. And per finding 2, these programs would have reddened under the
**normalised** comparison too, so the transcript is the only thing that shows the control demonstrates
what it claims. Re-run it and paste all three in full.

## Also correct

The report's claim that the mechanism "adds no work" is contradicted by its own deterministic
`+0.045%` / `+0.055%` on `arith instructions:u`, min equal to max. Below 1% and procedurally fine, but
it is a layout effect, not an absence of work; say that instead.

## Verification this round owes

* The five gate commands, each status read **unpiped** -- a command piped into `grep` or `tail`
  reports the pipe's status.
* Finding 1's in-crate test, shown failing before the narrowing and passing after.
* The negative control in full, all three programs, raw mode.
* If you take finding 3's fix: the three DO programs measured on both engines against the oracle,
  before and after.
