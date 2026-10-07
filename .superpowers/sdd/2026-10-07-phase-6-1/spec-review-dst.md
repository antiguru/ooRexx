# Spec review, lens `dst`

Spec: `docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md` (draft), section 4, D7, O3-O7, exit criterion 6.
Tree `be19fd06a`. Paths under `rust/crates/rexx-exec/` unless they say otherwise; ootest paths under
`ootest/ooRexx/base/`. No runs were made; every finding cites code or a report line.

## BLOCKER

### B1. Nothing shows the seeded gate can go red on a real bug

- Spec: exit criterion 6, "the seeded gate is green at tier 1 under the committed seed list ... every mutant of
  section 4 is judged with a run".
- Wrong: criterion 6 is satisfiable by a judge that cannot fail. The mutants are judged by crate tests (M11's
  witness is a `run_program_on_stack` crate test, scout C section 4), not by the gate. M11's own observable is
  `result The NIL object after 0`, rc 0 (scout C section 4): a value change at rc 0, which tier 1 sees only if an
  ooTest assertion covers it. The gate's tier 1 is crash, refusal, hang, rc, and ooTest failure counts; nobody has
  checked that any known concurrency defect lands in it.
- Evidence: scout C report section 3 (tier 1 list) and section 4 (M11 output); spec section 4 "Gate".
- Fix: add to criterion 6 a gate-level kill check. Re-introduce, one at a time, defects the gate exists for and
  require a tier 1 red from the seeded gate itself (not a crate test) within the committed seeds: at least M11
  under `fail=wait:K` in a group-shaped program, and two concurrency fixes from the Phase 6 S2-S5 ledger reverted.
  Also try the origin failure (queued item: REPLY under every opportunity, rc 1, release only,
  `bg/b6efbfd53/logs/g4-test-release.txt:2210-2213`): add `uniform:1` (every opportunity with the virtual clock) and
  record whether any seed reproduces it. A gate that kills none of these is recorded as such, not as green.

### B2. `pct:d` has no defined k, so it is either unimplementable or not replayable

- Spec: "`pct:d` places d preemption points uniformly over the run's clause steps".
- Wrong: the run's step count is known only at its end. Scout C's design had `k=N` or "the previous run's step
  count" (scout C section 2); the spec dropped both. Taking k from a previous run makes the schedule a function of
  run history, outside commit + seed + policy, which breaks the replay key the spec states ("commit, seed and
  policy replay a run").
- Evidence: spec lines 106-108; scout C report section 2, `pct:d[,k=N]` bullet.
- Fix: k is part of the policy string and the replay line (`pct:d,k=N`), with N per group committed beside the
  seed list and derived by a committed calibration command; or replace fixed points with a per-step probability
  d/k_est over contended steps (B2 and I1 share this fix).

### B3. Legal schedules turn tier 1 red, and the spec has no exemption mechanism for sim

- Spec: "Tier 1 fails the gate: ... a loud scheduler refusal ... a hang, a nonzero ooTest failure or error count
  where no oracle run had one, an rc no oracle run had."
- Wrong: the whole-groups harness already carries three per-mode allowances that do not transfer to a seeded mode,
  and the spec says nothing about them:
  1. `EVERY_FORCES_THE_RACE` (`tests/concurrency_tests.rs:2098-2105`, ruling P46): MutexSemaphore TEST_EXCLUSION
     deadlocks at the test's own race, and the oracle deadlocks in the same interleaving (3 of 3, rc 137). Any
     policy that preempts in that window reaches it; under sim a full deadlock is a loud refusal (spec section 4
     table, "refused loudly"; scout C section 2, "idle_for_posts with nothing that can post is refused loud").
     That is a tier 1 red on a schedule the oracle can take.
  2. `WALL_CLOCK` (`:2056-2097`) with the P48 rerun (`:3334-3355`). Under sim a rerun with the same seed is the
     same run, so P48 is void. Some of those rows compare the virtual clock with a real one: STREAM
     TEST_QUERYDIR_EXISTS takes `.DateTime~new` and asserts it equals the new directory's `query datetime`
     (`bif/STREAM.testGroup:864-876`). The spec makes `TIME`/`DATE` virtual and the file system real (section 4
     table), so once any earlier clause or sleep has advanced virtual time this assertion fails on every seed.
  3. `DIFFERING` (`:2475` on) is keyed per (group, part, mode in normal/every) with an exact failing-test list;
     no `sim` mode rows exist, and a failing list that varies with the seed cannot match a fixed key.
- Without a mechanism, criterion 6 is met only by choosing seeds that miss these, which the committed seed list
  invites.
- Fix: (a) seeds are derived by rule (`seed_i = splitmix(hash(group, part), i)`), never hand-listed; (b) a
  committed `SIM_EXEMPT` table of (group, test, reason, evidence), renamed out of sim runs the way rxapi tests are
  renamed out today (`:2390-2392`), each entry citing an oracle run or the virtual/real clock split; (c) state what
  `DIFFERING` means under sim (recommend: compare with failing tests from `SIM_EXEMPT` removed); (d) a deadlock in
  a test whose oracle twin is recorded as deadlocking is an exempt row, not a scheduler defect.

## IMPORTANT

### I1. "PCT's depth bound applied to preemption points" claims a guarantee it does not have

- Spec: "`pct:d` places d preemption points ... PCT's depth bound applied to preemption points; priorities do not
  reorder the ready queue."
- Wrong: PCT (Burckhardt et al., ASPLOS 2010) gets its bound, probability at least 1/(n k^(d-1)) of hitting a
  given depth-d bug, from random initial priorities over n threads plus d-1 change points; the scheduler always
  runs the highest-priority enabled thread. With FIFO and no priorities, a preemption lets only the ready queue's
  head run. For two activities that is close to a priority swap; for three or more, an ordering constraint like
  "C before B" may need a reorder that no placement of points produces, so no bound holds. The count also differs
  (d points vs PCT's d-1 change points). Further, a point placed on a step with an empty ready queue is discarded
  (`scheduler.rs:2013-2016` clears the slice and returns), and ooTest groups run tests sequentially with short
  concurrent windows, so most uniformly placed points over all clause steps land where nothing is ready. With
  k in the 10^5-10^6 range for a whole group and d=3, 16 seeds per group explore close to nothing.
- Fix: name it "random d-bounded preemption" and drop the PCT guarantee claim; count steps only where `ready` is
  non-empty and the clause `yields` (the contended steps), which is the standard refinement and also shrinks k;
  include d=1 and d=2 in the mix (O6).

### I2. Without background slicing, pct produces schedules the oracle cannot take

- Spec: `pct:d` places d points; nothing else preempts. Switch modes disarm the timer (`scheduler.rs:1957-1961`).
- Wrong: the oracle slices every 24 ms whenever another activity waits (`timer.rs:36`, `SLICE_LENGTH`). After the
  d-th point a pct run lets one activity run unboundedly while others are ready. That is outside the oracle's
  legal set, so outcomes from it should not be judged as divergences, and any program that polls a variable
  another activity sets spins until the run deadline: a tier 1 hang on a correct interpreter.
- Fix: every policy keeps a fairness floor: a forced preemption after F contended clauses (F a policy parameter,
  default sized to the oracle's 24 ms at its measured clause rate). Record the floor in the replay line.

### I3. One PRNG stream couples every knob to the schedule

- Spec: "One in-crate PRNG ... Every draw comes from it in program order".
- Wrong: the per-clause clock quantum, `gc=q` (one draw per allocation), shuffle picks and child seeds all advance
  the same stream as the preemption decisions. Turning `gc=q` on or off, or any change in allocation count,
  re-rolls the whole schedule. That defeats diagnosis ("same schedule, GC off") and any shrinking.
- Fix: derive one independent stream per decision kind from the seed (schedule, clock, gc, order, children).

### I4. Clock origin and quantum are unspecified, and both are semantics

- Spec: "virtual time advances a seeded quantum per clause and jumps to the next deadline when every activity
  waits". Origin is not stated (scout C section 2 says "real wall clock at start").
- Wrong: (a) a real origin makes `TIME`/`DATE` output differ between replays of one seed, and between the
  determinism self-test's two runs when they straddle a second; the midnight rows of `WALL_CLOCK` then depend on
  when the gate runs. (b) The quantum's distribution decides which timing-assuming tests pass: GUARD test_off
  sleeps 0.015 s and asserts the waiter has already run (`keyword/GUARD.testGroup:137-142`); EventSemaphore
  TEST_WAIT_CONCURRENT relies on SysSleep 0.1 and 0.01 (`class/EventSemaphore.testGroup:149-154`). A large
  quantum reds them; a tiny one makes `time('e')` busy waits take millions of clauses.
- Fix: origin derived from the seed (fixed epoch plus a seeded offset; `clock=midnight` as the knob scout C
  proposed; `clock=real` opt-in), and the quantum's range stated in the spec, sized from the oracle's measured
  per-clause time. Name the file-system stamp split (B3.2) as a known limit.

### I5. The replay key is incomplete

- Spec: "commit, seed and policy replay a run"; "A failing seed prints its replay line."
- Wrong: other inputs change the run: build profile (the origin failure was release only), interpreter stack size
  (stack-bound 11.1 conditions decide M9-M11's paths; scout C reached 11.1 at depth 5698 on a 64 MiB test stack and
  at the activation limit 5000 on the CLI's 512 MiB, scout C section 4), the entry point (in-process
  `run_crate_within` with an explicit environment, `tests/support/group_runner.rs:425-476`, vs `rexx-run` with the
  process environment), the renamed-out test set, stdin bytes, `TZ`, and the fresh copy's contents.
- Fix: the replay line is a harness command (test filter plus `REXX_SIM_ONLY=group:part:seed:policy`) that runs
  the same entry, stack and environment; the report line prints profile and stack size.

### I6. No shrinking, and no path from a found bug to a regression test that survives commits

- Spec: section 4 says seeds replay only per commit and stops there.
- Wrong: a failing seed is the only artifact. On the next commit the same seed takes another schedule, so a seed
  kept as a regression test goes green without testing anything. The spec has no minimiser.
- Fix: (a) the run records its decision trace (the contended-step indices where it preempted, each shuffle pick,
  each gc point); (b) `sim:trace=FILE` replays a trace; (c) a shrinker deletes trace entries while tier 1 stays red
  (delta debugging); (d) a found bug lands as a crate test with explicit switch points (the `AtClause` style),
  plus an assertion that the test reaches the defect's site, so a later commit that moves the schedule fails loud
  rather than passing.

### I7. Tier 1 depends on the sampled oracle set and on oracle runs inside the gate

- Spec: "an outcome outside it triggers extra oracle runs before it is judged"; tier 1 includes "an rc no oracle
  run had" and failure counts "where no oracle run had one"; the set "only grows".
- Wrong: the same seed on the same commit can get different verdicts depending on what the extra oracle runs hit,
  the argument the spec itself uses against byte membership in O5. A committed set that grows during gate runs
  also mutates tracked files from a gate, and an oracle outcome from a bad run (crash, a machine-specific leak)
  stays forever.
- Fix: the gate judges against the committed set frozen at the commit; extra oracle sampling is a separate
  command whose output is reviewed and committed. Each committed outcome records its run count and machine.

### I8. The determinism self-test is too weak, and there is no structural guard

- Spec: "A determinism self-test runs every group twice under one seed and requires identical outcomes and switch
  counts."
- Wrong: equal switch counts do not imply equal schedules, and two runs in one process share process state (the
  timer `REGISTRY`, `timer.rs:267`; signal statics, `signal.rs:36-40`). Hidden inputs are also detectable at their
  source: under bound 0 the only legitimate inbox poster is a real signal's halt, but the pool, stdin reader,
  output lender and native library recalls all post (`scheduler.rs:668-687`, `input.rs:380`,
  `dispatch/library.rs:1190`, `:1230`).
- Fix: compare a hash of the full decision trace (I6) plus outcome; run one of the two copies in a fresh process;
  in sim, any inbox post other than `Posted::Halt` is a loud determinism breach; forbid `Instant::now` and
  `SystemTime::now` outside the clock seam with clippy `disallowed_methods` (today's sites: `scheduler.rs:1164`,
  `:1398`, `:1482`, `semaphores.rs:223`, `builtin/rexxutil.rs:189`, `:353`, `dispatch/semaphore.rs:137`,
  `dispatch/time_support.rs:171`, `builtin/datetime.rs:760`, `builtin/numeric.rs:578`; the deadline reads in
  `clause.rs` are the allowed exception).

### I9. FIFO default leaves legal orders unexplored (O3)

- Spec: "FIFO, the oracle's order; `order=shuffle` picks among activities one event readied".
- Wrong: when one post or release wakes several waiters, the oracle's order is the order its OS threads reach the
  kernel lock, which varies. FIFO-only makes those legal orders unreachable by default, including in the gate.
- Fix: one-event shuffle on by default (legal); full reorder stays off.

## MINOR

- M1. `gc=q` sits on `alloc_with` (`lib.rs:2633-2640`), the hottest path; section 5 does not name it. Fold it into
  the existing `stress_collect` test (`lib.rs:2645`) so the default path keeps one branch.
- M2. The spec's table drops scout C's refusal of "waiting for a post ... a native's own thread calling back"
  (scout C section 2). Restore it; native library recalls (`dispatch/library.rs:1190`) are real-time input.
- M3. `SysFileTree` lists `read_dir` unsorted (`builtin/rexxutil.rs:766-770`); the order is the file system's,
  which can differ across machines. Name it in the "stays real" row.
- M4. Bound 0 never runs P52 lending, where the Phase 6 races were (scout C risks). Say in criterion 6 that sim is
  not evidence for P52, so a green gate is not read as covering it.
- M5. Under bound 0 a command that waits on something another activity produces (a pipe, a file) deadlocks. A
  loud refusal is right; list it in the refusal row.
- M6. M11's witness scans recursion depth for about 25 s on a 64 MiB stack (scout C decision 10). Once
  `fail=wait:K` exists, rewrite it to use the knob and assert the `cancel_wait` sleeper branch is reached.
- M7. Tier 2 has no end state. Add: every tier 2 outcome at the close is either added to the oracle set by a
  sampling run or recorded with a reason.

## OPEN QUESTIONS

| # | Question | Recommendation |
|---|---|---|
| O3 | (differ) Ready order default | One-event shuffle on by default (I9); FIFO as a knob; no full reorder. |
| O4 | (agree, with a condition) Pool inline | Inline first, with M4's sentence in criterion 6. |
| O5 | (agree in shape, differ in content) Judge | Tier 1 fails, but tier 1 is judged against a frozen committed set (I7), with a `SIM_EXEMPT` table (B3) and a gate-level kill check (B1). |
| O6 | (differ) Seeds and mix | Seeds derived by rule, not listed (B3a). Mix over contended steps: d in {1, 2, 3}, `uniform` at 0.01 and 0.2, plus `uniform:1`. Size the count from a timed run of the whole list; a separate exploration command with large N runs outside the gate. |
| O7 | (agree) `fail=wait:K` in the binary | Yes, sim only, refused in any harness run compared with the oracle. |
| O8 | (new) Clock origin | Seeded fixed epoch, `clock=real` opt-in (I4). |
| O9 | (new) Who rules a `SIM_EXEMPT` row | Same as `DIFFERING` today: a row needs an oracle run or the virtual/real clock argument, and Moritz rules the first batch at T8. |
| O10 | (new) Trace replay and shrinker in 6.1 | In 6.1, in T7 (trace record and replay) and T8 (shrinker, regression conversion); without them a found bug has no durable test (I6). |
