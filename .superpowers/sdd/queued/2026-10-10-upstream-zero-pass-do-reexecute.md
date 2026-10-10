# Upstream candidate: `=` at a zero-pass DO's pause makes a later END raise 10.1

Found by Phase 6.1 Task 6 (report concern 2); recorded as Deviation 26 in `docs/superpowers/plans/phase-4-exclusions.txt`, owner none. Queued by Phase 6.1 Task 12 (2026-10-10). Not a crate item. Filing upstream needs Moritz.

Probe `z26.rex`, run from a fresh empty directory:

    trace ?a
    do 0
    say 'never'
    end
    do i = 1 to 2
    nop
    end
    say 'end'

Standard input `z26.in` (its first lines; the rest are empty or more `.stderr~lineout` markers):

    =

Oracle, rc 246:

    [stdout]
    (empty)
    [stderr]
           +++ "LINUX COMMAND z26/z26.rex"
         2 *-* do 0
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
         2 *-* do 0
         5 *-* do i = 1 to 2
         6 *-*   nop
         7 *-*   end
         7 *-*   end
    Error 10 running z26/z26.rex line 7:  Unexpected or unmatched END.
    Error 10.1:  END has no corresponding DO, LOOP, or SELECT.

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    end
    [stderr]
           +++ "LINUX COMMAND z26/z26.rex"
         2 *-* do 0
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
         2 *-* do 0
         5 *-* do i = 1 to 2
         6 *-*   nop
         7 *-* end
         5 *-* do i = 1 to 2
         6 *-*   nop
         7 *-* end
         5 *-* do i = 1 to 2
         8 *-* say 'end'

The Task 6 review traced it to `RexxInstructionBaseLoop::execute` calling `terminate()` for a zero-pass loop before the re-execute; the block stack is one too deep at the next END.
