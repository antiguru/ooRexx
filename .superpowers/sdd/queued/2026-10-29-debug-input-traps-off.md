**CLOSED 2026-10-29 by Phase 9d Task 8.7c fix round 2 (9af69058d), ruled by Moritz 2026-10-29 ("Traps off only while the line runs"): traps are OFF while the typed line runs; the read of .DEBUGINPUT is an ordinary stream read, so its conditions reach their traps (Deviation 75's delayed rule included).**
**OPEN, queued 2026-10-29 by the controller; proposed 9d (Task 8.7c).**

# Conditions raised while debug input is processed are delivered

Found by Phase 9d Task 8.7b fix round 3 (task-8.7b-report.md, Fix round 3 Concerns 1); predates the task.
rexxref condtra.xml:489-494 (Note 4): traps are OFF while the input of an interactive debug pause is processed. This crate drops only a condition that input raises while its trap is delayed; one raised where its trap is not delayed is still delivered (dp2's first read in main; dp3's `h 2`).
Probes: docs/superpowers/records/2026-10-18-phase-9d/t8.7b/fix-r3/dp2/dp2.rex, docs/superpowers/records/2026-10-18-phase-9d/t8.7b/fix-r3/dp3/dp3.rex (both read debug input through .local~DEBUGINPUT; never run dp2 unbounded). Derive the full rule from Note 4 and the debug section of rexxref before changing the pinned debug case.
