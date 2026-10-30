**CLOSED 2026-10-30 by Phase 9d Task 8.8b (ecce67dad).**

**OPEN, queued 2026-10-29 by the controller from an external (codex) review of 9af69058d; Moritz 2026-10-29 "lgtm" to the placement: proposed 9d (Task 8.8).**

# The oracle harness writes all of stdin before draining output

`rexx-exec/tests/support/oracle.rs:217` writes the whole stdin with `write_all` before reading stdout/stderr or starting the deadline, so input larger than the pipe fed to a program that does not read (or writes much first) blocks the test with no timeout. Feed stdin on its own thread beside the readers and the deadline. Witness: a harness test with input over the pipe size and a program that never reads, ending at its deadline.
