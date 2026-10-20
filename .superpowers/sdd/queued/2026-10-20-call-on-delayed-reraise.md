**RULED: 9d by the controller, as Moritz ruled the post-Task-2 gaps 2026-10-19 ("proposal ok"); Task 8.** A CALL ON condition raised again while its delivery is delayed runs the handler twice.

# CALL ON: a second raise while the trap is delayed

Found 2026-10-20 by Phase 9d Task 6.13 fix round 3 (`task-6.13-report.md`, "Fix round 3",
Concerns). rexxref says a `CALL ON` trap is delayed from its first raise, and the same condition
raised again while it is delayed is ignored. Task 6.13 applies that to a command's own ERROR and
FAILURE. Two `raise error` calls in one clause (`zr = ra() + ra()`) still run the handler twice
here. The oracle segfaults on that shape (`rust/CLAUDE.md`, probes; `corpus/oracle-crashes.txt`),
so the rule comes from the documentation alone.
