**CLOSED 2026-10-29 by Phase 9d Task 8.7c (e620097f0).**
**OPEN, queued 2026-10-27 by the controller; proposed 9d (Task 8.7).**

# An operator on a variable reference leaves out the Compiled method traceback line

Found by the Phase 9d Task 8.5 review (task-8.5-review.md, queue candidate 2; implementer concern 4);
predates the task. When an operator sent to a VariableReference raises inside the referenced value's
method (arithmetic on `>v` whose value is not a number), the oracle's traceback carries a `Compiled
method "+"` line, as it does for an explicit `t~'+'(0)`, which this crate also prints; the operator path
on a reference omits it here. Probes p11 against p13, p14, p15 of the review (copied to
docs/superpowers/records/2026-10-18-phase-9d/t8.5/r1/probes/ by Task 8.5's fix round, if kept).
