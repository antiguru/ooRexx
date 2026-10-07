# Deterministic simulation testing of the scheduler (Phase 6.1)

Moritz, 2026-10-07: fuzz the scheduler instead of testing one interleaving, under deterministic
simulation (DST), with PCT as the schedule policy.

`SwitchMode` (`rexx-exec/src/invocation.rs:92`) has `AtClause(k)` (one switch) and
`EveryOpportunity` (round-robin at every boundary). Each explores one schedule per run; the oracle's
outcomes vary across schedules (P83, P86).

## Proposal

- A simulation mode where every source of nondeterminism comes from one seed, printed on every run
  and replayed by `REXX_SWITCH_MODE=sim:SEED`:
  - schedule: at each clause boundary the next activity is chosen by the policy (uniform random
    with probability `p`, or PCT: random priorities and `d` priority change points);
  - clock: virtual, advanced to the next deadline when every activity waits (`SysSleep`,
    `time("e")`, Alarm, Ticker, GUARD WHEN timeouts), replacing the timer thread;
  - pool: native calls that are lent the baton (P52) run inline;
  - signals and stdin: scripted from the seed; rxapi and the file system refused or stubbed.
- In-crate PRNG (no dependency).
- Gate: each whole group of criterion 1 under N seeds; every outcome is an oracle outcome or a
  recorded divergence (the whole-groups agreement rule, per seed). A failing seed is the bug report.

## Origin

The final Phase 6 gate (`bg/b6efbfd53/logs/g4-test-release.txt:2210-2213`): REPLY derived under
every opportunity passed every test with rc 1, once, in the release run only; wall-clock inputs made
it unreplayable.
