# Where a cross-activity HALT lands inside a spinning loop (licensed divergence, P32)

The oracle raises a HALT set by another thread at the end of the instruction after its target's
next yield check. `RexxActivation::run` (`execution/RexxActivation.cpp:611-655`) checks every
`yieldInstructions` = 50 instructions (`RexxActivation.hpp:591`) with a per-activation counter
(`instructionCount`, `:651`; INTERPRET is its own activation), counting hidden THEN/ELSE/end-of-IF
instructions. So in practice the site is "the activation's 51st instruction", deterministic only
because the hand-off finds the slice already over.

This crate honours SLICE at an interpreter-wide countdown visit (`CLAUSES_PER_CHECK` 1024), so the
site is (visit position) mod loop length and varies run to run under the timer.

Ruled a scheduling observable (like GC ordering, [[oorexx-gc-ordering-divergence-licensed]]),
not matched. Parity would need: a per-activation count reloaded on call/return (cold path every 51
clauses; spec section 4 measured N=50 at +0.27% emptyloop, +0.06% rexxcps), and counting the
oracle's hidden instructions.

Oracle-deterministic probes (8-12 runs each) and the full analysis: Task 4 review, "Re-review 2
addendum", `.superpowers/sdd/2026-10-01-phase-6-s2-s5/task-4-review.md`. Example: `p = 1` /
`do i = 1` / `a = 1` / `b = 2` / `end` with the halting main busy-waiting on a value set before the
loop: oracle names `b = 2`; we name `a = 1`, `b = 2` or `end`.
