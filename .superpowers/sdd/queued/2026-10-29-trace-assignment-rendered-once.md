**CLOSED 2026-10-30 by Phase 9d Task 8.8f (3979b7e31).**

# TRACE renders an assignment's value once for its >>> and >=> lines

Found by Phase 9d Task 8.7c (report Queue candidates 2; confirmed by its review); predates the task. Under `trace i`, `x = o` renders the value once for both lines here, where the oracle renders each line, so an object's own `DEFAULTNAME` runs twice here and three times there. Traced performance is no target; match the observable sends.
Probe: docs/superpowers/records/2026-10-18-phase-9d/t8.7c/probes/dn/dn7.rex.

Also confirmed by 8.7c re-review r2 (task-8.7c-rereview-r2.md queue candidate 1): a third send that raises is trapped on the oracle; docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr2/c1/d.rex.

## Task 8.8f evidence

Each emitted assignment trace line renders separately through the existing object-name protocol. The result render precedes the write and the assignment render follows it. A third DEFAULTNAME failure reaches SYNTAX after the handler can observe the assigned value. TRACE OFF, R and I controls and Deviation 81 coverage pass. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8f-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/`.
