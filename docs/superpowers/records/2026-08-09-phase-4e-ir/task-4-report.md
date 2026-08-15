# Task 4 report: promote `Do` and `Loop`

Status: partial.
The construct is promoted in the sense that matters structurally -- a `DO`/`LOOP` compiles to an op
of its own and its body's clauses are stepped from the compiled stream -- and it is **not** promoted
in the sense the plan's Interfaces line describes, which is a flattening into
`Clause`/`EvalExpr`/`Jump`/`JumpUnless` with a native control-value step.
The reasons are in "What was not done, and why" below, with a proposed split.

## The prediction, recorded before measuring

Written before any benchmark was run at this commit.

* `emptyloop` (anchor: 3.33x tree-walker vs oracle): **no movement, or a regression of up to about
  2%** of the IR arm against the tree-walker arm.
* `varlookup` (anchor: 4.40x): **no movement, or a regression of up to about 2%.**

The reasoning is a decomposition measured at `777fa4a9` (release, this machine, `n = 25000000`,
median of two identical runs each, `/usr/bin/time`):

| program | wall | ns/iteration |
|---|---:|---:|
| `do i = 1 to n; nop; end` (`emptyloop`) | 3.00 s | 120 |
| `do i = 1 to n; end` | 2.08 s | 83 |
| `do n; nop; end` | 1.40 s | 56 |
| `do n; end` | 0.65 s | 26 |

Subtracting: about 26 ns/iteration is the repeating loop's own framing (`in_clause`, the two trace
gates, `do_body_outcome`), about 57 ns is a controlled loop's control step (`loop_advance` reads the
control variable out of the pool, adds `BY` as a `Number`, compares against `TO`, writes it back),
and about 37 ns is one trivial body clause through `step_in_temps_frame`.
The oracle runs the same `emptyloop` at about 35.8 ns/iteration in total.

This promotion changes none of those three.
`run_loop`, `run_repeating` and `loop_advance` are one implementation entered from both engines, and
what the op adds is an `op_of` lookup, an `ops` index and a match per body clause -- work that did
not exist before and that replaces nothing.
So the prediction is a small regression rather than "no change" as such, and the honest form of it is
that the effect is at or below what a handful of paired runs can resolve.

## Measured, against the prediction

**The prediction was wrong in sign on both axes.** Predicted a regression of up to 2%; measured an
improvement of 1.7% on `emptyloop` and 3.2% on `varlookup`, IR arm against tree-walker arm, within
one binary. The full method and the negative control are in "Measurement" below.

| axis | predicted (IR vs tree-walker) | measured |
|---|---|---:|
| `emptyloop` | no movement, or up to +2% (slower) | **-1.7%** |
| `varlookup` | no movement, or up to +2% (slower) | **-3.2%** |

**The movement did not arrive by the stated hypothesis, and it has no attributed mechanism at all.**
The hypothesis behind the prediction was that the promotion adds an `op_of` lookup, an `ops` index
and a match per body clause and removes nothing, so the IR arm should cost slightly more. The added
work is real and is still there. What offsets it, and more than offsets it, is not established. So
this is a measured improvement with no explanation, which is a weaker result than a measured
improvement with one, and it should not be quoted as "promotion made loops faster" until something
names the mechanism.

The first sitting made the effect look about three times larger than it is, and that was an artifact
-- see "Measurement" for what it was and how it was caught. It is exactly the shape of the anchor's
own warning about a ratio flattered by its denominator.

## What was landed

### The op

`Op::Loop`, emitted by `compile` for `InstructionKind::Do` and `InstructionKind::Loop` (one
construct, two spellings, matched together in `step`'s own arm).
It carries no payload, for the same reason `Op::Generic` carries none: the driver reaches it through
`Chunk::op_of`, so the instruction index is already in hand.

### What it changes, which is one line

`Interp::run_loop` and `Interp::run_repeating` are unchanged in substance.
Header validation, the five `LoopKind` arms, the `COUNTER`/`DO WITH` refusal, `WHILE`/`UNTIL`, the
`HeaderClause` attribution, the `LEAVE`/`ITERATE` label search through `do_body_outcome` and
`pop_search_frame`, every `>K>` keyword trace, the per-pass `DO` re-echo and its `UNTIL` sibling, the
`END` echo -- all of it is one implementation, entered from both engines.

The one thing that was engine-specific inside it is the call that steps each body clause.
`Interp::run_bounded` now takes a `BodyEngine`:

* `BodyEngine::TreeWalker` steps the clause through `Interp::step_in_temps_frame`, as before;
* `BodyEngine::Chunk(chunk)` steps it through `Interp::step_from_chunk`.

`step_from_chunk` is the single instruction-to-op mapping and the single `chunk_map_too_short`
guard: the driver's own outer loop calls it too, rather than repeating the lookup. The first round
had the mapping in both places, which is the finding avoided on loop semantics reappearing at
smaller scale.

`BodyEngine` is threaded `step_in_temps_frame_with` -> `step` -> `run_loop` -> `run_repeating` ->
`run_bounded`.
**Both** of `run_loop`'s `run_bounded` calls take it: `run_repeating`'s, which every repeating
`LoopKind` reaches, and `LoopKind::Simple`'s own, which is a separate arm and needs a witness of its
own -- see I1 in the mutation log.
`step_in_temps_frame` is kept as a thin wrapper passing `TreeWalker`, so every existing call site is
unchanged.

`run_fragment` passes `TreeWalker` unconditionally and says why at the call site: `INTERPRET` text
compiles to no chunk, and its `Code` is a different body from the one an enclosing chunk's `op_of`
indexes, so handing it a chunk would index one body's ops with another body's instruction numbers.
`If`, `Select` and `OTHERWISE` also pass `TreeWalker`: their branch bodies are Task 5's, and passing
`engine` there would have promoted them in this commit without a test that names them.

### Why this is an extraction and not a second implementation

The brief's constraint is that promotion extracts shared semantics rather than writing a second one.
This form takes it to its limit: there is no second loop to keep in step, because there is only one
loop and both engines enter it.
That is also why the dual-engine sweep cannot see this promotion at all, and why it needed an
observable of its own -- see "The tests" below.

### `REXX_ENGINE`

`rexx-run` had no way to select an engine, so neither arm of a two-arm benchmark could be run.
`REXX_ENGINE=ir` / `REXX_ENGINE=tree-walker` now selects.
**Only an unset variable defaults**; set-but-unrecognised exits 2, and that includes a value this
platform will not decode as UTF-8, which the first round folded into the unset case and silently ran
the tree-walker for. Silently running the other engine is the worst of the three outcomes, because a
benchmark attributes the result to the wrong arm.
Measured after the fix: unset -> 0, `ir` -> 0, `REXX_ENGINE=` -> 2, `REXX_ENGINE=IR` -> 2, and a
lone `0xff` byte -> 2 with `REXX_ENGINE is a value that is not UTF-8`.

The library reads no environment variable to choose an engine at any depth, so an in-process harness
gets exactly what its own `Invocation` asked for. `Engine`'s own doc comment, which argues for a
field rather than a variable, now cross-references the one place a variable does exist and says why
that case is different.

**Verified to select, rather than assumed.** A temporary `eprintln!` in `run_chunk`, debug build,
three runs of the same program: unset -> 0 chunk entries, `tree-walker` -> 0, `ir` -> 1, and
`REXX_ENGINE=nonsense` -> the diagnostic and status 2. The probe was then reverted.

## The tests

### Added

1. `tests/ir_dual.rs::both_engines_agree_on_every_loop_shape` -- one `DO`/`LOOP` program per shape
   (controlled; controlled with a body that writes the control variable; `BY`/`FOR`; bare repeat
   count; `WHILE`; `UNTIL`; `FOREVER` with `LEAVE`; nested with a labelled `ITERATE`; `DO OVER` on a
   non-stem target; a top-level `Simple` block; the same block reached through an `IF`; a labelled
   `Simple` block left by name; a controlled loop under `trace i`; an `UNTIL` loop under `trace r`),
   each run on both engines and compared byte for byte, **and** each compared against the bytes the
   tree-walker produces for it.
   The two traced cases carry their whole expected trace sink.
   **Added before any compiler change**, per the brief's ordering, and green at that point.
2. `src/ir/golden_tests.rs::a_counted_loop_compiles_its_do_to_a_loop_op_and_its_body_to_generic` --
   the exact op stream for `do i = 1 to 3; nop; end`.
3. `src/ir/drive/tests.rs::the_ir_engine_steps_a_loop_body_from_the_chunk` -- the loop's `DO` clause
   and its three body clauses are stepped from the chunk, counted through a new per-thread
   `clause_op_entries`.
4. `src/ir/drive/tests.rs::the_ir_engine_steps_a_simple_blocks_body_from_the_chunk` -- the same for
   `LoopKind::Simple`, which is a separate `run_loop` arm with a `run_bounded` call of its own and so
   needs a witness of its own.
5. `src/ir/drive/tests.rs::the_tree_walker_steps_no_clause_from_a_chunk` -- their negative control.

### Mutation log

Every restore was from a `cp` backup and verified with `sha256sum -c` against the working tree.
No `git checkout --` anywhere.

| # | mutation | result | what else in the workspace caught it |
|---|---|---|---|
| M1 | `run_repeating`: drop the per-pass `END` echo | `both_engines_agree_on_every_loop_shape` red, on the `trace i` case | 8 other tests, all pre-existing trace units. **Adds no coverage for this mutation** -- the expectation half duplicates them |
| M2 | `run_repeating`: drop the per-pass `END` echo **only under `Engine::Ir`** | red | `both_engines_agree_across_every_population` only |
| M3 | `loop_advance`'s `Count` arm: run one fewer pass **only under `Engine::Ir`** | red | `both_engines_agree_across_every_population` only |
| M4 | `compile`: emit `Generic` for `Do`/`Loop` again | golden test red **and** `the_ir_engine_steps_a_loop_body_from_the_chunk` red | nothing else in the workspace, including the dual sweep |
| M5 | `run_bounded`'s `Chunk` arm falls back to `step_in_temps_frame` (op emitted, body not driven from it) | `the_ir_engine_steps_a_loop_body_from_the_chunk` red | nothing else, and the golden test stays green |
| C | `run_repeating`: drop `UNTIL`'s second, unconditional `DO` re-echo **only under `Engine::Ir`** | `both_engines_agree_on_every_loop_shape` red, on the `trace r` case | **nothing else, the corpus sweep included** |
| I1 | `LoopKind::Simple`'s arm passes `TreeWalker` (fix round 1) | `the_ir_engine_steps_a_simple_blocks_body_from_the_chunk` red | nothing else in the workspace |

What the log says, stated rather than left to be inferred:

* M1 is the honest negative result. The new dual cases *can* fail on a loop-semantics change, and for
  that change they add nothing the suite did not already have.
* M2 and M3 are the shape that matters after a promotion -- an engine that diverges -- and there the
  new cases are a second, ten-millisecond, named-by-shape catcher beside an 8.5-second sweep. Still
  not unique coverage.
* **C is where the traced loop cases are the only thing standing.** A first version of this report
  said M4 and M5 were the only such rows, which was a false universal drawn from three mutations: C
  is an engine divergence in a trace shape no corpus program has, so the sweep runs clean over it and
  only the `trace r` case in `LOOP_CASES` sees it. Writing the two traced cases' whole expected sink
  out in full is what makes that possible.
* **M4, M5 and I1 are where the counting tests are the only thing standing.** Nothing else in the
  workspace can tell a promoted loop from an unpromoted one, because both engines produce identical
  bytes by construction. M5 is the sharpest: it leaves the op emitted and the golden test green, and
  only the clause count says the body never reached the stream.
* **I1 is a hole the first round shipped**, found in review. `LoopKind::Simple` is a separate arm of
  `run_loop` with a `run_bounded` call of its own, and un-promoting it left the entire workspace
  green -- so that half of the promotion had no witness. Compounding it, the `LOOP_CASES` entry named
  "simple block" was `if 1 = 1 then do`, which `If`'s own tree-walker routing takes away from the
  chunk before the block is reached, so the case was not testing what its name said. Both halves are
  fixed: the case is now a top-level block, the `IF`-nested route is a second case that says so in
  its name, and `the_ir_engine_steps_a_simple_blocks_body_from_the_chunk` is the witness the mutation
  above reddens.

### Suite

Baseline at `777fa4a9`: 1349 passed, 0 failed, 4 ignored.
After: **1354 passed, 0 failed, 4 ignored** (+5), identical in dev, dev+STRICT, release and
release+STRICT.
`cargo fmt --all --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean.

## What was not done, and why

Three of the brief's items are not discharged. Each is blocked on the same thing, and the block is a
property of the tree rather than a judgement about effort.

### 1. `Op::Clause`, `Op::EvalExpr`, `Op::Jump`, `Op::JumpUnless` and the control-value step

The plan's Interfaces line describes flattening a `DO`/`LOOP` into a run of ops with jumps between
them. Three obstacles, in increasing order of how hard they are to work around.

**The driver has no op program counter.** `run_clause_ops` runs exactly one op and answers that
clause's `Flow`. A jump target is meaningless without an op pc, and an op pc is not enough on its own:
a flattened construct also needs the absorption rule `run_bounded` holds (an escaping `Flow::Goto`
whose instruction target lands inside my range is mine; anything else propagates), a frame stack so a
`LEAVE`/`ITERATE` reaching a body op is handed to the right enclosing construct, and a translation
between the instruction indices `Flow` carries and the op indices a jump uses. That is an op-level
interpreter, and building it is a driver change rather than a compiler change.

**`Op::Generic` cannot appear inside a jumped-to region.** It carries no instruction index, and the
driver supplies one only because the outer loop reaches it through `op_of` at a pc it already holds.
The moment ops are walked with an op pc, a `Generic` in the middle of a region has no way to say which
instruction it delegates. Fixing that means either a payload on `Generic` or a reverse map on `Chunk`,
and `ir/mod.rs`'s own doc says later tasks extend these shapes rather than reshape them.

**`Op::EvalExpr` cannot be emitted for a loop header before the trace ops exist.** The plan's premise
is that `EvalExpr` "calls the same function `step` calls, so trace output is identical by
construction". That holds only if `EvalExpr` is the whole evaluate-and-trace unit. Every header
evaluation in this crate interleaves the two: `setup_controlled` walks `ctrl.order` and emits
`>K> "TO"` *between* evaluating `to` and evaluating `by`; `LoopKind::Count` traces `>K> "FOR"`
between evaluating the count and validating it; `eval_condition` traces `>K> "WHILE"`/`"UNTIL"`
between evaluating the condition and checking it is logical. Splitting the evaluation out into an
`EvalExpr` op requires the trace to be an op too, so the two stay in order -- which is Task 6.
Emitting an `EvalExpr` that also traces would make it a loop-header op rather than the general
expression op Tasks 7 through 9 need.

### 2. The register allocator and its test

The allocator has no caller in this commit, because nothing addresses a register: a loop's control
value lives in `run_loop`'s own `LoopState`, which is richer than an `ObjRef` region can hold
(`SymbolId`, `ControlValue`, two `Number`s, two budgets and a `bool`). Landing it would fail
`-D warnings` on `dead_code`, which is the exact reason the plan moved it out of Task 2. It belongs
with the flattening, where its first caller is.

### 3. The compiler-side assertion that no `Clause` op precedes a `Generic` op

Still not writable, and for the reason Task 3 gave: `compile` emits no `Clause`, so the forbidden
sequence cannot be constructed and the assertion cannot fire. Writing it now would be the
cannot-fail-test defect this phase has already shipped once. It moves to the task that first emits
`Clause`, unchanged.

## Measurement

Release, `lto = "fat"`, this machine, `ulimit -v 8388608` on every child, every child run from one
fresh empty directory, `/usr/bin/time -f %e`.
Both arms are the same binary, selected through `REXX_ENGINE`, so there is no build-identity problem
between arms -- but there is one *between builds*, and it turned out to matter.

### The design, after the first sitting went wrong

The first sitting measured only the promoted binary, three interleaved pairs per axis, and read
`emptyloop` at tree-walker 3.05 s against IR 2.88 s -- a 5.6% improvement.
Reverting the promotion (Step 10) and re-measuring showed both arms of the reverted binary at 2.88 s.
That is the tell: **the tree-walker arm cannot be affected by the promotion**, because under
`Engine::TreeWalker` nothing is compiled at all, yet it read 3.05 s in one build and 2.88 s in the
other. The one-line change to `compile` moved code the tree-walker arm never executes, and the
resulting layout shift is worth about 2% on this axis. So most of that first 5.6% was the
denominator moving, not the numerator.

The final design fixes that by putting all four cells in one sitting and reading the estimator from
*within* a binary, with the reverted build as the negative control:

* estimator: IR arm against tree-walker arm, **within the promoted binary**;
* negative control: the same comparison **within the reverted binary**, which must read zero, because
  there the two arms delegate identically.

Four rounds, all four cells per round, one axis at a time.

### The numbers

Medians of four rounds, seconds. `promoted` is `37681dda4b3259f72c6619a221f356ba89de834166562f5a459daaf92ca82859`;
`reverted` is `f4673551c248f4e5ce2fd92879143a23910ce0b59fdf79d7a278f48ef6975d5d`, built from the same
tree with `compile` emitting `Generic` for `Do`/`Loop` again and nothing else changed.

| axis | binary | tree-walker arm | IR arm | IR vs tree-walker |
|---|---|---:|---:|---:|
| `emptyloop` (25e6) | promoted | 3.01 | 2.96 | **-1.7%** |
| `emptyloop` | reverted | 2.95 | 2.955 | -0.2% |
| `varlookup` (19e6) | promoted | 5.325 | 5.155 | **-3.2%** |
| `varlookup` | reverted | 5.345 | 5.335 | -0.2% |

Every cell's four samples spread by less than 1%: `emptyloop` promoted IR was 3.00/2.96/2.95/2.96 and
promoted tree-walker 3.01/3.03/3.01/3.01; `varlookup` promoted IR was 5.13/5.15/5.17/5.16 and promoted
tree-walker 5.31/5.33/5.33/5.32.

The negative control reads -0.2% on both axes, which is the resolution this many paired runs has. The
promoted rows are 8x and 16x that. Against the anchor's own ratios the promoted IR arm would put
`emptyloop` at about 3.31x and `varlookup` at about 4.27x, but **those are not comparable to the
anchor's 3.33x and 4.40x**: the anchor is a different sitting on a different binary with an
interleaved oracle beside it, and this sitting ran no oracle at all. The within-binary percentages
above are the result; the re-derived ratios are not.

### What this measurement does not establish

* **Any mechanism.** The promotion adds work per body clause and removes none, and the arm that does
  the extra work is the faster one. Nothing here says why.
* **That it survives a relink.** The tree-walker arm moved 2% between two builds that run identical
  code on that arm, so layout on this machine is worth about that much. The estimator is taken within
  a binary, which holds layout fixed for the binary but not for the two arms' own code paths inside
  it. A second promoted build from a perturbed tree would separate the two, and was not run.
* **Anything about the other four axes.** `alloc4c`, `arith`, `compound` and `strings` were not
  measured here.

## The proposed split

* **Task 4a (this commit).** `Op::Loop`; `BodyEngine`; `run_bounded` parameterised; the dual-engine
  loop cases; the golden test; the clause-count observable and its negative control; `REXX_ENGINE`.
  What it buys the phase is the thing every later task needs: an instruction inside a loop body is
  reachable from the compiled stream. Without it, Task 7's promoted assignment and Task 8's promoted
  variable access would run at top level only, and every benchmark's hot code is inside a loop.
* **Task 4b.** The op-level interpreter: an op pc in `run_clause_ops`, `Jump`/`JumpUnless`, the
  absorption rule and the construct-frame stack, and whatever `Op::Generic` needs to carry an
  instruction index. Its natural first consumer is `If`/`Select` (Task 5), which is a much smaller
  construct to flatten than a loop and has no trace-ordering entanglement in its header.
* **Task 4c.** Flatten the loop header, after Task 6's trace ops exist, with `Clause`, `EvalExpr`,
  the register allocator and the no-`Clause`-before-`Generic` assertion. This is where the three
  undischarged obligations land, together, with their first callers.

**Reordering 4b before Task 5 is worth considering**, since `If`/`Select` is the cheaper vehicle for
the same infrastructure.

## What this task cannot tell you

* **Whether the flattening moves any benchmark.** The decomposition above says the loop's own framing
  is about 26 ns of `emptyloop`'s 120, and flattening can attack at most that, against an oracle
  figure of 35.8 ns for the whole iteration. The remaining 94 ns is `loop_advance`'s control step and
  one body clause's `step_in_temps_frame` -- Tasks 6 through 9's territory, not Task 4's. This is a
  claim about where the time is, measured; it is not a claim about what a flattened loop would cost,
  which nobody has measured.
* **Whether the dual-engine sweep would catch a flattened loop's divergence.** M2 and M3 say it
  catches an engine-gated change to loop semantics on the corpus as it stands. They say nothing about
  a divergence in a shape no corpus program has.
