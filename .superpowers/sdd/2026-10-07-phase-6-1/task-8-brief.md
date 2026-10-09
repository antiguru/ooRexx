### Task 8: The simulation mode: seed, streams, clock seam, inline pool

**Files:** new `rexx-exec/src/sim.rs` (PRNG and streams, `SimConfig`, the virtual clock),
`invocation.rs` (`SwitchMode::Sim` beside `:92-98`), `bin/rexx-run.rs` (`:62-74` parsing, the
`rexx-sim:` line), `lib.rs` (`Interp::now`/wall reading; pool bound 0 in sim at `:233-238`;
`gc=q` in the `stress_collect` branch of `collect_if_due` `:2645`), `clause.rs` (`idle_until` `:318-339`, `idle_for_posts`
`:342-350`), `scheduler.rs` (`:1164`, `:1398`, `:1482`), `semaphores.rs:223`,
`builtin/rexxutil.rs` (`:189`, `:353`), `dispatch/semaphore.rs:137`, `dispatch/time_support.rs:171`,
`builtin/datetime.rs:759-764`, `builtin/numeric.rs:577-582`, `scheduler/pool.rs` (`reserve`
`:111-119`), `timer.rs` (no slices armed in sim), `command.rs` (child seed), new `rust/clippy.toml`
(`disallowed_methods`), `Outcome`.

**Interfaces:** `SwitchMode::Sim(SimConfig)`; `SimConfig { seed: u64, policy: Policy, knobs: Knobs }`
parsed from `sim:SEED[,policy][,knob...]` (`sim` alone takes a seed from the clock and prints it);
`Policy` is filled by Task 9 (this task parses `fifo`, no preemption); `Knobs { gc: Option<f64>,
halt_at: Option<u64>, fail_wait: Option<u64>, clock: ClockOrigin }`; `struct Streams { schedule, order,
clock, gc, children: Rng }` derived from the seed by splitmix64; `Interp::now() -> Instant`-like
value and `Interp::wall_now()`; `Outcome` gains `sim: Option<SimReport { seed, policy, steps,
switches, trace_hash: Option<u64> }>`, the hash filled by Task 9.

- [ ] **Step 1:** Tests first, crate side: under `sim:1` a program that sleeps 5 s, reads `time('e')`,
      sets an Alarm and waits on an EventSemaphore with a timeout finishes in under a second of wall
      time and prints the same as without sim (values as predicates); two runs of `sim:1` give identical
      output including `random()` without a seed; `sim:1` and `sim:2` give different `random()`.
- [ ] **Step 2:** The PRNG (splitmix64 seeding xoshiro256\*\*) and streams; the virtual clock: origin a
      fixed epoch plus a seeded offset (`clock=midnight` seconds before a day boundary; `clock=real`
      the wall clock at start), a seeded per-clause quantum whose range is sized from the oracle's
      measured per-clause time (measure `rexxcps` on the oracle, record the figure and the range in
      the gate record), and a jump to the next deadline when every activity waits.
- [ ] **Step 3:** Route every clock read through the seam. Derive the allow-list with
      `grep -rn -E '\.elapsed\(\)|thread::sleep|Instant::now|SystemTime::now' src --include=*.rs`
      and sort what it prints: the seam replaces the reads in `builtin/numeric.rs:578`,
      `builtin/datetime.rs:760` and the scheduler, semaphore, rexxutil and time-support sites; each
      remaining site gets an `#[allow]` with its reason (the seam itself, the run deadline in
      `clause.rs`, the loom clock `sync.rs:189-217` and its tests `:233-255`, `run.rs:573` under
      `cfg(test)`, threads sim does not run: `signal.rs`, `scheduler/pool.rs:43`).
- [ ] **Step 4:** Pool bound 0 in sim (every native call, command and stdin read on the baton); the
      timer arms no slices in sim; the `Activity::random_source` generator's seed (Task 1) and child `rexx`
      seeds come from their streams; `gc=q` inside the `stress_collect` branch of `collect_if_due`; `halt@K`; `fail=wait:K`. Loud
      refusals in sim (named semaphores, rxapi, a wait for a post only a native's own thread or a
      command's peer can deliver; any inbox post other than `Posted::Halt`), each with a LIMIT row in
      `refusal-dispositions.tsv`.
- [ ] **Step 5:** `rexx-run` prints `rexx-sim: seed=S policy=P steps=N switches=M trace=H profile=X
      stack=Z` to stderr at exit. The corpus run in default mode is byte-identical (Review Focus 4);
      the per-task check. Commit.

