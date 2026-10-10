# `List~remove` copies the list per call

Found by the Phase 6.1 final review (finding 4, `probes/b/t21a.rex`); pre-existing at the 6.1 base, which the review found timing out on the first loop. Queued by the Phase 6.1 final fix (2026-10-10). Same answers; quadratic time where the oracle is constant per call. Task 11a's scope named `Array~append`, `items` and `List~append`, not `remove`.

Probe `lr2.rex`, run from a fresh empty directory under `timeout -k 5 60`:

    l = .list~new; do i = 1 to 200000; l~append(i); end
    call time 'r'
    do i = 1 to 20000; l~remove(l~first); end
    say 'remove first x20000' time('r')
    do i = 1 to 20000; l~remove(l~last); end
    say 'remove last x20000' time('r')
    say l~items

Oracle, rc 0, one run:

    [stdout]
    remove first x20000 0.003279
    remove last x20000 0.004486
    160000
    [stderr]
    (empty)

This crate (`rexx-run` built from the final fix's finding 1 tree), rc 124 at the timeout, one run, `PROGRAM` standing for the program's path:

    [stdout]
    remove first x20000 29.150683
    [stderr]
         5 *-*   l~remove(l~last);
    Error 4 running PROGRAM line 5:  Program interrupted.
    Error 4.1:  Program interrupted with HALT condition.

Suspected site: `dispatch/collection/list.rs`. `list_position` copies the handles and items arrays (`array_slots_owned`) and scans them for the handle on every lookup, and `list_take` removes from both by position (`array_splice`, a `Vec::remove` that shifts every later slot).
