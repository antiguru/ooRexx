**RULED: 9d, ruled by Moritz 2026-10-19 ("proposal ok").** gap.

**OPEN: found by the Phase 9d Task 6.8 review, not yet ruled.** The `.local~novalue` hook is not implemented.

# The NOVALUE environment hook

Found 2026-10-19 by the Task 6.8 review (`task-6.8-review.md`, Minor 4). With
`.local~novalue = .nv~new`, the oracle sends `NOVALUE` to that object for `say zzz`, for `a.7`, and
for the selectors of `drop (zzz)` and `procedure expose (yyy)`. The crate prints `ZZZ`, at the
parent commit too. A search of `oodocs/rexxref` for NOVALUE beside `.local` or `.environment`
found no description of the hook.
