**CLOSED 2026-10-26 by Phase 9d Task 8.4 (7fe50bedd).**

**OPEN, queued 2026-10-22 by the controller; proposed 9d (Task 8.4).**

# A ::CONSTANT's stack frame and executable

Found by Phase 9d Task 8.3 (task-8.3-report.md, concern 3). Probe:
docs/superpowers/records/2026-10-18-phase-9d/t8.3/probes/constant_frames.rex.

Oracle: `.context~stackFrames` inside a ::CONSTANT's evaluation lists `METHOD ::CONSTANT 5` then
`PROGRAM <file> 3`, and `.context~executable` is a Method. Crate (99fd7ae2b): `PROGRAM ::CONSTANT 0`
alone, and a Routine.

Open: derive from rexxref (::CONSTANT defines a method; Ruled 13) before taking the oracle's answer.
