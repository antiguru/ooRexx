# SDD ledger — plan: docs/superpowers/plans/2026-09-15-file-split.md

Plan committed at `ec7824e4a`. It runs after the gate table C repair
(`5757857a4`) and before Phase 8 surface Task 4.

Pre-flight scan of the plan against itself, before Task 1 is dispatched:

- Tasks 2 through 10 each name a disjoint set of files, so no two tasks move the
  same lines. Task 1 is a survey that proposes split-or-leave per candidate and
  moves nothing; Task 11 closes.
- Task 1 produces the rulings that Tasks 2 through 10 consume. Every later task
  therefore depends on a controller ruling, not on another task's code, which is
  what keeps them independent.
- The plan mandates no test that asserts a file's length, and no allowlist. The
  review rubric treats an assertion-free test as a defect; nothing here creates
  one.
- The file-granular invariants (`unsafe_sites.rs` for D-U1, `environment_seam.rs`
  for D45, `dispatch_seam.rs`) name files, and a split that moved their subject
  would have to move the record with it. The plan says so; the conflict to watch
  is a task that splits one of those files without touching its record, which is
  a per-task review item rather than a plan defect.
- `ffi.rs` and `load.rs` stay as they are where self-containment would move
  `unsafe` out of them. That is a deliberate exception to the plan's own
  principle and is stated in it.

No conflict found that needs a ruling before execution begins.

## Task 1: complete

Commit `efe70f926`. Records committed under
`docs/superpowers/records/2026-09-15-file-split/`, not the ignored workspace
path my brief named. The agent was right and the brief was wrong: `.superpowers/`
is git-ignored and the 2026-08-15 ruling puts committed records under
`docs/superpowers/records/<plan-basename>/`. I still owe that directory copies of
this ledger and the task brief.

Enumeration was done twice, by length and by responsibility from 400 lines up,
with a third measurement that reordered the list: each candidate's inline
`#[cfg(test)] mod` span separated from the rest, because several files trip the
trigger only on their test module. The agent's first script for that span was
wrong, read a `#[cfg(test)]` on a function as a module start, and gave `lib.rs`
5713 test lines against a true 976; it caught this itself, rechecked every figure
the corrected script disagreed with, and re-ran every command the survey quotes.
That is the failure mode this project keeps hitting, found and corrected inside
the task rather than by a reviewer.

### Rulings on the survey, before any task moves code

1. **`ffi.rs` leaves Task 8.** "Split only by moving safe code out" has no safe
   code to move: the `#[cfg(test)]` stub region and `mod tests` both contain
   `unsafe` in the forms `unsafe_sites.rs` matches, so any child file holding
   them turns D-U1's record red, and `mod tests` alone would leave most of the
   file. Cost if wrong: `ffi.rs` stays long. Accepted, because D-U1 is a
   standing constraint and this plan is not the place to renegotiate it.
2. **Order changes: `run/tests.rs` and `rexx-bench-suite.rs` go first**, before
   `dispatch.rs`. They move no production code, so they exercise the four
   instruments, the `cargo doc` pass and the interleaved performance procedure
   while nothing depends on those working. `dispatch.rs` is the riskiest file
   and the one where the performance constraint bites hardest; it should not be
   the first to rely on unproven instruments. Cost if wrong: two low-value tasks
   land before the high-value one.
3. **Task 10 shrinks to `rexx-bench-suite.rs` and `gate_table_c.rs`.** The rest
   of its list meets the plan's own "one set of test cases for one function"
   exception and is a leave. Same for Task 5's `error.rs` and `activation.rs`.
4. **The plan's D-U1 statement is corrected** to include `rexx-core/src/bytes.rs`
   and `rexx-core/src/lib.rs`, and the pins the survey found are written into the
   tasks that would trip them: `mod seam`, `method_is_protected` (whose
   occurrences must sit in a path containing the literal `dispatch.rs`) and the
   `seam::clear(` call in `Interp::invoke` for the dispatch task; `collect_now`
   for the `lib.rs` task; `refusal-sites.tsv` column 3 being path-derived, which
   survives a dispatch split only because `/dispatch/` carries the same tag, and
   that is to be asserted rather than relied on; and the `module is
   src/docs/classes.rs` headers in `class-set.txt` and `class-methods.txt`.
5. **`ir_recorded.rs`'s inline `LOOP_CASES`/`BRANCH_CASES` stay out of this
   plan.** Moving them to a data file matches a recorded preference of Moritz's,
   but it is not a pure move and this plan's every instrument assumes one.
   Queued separately rather than folded in. Cost if wrong: the preference waits.

The plan amendments are held until the gate run in flight at `5b0b69170`
finishes, because a gate run spanning a controller edit is void.
