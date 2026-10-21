**OPEN, queued 2026-10-20 by the controller; proposed 9d (a rooting defect in a gated stress test).**

# collect_stress panics on lang/raise_additional_array.rex

Found by Phase 9d Task 6.15 (fix round 1 report, Concern 1) and confirmed by its re-review
(`.superpowers/sdd/2026-10-18-phase-9d/task-6.15-rereview-r1.md`) in a fresh worktree at
5698947b0 with its own target dir: the `collect_stress` test panics at dispatch.rs:1799 with
"a live value" on `lang/raise_additional_array.rex`. The program uses no streams, and the failure
predates Task 6.15. A collection that runs while a RAISE ... ADDITIONAL array is built frees a
value still in use, so something on that path is unrooted
(memory: fresh allocations in a closure are unrooted).
