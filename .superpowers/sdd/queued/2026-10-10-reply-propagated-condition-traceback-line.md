# A condition propagated in a REPLY continuation lacks the sender's traceback line

Found by Phase 6.1 Task 1 (report concern 1, the RF1 (d) SIGNAL ON variant). Queued by Phase 6.1 Task 12 (2026-10-10). Concurrent: 5 runs per engine. Code, message and rc agree; the crate test `scheduler::tests::a_reply_continuation_propagates_its_methods_condition` asserts only those.

Probe `rf1d.rex`, run from a fresh empty directory:

    o = .t~new
    say 'got' o~m
    call SysSleep 0.2
    say 'main done'
    ::class t
    ::method m
      signal on syntax
      x = 1/0
      return 0
    syntax:
      reply 1
      say 'cont propagates'
      raise propagate

Oracle, rc 0, 5 runs, 1 distinct outcome:

    [stdout]
    got 1
    cont propagates
    main done
    [stderr]
         8 *-* x = 1/0
         2 *-* say 'got' o~m
    Error 42:  Arithmetic overflow/underflow.
    Error 42.3:  Arithmetic overflow; divisor must not be zero.

This crate (`rexx-run` at `6a87cd616`), rc 0, 5 runs, 1 distinct outcome:

    [stdout]
    got 1
    cont propagates
    main done
    [stderr]
         8 *-* x = 1/0
    Error 42:  Arithmetic overflow/underflow.
    Error 42.3:  Arithmetic overflow; divisor must not be zero.

Suspected site: the traceback a propagated condition carries after REPLY moves the activation (`scheduler.rs` continuation, `run/condition.rs`): the oracle keeps the sender's frame `2 *-* say 'got' o~m` at the raise.
