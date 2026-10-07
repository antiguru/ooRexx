# Scout C: design survey for deterministic simulation testing of the scheduler

Read `scout-common.md` beside this file first. Your name for paths: `sc`.

Items: `.superpowers/sdd/queued/2026-10-07-seeded-random-switch-mode.md` (Moritz's proposal: DST with PCT
schedules, the gate instrument for the concurrency groups) and `2026-10-07-unjudged-scheduler-mutants.md`.
Phase 6 spec: `docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md`. Constraints: no new
dependencies beyond loom and libc (in-crate PRNG); `unsafe` only in ffi.rs, load.rs, bytes.rs, frame.rs,
island.rs, signal.rs; no per-clause cost in the default mode (any conditional in the IR driver's
`Op::Clause` arm costs ~0.5% on rexxcps; the existing switch-mode check is the place to hang off).

Survey, citing file:line:
1. Every source of nondeterminism a Rexx program run can see in this crate: scheduling (`SwitchMode`,
   `invocation.rs:92`, `scheduler.rs`), the timer thread and the slice, wall clock reads (TIME, DATE,
   SysSleep, Alarm, Ticker, GUARD WHEN / semaphore timeouts, `time('e')`), the native-call pool (P52
   baton lending), signals (`signal.rs`), stdin, RANDOM's unseeded default, hash iteration order,
   addresses/identity hashes, the file system, environment. For each: where it is read, and the seam
   at which a simulation could supply it.
2. A design for `REXX_SWITCH_MODE=sim:SEED[,policy]`: one seed drives all of the above; PCT (random
   priorities, d change points, depth bound) and a uniform-random policy; a virtual clock advanced to
   the next deadline when every activity waits; native pool calls run inline; seed printed on every
   run. What can stay real (file system with a fresh dir) and what must be stubbed or refused.
3. Harness: how a gate runs each whole concurrency group of criterion 1 under N seeds and judges each
   outcome against the oracle's outcome set (look at how whole-groups and `REXX_CORPUS_GATE` judge
   today; `rust/crates/rexx-exec/tests/concurrency_tests.rs`, `group_runs`). How the oracle's outcome
   set is obtained (it cannot be seeded).
4. The unjudged mutants M6, M7, M9-M12 (`.superpowers/sdd/2026-10-01-phase-6-s2-s5/final-review.md`
   "Declined to judge"): for each, the schedule that would make it observable if any, so the
   simulation's first seeds can target it. A prototype is welcome but optional: if you build one in
   your worktree, report what it found, not just that it ran.
5. Cost: files touched, size estimate per part (S <50 lines, M <300, L more), and risks.

Write `.superpowers/sdd/2026-10-07-phase-6-1/scout-c-report.md`: the nondeterminism table, the design,
the harness, the mutant schedules, the cost table, open decisions for Moritz (each with your
recommendation).
