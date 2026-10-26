**OPEN, queued 2026-10-26 by the controller; proposed 9d (Task 8.7).**

# A compile failure's condition object names the caller's package

Found by Phase 9d Task 8.4 (task-8.4-report.md, concern 3). Probe:
docs/superpowers/records/2026-10-18-phase-9d/t8.4/naming/probes/compile-condition/m.rex, run from a
directory holding its two callees. After a parse or translation error in a called file,
`condition('o')~program` and `~package` name the calling program here; the oracle names the file
being translated and hands out a package object for it. Needs a package object for a source that
did not translate. Derive from rexxref (condition object PROGRAM/PACKAGE) before following the oracle.
