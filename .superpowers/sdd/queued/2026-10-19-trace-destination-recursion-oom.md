**OPEN: found by the Phase 9d Task 6.10 round-2 re-review, a regression of Task 6.10 against `d2e55befa`.** A trace destination that recurses to the control-stack limit on every line is OOM-killed instead of ending.

# Recursion to the stack limit inside a trace destination

Found 2026-10-19 (`task-6.10-rereview-r2.md`, finding 2). Probe `rec3`: a destination's LINEOUT
runs `call spin`, and `spin` recurses with no end. Since Task 6.10 the crate is killed at a 2G cap
after about 8 s. Task 6.10's parent printed `after`, rc 0. The oracle segfaults (rc 139). Bounded
depths show the cliff: depth 20000 passes on the parent (77 MB peak) and is OOM-killed since Task
6.10, while depths 1000 and 5000 pass on both. The crate's control-stack limit is about 9998.
The same recursion on the first LINEOUT alone terminates (trapped `11.1`, untrapped rc 245), so
the growth appears when every delivery reaches 11.1. A likely cause, not confirmed: the long error
report is itself delivered line by line through the same destination, and each line recurses to
the limit again. `rec1`/`rec2`, a recursing destination waiting on a worker, behave the same way.
