**CLOSED 2026-10-28 by Phase 9d Task 8.7b (5053fbd9b).**
**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** no probe recorded; dispatch/library/surface.rs:859 still drops the lookup's Err through .ok()?.

# find_class's class_named branch drops a raised condition

Source: Phase 9b Task 1 review M1 (.superpowers/sdd/2026-10-13-phase-9b/task-1-review.md).
`surface.rs:846` (at 0368229d1): `find_class`'s branch for routines and for methods whose scope
is not a program class calls `class_named`, which uses `dot_variable().ok()`, so a condition
raised during the lookup is dropped instead of propagated. m2 fixed the `class_in_program` branch
only. Fix: propagate `Err` as `class_in_program` does, with a witness that fails before the fix.
