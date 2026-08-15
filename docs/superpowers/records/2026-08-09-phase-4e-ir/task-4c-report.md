# Task 4c: flatten the loop header -- report

BASE `de05ea58`. Commits `08137f3a`, `b6d54856`, `38e3dc89` (the anchor), `4d8e913a`, fix round `693ec3d0`.

Suite 1378 to **1382**, 0 failed, 4 ignored, green in dev, dev+STRICT, release and release+STRICT.
`cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean from an
empty target directory (`CARGO_TARGET_DIR` pointed at a fresh path, so cargo could not reuse a
per-crate result -- the failure mode `rust/CLAUDE.md` records).

## Fix round 1

Seven Minors, all addressed in `693ec3d0`; the Important was the criterion-4 re-measurement and the
coordinator did it (`6ce592ff`).

* **m1** -- the M1 mutation row undercounted its catchers, five golden tests rather than four. Re-run,
  and the reason it was wrong is recorded with the row.
* **m2** -- `step`'s new comment enumerated what "every other `run_bounded` call on this page" does,
  which is an in-repo enumeration in prose. Replaced with the property the signature now enforces:
  `step` takes no `BodyEngine`, so nothing can be threaded in without changing it.
* **m3**/**m3b** -- both in the anchor and both mine to correct: 4b-M's withdrawn ratios and Task 6 as a
  candidate cause are gone, replaced by the review's per-pass attribution, and the one-comment control's
  cause is corrected from code layout to measurement noise. See the two sections below.
* **m4** -- two loop-header divergences the table did not reach are recorded, measured rather than
  taken on report.
* **m5** -- a region's length was computed ahead of the ops it describes, which is the emitted shape
  written twice in one arm. Regions are now opened with a placeholder and closed from the stream's own
  length, at all four sites rather than only the loop's, since the shape is the same at each. The golden
  tests pin every `end` exactly, so the change is checked rather than argued.
* **m6** -- `echo_header_value` rendered the value's text before the `>K>` gate. Gated, the same shape
  and reason as `bind_control`'s own check.
* **m7** was the coordinator's own and is theirs.

`4d8e913a` corrects one false reason in a doc comment: `Op::LoopRun`'s credited the `DO` clause's temps
frame with rooting a `DO OVER`'s target across the loop, which is how the tree-walker roots it and not
how the compiled stream does -- the register is, and the enclosing-scope allocation is what keeps it.
The decision was right and the reason was wrong, which is the shape that passes review on the
decision's merits.

## The prediction, recorded before anything was measured

Against `docs/superpowers/plans/phase-4e-anchor.md`, and **not** against Task 4a's
-1.7%/-3.2%, which have no named mechanism.

**Predicted: no movement on either axis, on either arm.** Concretely, `|delta| < 1%` on
`emptyloop` and on `varlookup`, for the tree-walker arm and for the compiled-stream arm alike, and
no movement attributable to this change.

The mechanism for the prediction: everything this task moves is paid **once per loop entry**, and
neither benchmark enters a loop more than once. `bench-programs/emptyloop.rex` is
`do i = 1 to n / nop / end` with `n = 25000000`, and `varlookup.rex` is one loop over two
assignments; the header is evaluated once in each. What the compiled stream now does per entry is
`in_stepped_clause_with` plus six region ops instead of `step_in_temps_frame_with` plus one op, and
what it does per pass is unchanged -- `run_repeating` still drives every iteration and still enters
`run_bounded` once per pass for the body's range. The tree-walker arm's header evaluation moved from
`setup_controlled` to a loop over `loop_header_plan`, which is also once per entry.

So this task does **not** discharge the plan's "predicted movement: `emptyloop` and `varlookup`".
That prediction belongs to flattening the loop's **per-pass** header, which this task does not do,
and the "What this task is not" section below states why that is a separate piece of work rather
than a step left out.

## The op set, and the header's compiled shape

Three new ops, one removed.

* `Op::TraceKeyword { role, src }` -- one header value's `>K>` line, from a register, under the tag
  its `HeaderRole` gives it. **The emission is a separate op**, which is what makes the ordering
  expressible at all and is the shape Task 6 established.
* `Op::LoopHeaderValue { role, src }` -- that value's validation, filed for the op below.
* `Op::LoopRun { index }` -- the construct, from the values the ops before it filed, with the body's
  clauses stepped from this chunk.
* `Op::Loop` is **gone**. It existed to hand a `DO`/`LOOP` instruction to the clause unit with the
  chunk's engine attached, and a flattened header does not need it.

`do i = 1 to 3 / nop / end` compiles to, and this is asserted verbatim in
`a_counted_loop_compiles_its_header_to_a_clause_region_and_its_body_to_generic`:

```
0: Clause index=0 end=7
1: EvalExpr index=0 slot=0 dst=0
2: LoopHeaderValue role=Initial src=0
3: EvalExpr index=0 slot=1 dst=1
4: TraceKeyword role=To src=1
5: LoopHeaderValue role=To src=1
6: LoopRun index=0
7: Generic index=1
8: Generic index=2
```

**One group per header expression, in the order the expressions were written**, and inside a group
the order is evaluate, echo, validate. Both boundaries of that order are observable and both are
measured against the oracle: `do i = 1 to 'a' by zf()` raises before `zf` is called, so the
validation may not move after the next expression's evaluation; and `trace r` over
`do i = 1 to 'a' by 2` prints `>K>  "TO" => "a"` for the value that then fails, so the echo may not
move after the validation. A single evaluate-and-echo-and-validate op would satisfy both and is what
the plan rejected, because it is loop-header-specific where `EvalExpr` is the general op Tasks 7 to 9
need.

`LoopRun` is the **last op of the region, inside it**. The whole loop therefore runs inside the `DO`
clause exactly as it does on the tree-walker: the clause's temps frame stays open across every pass,
its boundary is where the tree-walker has it, and no behaviour moves. The body's clauses are not
region ops -- they are reached through `run_bounded`, which re-enters the driver for the body's range.

A refused `DO`/`LOOP` compiles to the same shape with an empty header region, and the refusal is
`run_loop_with_header`'s. See the defect section: getting that wrong was the one real bug in this
task.

## What was extracted, and what is now written once

* `loop_header_plan(&Loop) -> Option<HeaderPlan>` -- **the order**, and the refusal. The tree-walker's
  `eval_loop_header` iterates it; `compile` emits one group per entry from the same iteration. So the
  evaluation order exists once, not as a Rust loop and an op stream to keep in step.
* `Interp::accept_header_value(role, value, &mut values)` -- **the validation**, one implementation of
  what each role requires, entered from the tree-walker's loop and from `Op::LoopHeaderValue`.
* `Interp::echo_header_value(role, value)` -- **the echo**, likewise, entered from the tree-walker's
  loop and from `Op::TraceKeyword`.
* `Interp::run_loop_with_header(...)` -- **the construct**, which is `run_loop`'s old five-arm match
  with the evaluation lifted out. Both engines reach every iteration, `WHILE`/`UNTIL`, the
  `LEAVE`/`ITERATE` search and every trace echo through it.
* `setup_controlled` is gone, absorbed into the three above.
* `step_in_temps_frame_with` is gone, and so is `step`'s `BodyEngine` parameter. It had exactly one
  caller that passed anything but `TreeWalker` -- `Op::Loop` -- and with that op gone, everything
  reaching `step` is being stepped by the tree-walker. Leaving a parameter with one possible value is
  the shape that made Task 4a's I1 defect invisible.

Two behaviour-preserving unifications inside that extraction, both stated because they are changes
rather than pure moves:

* the `>K>` indent is now read from `clause_state.current_value_indent` **at each echo**, where
  `setup_controlled` captured it once before the header's first evaluation and `Count`/`Over` read it
  after their own. All three are the same answer, because `resolve_and_run_call` restores that field
  (pinned by `current_value_indent_is_restored_after_a_call`); reading it where it is used is what
  makes both engines read the same field at the same point rather than agreeing by that argument.
* the digits a bound is rounded at are read inside `accept_header_value` rather than once before the
  header. Same answer for the same reason: `NUMERIC DIGITS` changes only by executing an instruction,
  this activation executes none between its own header's expressions, and a callee's setting does not
  survive its return.

## Why the clause counts did not change, derived rather than adjusted

`the_ir_engine_steps_a_loop_body_from_the_chunk` still asserts **4** and
`the_ir_engine_steps_a_simple_blocks_body_from_the_chunk` still asserts **3**. Neither number was
touched.

The derivation. `count_clause_op_entry` fires on every op that opens a clause, which after this change
is `Op::Generic` and `Op::Clause`. A three-pass loop over a one-clause body emits `Clause` once for
the `DO` and `Generic` once for the body instruction; the body clause is stepped three times and the
`DO`'s clause is opened once. 1 + 3 = 4. Before the change the `DO` contributed one `Op::Loop` entry
instead of one `Op::Clause` entry -- **the same one**. The block is 1 + 2 = 3 by the same count.

The tripwire was expected to move and did not, and that is the honest reading of what this task did:
it changed *where the DO clause's work comes from* and not *how many clauses the stream opens*. What
would move the count is the loop's **per-pass** header becoming a region of its own, which is the
piece below. The count's own doc comment now carries this derivation, so the next task does not have
to rediscover which change the number is sensitive to.

## The defect this task shipped and then fixed

`08137f3a` put the refusal in `run_loop` -- the tree-walker's own entry -- and described it in
`Op::LoopRun`'s doc comment as if both engines reached it. They did not. On the compiled stream:

* `do with index i over 5` hit `unreachable!("DO WITH takes the loud path above")`;
* `do counter c i = 1 to 2` hit `expect("a controlled loop's plan always names its initial value")`;
* `do qq over a.` hit `expect("a DO OVER's plan always names its target")`.

All three at rc 101 where the tree-walker refuses loudly at rc 120, and **the whole workspace was
green**: nothing ran a refused loop on the compiled stream. It was found by a mutation probe aimed at
something else -- deleting a refusal check that turned out not to exist -- which is the second time in
this phase that the mutation exercise found a live bug rather than confirming a test.

Fixed in `b6d54856`: the check moved into `run_loop_with_header`, the function both engines enter, and
`tests/ir_dual_cases/loop-refusals` is the three rows that hold it there.

## Tests, with mutation and adds-coverage evidence

Every mutation was run with `cargo test --workspace --no-fail-fast`, so "nothing else caught it" is
measured rather than assumed, and every restore was from a `cp` backup verified with `sha256sum -c`.
The workspace was rebuilt after each restore -- the runs below are of freshly built binaries, not of
one that survived a source revert.

New tests: five golden tests and two `datadriven` case files (11 rows and 3 rows). Suite 1378 to
**1382**; the case files run inside the existing `both_engines_agree_on_every_case_file`, so they add
rows rather than test counts.

| mutation | red | red with the new tests removed | what that says |
|---|---|---|---|
| M1: emit every `LoopHeaderValue` after every `EvalExpr` (defer validation) | `loop-header-boundaries` + **5** golden | 5 golden only | the case file is the **only behavioural** catcher; the golden tests catch the shape, which is not the same claim |
| M2: emit `TraceKeyword` after `LoopHeaderValue` | `loop-header-boundaries` + 4 golden | 4 golden only | same, for the other half of the order. Four and not five, because the fifth golden test asserts an `Initial` group and `Initial` has no `>K>` line to move |
| M3: drop the `TraceKeyword` op | `LOOP_CASES`, the corpus population sweep, `loop-header-boundaries`, 5 golden, one count test | -- | broadly caught, and **a poor mutation**: it also leaves the region's `end` one op too long, so part of what goes red is a malformed stream rather than a missing echo |
| M4: release the header's registers at the region's end | `a_nested_loops_registers_sit_above_the_enclosing_loops_...` alone | -- | the new golden test is the only catcher |
| M5: no refusal in either engine's path | 4 pre-existing `run::tests` + `loop-refusals` | -- | the pre-existing tests already cover the tree-walker path, so this mutation does **not** measure the new file |
| M5b: refusal in the tree-walker's entry only -- **the arrangement that shipped in `08137f3a`** | `loop-refusals` alone | nothing at all | the new file is the only thing in the workspace that catches the real defect |
| M6b: `release_to` keeps the highest mark | `two_constructs_ending_at_one_instruction_release_to_the_lower_mark` alone | -- | the new golden test is the only catcher |
| M7: `Op::LoopRun` passes `BodyEngine::TreeWalker` | both clause-count tests | -- | the promotion keeps its witness: the body's clauses still reach the stream |

**The M1 row first said four golden tests and the count was mine to get wrong.** I ran M1 before
writing the fifth golden test and never re-ran it after adding one, so the row recorded a suite that no
longer existed. Both rows above are re-run against the tree as committed. The rule this earns is narrower
than "re-run mutations": **a mutation row is a measurement of a particular suite, so adding a test
invalidates every row taken before it**, and the cheap discipline is to re-run the table once after the
last test lands rather than to re-run each row when it is written.

**One test I wrote could not fail, and I caught it by running the mutation rather than by reading it.**
`two_constructs_ending_at_one_instruction_release_to_the_lower_mark` first asserted `chunk.registers`,
which is three whichever mark wins -- the high-water mark records how deep the stack went and a
release only decides which indices come *next*. It passed under M6b. The assertion is now on the
second loop's own register indices, and the doc comment says the count does not distinguish and why.

**The 11 oracle rows, and how "would this ever have failed" was recovered.** The cases were written
after the compiler change, not before it, which is the order the brief asks against. The property that
ordering buys was recovered by measurement instead: every one of the 11 programs was run on **both
engines** against a release build of `de05ea58` in a separate worktree, and all 22 outputs are
byte-identical to the same runs at head. So each row agreed on both engines before anything was
flattened, and each row's expected bytes are the oracle's -- measured directly, one program per fresh
empty directory, absolute paths, stdout and stderr as separate descriptors.

**Nine of the eleven rows are boundary cases**, built around where a clause boundary falls rather than
around what a loop computes:

* a handler queued by a `TO` expression, delivered at the `DO` clause's boundary before the first body
  clause reads what it set;
* the same handler **queueing a second trap behind itself** -- `SIGL 4`, the first body clause, so the
  boundary that delivered the first is not the one that delivers the second;
* a handler that *fails* at the `DO` clause's boundary, so the clause blamed is the `DO`;
* a handler queued by a `WHILE` condition, delivered at the per-pass re-test's own boundary;
* that handler queueing again -- `SIGL 5`, the `END`;
* a handler queued by an `UNTIL` condition -- `SIGL 5`, the `END`, because `UNTIL` is reached from
  whichever instruction transferred control back to the loop;
* a handler queued by a `DO OVER` target, which is the one kind whose header value outlives the header;
* `PROCEDURE` as a loop body's first instruction, 17.1, which says the promoted clause spends the
  first-instruction permission as the unpromoted one did;
* and the three ordering rows above.

## Divergences met, and which are new

**None are new.** One row of `loop-header-boundaries` differs from the oracle and says so in its own
comment: a `CALL ON` handler failing at a `DO` header's boundary echoes its own clause and the `DO`'s
two columns short of the oracle's. Measured at `de05ea58` too, on both engines, so it is not this
promotion's -- it is the family `task-4bp-report.md` catalogues, whose `SELECT`-header row is the same
two-column gap. `KNOWN_DIVERGENCES` gained no row: the two engines agree on every program measured
here.

**Two more that the first version of the table did not reach**, found by the review and measured by me
before recording them. Both engine-agreeing, both byte-identical at `de05ea58`, both the same elided-
instruction family:

* a **zero-pass** loop whose requeued handler has no body clause behind it -- `do i = 1 to raiser()`
  with `raiser` returning `0`: the oracle prints `after` then `G ran 6`, delivering the second trap at
  the next real clause, and this crate prints `G ran 3` then `after`, delivering it at the `DO` clause's
  own boundary. The one-pass row in the table reports `G ran 4` on both sides, so it does not cover
  this: what differs is that with zero passes there is no body clause between the header's boundary and
  the loop's end to take the delivery instead.
* an **`UNTIL` requeue** -- the oracle delivers the second trap at the `END`'s own clause (`G ran 5`) and
  this crate at the clause after the loop (`G ran 6`). The `END` is an instruction the oracle executes
  and this crate echoes without stepping.

They are written into `loop-header-boundaries`' own header rather than only here, because that file is
what the next reader of a loop-header boundary opens.

## Predicted versus measured

Prediction, recorded at the top of this file before anything was run: **no movement on either axis on
either arm**, `|delta| < 1%`, because everything this task moves is paid once per loop entry.

Measured, `perf stat -e instructions:u`, interleaved base-against-head within one sitting, three
rounds per cell:

| axis | arm | base | head | delta |
|---|---|---:|---:|---:|
| `emptyloop` | tree-walker | 38.0007e9 | 37.8507e9 | -0.395% |
| `emptyloop` | IR | 40.4007e9 | 40.3257e9 | -0.186% |
| `varlookup` | tree-walker | 71.8207e9 | 71.6307e9 | -0.265% |
| `varlookup` | IR | 74.0437e9 | 73.9297e9 | -0.154% |

**The prediction was wrong**: there is a movement, it is a saving, and it is per *clause* -- 150e6 over
25e6 passes is exactly 6 instructions, and `varlookup`'s 190e6 over 38e6 body clauses is 5. So I
under-counted my own change, which touches the per-clause path through `step`.

**And the mechanism I then proposed is refuted.** A control build of `b6d54856` with `step`'s
`BodyEngine` parameter put back and nothing else changed reads 37.8507e9 and 40.3257e9 -- head's
figures to eight significant figures, on both arms. So the saving comes from somewhere else in the
diff and no route has been named. It is recorded, not claimed.

Wall clock and cycles both read head **slower** on all four cells (+6.2%/+1.3% on `emptyloop`,
+1.8%/+4.5% on `varlookup`) while executing fewer instructions. That is not attributed either, and the
reason is a control: **two release builds of the base worktree differing by one comment** read 2.745 s
against 2.959 s on the tree-walker arm and 2.907 s against 3.152 s on the IR arm, which is larger than
three of those four deltas and comparable to the fourth.

**I called that control's cause code layout and it is not.** The review checked what I did not: the two
binaries have a byte-identical `.text` -- same sha256, same size, same load address -- so the executed
code is the same and codegen variance explains none of it (`68f29913`). It is run-to-run variance in the
measurement environment, and that is the worse answer, because it bounds **repeated runs of one
binary** and not only comparisons between two. The conclusion the control was drawn for survives and
gets stronger: the four deltas are inside the instrument's spread. Only my mechanism for the spread was
wrong, and it was wrong in the direction that would have made the instrument look fixable.

`varlookup`'s criterion-4 residual is Task 7's to discharge and is not touched here. What I measured
of it: the IR/TW **instruction** ratio is 1.0310 at base and 1.0321 at head, essentially unchanged.

## What this task is not, and why that is a boundary rather than an omission

The plan's "predicted movement: `emptyloop` and `varlookup`" is **not** discharged, and it cannot be by
flattening the header's evaluation: that evaluation happens once per loop entry and both axes enter one
loop. The movement the plan predicts needs the loop's **per-pass** header -- the re-test that
`run_repeating` runs inside `in_clause` on every pass -- to become stream ops, so that the body's ops
and the header's live in one `run_ops` range and the per-pass `run_bounded_from_chunk` entry disappears.
That entry is the 74-instructions-per-pass cost the plan's own `varlookup` residual section names.

It is a separate task rather than a step left out of this one, and the reason is structural rather than
budgetary. **Once the loop's iteration is driven by the op counter, the `DO` clause can no longer span
the loop.** Today `LoopRun` sits inside the region and the clause encloses every pass, which is why
this task moved no boundary and needed no new divergence row. A per-pass flattening ends the region
before the body, and three things follow at once:

* the `DO` clause's boundary moves to *before* the first body clause. That is where the oracle has it
  (its `DO` instruction is the header and the body's clauses are separate instructions) and not where
  the tree-walker has it, so it is a new `KNOWN_DIVERGENCES` row in the direction where the compiled
  stream is right -- the same shape Task 4b' produced twice.
* a header value that outlives the header loses its root. `LoopState::OverOnce` holds an `ObjRef`
  rooted today by the `DO` clause's temps frame; with that frame closed before the body, the register
  is the only root, which is exactly the case the plan's "allocate in the enclosing scope" rule was
  written for. This task already allocates and releases those registers that way, so that half is done.
* the per-pass header is `in_clause` and not the full clause unit, so it cannot be an `Op::Clause`
  region: it would gain a clock invalidation, a `>I>` decay and a temps frame per pass that
  `run_repeating` does not do today.

The pieces that task needs, none of which exist: a loop frame on the driver's frame stack carrying
`LoopState`, `first_pass`, `header_clause` and the loop's bounds; `settle` arms for it that call
`do_body_outcome`; `run_repeating`'s per-pass body split into shared functions the driver's ops call in
the same order; and ops at the `END`'s own `op_of` entry for the fall-through echo, the `UNTIL` test and
the back edge. The clause counts move there, and that is where the tripwire's number should be
re-derived.

## What I could not verify

* **The saving's mechanism.** 150e6 and 190e6 instructions, stable to 1e-8, with the one candidate
  mechanism refuted by a control. Consistent with codegen changing around `run_repeating` now that its
  caller is smaller, but that is untested and is not offered as an answer.
* **That the header registers' rooting is load-bearing.** M4 -- releasing them at the region's end --
  is caught only by a golden test on register indices. No behavioural test can catch it, because
  nothing collects yet: the clobbered register would matter to a collector and to nothing else today.
  The golden test is a proxy for the property and is not the property.
* **`emptyloop`'s IR-against-tree-walker standing at BASE -- resolved after this report was first
  written, and against 4b-M rather than against me.** I read the IR arm 6.7% slower within one binary at
  `de05ea58` where 4b-M's figures said parity, and flagged it without establishing which was right. The
  review did: **4b-M's 3.12x/3.13x are withdrawn** (`6ce592ff`), `1535b030` itself reads IR/TW 1.0299
  and 1.0265 in two independent builds with the IR arm faster in 0 of 9 pairs both times, and both
  registered axes fail criterion 4 at head (`emptyloop` 1.0237 wall, `varlookup` 1.0511). The per-pass
  attribution is in the anchor: **4b''s frame stack owns +33 instructions per `DO`-body pass, this task
  owns +3, and Task 6 owns zero**, so the remedy is the frame stack's rather than any promotion's.
* **Whether the `>K>` line should have become a compiled emission decision** the way the clause echo
  did. It is emitted unconditionally and gated at run time by `trace_mode().results`, because a second
  compiled decision needs a staleness rule of its own. The op form buys the ordering here, not the
  elision, and that trade was decided rather than measured.
