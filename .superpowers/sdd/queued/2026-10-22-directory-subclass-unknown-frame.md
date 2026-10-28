**CLOSED 2026-10-28 by Phase 9d Task 8.7b (5053fbd9b).**
**OPEN, queued 2026-10-22 by the controller; proposed 9d.**

# `.context~stackframes` lacks a Directory subclass's METHOD UNKNOWN frame

Found by Phase 9d Task 8.2 (task-8.2-report.md, "Found, not fixed"; identical at base). Inside a
Directory subclass's UNKNOWN-run entry method, `.context~stackframes` lacks the `METHOD UNKNOWN`
frame the oracle lists (probe docs/superpowers/records/2026-10-18-phase-9d/t8.2/probes/unknown-frame/sf.rex).
