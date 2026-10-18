**OPEN: found by Phase 9d Task 6.6, not yet ruled.** An install error in a called program names the caller's file in `Error N running <file>`.

# An install error in a called program names the caller's file

Found 2026-10-18 by Phase 9d Task 6.6 (`task-6.6-report.md`, Concerns). When a program reached by
`CALL` or `::REQUIRES` fails to install, the crate's `Error N running <file>` line names the
calling program's file, where the oracle names the called one. Probes:
`docs/superpowers/records/2026-10-18-phase-9d/t6.6/probes/superclass/superclass_e`,
`superclass_g` and `superclass_l`. `superclass_l` is a plain 98.909 that differs at Task 6.6's
parent commit too.
