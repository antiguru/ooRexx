**CLOSED 2026-10-28 by Phase 9d Task 8.7a (030af45a1).**

**OPEN, queued 2026-10-27 by the controller; proposed 9d (Task 8.7).**

# String~equals and ~caselessEquals ignore a makeString argument

Found by the Phase 9d Task 8.5 fix round 1 re-review (task-8.5-rereview-r1.md, Out of scope); predates
the task (base 6e0bf5e5b the same). `'kmake'~equals(k)` and `'KMAKE'~caselessEquals(k)`, `k` an object
whose makeString answers `kmake`, answer 1 on the oracle and 0 here; a reference argument follows it.
Probe p05/meth.rex of the re-review (copied to docs/superpowers/records/2026-10-18-phase-9d/t8.5/r2/probes/
by Task 8.5's fix round 2, if kept). Derive the argument conversion from rexxref (String equals, Required
String Values) before following the oracle.
