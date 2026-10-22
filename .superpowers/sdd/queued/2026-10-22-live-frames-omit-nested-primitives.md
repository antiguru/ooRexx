**OPEN, queued 2026-10-22 by the controller; proposed 9d (Task 8.7).**

# Live stack frames omit a primitive that runs a Rexx body nested

Found by the Phase 9d Task 8.3 review (task-8.3-review.md, Q1, probe p9 in its section 3); predates
the task. `.context~stackFrames` inside a Rexx body run by `Object~run`, `Array~sort` (compareTo),
`Array~hasItem` (`==`), `Object~request` (makearray) or `Set~put` (hashCode) omits the primitive's
frame. The oracle lists `METHOD RUN | 1`, `METHOD SORT | 0`, `METHOD HASITEM | 1`,
`METHOD REQUEST | 1`, `METHOD PUT | 1`. A condition's frames already agree. Task 8.3 added the frame
for primitives entered through a `NativeTail` begin half only.
