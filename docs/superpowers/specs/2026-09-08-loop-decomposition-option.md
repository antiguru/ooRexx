# Decomposing loops into RISC ops -- a deferred option, not a plan

**Status: OPTIONAL FUTURE PHASE. Nothing depends on it, and the defect that raised the
question was closed without it.** This document exists so the option is written down with
its costs rather than rediscovered, and so that a later reader does not mistake the
decision that was actually made for a decision about CISC and RISC.

## What raised the question

A use-after-free in `DO OVER` on the compiled engine's flattened loop path. The
loop-control state lives in `Interp::flat_top` / `Interp::flat_loops`, which the collector
does not walk, and `LoopState::OverItems` held a `Vec<ObjRef>` there. Measured 2026-09-08
from `f8eeff8d1`: the default engine panicked on an ordinary program in 0.032 s, and the
tree-walker and `REXX_NO_FLAT=1` both answered correctly. The diagnosis and every figure
quoted here are in the record beside this file.

**The axis that mattered was GC visibility, not instruction granularity.** The oracle is a
tree-walker with a CISC loop instruction and interpreter-side control state, and it is
safe: `DoBlock` is a Rexx heap object (`instructions/DoBlock.cpp:51`), `DoBlock::live`
marks its `to` field (`:88`), and `checkOver` reads `((ArrayClass *)to)->get(overIndex)`
(`:144`) -- one rooted reference and an integer cursor. A survey of CPython, Lua, LuaJIT,
SpiderMonkey, V8, the JVM and Ruby found no engine that keeps iteration state where a
precise collector cannot see it; the precise ones put it in traced stack slots or
registers, and the two that do keep it in C locals (Ruby, JSC) pay for a conservative
stack scanner. So the landed fix moved the state into the register file, which is already
a region of the `temps` the collector walks, and left the loop op alone.

## What the option is

Replace `Op::LoopRun` plus the `FlatLoop` side stack with ops that carry the whole
protocol explicitly: a setup op that materialises the iteration source into registers, a
step op that binds the control variable and branches, and ordinary jumps. V8's
`ForInPrepare` writing a register triple, then `ForInNext`/`ForInContinue`/`ForInStep`, is
the closest existing shape; Lua's `TFORPREP`/`TFORCALL`/`TFORLOOP` chained by `goto` is
the shape that keeps superinstruction dispatch while holding every live value in a
register.

## What it would buy

1. **The bug class becomes unrepresentable rather than mitigated.** With no side struct,
   there is no second place a future loop form can hide a handle. The landed fix is one
   register write and a doc comment; that is a rule someone can forget, and this is not.
2. **`Interp::flat_top`, `flat_loops`, `flat_spares` and `unwind_frames`' two-stack
   invariant all go away**, along with the `FlatLoop` recycling pool.
3. The audit that found the loop stack found nothing else unrooted, so this would leave
   the interpreter with no hand-maintained root outside `RootSet` at all.

## What it would cost, and why it is not scheduled

`flat_loop_step` is not an iterator. Per pass it also carries:

* the `DO`/`LOOP` clause's own re-echo, once per pass after the first, asked at the pass
  boundary rather than at entry because a `TRACE` inside the body changes the answer
  (there is an `ir_dual` case that loses the line if the decision is made on the way in);
* `END`'s echo, for a pass that fell through to it and for no other;
* `UNTIL`'s separate re-echo and test, which is deliberately not shared with the
  top-of-loop echo;
* `ITERATE` clause attribution -- a pass that ended in `ITERATE` gives the following
  re-test the `ITERATE`'s own clause where a pass that fell through gives it `END`'s;
* `LEAVE`/`ITERATE` label matching and the search-frame indent reset, whose owner is
  `is_loop || label.is_some()` and not `is_loop`.

Every one of those is measured against the oracle. Decomposing them into ops means either
reproducing each in op form or keeping a generic op that is CISC in all but name. That is
a phase of work with a large trace-comparison surface, for a defect that is already
closed.

**And the flattening itself must survive it.** Measured, interleaved, three runs each,
same binary: `emptyloop.rex` 0.53 s flat against 0.76 s under `REXX_NO_FLAT=1`;
`varlookup.rex` 0.86 s against 1.02 s. A decomposition that gives back the flattening's
~30% on the clause-dispatch floor is a regression, not a cleanup.

## What a reader who picks this up must not assume

* **Not that CISC caused the defect.** It did not; see above. If the motivation is
  correctness rather than structure, the work is already done.
* **Not that the item list can stay a `Vec<ObjRef>`.** Whatever drives the loop, the
  snapshot is one heap array and the cursor is an integer -- that is both the oracle's
  shape and what keeps the rooted-handle count O(1) in the collection's length.
* **Not that `REXX_NO_FLAT` is a supported fallback.** It is a spike switch. Both arms are
  the same binary on purpose, so the per-op checks are priced either way.

## Entry criteria, if it is ever scheduled

1. A trace-comparison harness that pins the five behaviours listed above **before** any op
   is added, so a decomposition is checked against them rather than against a suite that
   happens to be green.
2. A measured budget: the decomposed form must hold `emptyloop` and `varlookup` at the
   flattened numbers, interleaved against a baseline binary built in its own target
   directory.
3. A statement of what it removes. If `flat_top` and its pool do not actually go away, the
   correctness argument is gone and only the dispatch argument is left -- and no published
   measurement isolates fused loop instructions against decomposed iteration ops, so that
   argument would have to be made locally.
