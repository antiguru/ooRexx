**CLOSED 2026-10-22 by Phase 9d Task 8.3 (68e7f15d5).**

**OPEN, queued 2026-10-21 by the controller; proposed 9d.**

# A copied REXX-package routine's frame answers the canonical Routine

Found by the Phase 9d Task 7.1 round-1 re-review (task-7-rereview-r1.md, Q3); the same at base.
A copy of a routine from the REXX package, reached by name or run through `~call`, has a frame whose
EXECUTABLE is the canonical Routine here, where the oracle answers the copy (`1 0` against `0 1`).
`Resolved::MergedInternal` drops the object, as `MergedLibraryRoutine` did before Task 7.1.
