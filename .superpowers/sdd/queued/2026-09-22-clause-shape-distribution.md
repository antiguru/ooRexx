# How many clause shapes would a superinstruction set have to cover?

Derived 2026-09-22 from the rendered op stream of the pinned
`rust/bench-rexxcps/rexxcps.rex` at the `26ccef4ee` tree, by grouping each
`Op::Clause`'s region into the sequence of op kinds it contains. **Trace ops are
excluded**, because `rexxcps`' are an artifact of its own two `TRACE VALUE`
clauses and ten of the sixteen `bench-programs` axes emit none at all.

This is the evidence for the "widen the ops" direction, and it is static
structure rather than a measurement of a change.

## The timed body, lines 41 to 83

**80 clauses, 34 distinct shapes, mean 2.81 ops per region.**

| clauses | cumulative | shape |
|---:|---:|---|
| 18 | 22.5% | *(empty region)* |
| 6 | 30.0% | `Const Store` |
| 6 | 37.5% | `Const Say EndBranch` |
| 5 | 43.8% | `Const Say EndWhen` |
| 4 | 48.8% | `Load LoadConstant Binary ConditionJump EnterWhen` |
| 4 | 53.8% | `Exec` |
| 3 | 57.5% | `Parse` |
| 2 | 60.0% | `Load Load Binary ConditionJump` |
| 2 | 62.5% | `Load LoadConstant Binary ConditionJump` |
| 2 | 65.0% | `SelectCaseText` |

Region sizes: 18 clauses of 0 ops, 10 of 1, 12 of 2, 17 of 3, 6 of 4, 9 of 5,
and a tail to 15.

Whole program: 129 clauses, 57 distinct shapes, mean 2.98.

## What it supports and what it does not

**Ten shapes cover 65% of the timed body's clauses.** Every clause in lines 41
to 83 runs once per iteration, so for the straight-line part of the body the
static distribution is the executed distribution; the `IF` and `SELECT` arms are
where that stops being true, and their shares are not adjusted for it here.

The arithmetic that makes this interesting: the `Condition` + `JumpUnless`
fusion measured **54.8 instructions per removed op**, of which only **8 was
dispatch and 34.8 the value handoff between the pair**. A three-op region fused
to one removes two ops. **It does not follow that it removes 110 instructions**
-- that pair's handoff was entirely dead because the branch was the write-back's
only reader, and there is no reason to expect every pair to be. The measured
range for removing one op is 8 at the floor and 54.8 once, and nothing in
between has been measured.

**The tail is real.** 34 shapes for 80 clauses means a fixed superinstruction
set gets diminishing returns quickly, and each one is code in the driver, whose
frame is already 1,416 bytes with 42.9% of its instructions never executed on
this program. **A superinstruction set that grows the driver may pay for itself
in dispatch and lose it again in register pressure**, which is the interaction
nobody has measured and the reason this is a note rather than a plan.

**The empty regions are not the opportunity they look like.** 22.5% of the timed
body's clauses have no ops at all, and item 6 was killed on exactly that: the
executed count is 2,240,007 of 16,321,024 clause ops, and a region entry is
about 10 instructions, so 0.11%.

## The measurement that would settle it

Pick the two commonest non-empty shapes, `Const Store` and `Const Say
EndBranch`, fuse each to a single op, and measure. Two commits, each on its own
number. That prices one removed op in a shape nobody has priced, and it tells us
whether the 8-to-54.8 range collapses toward one end. **Until that exists, any
figure for a superinstruction set is arithmetic on an unmeasured constant.**
