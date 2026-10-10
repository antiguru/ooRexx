# `=` at the pause after an ITERATE echoes it at its static indent

Found by Phase 6.1 Task 6 (report concern 1; KNOWN GAP row in `docs/superpowers/plans/phase-4-exclusions.txt`). Queued by Phase 6.1 Task 12 (2026-10-10). Same stdout and rc; one echo line's indent differs.

Probe `e2.rex`, run from a fresh empty directory:

    trace ?a
    do i = 1 to 3
      if i = 1 then iterate
      say i
    end

Standard input `e2.in` (its first lines; the rest are empty or more `.stderr~lineout` markers):

    .stderr~lineout('p1')

    .stderr~lineout('p2')

    .stderr~lineout('p3')
    =
    .stderr~lineout('p4')

    .stderr~lineout('p5')

    .stderr~lineout('p6')

Oracle, rc 0:

    [stdout]
    3
    [stderr]
           +++ "LINUX COMMAND e2/e2.rex"
         2 *-* do i = 1 to 3
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p1
         3 *-*   if i = 1 
    p2
         3 *-*     then
         3 *-*       iterate
         2 *-* do i = 1 to 3
    p3
         3 *-*   iterate
         2 *-* do i = 1 to 3
    p4
         3 *-*   if i = 1 
    p5
         4 *-*   say i
    p6
         5 *-* end
         2 *-* do i = 1 to 3

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    3
    [stderr]
           +++ "LINUX COMMAND e2/e2.rex"
         2 *-* do i = 1 to 3
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p1
         3 *-*   if i = 1 
    p2
         3 *-*     then
         3 *-*       iterate
         2 *-* do i = 1 to 3
    p3
         3 *-*       iterate
         2 *-* do i = 1 to 3
    p4
         3 *-*   if i = 1 
    p5
         4 *-*   say i
    p6
         5 *-* end
         2 *-* do i = 1 to 3

Suspected site: the re-execute echo for a flowed ITERATE (`run/interpret.rs` debug pause, `ir/drive.rs` `flat_loop_step_escaped`): the oracle echoes at the loop body's indent.
