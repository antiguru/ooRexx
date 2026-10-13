# Error message position prints "&1" where the oracle prints "~"

Source: Phase 9b Task 7 report (.superpowers/sdd/2026-10-13-phase-9b/task-7-report.md), pre-existing.
`call o~string('x')` raises 35.1 on both sides; the crate's message says `at "&1"`, the oracle's
`at "~"`. Find which token the crate's error position names for a message-send CALL and make it the
oracle's, with a corpus witness.
