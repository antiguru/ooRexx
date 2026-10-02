# 2026-10-02-elapsed-clock-per-routine-and-reset

Found by the Phase 6 S2-S5 Task 10 review (I4). Not fixed; present at the S1 close 1a81353e3 and at
31e78a7a6, so not a Phase 6 regression and owned by no Phase 6 stage. Distinct from
`2026-10-01-time-elapsed-loop-hang.md` (there the clock does not advance in a loop).

Two defects, each failing a criterion-1 row in both switch modes: `TIME` TEST_4 and TEST_10
(`TIME.testGroup:1929`) and `CALL` TEST_4 (`CALL.testGroup:399`). Through the ooTest driver, one run
each: ours 5, 5 and 7 assertions with one failure, rc 1; the oracle 8, 8 and 18, rc 0.

1. A `::routine`, or an external program file, does not start with a fresh elapsed clock. Probe
   (`c2.rex`):

       call time 'r'
       x = 0
       do i = 1 to 300000; x = x + 1; end
       call r
       exit
       ::routine r
       t = time('r')
       say 'routine r zero:' (t = 0)

   Ours `0`, oracle `1`. An external file `extr.rex` (`t = time('r'); return t`) behaves the same
   way (`c5.rex`).

2. When the caller reads `TIME('E')` before an internal call, a `TIME('R')` in the subroutine
   resets the caller's clock. Probe:

       call time 'r'; call SysSleep 0.3; x = time('E'); call s
       say (time('e') >= 0.3); exit
       s: tt = time('R'); return

   Ours `0` at 31e78a7a6 and at 1a81353e3, oracle `1`. Without the `x = time('E')` line all three
   print `1`, which suggests the caller's clock state is the timestamp cached by the read and is not
   saved across the call.

Run counts: the driver rows one run each on the reviewer's tree; the probes one run each, the
oracle's answers as quoted. Probes are under the Task 10 review scratchpad `t10rev/c/`.
