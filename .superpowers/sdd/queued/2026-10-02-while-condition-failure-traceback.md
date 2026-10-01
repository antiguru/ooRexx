# 2026-10-02-while-condition-failure-traceback

Found by the Phase 6 S2-S5 Task 4 re-review (O1), ruled not that task's (queued 2026-10-02). Not
fixed; the reviewer measured the same at dd293bbdb.

A `WHILE` whose condition calls a routine that fails on a later pass: the oracle's traceback names
the loop's `END` clause, the clause that sent control back to the header; this crate names the
`DO` clause. The routine's own line and the error line agree.

Probe `o1.rex` (fresh empty directory):

    do i = 1 while i < 3 | f(i)
      nop
    end
    exit
    f: procedure
      use arg n
      if n = 2 then return 1 / 0
      return 0

Oracle (rc 214):

         7 *-*         return 1 / 0
         3 *-*   end
    Error 42 running .../o1.rex line 7:  Arithmetic overflow/underflow.
    Error 42.3:  Arithmetic overflow; divisor must not be zero.

This crate (debug `rexx-run` at the Task 4 fix round 3 tree, rc 214): the same, except the second
traceback line is `     1 *-*   do i = 1 while i < 3 | f(i)`.

Probe `o1b.rex`, the `&` form with a body:

    n = 0
    do while n < 3 & h(n)
      n = n + 1
    end
    exit
    h: procedure
      use arg n
      if n = 1 then return 1 / 0
      return 1

Oracle's second traceback line `     4 *-*   end`; this crate's `     2 *-*   do while n < 3 & h(n)`.
