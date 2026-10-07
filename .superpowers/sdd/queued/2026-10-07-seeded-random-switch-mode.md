# Seeded random switch mode (Phase 6.1)

Moritz, 2026-10-07: fuzz the scheduler instead of testing one interleaving.

`SwitchMode` (`rexx-exec/src/invocation.rs:92`) has `AtClause(k)` (one switch) and
`EveryOpportunity` (round-robin at every boundary). Each explores one schedule per run; the oracle's
outcomes vary across schedules (P83, P86).

## Proposal

- `SwitchMode::Random { seed, p }`: at each clause boundary switch with probability `p` to a ready
  activity chosen at random. In-crate PRNG (no dependency). The seed is printed on every run;
  `REXX_SWITCH_MODE=random:SEED:P` replays it.
- Optionally PCT (random priorities, `d` priority change points) for depth-bounded coverage.
- Gate: each whole group of criterion 1 under N seeds; every outcome is an oracle outcome or a
  recorded divergence (the whole-groups agreement rule, per seed).

## Caveat

Replay is exact only without wall-clock inputs (timers, `SysSleep`, `time("e")`). A virtual clock in
this mode (advanced when every activity waits) makes it exact.

## Origin

The final Phase 6 gate (`bg/b6efbfd53/logs/g4-test-release.txt:2210-2213`): REPLY derived under
every opportunity passed every test with rc 1, once, in the release run only.
