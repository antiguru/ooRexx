# Task 8 fix round 1: re-review

Reviewing `ed7f1e94..e25c9065` (one commit, `e25c9065`, 6 files, 199
insertions / 92 deletions) against `task-8-review.md`'s ten findings and
`task-8-report.md`'s fix-round account. Scope: did round 1 close the ten
findings, and did it introduce anything new. Task 8's implementation itself
is not re-litigated.

All mutations below were applied with `Edit`, verified, then restored with
`git checkout --`; `git status --short` is empty and the tree is at
`e25c9065` throughout.

## Baseline re-established at `e25c9065` (ran)

* `cargo test --workspace`: 979 passed, 0 failed. Matches report.
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`: 39 of 39,
  STRICT.
* `cargo fmt --all --check`: exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.

## Verdicts, finding by finding

**I1 -- CLOSED (ran).** `queue.rs`'s comment above
`interleaved_push_and_queue_match_the_oracle_order` no longer claims
`tests/loud.rs` exercises `run.rs`'s arms; it now says explicitly that
`loud.rs` "proves neither" (wiring nor evaluation), naming the two tests
that actually do. The report's own false claim (lines 138-142 of the
original report) is corrected in the fix-round section. Re-verified
independently: with both `step` arms replaced by `Ok(Flow::Next)`,
`cargo test -p rexx-exec --test loud` is 8 passed / 0 failed.

**I2 -- CLOSED (ran).** The KNOWN GAP row's central claim is no longer one
falsifiable sentence; it is three separately measured claims. All three
verified true by running:
* `Push | Queue => Ok(Flow::Next)` (both arms replaced, nothing evaluated,
  nothing stored) -- STRICT corpus drops from 39 of 39 to 38 of 39, exactly
  as claimed.
* Evaluate and trace, discard the line (delete only the two
  `self.queue.push`/`self.queue.queue` calls, `let _ = line;`) -- STRICT
  corpus stays 39 of 39 (confirmed), `cargo test --workspace` drops to 298
  passed / 1 failed, the failure being exactly
  `queue::tests::push_and_queue_actually_write_into_the_running_interpreters_queue`
  (confirmed, reproducing the controller's own number independently).
* The claim that the third test is a unit test comparing against a
  measured constant, not a live oracle run -- true by inspection of the
  test body (`assert_eq!` against a hardcoded `VecDeque::from([...])`).

**I3 -- CLOSED (ran), and the new test is not fragile.** The new test
`push_and_queue_actually_write_into_the_running_interpreters_queue` was
checked against four mutations, not just the one the controller already
tried:
1. Delete only the two queue-write calls (the original reviewer mutation)
   -- dies. Reproduced: 298 passed / 1 failed, same single failure.
2. Full no-op (`Push | Queue => Ok(Flow::Next)`) -- dies (`left: []`).
3. Swap which branch calls `push` vs `queue` in `run.rs`'s merged arm
   (order corruption, not deletion) -- dies (`left: [[98],[97],[99]]`,
   `right: [[99],[97],[98]]`).
4. Swap `push_front`/`push_back` inside `Queue`'s own methods (order
   corruption at the type level) -- dies, along with both original unit
   tests.
5. "Stores only the last value" (`self.lines.clear()` before each insert,
   a corruption neither deletion nor a swap) -- dies, along with both
   original unit tests (`left: [[99]]` vs `right: [[99],[97],[98]]`).

So the guard catches corruption as well as deletion; it is not a test that
only detects its own mutation.

**M1 -- CLOSED (read).** Struct doc now says "the head being the next line
`PULL` will remove," matching the module doc's own phrasing, no longer the
misleading "oldest `PULL` target at the front."

**M2 -- CLOSED (read).** The unit test's doc no longer claims to be "the
whole of Task 8's coverage"; it states precisely what it proves (wiring of
`push_front`/`push_back` to the right keyword) and points to the other two
tests/gates for what it does not.

**M3 -- CLOSED (read).** Section header changed from "Each is a real
divergence" to "Most rows are a real divergence... One row (PUSH/QUEUE's
queue storage) is not a divergence at all," matching what the review found:
only this one row is the exception.

**M4 -- CLOSED (ran).** `run.rs`'s two arms merged into one
`InstructionKind::Push { .. } | InstructionKind::Queue { .. }` match arm
with a single `if matches!(...) { push } else { queue }` at the end.
Compiles clean, clippy clean, and the merge does not blur the LIFO/FIFO
distinction -- confirmed by mutation 3 above, which dies exactly because the
`if`/`else` still selects the right sink.

**M5 -- CLOSED (reasoned from deterministic `VecDeque` semantics, not
run, since `PULL` does not exist yet to run).** `queue.rs`'s module doc now
carries an explicit, addressed-to-4c paragraph: "4c's own `PULL` must pop
the front... a `pop_back` implementation would leave every test in this
file green while printing `B`, `A`, `C` instead." Checked the arithmetic:
current order is `[c, a, b]` (front to back); `pop_back` removes `b` first,
then `a`, then `c`, so upcased output would be `B`, `A`, `C` -- exactly what
the comment claims. This hands 4c the premise as an unenforced guard to
satisfy, not a settled conclusion to inherit uncritically, which was the
concern the review raised.

**M6 -- CLOSED (ran).** `owners.rs`'s `ExprKind::Call` comment corrected to
say `InstructionKind::Call` is `Owner::Phase("Phase 5")` since Task 7 moved
`Call::Trap` in scope, leaving only `Call::Qualified` loud. Verified against
current code: `owners.rs:129` is
`InstructionKind::Call(_) => ("Call", Owner::Phase("Phase 5"))`, and
`lib.rs:723-728` is
`Call::Named { .. } | Call::Dynamic { .. } | Call::Trap(_) => None,
Call::Qualified { .. } => Some("Phase 5")`. The corrected comment is
accurate.

**M7 -- CLOSED (ran).** `lang/push_queue.rex` now also exercises `queue 42`.
Regenerated `crates/rexx-parse/tests/sourceline_oracle/push_queue.txt` from
scratch using the exact driver `sourceline_oracle.rs`'s module doc
documents, from a fresh `mkdir`, against a *copy* of the corpus file (never
the repository file) -- `md5sum` identical to the committed fixture
(`94d820b5a446a46c971ca94a77173499`). The oracle's own trace output for the
new line is `>>>   "42"`, and the new header's claim "a number renders
through a different path than a literal does" is true of this crate too:
`value.rs`'s `to_text` has a distinct `Body::Num` branch calling
`number.format_form(...)` versus `Body::Text { bytes, .. } =>
Cow::Borrowed(bytes.as_slice())` for a literal/concatenation. `push_queue.rex`
is confirmed present in `phase-4b.txt` and is part of the 39/39 STRICT
corpus pass.

## New findings

**NEW-1. `rust/crates/rexx-exec/src/lib.rs:783-786` -- the same false claim
M6 fixed in `owners.rs` still stands, uncorrected, in a second file (ran;
found during re-review, not in the original ten; pre-existing, not
introduced by round 1 -- round 1 did not touch `lib.rs`).** `expr_owner`'s
comment on `ExprKind::Call` reads: "unlike `InstructionKind::Call`, which
keeps an owner string because `Call::Trap`/`Call::Qualified` are still
loud, this variant has no later-phase arm hiding inside it." This is the
identical defect M6 named and fixed at `owners.rs:185-189`: `Call::Trap`
has been in scope since Task 7, not "still loud" -- verified against the
same code cited above (`lib.rs:723-728` itself, twenty lines below the
false comment, correctly puts `Call::Trap(_) => None`). CLAUDE.md requires
this be corrected, not hedged. Severity: Minor (documentation only, same
class as M6). **Load-bearing**: the fix round explicitly corrected this
exact sentence in one location while leaving its sibling false in another,
so a reader following either file gets a different (and in `lib.rs`'s case,
wrong) answer to the same question -- worth closing in the same pass as
M6 rather than parking, since it is now the only remaining instance of the
finding class the review already spent effort naming.

## What is not wrong

Re-checked, found nothing: `run.rs`'s merged arm still evaluates before
tracing before storing, in that order, for both keywords (confirmed by
mutation 5 above still producing a trace-consistent, storage-corrupted
result rather than skipping evaluation); no `unsafe` introduced; `cargo
fmt`/`clippy` clean on the full round-1 diff; no stray files left in the
tree after any restored mutation (`git status --short` empty throughout).
