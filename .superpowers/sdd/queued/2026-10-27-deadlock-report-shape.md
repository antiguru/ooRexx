**OPEN, queued 2026-10-27 by the controller; proposed 9d (Task 8.7).**

# A guard deadlock's reports differ from the oracle's in count and echoes

Found by Phase 9d Task 8.4 fix round 2 (task-8.4-report.md, "Concerns (fix round 2)" 1); predates the
task. `d2.rex` (an ordinary guarded method deadlocking through `m~result`): the oracle prints one
report (`Compiled method "RESULT"`, `10 *-* m~result`, `2 *-* o~go`), this crate two. `g15.rex` (the
same deadlock while a ::CONSTANT method's re-run waits for its guard): the oracle prints two reports,
both echoing the directive and naming the waiter's line 10; here the started report names line 6
(Deviation 67's answer) and the waiter's report echoes nothing. Probes:
docs/superpowers/records/2026-10-18-phase-9d/t8.4/r2/probes/d2.rex, g15.rex. Derive the report shape
from rexxref (98.905, condtra.xml) before following the oracle; separate the part Deviation 67 covers.
