**OPEN, queued 2026-10-29 by the controller; proposed 9d (Task 8.8).**

# TRACE renders an assignment's value once for its >>> and >=> lines

Found by Phase 9d Task 8.7c (report Queue candidates 2; confirmed by its review); predates the task. Under `trace i`, `x = o` renders the value once for both lines here, where the oracle renders each line, so an object's own `DEFAULTNAME` runs twice here and three times there. Traced performance is no target; match the observable sends.
Probe: docs/superpowers/records/2026-10-18-phase-9d/t8.7c/probes/dn/dn7.rex.

Also confirmed by 8.7c re-review r2 (task-8.7c-rereview-r2.md queue candidate 1): a third send that raises is trapped on the oracle; docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr2/c1/d.rex.
