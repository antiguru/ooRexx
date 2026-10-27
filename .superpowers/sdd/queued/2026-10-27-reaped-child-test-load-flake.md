**OPEN, queued 2026-10-27 by the controller; proposed 9d (Task 8.8).**

# rexxutil_process's reaped-child test fails under workspace load

`a_command_whose_child_was_reaped_elsewhere_fails_with_127` (rust/crates/rexx-exec/tests/rexxutil_process.rs:97)
failed once in a `cargo test --release --no-fail-fast --workspace` run at 75066653a (`cmd rc 5`, `wait
-1`; log docs/superpowers/records/2026-10-18-phase-9d/t8.4/close-workspace-tests.txt) and passed 3 of 3
runs alone. The program races a 0.6 s command against SysWait. Make the witness independent of load,
or show the crate's answer under load is wrong.
