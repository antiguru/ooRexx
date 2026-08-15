# Task 8 report: `PUSH`, `QUEUE`, and the in-process queue

Commit: `ed7f1e942daaadf8e425e3fd50497beaad24e774`
Parent: `1663538a73eb0b759725178fef5fd6afc2f6cae3` (HEAD at dispatch)

Status: **DONE**

## What shipped

* `rust/crates/rexx-exec/src/queue.rs` (new). `Queue`, a plain `VecDeque<Vec<u8>>`
  wrapper: `push` inserts at the head (LIFO), `queue` appends at the tail
  (FIFO), matching the oracle's shared `RexxInstructionQueue` class
  (`QueueInstruction.cpp`). Two `#[cfg(test)]` unit tests: the interleaved
  order from a 4c-shaped probe measured against the oracle, and the adjacent
  "queue alone is plain FIFO" success (CLAUDE.md's "pair a refusal with its
  adjacent success").
* `rust/crates/rexx-exec/src/lib.rs`: `mod queue;`, an `Interp::queue: Queue`
  field, `Queue::new()` in the constructor, and `instruction_owner`'s
  `Push`/`Queue` arm moved from `Some("4b")` to `None`.
* `rust/crates/rexx-exec/src/run.rs`: two new `step()` arms. Both evaluate
  the expression (or `Vec::new()` for a bare `PUSH`/`QUEUE`), render it with
  `to_text` and trace it with `trace_result`, exactly mirroring `SAY`'s own
  arm -- the oracle's `RexxInstructionQueue::execute` shares `SAY`'s
  `RexxInstructionExpression::evaluateStringExpression`, differing only in
  which end of the queue the value lands on.
* `rust/crates/rexx-exec/tests/loud.rs`: removed the `Push`/`Queue` witness
  rows (they would now assert a failure that no longer happens), updated
  every count the removal moves (28 in-scope `InstructionKind`, 12
  phase-owned witness tags), and corrected a **pre-existing** stale doc
  comment above `INSTRUCTION_WITNESSES` that Task 7 had already broken (see
  "Found along the way" below).
* `rust/crates/rexx-exec/tests/owners.rs`: `Push`/`Queue` moved from
  `Owner::Phase("4b")` to `Owner::InScope`, deleted from
  `EXPECTED_OUT_OF_SCOPE`, and `variant_counts_match_the_audited_split`'s
  counts updated (28 in scope, 0 left in `Owner::Phase("4b")`).
  `INSTRUCTION_TAGS.len() == 40` was **not** touched, per the dispatch note
  -- confirmed still 40, since only ownership moved, not the variant count.
* `rust/corpus/lang/push_queue.rex` (new) + `rust/corpus/phase-4b.txt`
  (updated list and header prose) + `rust/crates/rexx-parse/tests/
  sourceline_oracle/push_queue.txt` (new oracle fixture) -- see "Found along
  the way" for why this was necessary and not just a nice-to-have.
* `docs/superpowers/plans/phase-4-exclusions.txt`: one new KNOWN GAP entry
  covering the queue-storage coverage gap, the "what coverage exists
  instead" facts, a closes-when clause, and the corrected cross-process
  caveat with today's measurement.

## Found along the way (not named in the brief)

**1. `tests/coverage.rs`'s own criterion-1 gate requires a corpus witness,
and the brief's file list did not mention it.** Moving `Push`/`Queue` to
`Owner::InScope` in `owners.rs` immediately failed
`every_in_scope_variant_is_witnessed_by_the_phase_subsets`: that test
requires every in-scope variant to be *constructed* by some program in the
`phase-4a.txt` + `phase-4b.txt` union, independent of whether the
differential run can observe its *behavior*. The brief's "zero differential
coverage" claim is true and is about the queue's *storage* -- it does not
mean no corpus program may mention `PUSH`/`QUEUE` at all. I added
`lang/push_queue.rex` (under `TRACE R`, so its `>>>` lines give a real,
non-vacuous differential signal on the expression-evaluation/trace half,
even though the storage half stays untested by that program) and its line
to `phase-4b.txt`. This also required regenerating
`rexx-parse`'s `sourceline_oracle` fixture for the new file (that gate
walks every file under `corpus/lang/`, not just the phase subsets), using
the exact `.Package~new` driver procedure `sourceline_oracle.rs`'s own
module doc documents for this purpose -- the one place that rule is already
recorded as deliberately broken, from a scratch directory, on my own new
file only. Without this the workspace-wide `cargo test` would have
regressed. **Discovered by running the gates, not by re-reading the brief.**

**2. `lib.rs:425-431`, which the dispatch note names as "the loud-dispatch
list. Remove Push and Queue" -- this is wrong, and I did not make the
edit.** That match (`Loud::instruction`'s own `name` lookup) is an
exhaustive, no-`_`-arm total function over *every* `InstructionKind`
variant, used only to produce a display name for whichever kind reaches the
`other =>` catch-all in `run.rs`'s `step()`. It already lists several
in-scope, implemented variants alongside the out-of-scope ones --
`Interpret`, `Procedure`, `Return`, `Signal`, `Use`, `Do`, `If`, `Loop`,
`Assignment` are all there despite being implemented, because the match's
job is completeness-for-the-type, not a scope table. Removing `Push`/`Queue`
from it would be a straightforward non-exhaustive-match compile error with
no other arm to receive them, and no other in-scope variant is special-cased
out of this list the way the note implies these two should be. I left it
unchanged; `instruction_owner` (the actual scope table, `lib.rs:747`) is the
one edit this file needed, and it was made.

**3. The doc comment above `INSTRUCTION_WITNESSES` in `loud.rs` was already
false before this task touched it**, independent of Task 8: it still said
"17 entries -- 16 coarse... Call becomes two rows and Signal becomes one",
which was Task 6's arithmetic. Task 7 (the immediately preceding commit)
moved `Signal`, `Raise` and `Call::Trap` in scope, which drops the true
figures to 14/14, and nothing updated this comment when that landed --
independently re-derivable from `INSTRUCTION_TAGS`'s own `Owner::Phase(_)`
count at that commit (2 + 4 + 7 + 1 = 14), and from the fact the array
itself held exactly 14 entries. Since I was already editing this exact
array (removing the `Push`/`Queue` rows) and the comment directly describes
it, I corrected the whole history trail (Task 6's fix, Task 7's silent
regression, Task 8's fix) rather than leaving a comment that was false
before my own edit and would have become more wrong after it. This is
outside my named edit surface but follows directly from CLAUDE.md's "a
comment that states something false must be corrected, not hedged" and the
fact I was the one who noticed it while editing the surrounding lines.

## Measurements (Step 1, and the two corrections)

All three, run from a fresh `mkdir`, oracle-wrapped, stdout/stderr/rc read
as separate descriptors:

1. **The 4c-shaped probe** (`push "a"`; `queue "b"`; `push "c"`; three bare
   `PULL`s into `v1 v2 v3`; each `SAY`n): oracle prints `C`, `A`, `B`
   (`PULL` is `PARSE UPPER PULL`). Quoted literals were used instead of the
   brief's own bare `a`/`b`/`c` shorthand, deliberately: an unquoted symbol
   already reads as its own upcased name, so a probe built from bare symbols
   cannot tell "the queue stores values verbatim" apart from "the queue
   upcases at `PUSH`/`QUEUE` time" -- this probe can, and the answer is the
   former. Storage order before `PULL`'s transform: `c`, `a`, `b` (`PUSH` at
   the head, `QUEUE` at the tail). This is `queue.rs`'s own pinned unit test.
2. **Zero differential coverage, confirmed.** `push "X"` / `queue "Y"` as
   the whole program body: rc 0, empty stdout, empty stderr. Matches the
   brief exactly.
3. **Cross-process, re-confirmed and corrected.** A `push`/`queue` program
   run to completion, then a **separate** `rexx` process running `say
   queued()`: prints `0`. Re-confirms the brief's own 2026-08-03 measurement
   (I ran it fresh, dated 2026-08-04). The correction: the scoping document
   (`2026-08-01-phase-4bc-scoping.md`, I15) justified the single-program
   rule by "rxapi is confirmed running (pid 857), so the caveat is live
   rather than theoretical" -- that establishes the daemon is reachable, not
   that two process invocations actually observe each other's pushes, which
   is the thing that actually matters for a differential run. The KNOWN GAP
   row records this distinction and the corrected reasoning; I did not edit
   the scoping spec document or 4c's own pre-existing `QUEUED` row in
   `phase-4-exclusions.txt`'s "EXCLUSIONS -- partial" section (neither is in
   my named edit surface, and the latter is 4c's own row to update when that
   task lands) -- the new KNOWN GAP row instead notes that row "inherits the
   same single-program rule and the same caveat."

## Test that could fail

For the unit test in `queue.rs`: a degenerate `Push | Queue => Ok(Flow::Next)`
in `run.rs` cannot reach `queue.rs`'s own tests at all (they call `Queue`
directly, not through the executor) and so is caught instead by
`tests/loud.rs`'s removal of the `Push`/`Queue` witness rows -- the loud
gate would fail loudly on any real program touching either keyword until an
arm exists. Inside `queue.rs` itself: collapsing `push`'s `push_front` to a
second `push_back` (making the type a plain FIFO) fails
`interleaved_push_and_queue_match_the_oracle_order` (order would come out
`a, b, c` rather than `c, a, b`) but would leave `queue_alone_is_plain_fifo`
green -- which is why both tests exist, per CLAUDE.md's "pair a refusal with
its adjacent success": the second test is what shows the FIFO half is
correct for the ordinary reason, not by an accident of the LIFO half's
mistake cancelling out in the interleaved case alone.

For `lang/push_queue.rex`: without `TRACE R` this program is exactly the
"test that cannot fail" the brief warns about (empty/empty/rc0 regardless of
implementation). Its whole differential value is the `>>>` lines `TRACE R`
produces, which pin that `PUSH`/`QUEUE` evaluate, render and trace their
expression -- a stub `Ok(Flow::Next)` with no evaluation at all would either
skip the `>>>` line or trace the wrong value, and the corpus gate would
catch it.

## Gates (all five, `rust/`, exit status read unpiped)

* `cargo test --workspace` -- **978 passed / 0 failed** (was 976; +2 for
  `queue.rs`'s own two unit tests). Exit 0.
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` -- **39 of 39
  matching, mode STRICT** (was 38 of 38; +1 for `lang/push_queue.rex`).
  Exit 0.
* `cargo test -p rexx-exec --test assertions` -- **4224 / 4259** (unchanged
  from the stated baseline). Exit 0.
* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0.

## Files touched

* `rust/crates/rexx-exec/src/queue.rs` (new)
* `rust/crates/rexx-exec/src/lib.rs`
* `rust/crates/rexx-exec/src/run.rs`
* `rust/crates/rexx-exec/tests/loud.rs`
* `rust/crates/rexx-exec/tests/owners.rs`
* `rust/corpus/lang/push_queue.rex` (new)
* `rust/corpus/phase-4b.txt`
* `rust/crates/rexx-parse/tests/sourceline_oracle/push_queue.txt` (new)
* `docs/superpowers/plans/phase-4-exclusions.txt`

`rust/crates/rexx-exec/src/plan.rs` and `rust/crates/rexx-exec/tests/
coverage.rs` were left untouched, per the dispatch note -- their
`Push`/`Queue` match arms are expression-walkers grouping them correctly
with `Command`/`Say`/`Return` and are non-work.

## Concerns for the dispatcher

None that block. The two items above (the criterion-1 corpus-witness
requirement, and the `lib.rs:425-431` correction) are the two places I
deviated from the literal brief text, both with the reasoning and evidence
inline above and in the corresponding source comments/commit message.

---

## Fix round 1 (review response)

Commit: `e25c9065b8826321e8650f5ff378ddf002cbe1ff`, on top of `ed7f1e94`.

Review verdict: spec MET, quality GOOD, no Criticals. Three Important
findings and seven Minor. All addressed; each fix's test was verified to
fail before the fix and pass after, and I3's fix was additionally verified
to die to the exact mutation the review used (delete only
`self.queue.push(line)`/`self.queue.queue(line)` in `run.rs`, keeping
evaluation and tracing) before being restored and re-applied as the real fix.

**Correction to this report's own "Test that could fail" section above
(lines 138-142).** It claimed a degenerate `Push | Queue => Ok(Flow::Next)`
"is caught instead by `tests/loud.rs`'s removal of the `Push`/`Queue`
witness rows -- the loud gate would fail loudly on any real program
touching either keyword." That is false: `tests/loud.rs` only runs a
program for an *out-of-scope* variant (its own module doc says so), and
`Push`/`Queue` moved in scope this same task, so removing their witness
rows removed the only thing in that file that had ever run a `push`/`queue`
program at all. Measured directly (review's own finding, reproduced here):
with both `step` arms replaced by `Ok(Flow::Next)`, `cargo test -p
rexx-exec --test loud` is 8 passed / 0 failed, fully green. The gate that
actually catches that mutation is `corpus.rs`, via `lang/push_queue.rex`
(STRICT drops to 38 of 39). I did not verify this claim by running the
mutation before writing it; the review did. Per CLAUDE.md's own method
section, that is exactly backwards.

**I3, the substantive finding: the degenerate that survives everything was
named nowhere.** Deleting only the two queue-write calls in `run.rs` --
keeping the expression's evaluation and its trace -- left `cargo test
--workspace` at 978/0 and STRICT corpus at 39/39 before this round.
`queue.rs`'s two original unit tests construct a `Queue` directly and
cannot see whether `run.rs` ever calls into it; `lang/push_queue.rex` sees
only the trace, not the storage. Closed with a third test in `queue.rs`,
`push_and_queue_actually_write_into_the_running_interpreters_queue`: it
builds a real `Interp`, parses and activates a program (`push "a"; queue
"b"; push "c"`), runs it through `Interp::run_activation` -- the same entry
point `Interp::run` uses in production -- and asserts on `Interp::queue`'s
contents afterward, through the same private-field access the type-level
tests already use (matching this crate's own convention of each test module
keeping its own minimal `activate` helper rather than sharing one). Verified
against the exact reviewer mutation: fails with `left: [] right:
[[99], [97], [98]]`; passes once the mutation is reverted. Also verified it
still fails under the *fuller* mutation (`Push | Queue => Ok(Flow::Next)`
entirely), alongside the corpus gate (which also fails that one, 38 of 39) --
so the two tests now cover two different, non-overlapping halves of the
same property rather than one covering both by accident.

**I1 and I2** (the false "`loud.rs` catches it" claim in `queue.rs`'s own
comment, and the KNOWN GAP row's false central claim) are both corrected in
place -- see the commit's own message and `queue.rs`/`phase-4-exclusions.txt`
directly. The KNOWN GAP row is now three separately measured claims (the
no-op arm is caught differentially at 38/39; the storage-discarding
mutation was not, at 39/39, which was the real gap; that gap is now closed
by an in-crate unit test, which is not the same thing as a differential
witness) rather than one blanket, falsifiable sentence.

**M1-M7**: struct-doc wording corrected to match the module doc (M1); a
unit test's doc no longer claims to be "the whole" of anything it is not
(M2); the KNOWN GAPS section header no longer claims every row is a
divergence (M3); `run.rs`'s two near-identical `Push`/`Queue` arms merged
into one match arm sharing the eval/trace logic, picking the LIFO/FIFO
sink with one `if` at the end (M4); the front-vs-back `PULL` premise the
unit tests assume is now stated as an explicit, unenforced guard in
`queue.rs`'s module doc, addressed to whoever implements `PULL` (M5); a
second stale `owners.rs` comment from Task 7 (`ExprKind::Call`'s own entry,
still describing `InstructionKind::Call` as `Owner::Phase("4b")`) corrected
alongside M1's neighbour, on the same reasoning the review used for it
(M6); `lang/push_queue.rex` now also exercises a numeric `QUEUE 42`, not
only string literals and concatenation, re-verified byte-identical against
the oracle and its `sourceline_oracle` fixture regenerated (M7).

### Gates, re-run after all fixes (`rust/`, exit status read unpiped)

* `cargo test --workspace` -- **979 passed / 0 failed** (was 978; +1 for
  the new integration test). Exit 0.
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` -- **39 of 39
  matching, mode STRICT** (unchanged -- the new coverage is in-crate, not
  a new corpus mismatch). Exit 0.
* `cargo test -p rexx-exec --test assertions` -- **4224 / 4259**
  (unchanged). Exit 0.
* `cargo fmt --all --check` -- exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0.

### Files touched this round

* `rust/crates/rexx-exec/src/queue.rs`
* `rust/crates/rexx-exec/src/run.rs`
* `rust/crates/rexx-exec/tests/owners.rs`
* `rust/corpus/lang/push_queue.rex`
* `rust/crates/rexx-parse/tests/sourceline_oracle/push_queue.txt`
* `docs/superpowers/plans/phase-4-exclusions.txt`

### Concerns for the dispatcher

None that block. Status: DONE.
