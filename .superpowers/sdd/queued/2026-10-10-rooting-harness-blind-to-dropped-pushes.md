# The rooting harness cannot see a dropped push in the truth or condition-value paths

Found by the Phase 6.1 Task 4a review (`.superpowers/sdd/2026-10-07-phase-6-1/task-4a-review.md`). Queued by Phase 6.1 Task 12 (2026-10-10). A test-harness gap for the final review, not a program divergence.

Removing the truth judgment's root push or `condition_value`'s root push left every rooting test green, and so did the positive control (unrooting the user STRING answer in `reqstr.rs`). No program probe: the evidence is the mutation runs the review records.

Suspected site: the collection-stress harness (`tests/collect_stress.rs` and the `gc=` knob), which did not collect at a point where these values were held only by the dropped push.
