**CLOSED 2026-10-28 by Phase 9d Task 8.6 (7596ced8d).**

**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** own probe: streamtype, query handle, lines, query exists and qualify on STDIN still differ.

# .Stream~new('STDIN') answers differ from the oracle

Source: .superpowers/sdd/2026-10-13-phase-9b/task-9-report.md, concern 3 (pre-existing).
Before open: streamtype, query handle, chars, lines differ. After open: qualify and QUERY EXISTS
differ. Measure each on the oracle and match, with a corpus witness (reads only; never a write
shape that 12a forbids).
