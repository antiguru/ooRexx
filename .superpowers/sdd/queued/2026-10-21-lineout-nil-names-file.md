**OPEN, queued 2026-10-21 by the controller; proposed 9d.**

# `.stdout~lineout(.nil)` renders .nil with the running file

Found by the Phase 9d Task 7.1 review (task-7-review.md, queue candidates; the same at base and
head). `.stdout~lineout(.nil)` prints `running <file> line 1` here where the oracle prints
`running REXX`. Derive from rexxref what string `.nil` renders as there before matching.
