# Array~sort raises 98.975 after a trailing remove

Found by the Phase 6.1 final fix (concern 4 in `final-fix-report.md`); the fixer reports the same answer before and after finding 3. Queued by the Phase 6.1 controller (2026-10-10).

After `remove` empties the last slot, the oracle sorts the remaining items and keeps the size at 2. This crate raises 98.975 for the emptied position.

Probe `s.rex`, run from a fresh empty directory:

    b = .array~of('c','a','b')
    b~remove(3)
    b~sort
    say b~items b[1] b[2]

Oracle (`/home/moritz/dev/repos/ooRexx/build/bin/rexx` under `ulimit -v 1048576`), rc 0:

    2 a c

This crate (`rust/target/release/rexx-run` in the worktree, under `memcap 2G`), rc 158:

           *-* Compiled method "SORT" with scope "Array".
         3 *-* b~sort
    Error 98 running PROGRAM line 3:  Execution error.
    Error 98.975:  Missing array element at position 3.

Suspected site: the Array SORT native sorts over `size` rather than up to the last item, so it meets the emptied slot. `ArraySlots` now keeps the last item (`c10b30ff1`), which the sort can use as its bound. Probing an empty slot inside the range, not at the end, comes before choosing the fix.
