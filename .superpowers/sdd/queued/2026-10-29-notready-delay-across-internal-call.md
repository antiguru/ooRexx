**OPEN, queued 2026-10-29 by the controller; proposed 9d (Task 8.7c).**

# A NOTREADY inside an internal routine called in the clause does not see the clause's DELAY state

Found by the Phase 9d Task 8.7b fix round 3 re-review (task-8.7b-rereview-r3.md); predates the task. Under CALL ON NOTREADY (or ANY), a NOTREADY raised inside an internal routine called later in a clause that already queued the condition is not ignored: `linein(...) || rn()` runs the handler twice here and on the base (`hn 1 ...zz2`, `hn 2 ...zz`), because `notready_trap_delayed_at` checks the routine's own activation. Decide from rexxref condtra.xml (Note 2's inherited DELAY state; NOTREADY) whether the DELAY reaches the routine. Do not run these shapes on the oracle: two conditions queued at one clause are crash-shaped (rust/corpus/oracle-crashes.txt entries 66, 67).
Probes: docs/superpowers/records/2026-10-18-phase-9d/t8.7b/review-probes/notready-internal-routine-n4.rex, docs/superpowers/records/2026-10-18-phase-9d/t8.7b/review-probes/notready-internal-routine-n7.rex.
