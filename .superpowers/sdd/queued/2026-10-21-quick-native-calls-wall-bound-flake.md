**CLOSED 2026-10-30 by Phase 9d Task 8.8c (949a7806b).**

**OPEN, queued 2026-10-21 by the controller; proposed 9d (a flaky lib test).**

# quick_native_calls_at_the_smallest_bound_run_alike is wall-clock flaky

Found by the Phase 9d Task 6.15 round-6 re-review (task-6.15-rereview-r6.md, Minor 1): it failed
2 of 40 parallel `cargo test --release -p rexx-exec --lib` runs. It sets `block=0.001` (1 ms) on a
native call, and one of its inner runs occasionally exceeds that under load. It predates Task 6.15
(last touched in 629f2d79b, Phase 9c). Make it independent of wall time, or run it under
`REXX_SWITCH_MODE=sim:SEED`.

## Task 8.8c evidence

The repeated native-call control now supplies a completed watch outcome under the deterministic simulation schedule. An adjacent supplied timeout outcome still refuses, and the actual stalled-native integration test continues to exercise its real wall bound. The existing configuration test still accepts block=0.001 and rejects smaller values, while completed native calls no longer depend on host descheduling. This is a test-only outcome seam at the watch decision boundary rather than a clock framework.

Evidence: `.superpowers/sdd/2026-10-18-phase-9d/task-8.8c-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/`.
