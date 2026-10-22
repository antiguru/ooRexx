**CLOSED 2026-10-22 by Phase 9d Task 8.3 (68e7f15d5).**

**RULED: 9d, ruled by Moritz 2026-10-19 ("proposal ok").** gap.

**OPEN: found by the Phase 9d Task 6.6 re-review, not yet ruled.** `.context~name` names the wrong context in an install activation.

# `.context~name` during an install

Found 2026-10-19 by the Task 6.6 round-2 re-review (`task-6.6-rereview-r2.md`, finding 1). Both
cases predate Task 6.6.
* An executable's `::constant k (digits() .context~name)` gives `4 ::CONSTANT` on the oracle and
  `4 ` (empty name) here.
* The prologue of a file required by a `.routine~newfile` package prints `.context~name~right(5)`.
  The oracle gives `prolog 9 q.cls`, naming the required file, and the crate `prolog 9 n.rex`,
  naming the main file.
