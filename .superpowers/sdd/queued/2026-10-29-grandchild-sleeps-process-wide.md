**CLOSED 2026-10-30 by Phase 9d Task 8.8b (ecce67dad).**

**OPEN, queued 2026-10-29 by the controller; proposed 9d (Task 8.8).**

# suite fixture grandchild check scans every process

`fixtures::a_command_a_test_leaves_running_is_killed_with_the_run` failed once under load ("the command outlived the run", suite.rs:2002): `grandchild_sleeps` scans all of /proc for `sleep 41.5`, so an unreaped grandchild of an earlier run counts against this one. Scope the check to the run's own process tree (8.7c re-review r1, task-8.7c-rereview-r1.md "Fixture test under load").
