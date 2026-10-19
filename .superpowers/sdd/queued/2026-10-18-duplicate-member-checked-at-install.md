**RULED: 9d, ruled by Moritz 2026-10-19 ("proposal ok").** gap.

**OPEN: found by Phase 9d Task 6.6, not yet ruled.** Duplicate class members (99.902, 99.931, 99.932) are checked at install, not while parsing, so a later error wins.

# Duplicate members are checked at install

Found 2026-10-18 by Phase 9d Task 6.6 (`task-6.6-report.md`, Concerns). The duplicate half of the
oracle's `checkDuplicateMethod` runs in the directive parser. The crate keeps it in
`check_member_keys`, at install. A duplicate followed by a body error or a later parse error
therefore reports the later error: `docs/superpowers/records/2026-10-18-phase-9d/t6.6/probes/directive/directive_10`
gives 99.937 where the oracle gives 99.931, and `directive_11` gives 35.929 where the oracle gives
99.902. The crate's `rexxc` also accepts a file whose only fault is a duplicate. The fix is for the
parser to track each class's member keys, as `ClassDirective::checkDuplicateMethod` does. No test
of the 9d set depends on it.
