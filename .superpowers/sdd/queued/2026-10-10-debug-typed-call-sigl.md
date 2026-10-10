# A CALL typed at a debug pause gives the callee SIGL 1

Found by the Phase 6.1 Task 6 re-review (`c2call`); pre-existing at the 6.1 base. Queued by Phase 6.1 Task 12 (2026-10-10). Silent: rc 0 on both; the oracle gives the paused clause's line.

Probe `c2call.rex`, run from a fresh empty directory:

    call on error name h
    trace ?a
    'false'
    say 'after'
    exit
    sub: say 'in sub' sigl; return
    h: say 'handler sigl' sigl rc; return

Standard input `c2call.in`:

    call sub

Oracle, rc 0:

    [stdout]
    in sub 3
    handler sigl 3 1
    after
    [stderr]
           +++ "LINUX COMMAND c2call/c2call.rex"
         3 *-* 'false'
           >>>   "false"
           +++   "RC(1)"
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
         6 *-*   sub:
         6 *-*   say 'in sub' sigl;
         6 *-*   return
         7 *-*   h:
         7 *-*   say 'handler sigl' sigl rc;
         7 *-*   return
         4 *-* say 'after'
         5 *-* exit

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    in sub 1
    handler sigl 3 1
    after
    [stderr]
           +++ "LINUX COMMAND c2call/c2call.rex"
         3 *-* 'false'
           >>>   "false"
           +++   "RC(1)"
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
         6 *-*   sub:
         6 *-*   say 'in sub' sigl;
         6 *-*   return
         7 *-*   h:
         7 *-*   say 'handler sigl' sigl rc;
         7 *-*   return
         4 *-* say 'after'
         5 *-* exit

Suspected site: `run/interpret.rs` `run_debug_fragment`: the typed line's clause numbering is the fragment's, not the paused clause's.
