**CLOSED 2026-10-29 by Phase 9d Task 8.7c fix round 1 (6da312e62), ruled R1 "Default's value, stem named".**
**OPEN, queued 2026-10-27 by the controller; proposed 9d (Task 8.7).**

# A stem whose default value has no string value names the stem in arithmetic

Found by the Phase 9d Task 8.5 fix round 2 re-review (task-8.5-rereview-r2.md, queue candidate 1);
predates the task. A plain stem (no reference) whose default value has no string value, used as an
arithmetic operand under SIGNAL ON NOSTRING: the oracle forwards to the default value (`h2 s. |nstr|
NSTR`), this crate names the stem (`h2 s. |a NSTR| Stem 0 0 1`). Probe c/refs2.rex of the re-review
(copied to docs/superpowers/records/2026-10-18-phase-9d/t8.5/r3/probes/ by Task 8.5's fix round 3, if kept).
Derive the stem's string value in a required-string position from rexxref before following the oracle.
