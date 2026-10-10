# A labelled block DO does not pause after its header

Found by Phase 6.1 Task 6 (report concern 4; KNOWN GAP row). Unchanged from the 6.1 base. Queued by Phase 6.1 Task 12 (2026-10-10). Same stdout and rc; the pause positions differ from the first pause on.

Probe `g1.rex`, run from a fresh empty directory:

    trace ?a
    do label lbl
      nop
    end lbl
    do
      nop
    end
    do label rl i = 1 to 2
      nop
    end rl
    loop label fl
      leave fl
    end fl
    do label w while 0
    end w
    select label s
      when 1 then nop
    end s
    say 'end'

Standard input `g1.in` (its first lines; the rest are empty or more `.stderr~lineout` markers):

    .stderr~lineout('p1')

    .stderr~lineout('p2')

    .stderr~lineout('p3')

    .stderr~lineout('p4')

    .stderr~lineout('p5')

    .stderr~lineout('p6')

Oracle, rc 0:

    [stdout]
    end
    [stderr]
           +++ "LINUX COMMAND g1/g1.rex"
         2 *-* do label lbl
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p1
         3 *-*   nop
    p2
         4 *-* end lbl
         5 *-* do
    p3
         6 *-*   nop
    p4
         7 *-* end
         8 *-* do label rl i = 1 to 2
    p5
         9 *-*   nop
    p6
        10 *-* end rl
         8 *-* do label rl i = 1 to 2
         9 *-*   nop
    p7
        10 *-* end rl
         8 *-* do label rl i = 1 to 2
        11 *-* loop label fl
    p8
        12 *-*   leave fl
    p9
        14 *-* do label w while 0
    p10
        16 *-* select label s
    p11
        17 *-*   when 1 
    p12
        17 *-*     then
        17 *-*       nop
    p13
        19 *-* say 'end'
    p14

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    end
    [stderr]
           +++ "LINUX COMMAND g1/g1.rex"
         2 *-* do label lbl
         3 *-*   nop
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p1
         4 *-* end lbl
         5 *-* do
    p2
         6 *-*   nop
    p3
         7 *-* end
         8 *-* do label rl i = 1 to 2
    p4
         9 *-*   nop
    p5
        10 *-* end rl
         8 *-* do label rl i = 1 to 2
         9 *-*   nop
    p6
        10 *-* end rl
         8 *-* do label rl i = 1 to 2
        11 *-* loop label fl
    p7
        12 *-*   leave fl
    p8
        14 *-* do label w while 0
    p9
        16 *-* select label s
    p10
        17 *-*   when 1 
    p11
        17 *-*     then
        17 *-*       nop
    p12
        19 *-* say 'end'
    p13

The unlabelled block DO, `do label rl i = 1 to 2`, `loop label fl`, `do label w while 0` and `select label s` pause as the oracle does.

Suspected site: the nested path a labelled block DO takes in the IR driver (`ir/drive.rs`), whose `Flow` comes after the body, so no header pause is taken.
