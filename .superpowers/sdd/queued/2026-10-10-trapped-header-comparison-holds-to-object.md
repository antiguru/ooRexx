# A trapped error in a user `>` during a DO header holds the TO object

Found by the Phase 6.1 Task 3 re-review 3 (`.superpowers/sdd/2026-10-07-phase-6-1/task-3-rereview3.md`, out-of-scope observation 3, `lkJ.rex`); the base `6a3091cc6` binary does the same. Queued by Phase 6.1 Task 12 (2026-10-10). Silent: rc 0 on both; the UNINIT runs four collections late.

Probe `lkJ.rex`, run from a fresh empty directory:

    call pJ
    call gc 'force'; say 'gc1'
    call gc 'force'; say 'gc2'
    x = 'abc' || 'def'; call q
    call gc 'force'; say 'gc3'
    do n = 1 to 3; y = n * 2; end
    call gc 'force'; say 'gc4'
    call pK
    call gc 'force'; say 'gc5'
    exit
    q: procedure; return
    pJ: procedure
    signal on syntax name trapJ
    do i = .c~new('err') to .t~new('J') for 5; nop; end
    return
    trapJ: return
    pK: procedure
    do i = .c~new('ok') to .t~new('K') for 2; nop; end
    return
    ::class c
    ::method init; expose k; use arg k
    ::method '+'; return self
    ::method '>'
    expose k
    if k = 'err' then return 1 + 'x'
    return 0
    ::class t
    ::method init; expose n; use arg n
    ::method '+'; return self
    ::method uninit; expose n; say 'uninit' n

Oracle, rc 0:

    [stdout]
    uninit J
    gc1
    gc2
    gc3
    gc4
    uninit K
    gc5
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    gc1
    gc2
    gc3
    gc4
    uninit J
    uninit K
    gc5
    [stderr]
    (empty)

The same expression outside a loop (`lkJ3.rex` in the re-review) is freed at the first collection. The re-review's instrumented copy shows `hold_object_control` and `release_loop_objects` balanced, so the retention is elsewhere; Task 2's deferred minor (a trapped SYNTAX inside a DO WITH header and its temps) has the same shape.

Suspected site: the temps a DO header pushes before its first comparison, left in place when a SYNTAX trap unwinds the header (`run/loops.rs`).
