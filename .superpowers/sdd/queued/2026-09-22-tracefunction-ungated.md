# Defect: `Op::TraceFunction` is emitted without the gate every other echo has

Found 2026-09-22 by the driver-gap task, incidentally, and deliberately not
fixed there. Not a correctness defect: it emits a trace op that can never echo,
so the cost is wasted work rather than a wrong answer.

## What is wrong

Every value-echo op in `compile.rs` is pushed under `echoes_values`, which is
the per-clause answer the trace dataflow analysis produces. **`Op::TraceFunction`
is the exception: both of its push sites are unconditional.**

So `e=length("ab")+e` under `TRACE N` compiles a `TraceFunction` that the driver
will dispatch and the gate inside will refuse, once per execution, for the life
of the chunk.

## Why it was left

* **Six golden streams pin its `path=` address under the default setting.**
  Changing the emission moves them, and a golden that moves for a reason nobody
  wrote down is worse than the op it removes.
* **It is worth zero on `rexxcps`**, whose trace ops exist for a different
  reason: two `TRACE VALUE` clauses whose value the source does not fix, which
  the analysis carries around the timed loop's back edge. Removing those two
  clauses takes the rendered stream from 721 ops to 518 and its trace ops from
  207 to 8.

## Before sizing it, know that the axis does not exist yet

**Ten of the `bench-programs/` axes emit no trace op at all.** So this item
cannot be sized on the current benchmark set: on `rexxcps` it is zero for the
reason above, and elsewhere there is nothing to remove.

Whoever takes it should either bring a program that exercises it -- a body with
a function call under a setting that does not echo -- or size it as a static
property of the emitted stream and say plainly that the runtime figure is
unmeasured.

## What the work is

Gate both push sites the way every neighbouring echo is gated, then regenerate
the six goldens and **say in the commit message why each moved**, since a golden
diff with no stated cause is the thing this project's records warn about most.
