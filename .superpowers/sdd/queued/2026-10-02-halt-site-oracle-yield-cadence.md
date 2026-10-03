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

## Guard waiter wake cadence and native-call yield points (Task 12 review m1, m2; ruling P44)

A notify readies a GUARD WHEN waiter without a switch. The oracle hands over at its next
50-instruction check; this crate at the next slice. Busy notifier `d1e.rex`: oracle "woke at 24"
5 of 5, this crate about 200k to 370k. On the oracle every native call (SAY included) is also a
yield point to a queued activity; unswitched this crate always takes the oracle's minority order
(`d1a` 12 of 30, `d1b` 9 of 30, `d1d` 9 of 30). Every outcome this crate gives is oracle-observed.

`d1e.rex`:

```rexx
/* busy notifier without output: where does the woken waiter run? */
o = .k~new
m = o~start('waiter')
call SysSleep 0.1
o~run
m~wait
say 'woke at' o~seen
::class k
::method init
  expose v c seen
  v = 0; c = 0; seen = -1
::method seen unguarded
  expose seen
  return seen
::method waiter unguarded
  expose v c seen
  guard off when v = 1
  seen = c
::method run unguarded
  expose v c
  v = 1
  do 3000000
    c = c + 1
  end
```

`d1a.rex`:

```rexx
/* busy notifier: after the store main keeps computing, one SAY per clause */
o = .k~new
m = o~start('waiter')
call SysSleep 0.1
o~set
do i = 1 to 20
  say 'main' i
end
m~wait
::class k
::method init
  expose v
  v = 0
::method waiter unguarded
  expose v
  guard off when v = 1
  say 'woke'
::method set unguarded
  expose v
  v = 1
```
