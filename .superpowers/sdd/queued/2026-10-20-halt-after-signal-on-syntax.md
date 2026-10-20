**RULED: 9d, under Moritz's 2026-10-19 ruling that gaps found during 9d are fixed in 9d (Task 8).** A pending HALT taken at a SIGNAL ON SYNTAX label gives a broken report.

# HALT at a SIGNAL ON SYNTAX label

Found 2026-10-20 by Phase 9d Task 6.13 fix round 2 (`task-6.13-report.md`, "Fix round 2",
Concerns). After `SIGNAL ON SYNTAX` traps a condition, a HALT that is pending when the label runs
gives a broken report: untrapped, `0 *-* <no failing clause recorded>`; with `SIGNAL ON HALT`,
`Error 0 ... <no message 0.0>`. The oracle names the label line, or runs the HALT handler. It needs
no command to reproduce, and predates Task 6.13. Probes:
`docs/superpowers/records/2026-10-18-phase-9d/t6.13/r2/probes/halt/` (`h5`, `h6`).
