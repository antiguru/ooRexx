**CLOSED 2026-10-30 by Phase 9d Task 8.8c (949a7806b).**

**OPEN, queued 2026-10-27 by the controller; proposed 9d (Task 8.8).**

# rexxutil_process's reaped-child test fails under workspace load

`a_command_whose_child_was_reaped_elsewhere_fails_with_127` (rust/crates/rexx-exec/tests/rexxutil_process.rs:97)
failed once in a `cargo test --release --no-fail-fast --workspace` run at 75066653a (`cmd rc 5`, `wait
-1`; log docs/superpowers/records/2026-10-18-phase-9d/t8.4/close-workspace-tests.txt) and passed 3 of 3
runs alone. The program races a 0.6 s command against SysWait. Make the witness independent of load,
or show the crate's answer under load is wrong.

## Task 8.8c evidence

The integration witness now accepts either valid winner in one bounded attempt rather than retrying until SysWait wins. Deterministic mapping coverage reaps an exact child through waitpid before the existing collector sees ECHILD, then checks rc 127 and FAILURE without a runtime injection seam. An owned-child control preserves rc 5 and ERROR, while a process-isolated exec control checks SysWait raw status 1280. Retained runs show both race outcomes, and the fixture disables unrelated normal command RC tracing before asserting empty stderr.

Evidence: `.superpowers/sdd/2026-10-18-phase-9d/task-8.8c-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/`.
