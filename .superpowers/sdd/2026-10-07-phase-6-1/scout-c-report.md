# Scout C report: deterministic simulation testing of the scheduler

Tree: `be19fd06a`. Paths below are under `rust/crates/rexx-exec/` unless they say otherwise.
Prototype built in a throwaway worktree (removed); its commands and outputs are in section 4.

## 1. Nondeterminism a Rexx program can see

| Source | Where it is read | Seam for a simulation |
|---|---|---|
| Preemption point | `serve_requests` reads `SLICE` (`src/scheduler.rs:1969-2033`); the timer sets it every 24 ms (`src/timer.rs:36`, `:219-245`); in default mode the countdown visits only every 1024 clauses (`src/clause.rs:301-313`), so a timer `SLICE` can sit pending across a park | The existing switch-mode arm, `src/scheduler.rs:1990-1999`: a `SwitchMode::Sim` arm decides `due` from the policy. Reached only when a switch mode is set (`set_switch_mode` makes the countdown 1, `:1957-1961`), so the default path gains no branch |
| Which ready activity runs | `next_runnable`, `ready.pop_front()` (`src/scheduler.rs:1591-1630`); `ready` is FIFO, filled by `spawn` (`:497-511`), `unpark` (`:587-590`), `make_ready` (`:361-368`), `yield_at_slice` (`:592-595`), `post_timer` (`:1154-1210`), `wake_due_sleepers` (`:1397-1407`) | `next_runnable`'s pop: a policy pick in sim. One branch per switch in default mode, not per clause (measure it anyway) |
| Sleeper and timer deadlines | `SysSleep` `src/builtin/rexxutil.rs:189`; `SysSemWait` poll `:353`; semaphore `WAIT`/`ACQUIRE` timeouts `src/dispatch/semaphore.rs:137`, expiry `src/semaphores.rs:223`; Alarm/Ticker `src/dispatch/time_support.rs:171`; `post_timer` `src/scheduler.rs:1164`; `wake_due_sleepers` `:1398`; `idle_for_good` `:1482` | One `Interp::now()` replacing each `Instant::now()` listed; sim answers origin + virtual offset |
| Idling until a deadline | `idle_until` `src/clause.rs:318-339` blocks on the timer thread's wake (`src/timer.rs:374-387`); `idle_for_posts` `src/clause.rs:342-350` | In sim, `idle_until(due)` sets virtual time to `due` and returns; `idle_for_posts` with nothing that can post is refused loud |
| `TIME`, `DATE`, `time('e')`, `.DateTime` | `real_clock_base_time` `src/builtin/datetime.rs:759-764`, cached per clause by `now_base_time` `:770-788`; elapsed anchor `:857-863` | `real_clock_base_time` reads the same sim clock, in wall units |
| Local zone | `chrono::Local` `src/builtin/datetime.rs:825`, `src/dispatch/stream.rs:279` | Left real (process `TZ`) |
| GUARD WHEN timeouts | None exist: `ParkReason::GuardWhen` has no deadline (`src/scheduler.rs:106`) | Nothing to do |
| Native-call pool (P52) | `exit_for_native` `src/scheduler.rs:604-649` lends the baton to a pool thread; `exit_for_block` `:651-685`; stdin chunk reads `src/input.rs:364-400`; completions arrive through the inbox in real-time order (`file_completions` `:1247`) | Pool bound 0 in sim (`Pool::new` `src/lib.rs:233-238`, `Pool::reserve` `src/scheduler/pool.rs:111-119` answers `None`): every native call, command and stdin read runs on the baton, the existing fallbacks (`:639-648`, `src/command.rs:710-713`, `src/input.rs:371-373`) |
| Signals | Handler `src/signal.rs:84`; timer thread posts `Posted::Halt` (`src/timer.rs:418-427`); served at the next drain (`src/scheduler.rs:1278-1281`, `serve_halt_now` `:2236`) | Real signals stay real. A scripted halt (`halt@K`: `halt_all` at clause K) is the deterministic substitute |
| Standard input | `ProgramInput` `src/invocation.rs:117-127`; the harness supplies bytes or nothing | Already deterministic under the harness; the CLI's piped stdin is deterministic in content |
| `RANDOM` without a seed | `initial_seed` `src/builtin/numeric.rs:577-582` (clock nanos and pid) | Drawn from the sim PRNG |
| Hash iteration order | String keys hash by content (`hash_of`, `src/dispatch/hash.rs:549-553`); identity keys and `identityHash` use handle bits, slot and generation (`src/dispatch/object_protocol.rs:162-169`, `rexx-core/src/handle.rs:106`); interpreter maps use `NameHasher`/Fx, not `RandomState` (`src/lib.rs:1225`) | Deterministic given the allocation sequence, which is allocation-driven (`collect_if_due` `src/lib.rs:2644-2650`). The oracle's identity orders vary per run (addresses); the outcome set covers that |
| Collection timing | `collect_if_due` `src/lib.rs:2644` | Deterministic already; a sim knob `gc=q` collects at an allocation with probability q (licensed: GC timing is not an observable) |
| File system | `src/dispatch/files.rs:209`, `src/dispatch/stream.rs:274` (stamps) | Real, in the fresh copy the harness already makes (`fresh_copy`) |
| Environment | `Invocation::with_environment` `src/invocation.rs:171`; the harness passes an explicit one (`tests/support/group_runner.rs:439-455`) | Deterministic under the harness |
| Commands | Child processes, run on the baton when the pool has no thread (`src/command.rs:707-713`) | Real. A child `rexx` gets `REXX_SWITCH_MODE=sim:<seed drawn from the parent's PRNG>` |
| Run deadline | `src/clause.rs:36`, `:303`, `:326` | Stays real: it is the harness's bound, not program input |
| Loom model clock | `sync::now` `src/sync.rs:189-217` (cfg loom only) | Untouched; the sim clock is a separate seam on `Interp` |

## 2. Design: `REXX_SWITCH_MODE=sim:SEED[,policy][,knobs]`

- `SwitchMode::Sim(SimConfig)` beside `AtClause` and `EveryOpportunity` (`src/invocation.rs:92-98`), parsed in
  `src/bin/rexx-run.rs:62-74`. `sim` with no seed takes one from the clock. One in-crate PRNG (splitmix64 seeding a
  xoshiro256**), no dependency. Every draw below comes from it, in program order, so one seed replays the run on
  one commit. A seed is not portable across commits: any change to clause counts moves the schedule. The replay
  key is commit + seed + policy.
- Policies, both keeping the oracle's FIFO ready order (`ActivityManager`'s waiting queue):
  - `uniform:p`: at each clause boundary, preempt with probability p when `ready` is non-empty.
  - `pct:d[,k=N]`: d preemption points drawn uniformly over clause steps 1..k (k from `k=`, else from the
    previous run's step count, which the run report prints). This is PCT's depth bound applied to preemption
    points; PCT's priority reordering would break FIFO.
  - `order=shuffle` (opt-in): picks uniformly among the activities a single event readied (a post, a release),
    which is the OS freedom the oracle has; full random order is judged by tier 1 only (section 3).
- Clock: `Interp::now()` and a wall reading; real mode answers `Instant::now()`/`SystemTime`, sim answers origin +
  virtual offset. Virtual time advances by a seeded quantum per clause boundary (so `time('e')` busy-waits end)
  and jumps to the next deadline when every activity waits (`idle_until`). Origin: real wall clock at start; knob
  `clock=midnight` puts it seconds before a day boundary, which targets the `TIME.testGroup` midnight rows of
  `WALL_CLOCK` (`tests/concurrency_tests.rs:2056-2097`). Under sim the timer thread is never armed for slices and
  never waited on for deadlines; it stays registered for signals.
- Pool: bound 0, every native call inline (P52 lending is then not exercised; see decision 2).
- Knobs: `gc=q` (random collections), `halt@K` (scripted `halt_all`), `fail=wait:K` (the K-th pinned wait fails
  with 11.1, the generalisation of the test-only `fail_native_wait`, `src/scheduler.rs:990-995`). `fail=` diverges
  from the oracle by construction and is for crate-alone tests only.
- Report: `Outcome` gains the seed, policy, step count and switch count; `rexx-run` prints one `rexx-sim: ...`
  line to stderr at exit; the harness masks lines with that prefix.
- Refused in sim (loud, naming sim): waiting for a post with nothing in flight that this thread will deliver
  (a native's own thread calling back), named semaphores and anything reaching rxapi.
- Stays real: file system in a fresh directory, environment, `TZ`, child processes, the run deadline, real signals.

## 3. Harness

Today: `whole_groups::one_row` (`tests/concurrency_tests.rs:3357-3447`) runs each group part 5 times on the oracle
(`ORACLE_RUNS`, `:2418`), 30 when unsettled (`:2419`), and once per mode here (`n00`, `e00`, `:3377-3384`), in
process through `run_crate_within` (`tests/support/group_runner.rs:425-476`). `agree` compares masked stdout,
raw stderr and status (`:379-383`); `verdict` accepts any oracle outcome, P86 count-only differences, and the
`DIFFERING` rows (`:3312-3331`); `WALL_CLOCK` rows get one rerun (P48, `:3334-3355`). Gated by
`REXX_CORPUS_GATE` (`GATE_ENV`).

Proposed:
- `group_runner::SwitchMode::Sim { seed, policy }`; `run_crate_within` passes it to the `Invocation` and
  `REXX_SWITCH_MODE=sim:<derived>` to children (it strips the variable today, `:443`).
- A test `each_group_of_the_derived_list_under_seeds` beside `:3507`: each group of criterion 1, parts as now,
  under a committed seed list (`REXX_SIM_SEEDS` overrides the count, `REXX_SIM_POLICY` the policy). Virtual time
  makes the sleep-heavy groups fast.
- The oracle cannot be seeded, so its outcome set is sampled: the existing 5/30 runs, cached per group in a
  committed file and grown, never shrunk. A sim outcome outside it triggers a targeted batch of extra oracle runs
  before it is judged.
- Two tiers. Tier 1 fails the gate: a panic, a `scheduler_inconsistency` or other loud scheduler refusal, an
  inverted wait or a hang, a non-zero ooTest failure or error count where every oracle run had none, an rc no
  oracle run had. Tier 2 is reported, not failed: an outcome that passes tier 1 but is not byte-equal to a sampled
  oracle outcome. PCT reaches schedules the oracle's 24 ms timer almost never takes, so byte membership alone would
  gate on sampling luck.
- A failing seed prints its replay line: commit, `REXX_SWITCH_MODE=sim:S,...`, the group and part.
- A determinism self-test: every group under one seed twice, byte-identical outcomes and identical switch counts.
  Without it a hidden real-time input reads as a schedule.

## 4. The unjudged mutants

| Mutant | Schedule that would make it observable | Status |
|---|---|---|
| M6 `switch_to` keeps `SLICE` (`src/scheduler.rs:976`) | In every switch mode `serve_requests` clears a fresh `SLICE` before it yields (`:2010-2012`), so a pending `SLICE` at `switch_to` only comes from the timer thread between countdown visits. The incoming activity then yields at its first visit. Every such schedule is one the timer could produce anyway, so no outcome leaves the legal set | Equivalent under any outcome judge, sim included. Killable only by a slice-accounting instrument |
| M7 `switch_to` keeps `slice_deferred` (`:977`) | Under `every`: A, inside a pinned frame, defers a slice, then parks pinned (a `~result` in a sort comparator); B runs inside its own pinned frame. Correct code counts B's first fresh slice as deferred (`:2028-2031`); M7 takes `pinned_yield` at once (`:2023-2026`). Rexx output moves by one clause (P32 class); the pinning report's deferred and pinned-yield counts differ | Not run. A pinning-report test in existing switch mode, no sim needed |
| M9 `cancel_wait` keeps `when_parked` (`:1034`) | A `GUARD WHEN` wait fails trappably (11.1 at the stack bound in `run_round`, `:1503-1507`), the program traps it and later parks on `~result`; then a signal's `halt_all` reaches `wake_for_halt` (`:2170`), whose stale `when_parked` branch unparks that unrelated park. `post_guard` cannot reach it: `abandon_guard_exec` withdraws the watches (`end_guard_exec`/`abandon_guard_exec`, `src/guards.rs:642-660`) | Not run. Needs `halt@K` plus `fail=wait:K` or a stack-bound program |
| M10 `cancel_wait` keeps the guard-queue entry (`:1035`) | A guard-lock wait fails trappably while another activity holds the guard; its release grants the lock to the activity no longer waiting; a later send to the object deadlocks | Tried, not reached: see below |
| M11 `cancel_wait` keeps the sleeper (`:1047-1049`) | A `SysSleep` under a pinned frame fails trappably at the stack bound; the program continues and parks on `m~result`; the stale deadline readies it and the result answers early | **Killed** by the prototype below |
| M12 `object_roots` drops `failed_sends` (`src/activity.rs:607`) | Handles are generation-checked (`rexx-core/src/heap.rs:429-437`) and `set_native_entry` on a dead handle does nothing (`src/environment.rs:1241-1247`). Any party able to read the attached condition holds a strong reference, which roots the message, so the lost write has no reader. Not checked: a `Message` subclass with `UNINIT`, whether an uninit-pending object can lose the write | Argued equivalent for Rexx output, not shown. An instrument (count writes to dead handles in `set_native_entry` under test) plus `gc=q` and a program whose main drops the message first would show whether the write is lost |

Prototype (crate test appended to `src/scheduler/tests.rs` in the throwaway worktree, run with
`CARGO_TARGET_DIR=/tmp/claude-1000/p61/sc/target memcap 8G cargo test -j 4 -p rexx-exec --lib scout_c -- --nocapture`,
on `run_program_on_stack` with a 64 MiB interpreter stack): main scans for the `INTERPRET` recursion depth at which
`call SysSleep` under `signal on syntax` is trapped as 11.1 (it found 5698; 5701 failed at the call), sleeps 1 s
there and traps the 11.1, then runs `m = .w~new~start('slow')` (`slow` sleeps 3 s) and
`say 'result' m~result 'after' (time('e') >= 2.5)`.

- Unmutated: `bottom caught`, then `result slow done after 1`, rc 0.
- M11 (the `sleepers.retain` in `cancel_wait` removed): `bottom caught`, then `result The NIL object after 0`, rc 0.

So the "continue past a failed wait" path is reachable from Rexx with a trappable condition, which the final review
did not run, and M11 changes output there. On the shipped CLI (512 MiB stack) I did not reach it: an operator
recursion stopped at the activation limit (`src/run/call.rs:148`) at depth 5000 (`rexx-run depth.rex 5000` printed
`top 11.1`; 4999 printed `answer slept`).

M10 attempt, same harness: a worker holds `o`'s guard in `SysSleep`, main sends `o~touch` at depth. At depth 5698
the 11.1 came from the send's own check (`src/dispatch.rs:3125-3128`) before any park, and M10 printed the same
as unmutated (`bottom caught`, `result touched`). One `INTERPRET` level is wider than the gap between that check
and `run_round`'s, so this needs finer stack padding or `fail=wait:K`.

## 5. Cost

| Part | Files | Size |
|---|---|---|
| `SwitchMode::Sim`, parsing, `Outcome` report | `src/invocation.rs`, `src/bin/rexx-run.rs`, `src/lib.rs` | S |
| PRNG | new `src/sim.rs` | S |
| Policies (preempt decision, FIFO/shuffle pick, PCT points) | `src/scheduler.rs` (`:1990-1999`, `:1603`), `src/sim.rs` | M |
| Clock seam and the routed sites, idling in sim | `src/sim.rs`, `src/clause.rs`, `src/scheduler.rs`, `src/builtin/rexxutil.rs`, `src/dispatch/semaphore.rs`, `src/semaphores.rs`, `src/dispatch/time_support.rs`, `src/builtin/datetime.rs` | M |
| Pool bound 0 and `RANDOM` seed in sim | `src/lib.rs`, `src/builtin/numeric.rs` | S |
| Knobs `gc=`, `halt@`, `fail=wait:` | `src/lib.rs`, `src/scheduler.rs` | S |
| Harness mode, seeded gate test, outcome-set cache, tier judge, determinism self-test | `tests/support/group_runner.rs`, `tests/concurrency_tests.rs`, a committed outcome-set file | L |
| Mutant tests (M11 from the prototype, M7 pinning count, M9/M10 via knobs) | `src/scheduler/tests.rs`, `tests/concurrency_tests.rs` | M |

No `unsafe`, no new dependency.

Risks:
- Pool bound 0 means sim never runs the P52 lending path, which is where the Phase 6 races were.
- A hidden real-time input (a native's own thread calling back, a command's timing) makes a seed unreplayable;
  only the determinism self-test catches it.
- Tier-2 noise: legal schedules the sampled oracle set lacks.
- `next_runnable` gains a branch per switch in default mode; not per clause, but the ping-pong benchmark should
  be measured before and after.
- Seeds replay only on their commit.

## 6. Open decisions for Moritz

1. Ready order: FIFO with random preemption points (PCT over preemption points) as the default, or PCT priorities
   that reorder. **Recommend FIFO default**, `order=shuffle` opt-in, full reorder judged by tier 1 only.
2. Pool under sim: bound 0 (inline), or real pool threads whose completions are awaited at once and delivered at
   a seeded later clause. **Recommend bound 0 first**, the delivered variant second, since P52 is otherwise
   untested by sim.
3. Judge: **recommend tier 1 fails, tier 2 reported**, as in section 3.
4. Clock origin: **recommend real at start**, `clock=midnight` as a knob for the `TIME` rows.
5. Seed echo: **recommend** the `Outcome` field for the harness and one masked `rexx-sim:` stderr line from the CLI.
6. Signals: **recommend** scripted `halt@K` in sim and real signals left real.
7. Fault injection (`fail=wait:K`) in a shipped mode: **recommend yes**, sim-only, never in an oracle comparison;
   it reaches M9 and M10 without a stack-bound program.
8. M6: **recommend** recording it as equivalent under outcome judging rather than building a slice-accounting
   instrument.
9. M12: **recommend** the dead-handle write counter under test, then one `gc=q` run, before calling it equivalent.
10. M11: **recommend** landing the prototype as a test (it needs `run_program_on_stack`, about 25 s of scan; a fixed
    depth or `fail=wait:K` would make it fast).
