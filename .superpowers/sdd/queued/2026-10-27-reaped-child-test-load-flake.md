**CLOSED 2026-10-30 by Phase 9d Task 8.8c (d7581cc4a).**

**OPEN, queued 2026-10-27 by the controller; proposed 9d (Task 8.8).**

# rexxutil_process's reaped-child test fails under workspace load

`a_command_whose_child_was_reaped_elsewhere_fails_with_127` (rust/crates/rexx-exec/tests/rexxutil_process.rs:97)
failed once in a `cargo test --release --no-fail-fast --workspace` run at 75066653a (`cmd rc 5`, `wait
-1`; log docs/superpowers/records/2026-10-18-phase-9d/t8.4/close-workspace-tests.txt) and passed 3 of 3
runs alone. The program races a 0.6 s command against SysWait. Make the witness independent of load,
or show the crate's answer under load is wrong.

## Task 8.8c evidence

The integration witness accepts either valid winner in one bounded attempt, while exact-child waitpid coverage deterministically exercises ECHILD/127/FAILURE. The owned-child fixture uses waitid with WNOWAIT to establish that exit status 5 is available without consuming it, then retains Popen through successful exec so SysWait returns 1280. Its explicit dropping control consumes the already-waitable child and yields -1, exposing the discarded-object race without sleeps or probability. Owned-command rc 5/ERROR coverage and the bounded competing-waiter control remain intact, with unrelated command tracing explicitly disabled.

Evidence: `.superpowers/sdd/2026-10-18-phase-9d/task-8.8c-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/`.
