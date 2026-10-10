# An untrapped RAISE typed at a debug pause does not stop the typed lines

Found by the Phase 6.1 Task 6 re-review (out-of-scope observation, `t4`); pre-existing at `fe4956b36`. Queued by Phase 6.1 Task 12 (2026-10-10). Silent: rc 0 on both; stdout and stderr differ.

Probe `t4.rex`, run from a fresh empty directory:

    call on user u name h
    signal on syntax name s
    trace ?a
    x = 1
    say 'end'
    exit
    h: say 'h' sigl; return
    s: say 's' sigl; exit

Standard input `t4.in`:


    raise user u
    say 'mid'
    raise syntax 40.1
    say 'mid2'

Oracle, rc 0:

    [stdout]
    end
    [stderr]
           +++ "LINUX COMMAND t4/t4.rex"
         4 *-* x = 1
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
         5 *-* say 'end'

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    end
    mid
    mid2
    [stderr]
           +++ "LINUX COMMAND t4/t4.rex"
         4 *-* x = 1
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
         5 *-* say 'end'
    +++ Interactive trace.  Error 40:   Incorrect call to routine.
    +++ Interactive trace.  Error 40.1:  External routine "&1" failed.
         6 *-* exit

Suspected site: `run/interpret.rs` `run_debug_fragment`: the oracle stops running typed lines and tracing after an untrapped RAISE USER in one.
