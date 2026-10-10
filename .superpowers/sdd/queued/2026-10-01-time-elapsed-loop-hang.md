# 2026-10-01-time-elapsed-loop-hang

RESOLVED by e25925ad8 and a8593f6e6 (Phase 6.1 Task 11a Step 1): a WHILE re-test after the first pass and an UNTIL test read a fresh clock, so a loop with no body clause ends; witness `rust/corpus/lang/time_elapsed_empty_loop.rex`.

Found by the Phase 6 S2-S5 Task 4 review (M5), ruled not that task's (queued 2026-10-01). Not fixed;
present at dcf9fd554 (the Task 4 base) as well as after Task 4.

A loop whose `WHILE` reads `TIME('E')` never ends: the elapsed time it reads does not advance.
`TIME('E')` between ordinary clauses advances (the reviewer's `tm3.rex`, identical to the oracle), so
the loop's header clause does not invalidate the clause timestamp the way each oracle instruction
does (`settings.timeStamp.valid = false` after every instruction, `RexxActivation.cpp:647`). A busy
wait on elapsed time is a common way to wait for another activity.

Probe `elapsed.rex` (fresh empty directory):

    call time 'r'
    do while time('e') < 0.3
    end
    say 'empty body ended'
    call time 'r'
    do while time('e') < 0.3
      nop
    end
    say 'nop body ended'

Oracle: `empty body ended` / `nop body ended`, rc 0, in about 0.6 s. This crate (debug `rexx-run`,
at the Task 4 fix round 1 tree): no output, killed by `timeout -s KILL 10` (rc 137), so the empty
body loop already never ends. The reviewer measured the body form on its own, and both at the base,
each killed at 10 s.
