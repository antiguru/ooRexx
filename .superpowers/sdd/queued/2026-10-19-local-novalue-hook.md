**OPEN: found by the Phase 9d Task 6.8 review, not yet ruled.** The `.local~novalue` hook is not implemented.

# The NOVALUE environment hook

Found 2026-10-19 by the Task 6.8 review (`task-6.8-review.md`, Minor 4). The oracle consults a
`NOVALUE` entry in `.local` (and then `.environment`) before it raises NOVALUE for an unset
variable, as rexxref describes for the NOVALUE condition. The crate has no such lookup anywhere,
and no queued item covered it.
