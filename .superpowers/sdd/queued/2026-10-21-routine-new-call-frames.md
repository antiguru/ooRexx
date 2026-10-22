**CLOSED 2026-10-22 by Phase 9d Task 8.3 (68e7f15d5).**

**OPEN, queued 2026-10-21 by the controller; proposed 9d.**

# Frames of `.Routine~new(...)~call` name the wrong executable

Found by Phase 9d Task 7.1 (task-7-report.md, Concern 4; probe
docs/superpowers/records/2026-10-18-phase-9d/t7.1/probes/rn.rex). The stack frames of a routine
built with `.Routine~new(...)` and run by `~call` read `ROUTINE CALL` here, where the oracle lists
`ROUTINE m` and then `METHOD CALL`. Derive from rexxref (StackFrame, Routine~call) before matching.
