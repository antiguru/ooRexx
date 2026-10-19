**OPEN: found by the Phase 9d Task 6.10 re-review, a regression of Task 6.10 against `d2e55befa`.** A guarded trace destination that spins waiting on another thread hangs.

# A guarded trace destination that spins

Found 2026-10-19 by the Task 6.10 re-review (`task-6.10-rereview.md`, finding 2). Probe `gw1`: a
trace destination with the default, guarded `LINEOUT` spins until another thread sets a flag. The
oracle prints `done 1` on 8 of 8 runs, and so did Task 6.10's parent `d2e55befa` under wall clock
and three simulation modes. Since Task 6.10 the crate hangs in every mode (killed, rc 137).
Blocking waits in a guarded destination (`guard on when`, `syssleep`) still finish.

Task 6.10 suppresses the pinned yield while a delivery's activation holds a guard, because a pinned
yield while another delivery waits for the same guard reaches the refusal "a pinned wait for a
guard that only an activity pinned below it can end" (`rexx-exec/src/lib.rs`). That suppression
fixed a two-thread trace deadlock and lost REPLY trace lines the parent had. Closing this needs
the pinned-guard-wait limitation lifted.

The round-3 re-review (`task-6.10-rereview-r3.md`, Minor 1) found a second shape with the same
root. Two started activities trace two lines each into one destination whose default, guarded
LINEOUT calls `syssleep 0.01`. The oracle prints `lines 10 results A B` on 8 of 8 runs. Since Task
6.10 round 1 the crate ends with rc 120, `rexx-exec: a pinned wait for a guard that only an
activity pinned below it can end is not implemented`, in every mode. The parent printed `lines 5`,
rc 0: it failed the second activity's deliveries too, but swallowed the failure. This is an
ordinary shape (a guarded logging destination, two threads, a blocking call in LINEOUT), so the
pinned-guard-wait limitation is worth weighing for 9d.
