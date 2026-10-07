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

Task 25 (2026-10-06): one elapsed clock is shared by every method activation of an activity and
its caller, both ways; a started activity starts fresh. Probes and 30-run tallies, from the
repository root, `bash docs/superpowers/records/2026-10-01-phase-6-s2-s5/whole-groups/clock/run.sh
REXX_RUN SCRATCH 30` (output `clock/summary.txt` beside it): `a.rex` (a method's reset reaches a
later method of the same object) oracle 30 `m2 fresh: 1`, ours 30 `0`; `b.rex` (main's reset
reaches a method) oracle 30 `1`, ours 30 `0`; `d.rex` (a method's reset reaches its caller) oracle
30 `caller untouched: 1`, ours 30 `0`; `e.rex` (a started method) 30 `1` on both. So `TIME`
TEST_VALIDOPT_BIGCHAR_R and TEST_VALIDOPT_LITTLECHAR_R pass alone and fail in a whole-group run
after earlier tests (`whole-groups/table.txt`, their failing-test lists).

REPLY diagnosis (2026-10-07, `.superpowers/sdd/2026-10-01-phase-6-s2-s5/reply-rc-diagnosis.md`): a
method's first `TIME('E')` reads the caller's running clock, where the oracle starts the method
fresh. Probe, one run each, at 05aac0c58:

    say "main" time("e")
    call SysSleep 0.05
    o = .t~new
    say "main got" o~m
    say "main after" time("e")
    call SysSleep 0.3
    ::class t
    ::method m
      say "m before" time("e")
      call SysSleep 0.02
      say "m before2" time("e")
      reply 7
      call SysSleep 0.01
      say "m after" time("e")

Oracle `m before 0`, `m before2 0.020647`, `main after 0.071721`, `m after 0.031880`; ours
`m before 0.050234`, `m before2 0.070365`, `main after 0.070484`, `m after 0`. The `m after 0`
half was the REPLY continuation starting a fresh clock, fixed under ruling P89. The rest is this
item's per-activity clock: in the oracle it is `ActivationSettings::elapsedTime`, which moves with
the activation. `RANDOM`'s `random_seed` also sits on `Activity` (`activity.rs:258`); P89 carries it
across a REPLY. The oracle seeds each activation from its activity (`RexxActivation.cpp:174`,
`:312`). Measured by the Phase 6 final review (`final-review.md` M2): main `random(1,1000,7)`, a
method `random(1,1000,11)`, main `random()`: oracle 517, ours 99. A method that never read the
clock, replying after main ran `time('R')`, reads `time('E')` 0.70 in its continuation where the
oracle reads 0 (3/3). Fixing the scope moves both fields to the activation, and P89's copy with them.
