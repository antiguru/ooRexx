# Task 4b: the op-level interpreter, debuting on `IF`

> **Fix round 1 applied.** C1 (a promoted clause not recording its boundary's failure site, an
> oracle-confirmed byte divergence between engines) is fixed in the clause unit rather than at the
> call site, so no promoted clause can repeat it; two boundary witnesses are in the regression set
> and M13 shows they are the only thing in the workspace that catches the defect. I1's control
> reasoning is corrected -- the tree-walker arm does not run identical code across the binaries, and
> that is why no within-binary control could read zero. I2, I3 and M1-M5 are addressed below.
> Sections written before the round and still true are unchanged; every section the round touched
> says so.

Status: **DONE_WITH_CONCERNS**.
The machinery landed and `If` is promoted; `Select` is not, and the boundary is proposed below.
The op-level driver costs the IR arm about 7% on the clause-dispatch axis, measured against the
parent commit in three sittings, and that is the concern.

Commits, read back with `git log`:

* `a2b6e225` -- the dual-engine branch cases, added while both engines still delegate.
* `08366e09` -- the op program counter, the register allocator, the compiler-side assertion, and
  the `If` promotion.
* `ac7b9bec` -- the first-instruction permission granted where the tree-walker spends it.
* `cfc4a688` -- fix round 1: C1 and the findings below.

## The blind spot the round named, and it is the finding to carry forward

The table was built around branch **shapes** and never around the clause **boundary**.
Every case asked "does this branch shape behave the same" and none asked "is the clause unit
discharged exactly once here".
The defect landed exactly in the gap: a promoted `IF` recorded its own failure site and not its
boundary's, every shape agreed on both engines, and the divergence was visible only when a `CALL ON`
handler queued by the `IF`'s own condition failed at the `IF`'s own boundary.

The structural fix is that `in_stepped_clause` now records both sites, so the obligation is
discharged by the clause unit rather than remembered by each caller -- a caller that has to remember
is a caller that can forget, and one did.
`BRANCH_CASES`' doc comment now says the same thing where the next task will read it, and so does
Task 4b''s section of the plan.

Suite: **1364 passed, 0 failed, 4 ignored**, green in dev, dev+STRICT, release and release+STRICT,
re-run after fix round 1.
Baseline was 1354/0/4, so ten tests are new; the round's three new cases are rows in an existing
table rather than tests of their own, so the total is unchanged.
`cargo fmt --all --check` exits 0 and `cargo clippy --workspace --all-targets -- -D warnings` exits 0
**from a clean target directory** (`CARGO_TARGET_DIR` pointed at an empty path, so every crate was
re-linted rather than reusing a warm result).

## The machinery, and why each piece is shaped the way it is

### The program counter is an op index

Task 4a stopped because a flattened construct sits *inside* one instruction's position, and a
counter that walks instructions cannot enter one.
`Interp::run_ops` walks ops, and every op that opens a clause carries its own instruction index --
`Generic { index }`, `Loop { index }`, `Clause { index, end }` -- because there is no instruction
counter travelling alongside to read it off.
The instruction space has not gone away: `Flow::Goto` and `Flow::Signal` both carry instruction
indices, and `Chunk::op_of` is the one table where the two spaces meet.

The activation's own `pc` is read where a body is entered and written by `apply_flow`.
**What licenses the op counter being a local is a property of the `pc`, not a claim about who reads
it** -- an earlier version of this paragraph said "nothing in between reads it", which is an
exhaustiveness claim nothing asserts. The property: the tree-walker already leaves the `pc` sitting
on an `IF` for the whole of that `IF`'s branch and on a `DO` for the whole of its loop, so a `pc`
that does not name the clause currently running is a state every reader already has to tolerate, and
this driver leaves it in exactly those states.

### The absorption rule is one function both loops decide through

`run_bounded` is now a two-arm match over one shared rule.
`absorb(flow, start, end) -> Absorbed` is `Advance` / `Resume(instruction)` / `Escaped(flow)`, and
the two loops differ only in what they do with the answer: the instruction loop moves its counter to
the instruction, the op loop maps it through `Chunk::op_at`.
The range test itself, `end` inclusive, exists once.

### The frame stack: **not built, and this is the deviation to flag**

The brief lists a frame stack among the deliverables.
`If` does not need one and I did not build one, for a reason rather than by omission.
A frame is what a construct needs when a `Flow` escaping its body has to be *consumed* or
*rewritten* by it: `Select` consumes a `LEAVE` naming its own label, raises 28.5 on a matching
`ITERATE`, and resets `origin.indent` through `pop_search_frame` on everything it forwards.
`If` does none of those -- its `run_bounded` result is "`Next` becomes `Goto(resume)`, everything
else propagates unchanged" -- so flattening it needs jumps and nothing else, and every `Flow` that
escapes the branch is already handled correctly by whatever range encloses it.

So the frame stack is the piece `Select` needs and `If` does not, and it moves with `Select`.
Building it now would be building an abstraction for a future change, with no caller to shape it and
no test that could fail if it were wrong.

### `Op::Generic` carrying its index, and the register file

Registers are an indexable region of the temporaries stack, opened by `RootSet::reserve_temps` in
`run_chunk` and truncated on every path out.
The region handle travels with the chunk in `BodyEngine::Chunk { chunk, registers }` rather than
living on `Interp`, because a register index means nothing without the region it indexes, and a
callee's chunk has a region of its own stacked above its caller's.

`Chunk::holds_register` is a debug tripwire at both the write and the read.
It exists because a register index past the region's end does not fail -- it addresses whatever a
clause happened to push onto the temporaries stack there, which is a wrong value found by chasing
it.
Mutation M4 below is what showed that: an undercounting high-water mark silently overwrote a live
temporary and the dual sweep stayed green.
With the tripwire in place the same mutation goes red on three test targets that previously missed
it.

### The register allocator

`Registers` with `mark()` / `alloc()` / `release(mark)` and a high-water mark, exactly the plan's
Decisions section, implemented rather than redesigned.
An `IF` takes a mark when its `Clause` op is emitted, allocates one register for the condition, and
releases to the mark when the region's three ops are emitted -- so nested `IF`s reuse one register
and `Chunk::registers` is 1 for any body containing any number of them.
`Mark` is a newtype so a mark and a register index cannot be passed to each other's function.
`alloc` refuses past `u16::MAX` with `ChunkTooLarge { what: "registers" }`, which is the second value
that field has ever taken; its doc comment said "always `op stream` today" and is corrected.

## The op set

| op | meaning |
|---|---|
| `Generic { index }` | delegate instruction `index` to the tree-walker's clause unit |
| `Loop { index }` | a `DO`/`LOOP` at `index` whose body clauses the driver steps |
| `Clause { index, end }` | open instruction `index`'s promoted clause; its ops run to `end` |
| `EvalExpr { index, slot, dst }` | evaluate expression `slot` of instruction `index` into register `dst` |
| `Jump { target }` | continue at op `target` |
| `JumpUnless { reg, target }` | continue at `target` unless register `reg` holds `1` |

`#[expect(dead_code)]` is gone from `Clause` and `EvalExpr`; both are constructed.
`ChunkTooLarge::what` keeps its `#[allow(dead_code)]`, which was not mine to remove.

### What `EvalExpr` stores, and why `JumpUnless` does not re-derive it

`EvalExpr` for an `If`'s slot `0` calls `Interp::eval_if_condition`, which is the same one function
`step`'s own `If` arm calls -- so the `>>>` line, the 34.1 raise and the `ExprKind::Logical`
readback all happen in exactly one place, and `EvalExpr` is trace-identical because it *is* the same
call.
The answer is a Rexx logical value, which `eval_condition` has already validated as exactly `0` or
`1`, so it is stored as the small integer of that name -- allocation-free -- and `JumpUnless` reads
it back by decoding.
The alternative considered and rejected was storing the condition's own `ObjRef` and having
`JumpUnless` re-derive the boolean from its text: that is an extra `to_text` allocation per branch
*and* a second expression of the rule that decides a branch.
A register holding anything but `0` or `1` is `Loud::register_not_logical`, not a defaulted branch.

## The register allocation emitted

For `if C then A else B`, at instruction `i`, with `f = false_target` and `r = skip_else(f)`:

```
op_of[i]     Clause { index: i, end: +3 }      mark taken, r0 allocated
             EvalExpr { index: i, slot: 0, dst: r0 }
             JumpUnless { reg: r0, target: first_op_of[f] }
                                               mark released
op_of[i+1]   ... the THEN branch's own ops ...
             Jump { target: op_of[r] }         only when r != f
op_of[f]     ... the ELSE marker and its branch ...
```

`chunk.registers` is 1 for a body with any number of `IF`s, nested or not, which
`nested_ifs_reuse_one_register` pins.

### The defect the dual sweep found, which is the part worth reading

Arriving at an `ELSE` means two different things, and `run_bounded`'s own doc comment already said
so: "the true path (fall through A, land on `Else` by `pc += 1`) and the false path (`Goto` straight
to `Else`) arrive at the identical `(instruction, pc)`, and only one of the two arrivals is supposed
to enter B".
The tree-walker tells them apart by which loop is running.
My first cut resolved both jumps through `op_of`, which collapsed them again, and
`corpus lang/if_else_chain.rex` plus the `nested if, both true` case went red immediately: the inner
`IF`'s true branch resumed at the outer `ELSE`'s index and ran the outer `ELSE`.

The fix is that a flat stream has *two* op positions there.
`op_of[k]` is now **where control resumes when it arrives at instruction `k`** -- the branch-end jump
when there is one in front of it -- and the `IF`'s own false path is the one arrival resolved against
the instruction's first op instead.
So a true branch that finishes skips the `ELSE` however it finished: by falling off its end, or by a
nested `DO` block answering `Flow::Goto` exactly at the boundary.
`an_if_with_an_else_compiles_to_a_clause_region_and_two_jumps` pins `op_of[3] == 5` for that reason
and says so.

A branch-end jump is emitted **only when the two targets differ**, which is only when there is an
`ELSE`; without one the true branch already falls through to where the false path lands, and a jump
to where control was going anyway is an op the driver would run for nothing.

## The assertion, and it fires

`assert_clause_regions_hold_no_clause_op` runs over what `compile` emitted and refuses any `Generic`
or `Loop` op inside a `Clause` region: both run a whole clause through `step_in_temps_frame`, which
echoes the clause on the way in, and the echo is not idempotent.

An unconditional `assert!` rather than a `debug_assert!`, so the release gate carries the same
guarantee -- one linear scan per body compile, against a pass that has already walked the same list.

**Proved to fire**, by `a_generic_op_inside_a_clause_region_is_refused`, which builds the forbidden
sequence directly and is `#[should_panic]`.
Built in the test rather than by mutating the compiler so the witness stays in the tree.
Its adjacent success is `a_generic_op_after_a_clause_region_is_accepted`: the same three ops with the
`Generic` one past the region's end, which is what stops the assertion being satisfied by a check
that refuses every stream.

## The clause-count tripwire

`the_ir_engine_steps_a_loop_body_from_the_chunk` (4) and
`the_ir_engine_steps_a_simple_blocks_body_from_the_chunk` (3) are **unchanged**, and that is derived
rather than tuned.
The counter now fires where a clause *begins* -- at a `Generic`, a `Loop`, or a `Clause` region --
which is the same set of events it counted before for a body containing no promoted `IF`.
`do i = 1 to 3 / nop / end` is still one `Loop` op plus one body clause per pass, and
`do / nop / nop / end` still one `Loop` plus two.
Both counts went to 7 and 9 at one point mid-task, from a real defect: the first version of
`run_chunk_clauses` let `apply_flow` advance a `pc` that `run_ops` had never moved, so the body ran
its tail twice.
The counts came back to 4 and 3 when that was fixed, which is the tripwire doing its job.

`the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk` is the new one, 4 on each path.
Under `Op::Generic` for `If` the count is 2 on each path, because the branch runs through
`run_bounded`'s tree-walker arm and is counted nowhere -- mutation M6 below.

## Every test added, with its mutation

Ten new tests.
Every mutation was applied with a `python` patch that asserts exactly one occurrence of its site, run
with `--no-fail-fast`, then restored from a `cp` backup and verified with `sha256sum -c` against the
recorded source sums.
No `git checkout --` was used anywhere.

| # | mutation | caught by |
|---|---|---|
| M1 | `PatchKind::Enter` resolves through `op_of` (collapse the two arrivals at an `ELSE`) | population sweep, branch shapes, the `IF` clause count, the `IF`-with-`ELSE` golden |
| M2 | never emit the branch-end jump | population sweep, branch shapes, the `IF` clause count, the `IF`-with-`ELSE` golden |
| M3 | `Registers::release` is a no-op | the two allocator tests, `nested_ifs_reuse_one_register` |
| M4 | `high_water()` answers the live count | `the_high_water_mark_...`, all three `IF` goldens; **and, after the `holds_register` tripwire was added**, the population sweep, branch shapes and loop shapes as well |
| M5 | the clause-region check scans nothing | `a_generic_op_inside_a_clause_region_is_refused` |
| M6 | `compile` emits `Op::Generic` for `If` again | the `IF` clause count and all three `IF` goldens -- **and nothing else in the workspace**, because both engines print identical bytes either way |
| M7 | `Op::Jump` falls through instead of jumping | population sweep, branch shapes, the `IF` clause count |
| M8 | `register_holds` answers `true` for `0` | branch shapes, loop shapes |
| M9 | `alloc` wraps instead of refusing | `a_register_file_wider_than_u16_is_refused` |
| M10 | the `Clause` op does not grant the first-instruction permission | **nothing** |
| M11 | the clause region does not consume the permission | **nothing** |
| M12 | the clause unit does not record its boundary's failure site | branch shapes, `a_handler_that_fails_at_a_clause_boundary_blames_that_clause` |
| M13 | **the exact defect C1 names**: the site recorded by `step_in_temps_frame_with` instead of by the clause unit | branch shapes, and nothing else in 1364 tests |

M6 is the "watch the golden fail" step the plan asks for, run as a mutation rather than as a
sequence, and it is the more informative form: it says the goldens and the clause count are the
*only* things in the workspace that can see whether the promotion happened at all.

M8 had to be run against the lib tests and the two inline dual tables rather than the whole
workspace: with every branch taken, some corpus program runs forever and the population sweep has no
time bound.
That is recorded rather than hidden -- the narrower run is what the row above reports.

### "Can fail" is not "adds coverage", and for the shape cases it is not

M1, M2 and M7 are each caught by the population sweep independently of `BRANCH_CASES`, so for those
three the new table adds nothing the corpus did not already have.
M8 is caught by `BRANCH_CASES` and by `LOOP_CASES` (whose `forever with leave` case contains an
`IF`), so not by the new table alone either.

**The boundary cases are different, and M13 is the proof.** M13 reinstates the exact defect this
round fixed -- the failure site recorded by `step_in_temps_frame_with` rather than by the clause unit
-- and across the whole workspace, `--no-fail-fast`, it is caught by
`both_engines_agree_on_every_branch_shape` and by **nothing else**: 1363 passed, 1 failed. The
pre-existing `a_handler_that_fails_at_a_clause_boundary_blames_that_clause` does not see it, because
that test exercises the tree-walker's path, which was never broken.

So the table earns its place through its *boundary* cases, and the shape cases earn theirs by
carrying recorded oracle bytes -- `trace i` and `trace r` transcripts, 7.3, 34.1, 17.1 -- which is a
different kind of value from mutation coverage.

### Two surviving mutations, stated without implying a guarantee

M10 and M11 both survive.
`grant_procedure_permission` on the `Clause` arm and the `mem::take` inside the clause region are
both unobservable, because `compile` currently puts the `THEN` marker's own `Generic` immediately
after every `Clause` region and that `Generic` grants again before any `PROCEDURE` in the branch can
be reached.

**Nothing enforces that.** No assertion states it, and a promotion that emits a region followed by
something else would make both lines load-bearing with nothing going red in between.
I considered enforcing it -- a compiler assertion that every `Clause` region is followed by an op
that grants -- and did not, because it would pin the *current* op stream's shape rather than the
permission rule, and the next promotion would have to relax it before it had ever caught anything.
So the honest position is the one now written at both call sites: the lines stay because the
obligation belongs to the clause unit, they are unobservable today, and they must not be read as a
guarantee that anything checks them.

## Prediction, recorded before measuring

Written at `08366e09`, before the release binaries were built:

| axis | predicted (IR vs tree-walker, promoted binary) |
|---|---|
| `emptyloop` (25e6) | no movement beyond +/-1%; the program holds no `IF` |
| `varlookup` (19e6) | the same, +/-1% |
| `branchloop` (8e6, ad hoc) | the IR arm **faster by 3-8%** |
| negative control (reverted binary) | both arms within +/-0.5% |

`emptyloop` being unaffected *by the promotion* was part of the prediction, as the plan asks.

## Measured

Release, `lto = "fat"`, `ulimit -v 8388608` on every child, every child from one fresh empty
directory, `/usr/bin/time -f %e`, medians of five rounds with a discarded warm-up round, all cells
alternating within one sitting.

Four binaries, all built from this tree:

| binary | sha256 | what it is |
|---|---|---|
| `parent` | `d9689fd82ac7cfd8a3ba5e05659869942d755676d1e76fc8d7e8e8a193d5039d` | `736bf080` in a git worktree -- before this task |
| `reverted` | `27da75a412248da6b5d2e08dfe1d4955e1289ce6afea3183d078cb0f77e64d52` | this task's driver, with `compile` emitting `Generic` for `If` |
| `promoted` | `e935ac5b20e9817f9153db5fb5f8865b5d13c4dfeee580ade4df2d4a0c5f59a7` | `08366e09` |
| `granting` | `48831fd265230eceb1ee8daedfce48aa02a791830893f584f4caef5c50d73262` | `ac7b9bec`, the tree as committed |

`branchloop.rex` is an ad hoc probe, not a registered axis: `do i = 1 to 8000000 / if 1 = 1 then nop
/ else nop / end`.
None of the six committed axes contains an `IF`, so without it the promotion has nothing to move.
It is not added to `AXES`/`PROGRAMS`, because adding an axis changes the anchor's denominator set and
that is not this task's to do.

### The within-binary estimator did not survive this sitting, and the reason is not layout

Task 4a's estimator is the IR arm against the tree-walker arm *within* a binary, with a reverted
build reading zero as the control.
It does not work here, and the first version of this section got the reason wrong.

**It said the tree-walker arm "runs identical code in all four binaries" and attributed its movement
to layout. That premise is false.** Three of this task's changes are on the tree-walker's own path:

* `absorb` is now a function call taking `Flow` by value and returning `Absorbed`, which contains a
  `Flow`, once per clause -- **on both arms**, because `run_bounded_instructions` decides through it
  too. `Flow` is 64 bytes.
* `step_in_temps_frame_with` now routes through the generic `in_stepped_clause` with a closure,
  where it used to be one function.
* `if_targets` computes `skip_else` on **both** of `If`'s paths, where the old arm computed it only
  when the condition held.

So there is no arm in any of these binaries whose code did not change, and **no within-binary
control could have read zero.** That is the single most useful thing this task established about the
instrument, and it invalidates reading Task 4a's `-0.2%` control as "the two arms are equal" for any
task that touches shared code -- which every task from here on does.

The estimator reported below is therefore **the IR arm across binaries**, with the tree-walker arm
quoted beside it. The tree-walker arm's own movement is a mixture of layout *and* the three items
above, and this measurement does not separate them.

### The numbers

Medians, seconds. Three sittings; the first two are quoted, the third agreed with the second.

| axis | arm | parent | reverted | promoted | granting |
|---|---|---:|---:|---:|---:|
| `emptyloop` | IR | 2.870 | 3.130 | 3.090 | 3.070 |
| `emptyloop` | tree-walker | 3.010 | 3.000 | 2.990 | 3.060 |
| `varlookup` | IR | 5.230 | 5.390 | 5.400 | 5.360 |
| `varlookup` | tree-walker | 5.360 | 5.260 | 5.500 | 5.520 |
| `branchloop` | IR | 4.450 | 4.600 | 4.440 | 4.430 |
| `branchloop` | tree-walker | 4.510 | 4.520 | 4.610 | 4.790 |

Read as three separate effects:

| effect | comparison | `emptyloop` | `varlookup` | `branchloop` |
|---|---|---:|---:|---:|
| the op-level driver | IR arm, parent -> reverted | **+9.1%** | **+3.1%** | **+3.4%** |
| the `If` promotion | IR arm, reverted -> promoted | -1.3% | +0.2% | **-3.5%** |
| the permission fix | IR arm, promoted -> granting | -0.6% | -0.7% | -0.2% |
| net, as committed | IR arm, parent -> granting | **+7.0%** | **+2.7%** | **-0.7%** |

### Predicted versus measured

| axis | predicted | measured | verdict |
|---|---|---|---|
| `emptyloop` | +/-1% | promotion -1.3%, but **+7.0% net** from the driver rewrite | **wrong**, and wrong about the thing I did not separate |
| `varlookup` | +/-1% | promotion +0.2%, **+2.7% net** | same shape |
| `branchloop` | -3 to -8% | **-3.5%** for the promotion | right, at the bottom of the range |
| negative control | +/-0.5% | +4.3% / +2.5% / +1.8% | **wrong**; the control's premise does not hold once the driver itself changes |

The prediction was right about the *promotion* on every axis and wrong about the *task*, because I
predicted per-axis movement without separating the machinery from the promotion it debuts.
That separation is what the parent binary bought, and it was added after the first sitting made the
control look broken.

### The mechanism, from the coordinator's profile

The coordinator profiled the driver with samply (release with symbols, `emptyloop` at n=25e6, one run
per arm, analysed with pollard) while this review round ran, and it converges with the candidate list
above. Wall 3013 ms tree-walker against 3097 ms IR. Per-function self time:

* `run_ops` **96 ms, new**, replacing `run_bounded_instructions` at 49 ms -- the op loop costs about
  twice the instruction loop it replaced.
* `step_in_temps_frame_with` self **617 -> 674 ms (+57)**, with its closure 151 -> 111.
* `_memcpy_avx512_unaligned_erms` **17 -> 53 ms**, on an identical stack through `run_repeating` in
  both arms: same code, three times the time, which is a larger value being copied per clause. That
  is `absorb`'s 64-byte `Flow`.
* `free` +20 ms against `malloc` -17 ms.

About **84%** of the total regression sits on the dispatch path itself
(`run_bounded_instructions` + `step_in_temps_frame_with` + closure, 817 ms, against `run_ops` +
`step_in_temps_frame_with` + closure, 881 ms). The named cause is two bounds-checked `op_of` loads
and an extra call frame per `DO`-body pass, which on a 25e6-pass axis is 50e6 loads and 25e6 frames.

**Task 4b-M owns the fix and now has a mechanism to test rather than a search.** Nothing in this
round changes it.

### What this measurement does not establish

* **How much of the +7% each named cause is worth.** The profile attributes 84% of it to the
  dispatch path and names the candidates; it does not apportion them, and no candidate has been
  removed and re-measured.
* **The tree-walker arm's own movement.** It is a mixture of layout and the three shared-path
  changes above, and this sitting separates neither.
* One added per-clause cost *was* found and removed (the permission grant, worth 0.6%). `#[inline]`
  on `absorb`, `Chunk::op_at`, `Chunk::op_at_index` and `run_bounded_from_chunk` was built and
  measured at 0.2% -- inside the noise -- and reverted rather than committed as an unexplained
  annotation.
* **That `branchloop` is representative.** It is one branch shape in one loop, chosen to isolate the
  promotion, not sampled from anything.
* **Anything about `alloc4c`, `arith`, `compound` or `strings`.** Not measured.
* **Anything against the oracle.** No oracle ran in any of these sittings, so no ratio here is
  comparable to the anchor's.

## `Select` is not promoted, and here is the boundary

The brief allows the split and this is it.

`Select` needs three things `If` does not, and all three are the frame stack:

* it consumes a `LEAVE` naming its own label and raises **28.5** on a matching `ITERATE`, so a `Flow`
  escaping a matched `WHEN`'s body has to reach `leave_select` rather than the enclosing range;
* it resets `origin.indent` through `pop_search_frame` on every `LEAVE`/`ITERATE` it forwards, which
  is a rewrite of the escaping flow rather than a pass-through;
* F-EX1's redirect -- an absorbed `WhenCase`'s false-branch `Goto` landing exactly on this `SELECT`'s
  `OTHERWISE` marker has to enter `run_otherwise` rather than being forwarded -- is a decision made
  by the construct about a flow that would otherwise escape it.

Flattening it therefore means an explicit stack of open construct frames in the driver, each with its
label and resume and an outcome handler, and a `Flow` that escapes walking that stack applying each
frame's handler -- which is what the Rust call stack does today through `run_bounded(...)?` followed
by `leave_select`.
That is a second, larger piece of machinery than everything in this task, and `indent_offset`,
`current_case_text` and `test_case_when`'s per-value trace lines all ride on it.

**Proposed boundary:** a Task 4b' that builds the construct-frame stack and promotes `Select` with
it, before or after Task 4c depending on whether Task 6's trace ops land first.
`BRANCH_CASES` already carries `SELECT` shapes, added before anything was promoted, so part of the
regression net is in the tree before that task starts.
**It is not sufficient**, and the axis on which to extend it is boundaries rather than shapes -- see
"the blind spot" at the top of this report. A `SELECT` opens a clause for its own header *and* one
per listed `WHEN`, so it has more of those boundaries than an `IF`, not fewer.

## Corrections made to the plan

The plan and this task's brief both said a `SELECT` with no matching `WHEN` is "error 93.4".
Measured against the oracle, it is **7.3** -- "All WHEN expressions of SELECT are false; OTHERWISE
expected", rc 249, for both `SELECT` and `SELECT CASE`.
Corrected in `docs/superpowers/plans/2026-08-09-phase-4e-ir.md` and in the brief, where the next
reader will see it, rather than only here.

## Things I could not verify

* **Why the op-level driver costs 7% on `emptyloop`.** Named above as unattributed. No profiler was
  run.
* **That `op_of` pointing at a branch-end jump is safe for every range start.** The argument is that
  a `run_bounded` range never *starts* at an `Else` -- every range start is `X + 1` for a `DO`,
  `WHEN`, `OTHERWISE` or `IF`, and none of those can be immediately followed by an `Else`. That is
  reasoning about the parser's output, not a measurement, and nothing asserts it. The 10,391-program
  population sweep passing is the evidence there is.
* **Whether the *shape* half of `BRANCH_CASES` adds mutation coverage over the corpus.** I looked
  and did not find a mutation only a shape case catches; I did not prove none exists. The boundary
  half is settled the other way by M13.
* **That the `granting` split is observable.** It is a parity fix and a 0.6% saving; no test
  separates it from granting everywhere, for the same reason M10 survives.

## One process note worth recording

`git worktree add` for the parent binary landed on a stale `parent/` directory left in the scratchpad
by a session on 5 August, and `cargo build` in it reported "Finished in 0.04s" -- producing a
`rexx-run-parent` from an unrelated commit that would have gone straight into the table above.
It was caught by the directory not being a git repository at all.
The fix used was a fresh worktree path (`wt-736bf080`) whose `git log` was read back before building.
This is the same shape as the brief's own warning about a probe binary surviving a source revert: a
build that does no work reads exactly like one that succeeded.
