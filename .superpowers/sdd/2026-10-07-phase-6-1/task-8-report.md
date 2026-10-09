# Task 8 report: the simulation mode (seed, streams, clock seam, inline pool)

Base `0765d19ef`. Commits `b7a050d6b` (the mode), `909d87b09` (a refused endless wait ends the
program's end), and the commit carrying this report and the gate record. Paths below are under
`rust/crates/rexx-exec/` unless they say otherwise; line numbers at `909d87b09`.

## Design

- `SwitchMode::Sim(SimConfig)` (`src/invocation.rs:98`), reached only through the existing
  switch-mode arm of `serve_requests` (`src/scheduler.rs:2025`), so the default mode has no new
  per-clause branch. `set_switch_mode` calls `start_sim` (`src/scheduler.rs:1983`,
  `src/sim.rs:395`), which boxes a `Sim` on `Interp::sim`, sets the pool's bound to 0, clears main's
  `RANDOM` generator and, under `gc=`, makes a collection due.
- `SimConfig::parse` (`src/sim.rs:100`) reads `sim` (seed from the clock, `clock_seed`
  `src/sim.rs:206`) or `sim:SEED[,fifo][,gc=Q][,halt@K][,fail=wait:K][,clock=midnight|real]`;
  `Display` prints the same text back. `Policy` has `Fifo` only (no preemption); Task 9 fills it.
- PRNG: splitmix64 seeding xoshiro256\*\*, in crate. `Streams` (`src/sim.rs:264`) splits
  `schedule, order, clock, gc, children` and a sixth, `random`, from consecutive splitmix64 outputs,
  so each stream's seed does not depend on how much another draws. `fifo` draws neither `schedule`
  nor `order`; they are split anyway so the later streams keep their seeds (an `#[expect(dead_code)]`
  says so).
- Clock (`Clock::new` `src/sim.rs:305`): the quantum (29 to 116 ns, `QUANTUM_NANOS`
  `src/sim.rs:35`) and the origin are drawn from the clock stream, always in the same order whatever
  the `clock=` knob is. Seeded origin: 2026-01-01T00:00:00Z plus an offset under a year.
  `clock=midnight`: 0.5 to 5 s before the local midnight ending the seeded day (local offset from
  `datetime::local_offset_micros`). `clock=real`: the wall clock at start. Virtual instants are a
  real `Instant` taken at start plus the virtual elapsed time; the wall reading is the origin plus
  the same elapsed time. `sim_clause` (`src/sim.rs:417`) adds a quantum at each boundary;
  `sim_idle_until` (`src/sim.rs:440`) jumps to the deadline, after checking the run's deadline on
  real time.
- Seam: `Interp::now` and `Interp::wall_now` (`src/sim.rs:375`, `:386`).
- Knobs: `gc=q` draws at the allocation where `collect_if_due` found a collection due
  (`src/lib.rs:2726`); with the knob on, `collect_now` leaves the next allocation due
  (`src/lib.rs:3036`), and the draw declines only where neither the slot nor the byte trigger would
  collect (`sim_declines_collection` `src/sim.rs:489`). The default allocation path is unchanged:
  the call is inside the taken branch. `halt@K` calls `halt_all` at boundary K, as a signal's halt
  does. `fail=wait:K` fails the K-th pinned wait with 11.1 after it parked, withdrawing it with
  `cancel_wait` (`src/scheduler.rs:1023`).
- Children: `run_command` draws a seed from the children stream and sets the child's
  `REXX_SWITCH_MODE` to the parent's config with that seed and without `halt@`/`fail=`
  (`src/command.rs:713`, `SimConfig::child`).
- Refusals: `sim_screen_posts` (`src/sim.rs:513`, called from `file_completions`
  `src/scheduler.rs:1263`) records any post other than `Completed` and `Halt`; under bound 0 a
  `Completed` is only ever posted by the baton thread itself, inline. The breach is refused at the
  next boundary, at a sim idle and at the run's end (`src/lib.rs:3502`) as `Loud::sim_foreign_post`.
  `idle_for_good` in sim refuses `Loud::sim_endless_wait` (`src/scheduler.rs:1499`): a wait only a
  real signal can end. `idle_for_posts` in sim is `scheduler_inconsistency` (`src/clause.rs:354`):
  with bound 0 every completion is posted before it is waited for. Both mark the run stuck, which
  stops the program-end loop (`src/scheduler.rs:1432`).
- Report: `Outcome::sim: Option<SimReport>` (`src/lib.rs:3546`): seed, policy, steps (the switch
  mode's clause count), switches (a counter on the activity table incremented in `switch_to`,
  `src/scheduler.rs:974`, unconditionally, so default mode pays an add and no branch), `trace_hash:
  None`. `rexx-run` prints `rexx-sim: seed=S policy=P steps=N switches=M profile=X stack=Z` at exit
  (`src/bin/rexx-run.rs:129`); `trace=H` is printed when `trace_hash` is `Some`. `sim` alone also
  prints `rexx-sim: seed=S` at start (`:69`), so a run that never ends has its seed.
- The timer arms no slice in sim: the switch-mode arm never arms it and `set_switch_mode` disarms
  it; `timer.rs` needed no change. A test pins it.

## Changes

`src/sim.rs` (new) and `src/sim/tests.rs` (new); `src/invocation.rs` (the variant; `SwitchMode`
loses `Eq` for the `f64` knob); `src/lib.rs` (field, exports, `Outcome::sim`, gc hook, end-of-run
breach); `src/scheduler.rs` (sim arm, `start_sim` call, `post_timer`/`wake_due_sleepers`/
`idle_for_good` on the seam, `fail=wait`, screen, switch counter, `switch_clauses`, the stuck stop);
`src/clause.rs` (sim idles, `deadline_passed`, `expire_deadline` made `pub(crate)`);
`src/scheduler/pool.rs` (`set_bound`, `reserve` answers `None` at bound 0, a test spawn counter);
`src/semaphores.rs` (`retest_semaphore`), `src/builtin/rexxutil.rs` (`SysSleep`, `SysSemWait`
poll), `src/dispatch/semaphore.rs` (`semaphore_wait` takes the interpreter),
`src/dispatch/time_support.rs` (`timed_wait`), `src/builtin/datetime.rs` (`real_clock_base_time`),
`src/builtin/numeric.rs` (`initial_seed`), `src/run.rs` (the scripted park's deadline) on the
seam; `src/command.rs` (child mode); `src/bin/rexx-run.rs`; `rexx-core/src/heap.rs`
(`bytes_since`); `rust/clippy.toml` (new); `#[allow]`s listed below; `sim: None` in the test
harnesses' `Outcome` literals; `rust/corpus/refusal-dispositions.tsv`, `refusal-sites.tsv`.

## Clock-read allow-list

`grep -rn -E '\.elapsed\(\)|thread::sleep|Instant::now|SystemTime::now' src --include=*.rs` at
`909d87b09`, sorted. `rust/clippy.toml` disallows `Instant::now`, `SystemTime::now`,
`chrono::Local::now` and `chrono::Utc::now`; `.elapsed()` and `thread::sleep` are not disallowed
and appear only where listed.

Routed through the seam (no longer printed): `scheduler.rs` `post_timer`, `wake_due_sleepers`,
`idle_for_good`; `semaphores.rs` `retest_semaphore`; `builtin/rexxutil.rs` `SysSleep` and the
`SysSemWait` poll; `dispatch/semaphore.rs` `semaphore_wait`; `dispatch/time_support.rs`
`timed_wait`; `builtin/datetime.rs` `real_clock_base_time`; `builtin/numeric.rs` `initial_seed`;
`run.rs` the `cfg(test)` scripted park (routed rather than allowed).

Allowed, each with its `reason`:

| site | reason |
|---|---|
| `sim.rs:208` `clock_seed` | the seed of `sim` alone |
| `sim.rs:323` | `clock=real`'s origin |
| `sim.rs:332` | virtual instants count from a real `Instant` |
| `sim.rs:377`, `:388` | the seam's default arms |
| `clause.rs:37` `Deadline::starting_now`, `:309` `countdown_reached`, `:336` `idle_until`, `:376` `deadline_passed` | the run's deadline, real in every mode |
| `sync.rs:197` `now` | the timer thread's clock, which sim does not arm |
| `sync.rs:220` | the loom model clock (`cfg(all(loom, test))`; clippy does not compile it) |
| `sync.rs:239`-`:261` | `sync` tests time real waits (allow on the module) |
| `scheduler/tests.rs`, `scheduler/tests/pool.rs`, `sim/tests.rs`, `dispatch/time_support/tests.rs` | tests time real runs (allow on each `mod tests;`) |
| `signal.rs:238`, `:270` | `thread::sleep` in `signal.rs`'s `cfg(test)` module; not a clock read, not disallowed |
| `scheduler/pool.rs:43` | `thread::sleep` under `cfg(test)` on a pool thread, which sim does not run |
| `scheduler/tests/callbacks.rs:53` | `thread::sleep` in a test native |

Outside `src`: `rexx-api/src/load.rs` `await_buffer_changed` (a test native's own bound),
`rexx-bench/src/timing.rs` (a benchmark times real runs), and each `tests/*.rs` harness that reads
the clock (this harness times or bounds real runs), as crate-level `#![allow]`.

## Tests

Failing first: `src/sim/tests.rs`'s three Step 1 tests did not build before the mode existed
(`E0432 unresolved import crate::SimConfig`, `E0599 no variant ... Sim`;
`/tmp/claude-1000/p61/t8/logs/step1-red.txt`). Then, all passing at `909d87b09`:

- `a_timed_program_runs_on_virtual_time_and_prints_what_it_prints_on_real_time`: `SysSleep 5`,
  `time('E')`, an `.Alarm` ringing a class method, `EventSemaphore~wait(1)` timing out. Under
  `sim:1` under a second of wall time; with no switch mode (8 s) the same four lines. The oracle
  prints `slept 1 1`, `rang 1`, `waited 0`, `elapsed 1 1` in 8.01 s (probe
  `/tmp/claude-1000/p61/t8/probe1/s1.rex`).
- `one_seed_gives_one_output`, `two_seeds_draw_different_randoms`: unseeded `RANDOM`, `TIME('L')`,
  `DATE('S')` in main and a started activity. Read through `rexx-run` in separate processes:
  `sim:1` twice gives `780 691 664166` / `07:01:26.808042 20260615 0` / `707 07:01:26.808042`;
  `sim:2` gives `906 819 48982`.
- `a_config_reads_back_what_it_prints`, `the_generators_give_their_reference_outputs` (splitmix64
  from 0 is `0xE220A8397B1DCDAF`; xoshiro256\*\* from `[1, 2, 3, 4]` gives 11520, 0, 1509978240,
  1215971899390074240; the third was worked by hand).
- `a_wait_only_a_signal_can_end_is_refused`, paired with the default mode reaching the run's
  deadline. This test found the program-end loop defect fixed in `909d87b09` (OOM at a 2G cap
  before the fix, 0.33 s after).
- `halt_at_k_halts_at_its_boundary` (`halt@50` prints `halted 48` for seeds 1 and 9, `halt@60`
  `halted 58`, none `done`), `fail_wait_k_fails_the_kth_pinned_wait` (`fail=wait:1` traps 11.1,
  `fail=wait:2` and no knob sort), `gc_q_collects_at_random_allocations_the_seed_chooses` (0
  collections without the knob and at `gc=0`, some at `gc=0.01`, equal across two runs),
  `a_knob_leaves_the_other_streams_draws` (`gc=0.5` on and off print the same `RANDOM` and `TIME`),
  `a_child_gets_a_seed_from_the_children_stream` (two `echo $REXX_SWITCH_MODE` print
  `sim:7735830288798199505,fifo,gc=0.5` and `sim:7407052850021197627,fifo,gc=0.5` under
  `sim:1,gc=0.5,halt@1000`), `clock_midnight_crosses_a_day_boundary` (86396 s, then the next day
  after `SysSleep 6`), `a_post_from_another_thread_is_refused` (an `Output` posted from another
  thread is refused at the next boundary; a `Halt` is not).
- `scheduler/tests/native.rs`: `the_timer_arms_no_slice_in_the_simulation_mode` (a call in flight
  arms the timer in default mode, `the_timer_is_armed_for_a_call_in_flight`, and not in sim);
  `a_native_call_in_the_simulation_mode_spawns_no_pool_thread` (the same two native calls: under
  `every` the pool spawns threads, under sim none; both print `5`, `7`).

## LIMIT rows

`rust/corpus/refusal-dispositions.tsv`:

- `sim_endless_wait`: spec section 4, a deadlock in the simulation mode is a loud refusal.
- `sim_foreign_post`: spec section 4, any inbox post other than a signal's halt is a determinism
  breach.

`scheduler_inconsistency` keeps its GUARD row; its new sim site is unreachable for the reason in
Design. `refusal-sites.tsv` re-derived with `REXX_REFUSAL_SITES_REFRESH=1`.

## Performance

`docs/superpowers/plans/phase-6-1-gate.md` `## Task 8`. Callgrind against `0765d19ef`: rexxcps
+0.0001%, emptyloop +0.0006%, startup +0.0004%. Wall clock at load 1.0: rexxcps +1.32%,
emptyloop -0.45%, startup -4.00% (one millisecond of 25). The oracle's rexxcps median is 17,229,378
clauses per second, 58 ns per clause, recorded there with the quantum range.

## Commands and results

At `909d87b09`: `cargo fmt --all --check` exit 0; `memcap 8G cargo clippy -j 4 --workspace
--all-targets -- -D warnings` exit 0; `memcap 8G cargo test -j 4 --workspace --no-fail-fast` exit 0,
3094 passed, 0 failed, 4 ignored (`/tmp/claude-1000/p61/t8/logs/ws2.txt`); `REXX_CORPUS_GATE=1
memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test ir_recorded_oracle` exit 0, corpus 29
passed 1 ignored, ir_recorded_oracle 21 passed; `refusal_dispositions`, `refusal_sites`,
`closed_phases`, `loud`, `no_process_state` pass. The first workspace run, at `b7a050d6b`, was
OOM-killed at the 8G cap in the lib tests: the defect `909d87b09` fixes.

## Concerns

1. **A native that waits for its own thread's callback deadlocks in sim.** With bound 0 the call
   runs on the baton thread; a native that spawns a thread, has it call back, and joins it (the
   test library's `SENDFROMANOTHERTHREAD`) blocks the baton thread inside native code while the
   other thread waits for the baton. Nothing on the baton thread runs, so neither the refusal nor
   the run's deadline fires; only an outer timeout ends it. Measured on the same inline path in
   default mode with the pool's bound set to 0 (a scratch test over `run_shaped`, not committed):
   `SENDFROMANOTHERTHREAD` under a 3 s run deadline had not answered after 10 s. Default mode
   takes that path too whenever the pool has no thread to give. A post from another thread that
   the baton thread drains is refused (`sim_foreign_post`, `a_post_from_another_thread_is_refused`);
   no end-to-end witness through a native was built. A
   command whose child waits on another activity is the same shape as the join (not run). The spec's refusal for "a wait for a post only a native's own thread or a command's
   peer can deliver" is therefore not reachable as a scheduler wait.
2. **Named semaphores and rxapi** are refused in every mode today (`Loud::named_semaphore` and the
   Phase 10 rows), so no sim-specific refusal was added: one would be unreachable. Phase 10's
   implementation is where sim needs its own refusal.
3. `Streams` has a sixth stream, `random`, beyond the brief's interface, for each activity's
   `RANDOM` starting state; it comes last so the five named ones keep their seeds.
4. The quantum is drawn once per run, not per clause.
5. `rexx-run` prints `trace=H` only when `trace_hash` is `Some`, which Task 9 fills.
6. The oracle measurement is on the RelWithDebInfo (`-O2`) build at `build/`, the only one here.

## Fix round 1

Commit `95df78c16`, on `task-8-review.md` and the ruling on I1.

- **I1 (a), a native's own thread.** `rexx_api::values::Baton` gains `refuses_foreign` (default
  `false`); `ffi.rs`'s `reached` answers a new `Addressed::Refused` for a callback from a thread
  running none of the interpreter's native calls when the requester refuses, and every member then
  does nothing, without taking the baton. `Requester::refuses_foreign`
  (`src/dispatch/library.rs`) refuses while the inbox's `SIM` request bit is set (by `start_sim`)
  and sets `FOREIGN` (`src/timer.rs`); `sim_breached` turns `FOREIGN` into `sim_foreign_post("a
  callback")` at the next boundary, idle or run end. So a native joining its calling-back thread
  returns and the run is refused, deterministically. Witness
  `scheduler::tests::callbacks::a_callback_from_another_thread_is_refused_in_the_simulation_mode`
  (`run_shaped`'s `Shape` gains `switch`), under a 20 s channel timeout: stdout `1` (the call's
  logical answer), rc 120, the refusal. Before the fix the same shape hung (the review's probe and
  this task's scratch probe).
- **I1 (b), a command's peer.** In sim, `run_command`'s two waits on the baton go through
  `collect_on_baton` (`src/command.rs`): the wait runs on a helper thread and the baton thread waits
  `block=` seconds of real time (default 2, `Knobs::block`); past that the child's process group is
  killed (sim children start in a group of their own, `process_group(0)`; `rustix`'s `process`
  feature, no new crate) and the clause raises `sim_blocked_command`, a new LIMIT row stating it is
  timing-dependent. Witnesses: the review's `p3/fifo.rex` through `rexx-run` under `sim:1`, rc 120
  in 2.14 s, no `cat` left; `sim::tests::a_command_only_another_activity_can_end_is_refused_after_its_bound`
  (default mode prints `hi`, `read done wrote`; `sim:1,block=0.5` refuses in 0.5 to 5 s).
  A native blocked inline on something other than a callback is not bounded: the baton thread is
  inside its code, which cannot be abandoned.
- **Minors.** The stream fields lost their forward-looking docs. `clippy.toml` (comment now says
  workspace-wide) also disallows `Instant::elapsed`, `SystemTime::elapsed` and `thread::sleep`; the
  new sites are `scheduler/pool.rs`'s `cfg(test)` sleep, `signal.rs`'s test module and
  `rexx-api/src/load.rs`'s test natives (one module-level exemption). Every exemption is now
  `#[expect]`, and clippy passes, so each fires. `file_completions` screens the inbox after a
  recall's re-drain. `tests/sim_processes.rs` runs `rexx-run` under `sim:1` in two processes
  (sleeps, `TIME`, `DATE`, `RANDOM` in main and two started activities) and compares stdout and
  stderr, and `sim:2` differs.
- **P1** is queued by the controller, not addressed here.
- Per-task check and perf: the gate record's `### Fix round 1`.
