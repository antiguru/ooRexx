**CLOSED 2026-10-30 by Task 8.8d (03d74f029).**

# The rooting harness cannot see a dropped push in the truth or condition-value paths

Found by the Phase 6.1 Task 4a review (`.superpowers/sdd/2026-10-07-phase-6-1/task-4a-review.md`). Queued by Phase 6.1 Task 12 (2026-10-10). A test-harness gap for the final review, not a program divergence.

Removing the truth judgment's root push or `condition_value`'s root push left every rooting test green, and so did the positive control (unrooting the user STRING answer in `reqstr.rs`). No program probe: the evidence is the mutation runs the review records.

Suspected site: the collection-stress harness (`tests/collect_stress.rs` and the `gc=` knob), which did not collect at a point where these values were held only by the dropped push.

## Task 8.8d disposition

Both dropped pushes now fail independently in compiled current mutants. TRACE R allocates between a fresh loop-condition concatenation and its read, while a three-element Array returned by == leaves a future receiver unowned after OBJECTNAME replaces the exit-value global. Normal and collect-every-allocation baselines pass, exact restorations pass, and the measured M12 collection/slot-reuse witness supplies a functioning GC positive control. The surviving two-element truth candidate and its residual owner remain documented rather than counted as a catch.

See [Task 8.8d report](../2026-10-18-phase-9d/task-8.8d-report.md) for per-subpart evidence and limits.
