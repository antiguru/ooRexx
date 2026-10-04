# Task 18 report: the driver pool, the `Send` grant and blocking operations

Base `837482aa0`. Code commit `2fd6b05b3`; this report is committed after it.

## Design

### Departure from the plan: the call moves, the baton is lent

The plan (Step 2) and spec 2.2 hand the baton to a pooled thread on a release with ready
activities, the releasing thread running the C call. In this tree every thread that exits has
scheduler frames below the exit holding `&mut Interp` (main's `run_activity_root`, every nested
loop), so a pooled thread taking the baton could resume only above those frames, and the releasing
thread could not resume until it ended (carry note 2, generalised: `owners` is non-empty in the
common case). Spec 2.6 also asks for the opposite shape in the pinned case: the nested loop blocks
on the inbox while the call is in flight.

So the call goes to the pool and the baton stays with the thread whose frames hold it:

- **Exit** (`Scheduler::exit_for_native`, `scheduler.rs`). The call's parts (`OffBaton`, the
  activity's `ThreadContext` clone, frame, pending condition) go into a `PooledCall`, wrapped in
  `Islanded`. A reserved pool thread gets the job and the holder lends it the baton
  (`Baton::lend`); the pool thread enters the call (builds its `Activation` over
  `HostRef::through`, `ThreadContext::enter`), gives the baton back, and calls the stub. The holder
  goes on running other activities.
- **Recall.** A callback on the pool thread (`Recalling::take_unless_held`,
  `dispatch/library.rs`) posts `Posted::Recall` to the inbox and waits for a lend. The holder
  serves it at its next drain (scheduler entry, cold visit, or idle on the inbox):
  `Interp::serve_recall` switches the call's activity in (`swap_running`, the outgoing activity
  pushed as a buried owner), pins it (`NativeApiCallback`), counts the call as `runs_below`, sets
  the stack fields to the pool thread's, and lends the baton with a root pointer derived from its
  own `&mut self`. When the stub returns, the pool thread recalls once more for the call's end:
  it leaves the thread context, puts the parts back on the record (`NativeInFlight::back`, frame
  checked), drops the `ThreadContext` clone, posts `Completed` and gives the baton back.
- **The baton** (`baton.rs`) is now `Baton<L>`: plain take and release as before, plus nested
  lends (`lend`, `await_lend`, `lent`, `give_back`) whose holders form a stack; only the top runs.
- **`HostRef`** (`rexx-api`): `HostRef::through(baton)`, and `Activation::conversion` refreshes a
  guarded host from `Baton::host()` at each take, so a callback derives its `&mut` from the
  pointer the current lender derived, never from one taken at an earlier lend.
- **Fallback.** Where `Pool::reserve` answers `None` (the bound is reached or the spawn fails),
  the call runs on the exiting thread exactly as Task 17 runs it; the ready activities wait for
  the next thread that takes the baton.

### The pool

`scheduler/pool.rs`. One per interpreter, the last field of `Interp`. Threads have 512 KiB stacks
(`POOL_STACK_BYTES`, `SysThread.hpp:67`), are spawned on demand up to `POOL_BOUND` (64, a value
of this task's choosing; the oracle bounds only idle threads), and return to an idle list after
each job. Dropping the pool ends and joins the idle threads. A run that ends with a call or command
still in flight (a deadline, a loud refusal) leaks its `Interp` (`execute_on`) rather than free
state a pool thread will still reach under a lend.

### Error 11 on a pool thread

`POOL_STACK_MARGIN` is 64 KiB, the oracle's `errorRecoveryStack` (`Activity.hpp:445`).
`serve_recall` points `stack_base`/`stack_room` at the pool thread for the length of the lend, so
every existing check (`run/call.rs`, `eval.rs`, `dispatch.rs`, `run_round`) measures that thread;
`run/call.rs` needed no change. Measured with `deep_pinned_recursion_on_a_pool_thread_raises_11`'s
program: 55 levels here, rc 245 (the oracle: 7744 levels, rc 245, on its main thread). At margins
of 32 KiB and 192 KiB the same program raised 11.1 at 64 and 43 levels; the test profile is
optimised, and debug assertions changed nothing.

### `exit_for_block` and `ADDRESS`

`command.rs`: `spawn` is split into `start` (on the baton) and `collect`. Where nothing is
redirected and `blocks_off_baton` holds (another activity alive or a test mode, not in a park's
continuation), `run_command` hands `Block { running }` to `Scheduler::exit_for_block`, which runs
`collect` on a pool thread and posts `Posted::Unblocked`. The clause parks with
`ParkReason::Block`; `exec_command` stashes the command text and indent on `Activity::blocked`;
the re-run (`exec_instruction`'s new arms) settles it through `end_blocked_command` /
`settle_command` without evaluating anything again. A pinned clause (every `Op::Exec` runs at pin
depth 1) waits in a nested loop, which blocks on the inbox. No pool thread free: the child is
waited for on the baton. `in_flight` counts blocks too.

### The `Send` grant

`island.rs` holds `Islanded<T>` with the only `unsafe impl Send`, `Island` (the root pointer) and
`Lent` (a lend whose `interp()` borrow ends before the give-back its drop does). `unsafe_sites.rs`
lists the file in both lists.

## Carry-forward notes

1. **SAFETY wording.** The note says "no refcount operation and no interior access off the baton",
   and names both `ThreadContext` clones: the fallback's, made before the release and dropped after
   the reacquire (as Task 17), and a pool thread's, moved in an `Islanded` and dropped under a lend.
2. **Pinned frames below an exit.** The baton never leaves a thread whose frames hold it, except as
   a lend that ends before that thread runs again; pinned frames resume on their own thread.
   `a_pinned_wait_blocks_on_the_inbox_while_a_pool_thread_runs_the_call` records the threads: the
   native ran on a thread other than the interpreter's, and `HERE`, called by the comparator after
   the wait, ran on the interpreter's.
3. **Starvation.** Decided: the P43 shortcut stands. A lone activity's call keeps the baton for its
   whole length, so an activity its callback starts runs once the call returns, and a call that
   then waits for that activity in C waits in vain. Spec 2.1 has every call from a resumable entry
   leave its driver and the oracle releases its kernel lock around every native call
   (`NativeActivation.cpp:1304`); P43 trades that for no cost to a single-activity program. Test:
   `a_lone_call_keeps_the_baton_after_its_callback_starts_an_activity` (`0\nw\n`, no exit), over
   `rexx_api::load::send_then_await`. Recorded in `phase-6-gate.md` S4.
4. **Mutants.** The inbox block is now necessary (M1 below). The entry drain is a latency-only
   optimisation: removing it leaves the scheduler tests green (M2), since the idle on the inbox and
   the cold visit drain the same posts; no test witnesses it.
5. **Close and call race.** Argued away: `entry.held()` (`dispatch/library.rs:98`, `:188`) runs in
   prepare and `library.close()` (`dispatch/library.rs:508`, `libraries.rs:96`) at termination or
   on drop, all on the baton; pool threads never take a row. Derived by
   `grep -rn '\.held()\|\.close()' crates/rexx-exec/src --include=*.rs | grep -v tests`.
6. **Blocking operations.** The command now gives 123 hits (113 at `bddcd480e`, ten from this
   task); each is classified in `phase-6-gate.md`'s new S4 section, with the output. The `ADDRESS`
   wait is `command.rs:595`.
7. **Unsafe sites, dependencies, globals.** `unsafe` is added in `island.rs` (listed) and in the
   granted `rexx-api` files. No dependency added. No new global: the pool belongs to the
   interpreter; the test natives' statics are in the test module.

## Other changes

- **`Activation::conversion` built a `Taken` eagerly** (`then_some(Taken(..))`), whose drop
  released a baton it had not taken. Harmless while only plain takes existed; with lends it gave
  the baton back mid-call. Now `then(|| Taken(..))`.
- **The home-thread check** (`AttachThread`, `AddCommandEnvironment`, `ffi.rs`) also accepts the
  thread running the innermost call with the baton released (`Thread::runner`, written by
  `enter`); otherwise a library loader registering a command handler from a pool thread panicked.
  The full thread check is Task 19's.
- **A failure ending a park while the call is in flight** found no park (`resume_parked`): the park
  lives on the call record until the call's end. It now takes it from there
  (`NativeInFlight::take_park`). Found as a 1-in-15 failure of
  `a_failure_ending_a_native_park_abandons_the_call`; 40 of 40 after.
- `ParkKind::Command` in the `pinning` feature; `refusal-sites.tsv` re-derived (line numbers only).

## Tests

`scheduler/tests/pool.rs`, over routines defined in the test (`rexx_api::load::routines_only`):

| Test | Shows |
|---|---|
| `two_native_calls_waiting_for_each_other_meet_on_pool_threads` | both calls meet (2, 0), 2 exits |
| `a_spawn_failure_leaves_ready_activities_to_the_next_baton_holder` | pool stack `1 << 46`, spawns fail: both give up (0, 2), same stdout |
| `deep_pinned_recursion_on_a_pool_thread_raises_11` | `Error 11.1:` on stderr, no crash |
| `a_pinned_wait_blocks_on_the_inbox_while_a_pool_thread_runs_the_call` | Task 17's pinned-waiter program with a 200 ms native: `7\nsorted 1,2\n`, thread identities as in note 2 |
| `a_long_command_does_not_stop_another_activitys_output` | `tick 1..3`, `after 0`, `done`; the oracle prints the same, 3 of 3 runs from a fresh directory |
| `a_lone_call_keeps_the_baton_after_its_callback_starts_an_activity` | note 3 |

Task 17's `native.rs` tests pass unchanged; those whose calls leave their driver now run them on
pool threads. Two loom models are
added (`tests/loom.rs`): `a_lent_baton_comes_back_before_its_lender_runs` and
`a_recall_posted_to_an_idle_holder_is_lent_the_baton`.

## Mutation evidence

`scratchpad/t18/mut/run.py`: each mutant applied to the working tree, restored from a saved copy;
`CARGO_TARGET_DIR=<scratch> memcap 8G cargo test --profile mutation -p rexx-exec --lib
scheduler::tests --no-fail-fast -- --test-threads=4`.

| Mutant | Result |
|---|---|
| M1 no inbox block (`next_runnable` answers `None`) | exit 101, 10 failed, among them the pool pinned-waiter, meeting and command tests |
| M2 no entry drain | exit 0, 68 passed: latency only (note 4) |
| M3 recall served without switching the activity in | exit 101, panic "a native activation is running" |
| M4 native calls never use the pool | exit 101: the meeting and pool pinned-waiter tests |
| M5 pool stack unbounded (`stack_room = usize::MAX`) | exit 101, "thread 'rexx-pool' has overflowed its stack" |
| M6 a callback takes no baton (no recall) | exit 101, panic "a native activation is running" |
| M7 call parts not put back at the call's end | exit 101, 11 failed |
| M8 commands keep the baton | exit 101: the command test |

M1 and M2 were rerun after the `take_park` fix (results above); the first M2 run failed only the
abandon test, the race that fix closed.

## Checks

At `2fd6b05b3`'s tree, from `rust/`:
- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. Also clean:
  `--features pinning` on `rexx-exec`, and `RUSTFLAGS="--cfg loom"` on `rexx-exec`.
- `cargo test --workspace --release --no-run` (exit 0), then
  `memcap 8G cargo test --workspace --release --no-fail-fast`: exit 0, 141 test binaries, 2947
  passed, 0 failed (rexx-exec lib 963). Built outside `memcap`: the first attempt under the cap
  was OOM-killed at 8G while compiling, before any test ran.
- `RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=<scratch> memcap 8G cargo test -p rexx-exec --test
  loom` at the commit: exit 0, 10 passed (328 s).
- Scheduler tests repeated 20 times (`--profile mutation`): 20 of 20 at 68 passed.
- "It works": release `rexx-run` built from `git archive` of `837482aa0` and `2fd6b05b3` into
  their own target directories (each printed `Compiling rexx-exec`), every `bench-programs/*.rex`
  and `bench-rexxcps/rexxcps.rex`, `LD_LIBRARY_PATH` at the oracle's lib: every exit status 0 on
  both; stdout and stderr identical except `heapshape`'s `build_seconds=`/`gc_pause_seconds=` and
  `rexxcps`'s `Performance:` lines; `extcall` prints `3000000`.

## Concerns

- **Plan departure** (Design, first section): the pool runs calls, not drivers; the baton moves only
  as a lend. Review should rule.
- **Callbacks from pool threads are served at the holder's drains**, at most a countdown (1024
  clauses) late while it runs, even when it is pinned. That is most of Task 19's request priority
  and delegation; Task 19 keeps the thread check, foreign threads and timer arming for requests.
- **`Running` is no longer LIFO** once calls interleave (Task 17 review); `Addressed::Elsewhere`
  detection can miss. Task 19.
- **Leaks on an unfinished run**: an `Interp` ended with work in flight is leaked, and its pool
  threads stay blocked.
- **Test helpers in `rexx-api`**: `routines_only`, `send_then_await` and `pub` `NativeRoutine` /
  `Stub` types, `doc(hidden)`.
- Filesystem builtins stay on the baton (gate record).
