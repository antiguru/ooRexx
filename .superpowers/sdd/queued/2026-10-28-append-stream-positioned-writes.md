**CLOSED 2026-10-29 by Phase 9d Task 8.7c fix round 1 (6da312e62), ruled R3 "Enforce rexxref".**
**OPEN, queued 2026-10-28 by the controller; proposed 9d (Task 8.7).**

# Positioned writes on an APPEND stream

Found by the Phase 9d Task 8.6 fix round 1 re-review (task-8.6-rereview-r1.md, Out of scope); predates
the round. File `aaaa|bbbb|`, `open('write append')`, then `seek('=1 write char')` (answered `1` on all):
the oracle reports `pos3 4`, `charout('C', 1)` `0` then `pos4 2`, `lineout('L4', 1)` `0`, file
`aaaa|bbbb|L1|L2|CL4|`; this crate reports `pos3 17`, accepts the CHAROUT but refuses the LINEOUT (`w4 1`),
file `aaaa|bbbb|L1|L2|C`. rexxref funct.xml:4309-4312: the write pointer cannot move into the file as it was
opened, which neither side's SEEK enforces, and this crate's CHAROUT and LINEOUT disagree. Probe:
docs/superpowers/records/2026-10-18-phase-9d/t8.6/r2/probes/a1.rex. Derive from rexxref first.
