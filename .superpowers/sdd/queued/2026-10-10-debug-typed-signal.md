# A SIGNAL typed at a debug pause does not transfer control

Found by the Phase 6.1 Task 6 re-review (`c1sig`, `c2sig`); pre-existing at the 6.1 base. Queued by Phase 6.1 Task 12 (2026-10-10). Silent: rc 0 on both, stdout differs.

Probe `c1sig.rex`, run from a fresh empty directory:

    call on error name h
    trace ?a
    'false'
    say 'after'
    exit
    lbl: say 'at lbl' sigl; exit
    h: say 'handler sigl' sigl; return

Standard input `c1sig.in`:


    signal lbl

Oracle, rc 0:

    [stdout]
    at lbl 7
    [stderr]
           +++ "LINUX COMMAND c1sig/c1sig.rex"
         3 *-* 'false'
           >>>   "false"
           +++   "RC(1)"
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
         7 *-*   h:
         6 *-*   lbl:
         6 *-*   say 'at lbl' sigl;
         6 *-*   exit

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    handler sigl 1
    after
    [stderr]
           +++ "LINUX COMMAND c1sig/c1sig.rex"
         3 *-* 'false'
           >>>   "false"
           +++   "RC(1)"
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
         7 *-*   h:
         7 *-*   say 'handler sigl' sigl;
         7 *-*   return
         4 *-* say 'after'
         5 *-* exit

Suspected site: `run/interpret.rs` `run_debug_fragment`, which runs the typed line as a fragment and does not let a SIGNAL in it leave the paused clause.
