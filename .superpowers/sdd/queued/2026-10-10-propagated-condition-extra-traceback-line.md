# A condition propagated from an internal call in a handler prints the call's traceback line

Found by Phase 6.1 scout B (2026-10-07, `cond3`); out of 6.1 scope by spec D9. Queued by Phase 6.1 Task 12 (2026-10-10). Code, message and rc agree; stderr has one extra line here.

Probe `b6_cond3.rex`, run from a fresh empty directory:

    call outer
    say 'main after outer'
    exit
    outer:
      signal on syntax
      say 1/0
      return
    syntax:
      call s
      say 'no propagate from s'
      return
    s:
      raise propagate

Oracle, rc 214:

    [stdout]
    (empty)
    [stderr]
         6 *-*   say 1/0
         1 *-* call outer
    Error 42:  Arithmetic overflow/underflow.
    Error 42.3:  Arithmetic overflow; divisor must not be zero.

This crate (`rexx-run` at `6a87cd616`), rc 214:

    [stdout]
    (empty)
    [stderr]
         6 *-*   say 1/0
         9 *-* call s
         1 *-* call outer
    Error 42:  Arithmetic overflow/underflow.
    Error 42.3:  Arithmetic overflow; divisor must not be zero.

Suspected site: the traceback a `RAISE PROPAGATE` carries (`run/condition.rs`): the oracle reports the original raise and the outer call, without the handler's `call s`.
