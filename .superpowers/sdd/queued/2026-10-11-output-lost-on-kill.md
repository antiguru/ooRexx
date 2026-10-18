**CLOSED: e4bbcbba4.** Phase 9c Task 7 hands standard error over at each line end, so a kill keeps it as on the oracle (closed 2026-10-17), while a redirected standard output is still lost at a kill, as Python's is: Deviation 40 (`docs/superpowers/plans/phase-4-exclusions.txt`).

# Output written before a KILL is lost

Found 2026-10-11 by the Phase 9a Task 2 implementer, building the suite harness.

`.error~say('started X'); do forever; end` under `timeout -s KILL 2`: the oracle prints `started X`
on stderr; `rexx-run` prints nothing (rc 137). `rexx-run` hands its stdout and stderr buffers to fd 1
and 2 only before a read, before the thread idles, and at exit (`input.rs` `hand_over_output`,
`clause.rs` `idle_until`), so anything written before a kill, a crash or an OOM is gone.

Owner: Phase 9c, when `rexx` replaces `rexx-run` (a launcher whose output a kill discards differs from
the oracle's observably). The cost of flushing per write must be measured against section 8's budget.
The suite harness does not depend on it: its started marker is a file closed per test.
