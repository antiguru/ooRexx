**CLOSED 2026-10-28 by Phase 9d Task 8.7b (5053fbd9b).**
**OPEN, queued 2026-10-26 by the controller; proposed 9d (Task 8.7).**

# The main program's PROGRAM level is missing while its own ::REQUIRES install

Found by Phase 9d Task 8.4 (task-8.4-report.md, concern 1). Probe:
docs/superpowers/records/2026-10-18-phase-9d/t8.4/constant/probes/f1.rex (records
t8.4/constant/residual-*.txt). While a main program's ::REQUIRES package installs (a ::CONSTANT
expression there answering .context~stackFrames), the oracle lists the main program's PROGRAM
level below the required package's ROUTINE level; the crate has no activation for the main program
yet, so the level is missing.
