# Task 10 carry-ins (controller)

Decisions and interfaces from earlier tasks that `task-10-brief.md` cannot know.

* **Sim mode as built (Tasks 8, 9).** `REXX_SWITCH_MODE=sim:SEED[,policy][,knobs]`. Policies
  `pre:d,k=N`, `uniform:p`, `pct:d,k=N`, `order=fifo`; knobs `gc=q`, `halt@K`, `fail=wait:K`,
  `block=S` (0.001 to 86400 s, default 2 s), `floor=F`, `trace=FILE`, `replay=FILE`. The run's
  stderr carries `rexx-sim:` lines including the trace hash and the contended-step count
  (`contended=`): use that count as the part's k in Step 1. Sim refusals: `sim_blocked_command`,
  `sim_blocked_native`, `sim_foreign_post`, `sim_endless_wait`, and the invariant refusals
  (`scheduler_inconsistency` family). Replay refuses loudly on divergence since Task 9 fix round 1.
  Read `task-9-report.md` and `task-9-fix1-report.md` for the exact stderr formats.
* **Process-level timeout (Task 8 ruling).** The sim watchdog covers a blocked inline command or
  native, not every hang. Every gate run therefore has a process-level deadline, enforced by the
  harness (kill the child's process group on expiry) and reported as a hang red with the replay
  line. The deadline per part comes from Step 1's calibrated wall time, with a stated multiplier.
* **Memory.** The harness runs each child under the existing group_runner limits; the whole gate
  command runs under `memcap 8G`. An uncapped interpreter run once OOM-killed the whole session.
* **Ruling, SysSleep `TEST_SLEEP_DURATION`.** Under sim it fails on 4 of 5 policies because the
  virtual clock jumps to the sleep's deadline, so `time('e')` after `SysSleep(1e-8)` reads `0`; the
  test's lower bound needs at least 1 µs of measured time, which on a real system comes only from
  nanosleep's 51 to 110 µs overshoot (review of Task 9, check 6). It is a `sim-exempt.tsv` row with
  that evidence: the test asserts host sleep latency, not a property of the program, and the spec
  names no modelled sleep latency. Cost if wrong: one test unexercised under sim; a modelled
  overshoot can be added later without changing the gate's shape.
* **Reds while building the gate (Step 2).** The plan sends each red to Moritz case by case. In this
  run the controller rules on each instead (recorded in the ledger, and listed for Moritz at the
  phase close). So: for each red, stop and report it to the controller via your report file and a
  message (part, seed, policy, replay line, the failing line, and your evidence for defect versus
  test-environment assumption) before fixing or exempting it. Do not exempt on your own.
* **Revert checks (Step 5).** The ledger to grep is
  `.superpowers/sdd/2026-10-01-phase-6-s2-s5/progress.md`. Reverts happen in a scratch copy
  (`git archive` into `/tmp/claude-1000/p61/t10/`), never on the branch: no checkout, no reset,
  no stash in the shared tree.
* **Queued, not this task's:** `.superpowers/sdd/queued/2026-10-09-full-pool-inline-command-hang.md`
  (default-mode hang with a full native pool). If a gate part hits it, report it; do not fix it here.
