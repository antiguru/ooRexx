**CLOSED 2026-10-22 by Phase 9d Task 8.3 (68e7f15d5).**

**OPEN, queued 2026-10-21 by the controller; proposed 9d.**

# CallExpr and Call read the kept routine before the arguments

Found by the Phase 9d Task 7.1 round-3 re-review (task-7-rereview-r3.md, Q5); head, base and round 2
alike. `Op::CallExpr` and `Op::Call` read the site's kept routine before evaluating the arguments,
where the oracle reads `externalTarget` after them. An argument that re-enters the same site and then
rebinds the name gives a different result (probe t7-rereview3-scratch/n/reentry.rex).

Widened 2026-10-21 (Task 7.1 round-4 re-review, task-7-rereview-r4.md, Q6). A related shape is a
Task 7.1 regression, not pre-existing: in `Op::CallExpr` and `Op::Call`, an argument that re-enters
the site (its inner call returns, so the site keeps its routine), then rebinds the name and raises,
makes the outer call's forget wipe what the inner call kept. The oracle prints `answered 3`; head and
round 2 print `raised 98.941` (probe t7-rereview4-scratch/n2/reenter-raise.rex). Base matches the
oracle where the site starts empty. Fix both shapes together: read and forget the kept routine after
the arguments, as the oracle reads `externalTarget`.
