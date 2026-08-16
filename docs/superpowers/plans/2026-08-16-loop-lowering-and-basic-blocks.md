# Hoisting loops into the IR, and whether the IR should keep basic blocks

**Goal:** Decide, later, whether a `DO`/`LOOP`'s control should be lowered into the IR instead of resolved by a shared function call, and whether the IR should grow explicit basic blocks -- so that loop-level optimisation is expressible at all.

**Status:** open, unassigned, and **deliberately undecided**. Recorded 2026-08-16 by Moritz during the review of Task 7 of `2026-08-15-phase-5a-native-layer.md`. Nothing here is a ruling; it exists so the question survives the session it was raised in.

## What the shape is today

`Op::LoopRun { index }` (`rust/crates/rexx-exec/src/ir.rs:164`) is the last op inside its `Op::Clause` region. Driving it (`rust/crates/rexx-exec/src/ir/drive.rs:1346`) hands the whole construct to `Interp::run_loop_with_header` (`rust/crates/rexx-exec/src/run.rs:6381`), the same function the tree-walker reaches.

**The body is already compiled.** The driver passes `BodyEngine::Chunk { chunk, registers }`, so the body's clauses are stepped as ops out of the same chunk through `run_bounded_from_chunk` (`run.rs:6122`). What is not lowered is the loop's *control*: iteration, `WHILE`/`UNTIL`, and the `LEAVE`/`ITERATE` label search. The distinction matters for anyone reading this later -- "the loop is interpreted" is wrong; "the loop's control is a call into shared code, and the body is re-entered through it once per pass" is right.

## Why it is that way

Each of these is documented at its own site, and each is a thing a lowered loop would have to reproduce:

- **One implementation of loop semantics for both engines.** `Op::Jump` and `Op::JumpUnless` already exist, so lowering is mechanically available; it is not used because a jump-lowered loop makes the IR's iteration semantics separate code from the tree-walker's. `ir_dual` cannot police a difference both arms route through identically, so the equality would have to be proven some other way.
- **The clause's temps frame stays open across every pass** (`ir.rs:145`), because it roots the per-pass temporaries a `WHILE`/`UNTIL` test pushes. That is why `LoopRun` sits *inside* its `Clause` region rather than after it. Lowering to jumps decouples frame lifetime from region extent, and the frame is a GC root.
- **Header evaluation order is observable.** `do i = 1 to 'a' by zf()` raises 41.1 on `TO` and never calls `zf`, so `Op::LoopHeaderValue` validates each value in front of the next one's evaluation (`ir.rs:119`). A stream that gathered the header then validated it would call `zf`.
- **`LEAVE`/`ITERATE` do a dynamic label search** and can name an enclosing loop, so the branch target is not derivable from this loop's own text.
- **Refusals are decided once, for both engines**, by `loop_header_plan` before anything is evaluated (`run.rs:6391`). The doc there records what happened before that check existed: the refused forms *panicked* on the compiled stream while failing loudly on the tree-walker, and no test in the workspace was red.

## What it costs

`run_bounded` is called inside the pass loop, so **each iteration re-enters the driver** rather than continuing a program counter in the same dispatch loop. Every pass pays a call and driver setup, and the body's `pc` cannot stay hot across passes.

**This has not been measured.** The `emptyloop` axis is the instrument that would attribute it. No number in this document is a measurement, and none should be quoted as one.

## What a decision would unlock

Loop unrolling, loop-invariant code motion, and anything else that reasons about a loop as a region are not expressible while the body is entered through a function call per pass -- there is no object in the IR that *is* the loop. Two shapes are worth considering together rather than in sequence:

- **Lower the loop's control into the existing linear op stream** (jumps plus a header prologue), keeping `run_loop_with_header`'s decisions where they cannot be duplicated.
- **Give the IR explicit basic blocks.** This is the larger change and the one that makes later optimisation work ordinary rather than bespoke; it is also the one that most changes what `compile.rs` and the driver are. Recorded here so the cheaper option is not chosen merely because it is the one that fits the current representation.

## The standing position that changes the calculus

**Moritz is not opposed to retiring the tree-walker and keeping only the IR.** Recorded here because it is the single fact that most changes the answer, and it is not derivable from the code.

If the tree-walker goes, the "one implementation for both engines" argument for `Op::LoopRun` largely dissolves, and lowering becomes much cheaper to justify. What goes with it is the differential net: `Invocation::none()` already runs the compiled engine, so `ir_dual` is nearly the only thing pinning tree-walker bytes, and it is also what catches an IR bug that the oracle corpus happens not to cover. Removing the tree-walker makes the C++ oracle the only oracle. That trade deserves its own record and its own decision; it is named here, not settled here.

## Why this is not an exclusion

`phase-4-exclusions.txt` holds work assigned to a later phase and permanent chosen differences from the oracle. This is neither: there is no behavioural difference at all, on either engine. It is an internal representation question, so it belongs in `plans/`.

## What evidence a decision needs, before anyone builds

- [ ] **Measure the per-iteration re-entry.** Attribute it on `emptyloop`, both arms, against a build that does nothing else differently. Until that number exists, "the loop is slow because of re-entry" is a hypothesis.
- [ ] **Any prototype must still run the awkward cases.** The precedent is on file: the bytecode-VM spike's speedups were **withdrawn** once it emerged they were measured on a design that could not run `CALL`, `INTERPRET` or `DROP`. A loop-lowering prototype that does not reproduce header ordering, the temps-frame lifetime, the `LEAVE`/`ITERATE` label search and the shared refusal will produce a number of exactly that kind.
- [ ] **Decide the tree-walker's future first, or explicitly decide to lower without depending on it.** The two questions are entangled and answering them in the wrong order costs the work twice.
