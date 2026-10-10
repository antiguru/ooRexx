# Upstream candidate: the oracle's DO TO test compares a user answer with TheTrueObject by pointer

Found by Phase 6.1 Task 3 (memory `oorexx-do-to-true-identity-bug`; Moritz 2026-10-08 ruled it an oracle bug). Queued by Phase 6.1 Task 12 (2026-10-10). Not a crate item: Deviation 25 in `docs/superpowers/plans/phase-4-exclusions.txt` records the crate's rule. Filing upstream needs Moritz.

Probe `doto.rex`, run from a fresh empty directory:

    n = 0
    do i = .c~new to 5 for 3
      n = n + 1
    end
    say 'computed one:' n
    n = 0
    do i = .d~new to 5 for 3
      n = n + 1
    end
    say 'literal one:' n
    ::class c
    ::method '+'; return self
    ::method '>'; return 0 + 1
    ::class d
    ::method '+'; return self
    ::method '>'; return 1

Oracle, rc 0:

    [stdout]
    computed one: 3
    literal one: 3
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    computed one: 0
    literal one: 0
    [stderr]
    (empty)

A user `>` answering `1`, computed or literal, never ends the oracle's loop: `DoBlock.cpp:213` and `DoBlockComponents.cpp:173` test the answer against `TheTrueObject` by identity, where WHILE and UNTIL use `truthValue`.
