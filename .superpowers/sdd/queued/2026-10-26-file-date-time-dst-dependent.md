**OPEN, queued 2026-10-26 by the controller; proposed 9d (Task 8.8).**

# rexxutil_file_date_time.rex depends on the local DST state

Found by Phase 9d Task 8.3 fix round 2 (task-8.3-report.md, concern 3). The corpus program fails the
gate on 2026-10-26 with the base build too: line 9, the epoch of 2020-10-25 02:30 (an ambiguous
local time in the Europe/Zurich DST fall-back hour), the crate answers 1603585800 and the oracle
1603589400. It passed on 2026-10-22, before the DST change. Open: whether the crate resolves an
ambiguous local time differently from the oracle (a divergence) or the program is environment
dependent (pin TZ in the program or the harness).
