**CLOSED 2026-10-21 by Phase 9d Task 8.1 (fb86ce94f).** Part 2 is not fixed: any expression register keeps its value until reused, not only a call's; reported in `.superpowers/sdd/2026-10-18-phase-9d/task-8.1-report.md`.

**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** own probe: a panic "a live activation", rc 101, where the oracle prints 1; its file marks it PRIORITY.

# Routine~start('CALL') panics; a call's return value outlives the call

Source: Phase 9b1 Task 5 report (.superpowers/sdd/2026-10-14-phase-9b1/task-5-report.md), concern 5.
Both pre-existing. PRIORITY: the first is a panic ("a live activation"), i.e. a crash.
1. Routine~start('CALL') panics with "a live activation". Reproduce from the report's probe, find the
   activation-lifetime assumption, fix with a corpus witness (oracle first).
2. A call's return value stays reachable until the next call (held in a slot past the call's end).
   Observable only as GC timing (licensed), but it pins memory; fix by clearing the slot when the
   value is consumed.
