**CLOSED 2026-10-30 by Phase 9d Task 8.8c (949a7806b).**

**OPEN, queued 2026-10-26 by the controller; proposed 9d (Task 8.8).**

# rexxutil_file_date_time.rex depends on the local DST state

Found by Phase 9d Task 8.3 fix round 2 (task-8.3-report.md, concern 3). The corpus program fails the
gate on 2026-10-26 with the base build too: line 9, the epoch of 2020-10-25 02:30 (an ambiguous
local time in the Europe/Zurich DST fall-back hour), the crate answers 1603585800 and the oracle
1603589400. It passed on 2026-10-22, before the DST change. Open: whether the crate resolves an
ambiguous local time differently from the oracle (a divergence) or the program is environment
dependent (pin TZ in the program or the harness).

## Task 8.8c evidence

The accepted DST diagnosis isolates the oracle overlap choice to prior access-time conversion history, and the reference does not select an occurrence. Named-zone overlap rows now check membership in the valid epoch pair and local readback, retaining exact ordinary/gap comparisons and distinct output across zones. The helper preserves absent, empty and present TZ settings, with a red control showing that converting absence to an empty setting changed a subsequent ordinary epoch. Runtime make_time semantics remain unchanged, and the controller ruled that this fixture-contract repair needs no Deviation.

Evidence: `.superpowers/sdd/2026-10-18-phase-9d/task-8.8c-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/`.
