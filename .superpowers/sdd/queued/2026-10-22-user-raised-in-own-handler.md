**OPEN, queued 2026-10-22 by the controller; proposed 9d.**

# A USER condition raised during its own CALL ON handler is never redelivered

Found by the Phase 9d Task 8.2 review (task-8.2-review.md, Minor 6). Both interpreters agree: a USER
condition raised while its own CALL ON handler runs is never delivered after the handler returns.
rexxref (condtra.xml, CALL ON: the condition is delayed while its handler runs) suggests the delayed
condition is delivered when the handler ends. Derive the rule; if both depart from it, it goes to
Moritz as a Deviation candidate.
