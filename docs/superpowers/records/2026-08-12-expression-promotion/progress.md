# SDD ledger -- plan: docs/superpowers/plans/2026-08-12-expression-promotion.md

Branch: plan/rust-rewrite. Plan committed at ec649c0bc.

Open controller work, not a task:
* The `Op` width measurement. Moritz offered 16 bytes ("aligns better with
  cache lines anyway"), which would delete Task 3's side table. Two binaries
  are built and kept at `<scratchpad>/width/rexx-run-12` and `rexx-run-16`;
  the 16-byte one is HEAD plus an unreachable `Op::Pad { a, b, c }` variant and
  nothing else, which is the null control's right shape. The sitting was
  started and abandoned: the host would not pass the idle gate (`/proc/stat`
  ~1% idle, loadavg ~20 on 32 cores, from outside this sandbox), and neighbour
  contention is precisely what contaminates a cache-width read. **Take it in a
  quiet window before Task 3**, and shape Task 3 by the answer.

Task 1: implemented at 5a801e1e9, "Give the other twenty-four binary operators
one op, and one dispatch". Gates re-run by the controller independently of the
report: fmt 0, clippy 0, `memcap 8G cargo test --workspace --no-fail-fast` 0
with 1460 passed / 0 failed / 4 ignored over 82 result lines. Width assertion
still reads 12; `hints.reserve()` has exactly one call site and it is inside the
`is_arithmetic` branch. Task review dispatched.

Task 1: controller-verified finding, folded into the plan rather than left in a
report -- the tree-walker/compiled divergence on `MAX_EVAL_DEPTH` is
**pre-existing**, not introduced here: `zv = 1` plus 100,001 `+0` terms gives
tree-walker rc 245 and IR rc 0 at d0101a39a, before this plan. No compile-time
recursion cliff either: `push_native` at 700,000 terms exits 0. Task 1 re-pinned
four tests to `Engine::TreeWalker` because promoting `||` emptied them; the
reviewer is asked whether they still assert what their names claim.

Task 1 fix round 1: `1fdc2a9b1` (the eight findings) and `a8d36deb8` (a sweep
after Moritz's no-set-cardinality rule landed mid-round at `d2efbaf5b`). Gates
re-run by the controller: fmt 0, clippy 0, tests 0 with 1460 passed / 0 failed /
4 ignored -- unchanged from before the round, which is what a prose round plus
extra rows in one datadriven file should read.

Controller-verified, not taken from the report:
* Finding 4's hole is closed. `say 1 | 1` and `say 1 && 1` are both rows now, so
  `Or` computed as `Xor` fails.
* Finding 7's argument holds. `Failure::Exited(Option<ObjRef>)` is the only
  variant carrying a value, it is constructed only in `eval_call`, and
  `apply_binary`'s three functions take values rather than expressions, so none
  can produce one. An operand's own `Exited` still escapes through `?` in front
  of the new unconditional `pop_frame`, so nothing live is discarded.

Tracked follow-up, NOT part of this plan: pre-existing counts in `eval.rs` now
violate the new rule -- `arith_general`'s `unreachable!` message and
`small_int_arith`'s doc both count the arithmetic set, and `golden_tests.rs`
describes committed streams row by row. The implementer scoped its sweep to this
task's own prose, which is defensible; the repo-wide sweep is its own piece of
work and wants doing before the phase closes.

Task 1: COMPLETE. Fix round 2 at `8d7f086ec`, controller correction at the
commit above it. Commits: 5a801e1e9, 1fdc2a9b1, a8d36deb8, 8d7f086ec, plus the
controller's one-sentence fix.

Controller-verified rather than taken from the report: the NF-2 measurement was
re-taken independently -- `logical_values` mutated so `Or` computes exclusive-or,
`operators` held out, and `the_exempt_set_matches_the_current_blocked_rows`
fails. So the case file is not the sole catcher and the header's headline claim
survives. `eval.rs` restored from a copy, `sha256sum -c` OK, rebuilt, workspace
green at 1460 passed / 0 failed / 4 ignored.

Round tally, for the record this project keeps on correction rounds: round 1
answered eight findings and produced four new false statements; round 2 answered
those and produced one more of a different shape (a count of a mutable in-repo
aggregate, inside the fix for count-rot), which the controller fixed directly
rather than opening a third round for one sentence.

Task 2: COMPLETE. `1ccb81199` (the two ops) and `87f0e0d32` (fix round 1).
Gates re-run by the controller after each: fmt 0, clippy 0, tests 0, 1461 passed
/ 0 failed / 4 ignored.

Controller-verified rather than taken from the report:
* `Engine::DEFAULT` is `Engine::Ir` (`invocation.rs:160`), so `Invocation::none()`
  runs the compiled engine. Only `ir_dual.rs`, `spike.rs` and `collect_stress.rs`
  name `Engine::TreeWalker` in the test tree -- which makes `ir_dual_cases`
  very nearly the only thing pinning the tree-walker's own bytes. The committed
  header had this backwards and is corrected. Saved to memory, because the
  failure mode is a wrong premise about the harness generating several confident
  wrong sentences.
* `datadriven` reads case files through `fs::read_to_string`
  (`datadriven-0.9.0/src/lib.rs:483`), so a `0xAA`/`0xAC` prefix row genuinely
  cannot live in one. The implementer found that by reading rather than assuming,
  and the header says it.
* `form_name`'s prefix arm now calls `PrefixOp::spelling()`, which returns the
  same three strings the deleted match did, and `lib.rs:2825` asserts the
  message. Byte-identical, and covered.

Round tally: Task 2 needed one round against six findings, and the controller
found no new false statement in it -- against Task 1's four. The difference the
dispatch made was naming the specific traps rather than only the findings.

NEXT: the `Op` width sitting, before Task 3.

Entry 24 (the `Op` width) is measured and committed at `6fe41e092`. Sixteen
bytes is free on these axes; the cost a two-arm reading found belongs to having
an extra variant, which widening an existing one is not. Task 3 was reshaped by
that answer: the side table, its reserve call, its refusal and its hot-path
lookup are all withdrawn and the path went into the op.

Task 3: implemented at `266acca07` (the widening) and `b930290ec` (a correction
to this plan, which still built the withdrawn table in a passage below the
paragraph withdrawing it -- and whose Task 4 `Consumes:` line named types that
would not exist). Gates re-run by the controller: fmt 0, clippy 0, tests 0 with
1464 passed / 0 failed / 4 ignored. `NodeAddr` and `Chunk::nodes` are absent
from the tree. Review dispatched.

Controller note on the width check, which is worth more than the result: my own
first attempt reported a pass it never took. Adding the fields leaves
`E0063`/`E0027` errors, const evaluation does not run while those stand, and the
absence of an `E0080` reads exactly like the assertion holding -- setting it to a
value that cannot be true produced no panic either, which is what exposed it.
The implementer then refined it: `E0308` type errors do NOT suppress const
evaluation, only the field-shaped ones do. That is in the plan's Task 3 text so
the next reader of the check sees it.

Moritz's ruling on `NodePath`, asked and answered 2026-08-12: keep it as built,
including `Op::TraceFunction`'s copy of the path.

NEXT: Task 4, dispatched once Task 3's review verdict is in -- Task 4 builds
directly on `chunk_node_at`, which is the one place a real defect would live.

Task 3: COMPLETE. `266acca07`, `b930290ec`, `4cc6f8943`, plus the controller's
`464c5b31b`. Gates: fmt 0, clippy 0, tests 0, 1464 passed / 0 failed.

Controller-verified: removing `.rev()` from `NodePath::steps` now reddens both
`a_node_paths_steps_come_back_outermost_first` and the descent test
`a_paths_steps_land_on_the_node_it_names`. Before the fix round the second
passed under that mutation, which was the review's finding 3. Restored from a
copy, `sha256sum -c` OK, suite green.

**The const-eval mechanism is withdrawn and no mechanism replaces it.** Adding a
field to `Op::CallExpr` with every site unedited prints `E0063`, `E0027` AND
`E0080` together -- measured unpiped by the reviewer and again by the controller.
The original claim was a `head -4` truncation artifact of mine, written into the
plan and into `faf6360ce`'s message, which cannot be edited; the implementer had
refined it into a second false claim about `E0308`. Corrected at `464c5b31b` and
saved to memory as its own failure shape.

Recurring defect worth naming: **a comment claiming a discrimination its own test
or rows cannot make** has now appeared in three of this plan's four tasks --
Task 1's logical stanza, Task 2's sign-flip and source-span comments, Task 3's
descent-test doc. It is in every dispatch from here.

NEXT: Task 4, the promotion the address widening was for.

Task 4: COMPLETE. `b73ef0b6e` (the promotion) and `8a48bbb1d` (fix round 1),
plus the controller's `f1342b15f`. Gates: fmt 0, clippy 0, tests 0, 1464 passed.
Review verdicts: spec compliance PASS, code quality PASS WITH FINDINGS, all
findings answered.

ALL FOUR TASKS DONE. Remaining: the two interleaved sittings the plan owes.

Fact about the loop, worth carrying to the next plan: **a comment claiming a
discrimination its own tests or rows cannot make appeared in all four tasks**,
was caught by review every time and by the implementer none. Task 4's dispatch
named the defect class explicitly and asked for exactly that check, and the
task shipped an instance anyway (`Op::CallExpr`'s doc claiming a call promotes
"wherever it sits", where a single non-native sibling term leaves the whole slot
general and the call with no op -- untested until the fix round). Dispatch prose
is not a control; the review is.

PLAN COMPLETE. Final whole-branch review done (`final-review.md`): code coherent
across all four tasks, two prose defects spanning tasks.

* Finding 2 was the plan's Global constraints still saying the twelve-byte
  assertion "stays and is not relaxed" and telling a task that trips it to
  report rather than widen -- after Task 3 widened it to sixteen with Moritz's
  agreement. Fixed by the controller at `e4b939b02`. That section is what briefs
  are generated from, so it was instructing future tasks to refuse a landed
  change.
* Findings 1, 3, 4, 5, 6 are crate prose and are dispatched to `final-fix`,
  IN FLIGHT at the time of writing: `Op::Arith`'s doc claiming
  `length('ab') + 1` is one `EvalExpr`, `Op::Load`'s claiming `x + 1` is,
  `ir/mod.rs` keeping counts `eval.rs` dropped, `Loud::call_op_off_its_node`'s
  doc not widened for Task 3's two new failure routes, and a stale "neither is
  arithmetic" justification. Uncommitted edits to `drive.rs`, `golden_tests.rs`,
  `ir/mod.rs` and `lib.rs` in the tree are that agent's.

PICKUP STATE. HEAD `e4b939b02`. Entry 25 records the sitting Moritz accepted:
strings -10.04% and alloc4c -7.98% (both 9/9), against arith/varlookup/emptyloop
losing every round at +3.96% to +6.67% -- and emptyloop is instruction-identical,
83 apart out of 27.5 billion, so its regression is layout and not work.

NEXT, in the order I would take it:
1. Merge `Op` variants (`Op::Arith` into `Op::Binary` carrying the hint; the
   trace ops into one with a tag). Entry 25 makes variant count a measured cost.
   Build the control entry 25 says is missing -- head with the new ops present
   and nothing emitting them -- which would also settle the regression's
   mechanism.
2. The comma list, then RETURN/EXIT/PUSH/QUEUE, planned in
   `docs/superpowers/plans/2026-08-12-remaining-promotion-survey.md`.
3. `rexxcps`, still the worst axis by fraction still to lose. Before ranking
   anything off its 86-of-142 static coverage, take an execution count per
   instruction -- static text is not executed clauses.

Final-fix round: `5acdd69f0`, all five crate prose findings. Gates re-run by the
controller: fmt 0, clippy 0, tests 0, 1464 passed / 0 failed.

One residual adjudicated by the controller rather than deferred: the loud
message `call_op_off_its_node` still read "a compiled Call op does not name a
CALL name of its own body", narrower than the ops that report it since Task 3 --
a failing `Op::TraceFunction` descent printed a sentence about a `Call` op. The
fix round reported it instead of fixing it because widening the string changes
output and the round was scoped to comments, which was the right call. Widened
here after checking nothing in the sources pins the text.

PLAN CLOSED. Nothing outstanding.
