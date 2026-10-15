# Error message position prints "&1" where the oracle prints "~"

Source: Phase 9b Task 7 report (.superpowers/sdd/2026-10-13-phase-9b/task-7-report.md), pre-existing.
`call o~string('x')` raises 35.1 on both sides; the crate's message says `at "&1"`, the oracle's
`at "~"`. Find which token the crate's error position names for a message-send CALL and make it the
oracle's, with a corpus witness.

## Added 2026-10-15 (9b1 Task 7 review)

Error 47.2 prints the label as "&1" where the oracle prints "S1" -- the same unsubstituted-insert
shape. Also found: `instanceMethod` answers .nil after `setMethod` with a Method object (oracle
answers the method); probes in .superpowers/sdd/2026-10-14-phase-9b1/task-7-review-files/.
