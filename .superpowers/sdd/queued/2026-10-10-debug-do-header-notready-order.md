# A NOTREADY raised by a DO header under `trace ?a` runs its handler before the header's pause

Found by the Phase 6.1 Task 6 re-review (`dh1`); pre-existing at the 6.1 base; without debug both agree (`dh1n`). Queued by Phase 6.1 Task 12 (2026-10-10). Same stdout and rc; the pause order and banner differ.

Probe `dh1.rex`, run from a fresh empty directory:

    call on notready name h
    trace ?a
    do i = 1 to length(linein('/nonexistent/dh')) + 1
      say 'body' i
    end
    say 'after'
    exit
    h: say 'h sigl' sigl; return

Standard input `dh1.in` (its first lines; the rest are empty or more `.stderr~lineout` markers):

    .stderr~lineout('p1')

    .stderr~lineout('p2')

    .stderr~lineout('p3')

    .stderr~lineout('p4')

    .stderr~lineout('p5')

    .stderr~lineout('p6')

Oracle, rc 0:

    [stdout]
    h sigl 3
    body 1
    after
    [stderr]
           +++ "LINUX COMMAND dh1/dh1.rex"
         3 *-* do i = 1 to length(linein('/nonexistent/dh')) + 1
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p1
         8 *-*     h:
    p2
         8 *-*     say 'h sigl' sigl;
    p3
         8 *-*     return
         4 *-*   say 'body' i
    p4
         5 *-* end
         3 *-* do i = 1 to length(linein('/nonexistent/dh')) + 1
         6 *-* say 'after'
    p5
         7 *-* exit

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    h sigl 3
    body 1
    after
    [stderr]
           +++ "LINUX COMMAND dh1/dh1.rex"
         3 *-* do i = 1 to length(linein('/nonexistent/dh')) + 1
         8 *-*     h:
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p1
         8 *-*     say 'h sigl' sigl;
    p2
         8 *-*     return
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p3
         4 *-*   say 'body' i
    p4
         5 *-* end
         3 *-* do i = 1 to length(linein('/nonexistent/dh')) + 1
         6 *-* say 'after'
    p5
         7 *-* exit

Suspected site: the DO/LOOP header arm of the debug pause (`ir/drive.rs` `debug_pause_in_region`): the oracle pauses after the header before it delivers the condition the header queued.
