# Under interactive trace, REPLY does not pause and the continuation prints the banner again

Found by Phase 6.1 Task 6 (report concern 3; re-review M4 for the banner; KNOWN GAP row for the pause). Queued by Phase 6.1 Task 12 (2026-10-10). Concurrent: 5 runs per engine. Same stdout and rc. The method `U` part of this probe also shows the `>I>` gap queued as `2026-10-02-trace-entry-after-expose`.

Probe `k3.rex`, run from a fresh empty directory:

    o = .obj~new
    o~u
    o~r
    say 'end' result
    call syssleep 0.3
    exit
    ::class obj
    ::method u
      expose w
      trace ?a
      w = 1
      guard on
      guard off when w = 1
      return
    ::method r
      trace ?a
      reply 5
      say 'after reply'
      nop
      return

Standard input `k3.in` (its first lines; the rest are empty or more `.stderr~lineout` markers):

    .stderr~lineout('p1')

    .stderr~lineout('p2')

    .stderr~lineout('p3')

    .stderr~lineout('p4')

    .stderr~lineout('p5')

    .stderr~lineout('p6')

Oracle, rc 0, 5 runs, 1 distinct outcome:

    [stdout]
    end 5
    after reply
    [stderr]
           >I> Method "U" with scope "OBJ" in package "k3/k3.rex".
        11 *-* w = 1
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p1
        12 *-* guard on
        13 *-* guard off when w = 1
        14 *-* return
           <I< Method "U" with scope "OBJ" in package "k3/k3.rex".
           >I> Method "R" with scope "OBJ" in package "k3/k3.rex".
        17 *-* reply 5
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p2
           <I< Method "R" with scope "OBJ" in package "k3/k3.rex".
           >I> Method "R" with scope "OBJ" in package "k3/k3.rex".
        18 *-* say 'after reply'
    p3
        19 *-* nop
    p4
        20 *-* return
           <I< Method "R" with scope "OBJ" in package "k3/k3.rex".

This crate (`rexx-run` at `6a87cd616`), rc 0, 5 runs, 1 distinct outcome:

    [stdout]
    end 5
    after reply
    [stderr]
           +++ "LINUX METHOD k3/k3.rex"
        11 *-* w = 1
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p1
        12 *-* guard on
        13 *-* guard off when w = 1
        14 *-* return
           >I> Method "R" with scope "OBJ" in package "k3/k3.rex".
        17 *-* reply 5
           <I< Method "R" with scope "OBJ" in package "k3/k3.rex".
           >I> Method "R" with scope "OBJ" in package "k3/k3.rex".
        18 *-* say 'after reply'
    +++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
    p2
        19 *-* nop
    p3
        20 *-* return
           <I< Method "R" with scope "OBJ" in package "k3/k3.rex".

Suspected site: the REPLY instruction's debug pause and the per-activation banner flag carried across the continuation (`scheduler.rs` REPLY, `run/interpret.rs`).
