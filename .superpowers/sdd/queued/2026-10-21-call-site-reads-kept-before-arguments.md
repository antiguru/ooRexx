**OPEN, queued 2026-10-21 by the controller; proposed 9d.**

# CallExpr and Call read the kept routine before the arguments

Found by the Phase 9d Task 7.1 round-3 re-review (task-7-rereview-r3.md, Q5); head, base and round 2
alike. `Op::CallExpr` and `Op::Call` read the site's kept routine before evaluating the arguments,
where the oracle reads `externalTarget` after them. An argument that re-enters the same site and then
rebinds the name gives a different result (probe t7-rereview3-scratch/n/reentry.rex).
