**OPEN, queued 2026-10-29 by the controller; proposed 9d (Task 8.8).**

# suite fixtures test fails once under load

`fixtures::a_child_refusing_on_the_inherited_stderr_leaves_the_run_ended_normally` (rexx-bench-suite or suite crate) failed once in the Phase 9d Task 8.7c fix round 2 workspace run and passed 3 of 3 alone (docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r2/close-workspace-tests.txt). Find the timing dependence and fix it, or add it to the known-flake list with its cause.

Re-review r1 of 8.7c (task-8.7c-rereview-r1.md) did not reproduce it: alone and with the fixtures module, plain and under lib-test load, every run passed. See also 2026-10-29-grandchild-sleeps-process-wide.md.
