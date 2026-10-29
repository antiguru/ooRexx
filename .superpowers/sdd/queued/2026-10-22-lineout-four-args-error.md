**CLOSED 2026-10-29 by Phase 9d Task 8.7c (e620097f0).**
**OPEN, queued 2026-10-22 by the controller; proposed 9d.**

# LINEOUT with four arguments raises a different error

Found by the Phase 9d Task 8.2 review (task-8.2-review.md, Minor 6; the same at base). `lineout`
with four arguments raises 88.907 on the oracle and 88.922 here. Derive the right error from rexxref
(funct.xml LINEOUT, errnums.xml) before matching.
