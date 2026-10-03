# Task 13 report: MutexSemaphore, EventSemaphore and the unnamed Sys*Sem routines

Implementer s3-t13. Base `05f6b5d41`. Commits: `7451743ce` (code, witnesses, tables), and a
second commit holding this report and the gate record rows.

## Design decisions

- **State per interpreter, keyed by the instance** (`rexx-exec/src/semaphores.rs`, field
  `Activities::semaphores`). One record type holds an event's post, a counting semaphore's value
  and a mutex's lock (owner `ActivityId`, the pthread lock count `depth`, the oracle's `nestCount`
  `nest`, `closed`), plus the queue of parked waiters. Object-keyed entries are created at first use
  and pruned after each collection beside the guard pools (`lib.rs` `collect_now`); `ObjRef` carries
  a generation, so a reused slot never aliases. Handle-keyed entries live from `SysCreate*Sem` to
  `SysClose*Sem`, as the oracle's `malloc`ed `RXSEMDATA` does.
- **Parking.** One new `ParkReason::Semaphore(SemaphoreWait { key, timed })`. `park` files the
  waiter on the semaphore's queue and, for a timed wait, as a sleeper; `cancel_wait` withdraws it.
  WAIT and ACQUIRE are `NativeBegin` rows (`RESUMABLE_METHODS`) through `park_native`; the Sys*Sem
  waits go through a new `park_routine_with` (a routine whose answer is computed at wake). A park
  under a pinned frame is a pinned wait as before.
- **Hand-off, not retry.** A post or release gives the semaphore to a waiter directly
  (`wake_semaphore_waiter`): an untimed waiter, or a timed one still asleep and within its last
  instant. The resume reads the outcome: an event or counting waiter still on its queue timed out;
  a mutex waiter is the owner or timed out. An event post wakes every eligible waiter
  (`SysSemaphore::post` broadcasts); a mutex release or a counting post wakes the first in park
  order. The oracle's pthread wake order is unspecified; FIFO is this crate's.
- **Mutex semantics** (`classes/MutexSemaphore.cpp`, `common/platform/unix/SysSemaphore.cpp`):
  recursive for the owner; `release` answers 0 where `nest` is 0 or the caller is not the owner
  (`EPERM`); after REPLY the owner is still the caller (the continuation is the new activity);
  `cleanupMutexes` runs in `run_until_park` after the ending activity's UNINITs, as the oracle runs
  it after `checkUninitQueue`, releasing once per nesting level and handing the lock on. Main never
  ends, so main's locks are never released (oracle: o5 below hangs). UNINIT is the oracle's
  `close`: `nest` to 0 and `closed`, after which `acquire` with a non-zero timeout answers 0 and a
  trylock still works, while a lock held stays held (measured, o1). Held mutexes are rooted
  (`heldMutexes` marks them).
- **Event semantics:** WAIT answers 1 at once where posted, the post for a timeout of zero, else
  parks; UNINIT leaves the post (measured: `isPosted` 1 after `e~post; e~uninit`).
- **Timeout argument** (`floatingPointArgument(seconds, "timeout")`): a TimeSpan is read through
  `totalSeconds` (a pinned send), anything else through the required-string protocol and
  `double_of`, `nan`/`+infinity`/`-infinity` as the oracle's `doubleValue`; seconds in
  `[0, 4294967]` become whole milliseconds, anything else waits for ever; 88.902 otherwise.
- **Unnamed Sys*Sem** (`platform/unix/SysRexxUtil.cpp:642-1075`): counting semaphores, mutex
  created at value 1; `SysReleaseMutexSem` posts only at value 0; a timeout of 0 waits for ever,
  one below 0 answers 0 and takes nothing (the oracle's loop never runs), one above 0 polls every
  100 ms (`SEM_WAIT_PERIOD`): a post up to the last poll ends it, a later one is left, and the
  answer after the polls is 121. An unknown or closed handle answers 6 (`EINVAL`'s code; the oracle
  dereferences it). Handles are whole numbers from 2^40 so that `datatype(h, 'W')` is 0 as for the
  oracle's addresses; no witness prints one. Argument errors match the oracle's `uintptr_t` and
  `int` conversions (88.901, 88.907, 88.922 under the `REXX` package).
- **Named semaphores**: `SysCreateEventSem(name)` and `SysCreateMutexSem(nonempty)` refuse with
  `routine "..." with a named semaphore is not implemented (Phase 10)` (`Loud::named_semaphore`,
  `sem_open` is shared between processes, roadmap D11); `SysOpenEventSem`/`SysOpenMutexSem` rows
  move to Phase 10. No named semaphore was created on the oracle.
- **`SysCreateMutexSem()` with no argument segfaults the oracle** (`strlen(NULL)`, 3 of 3, rc
  139): `corpus/oracle-crashes.txt` entry 27. Here it creates an unnamed semaphore.
- **Deadlock detection:** the oracle's `checkDeadLock` reads only Message and VariableDictionary
  waits; a semaphore park sets no `Waiting`, so the chain stops there. A cycle through a semaphore is
  the P27 refusal where the oracle hangs (`a_semaphore_wait_is_outside_the_deadlock_check`).
- **Removed:** `ParkKind::unimplemented_method` and `ParkKind::routine` and their call sites (the
  would-be park points for the methods now built); the natives record `park_point!` where they park.
  `REFUSAL_FOLLOWS_THE_SCHEDULE` is deleted (Step 3).
- **Hot paths:** none changed. Callgrind, 3 rounds, base `05f6b5d41` against `7451743ce`'s
  release binary (`bench-programs/callgrind.sh -r 3 -p "dispatch fibcall rexxcps emptyloop
  sendloop"`): dispatch +0.0235%, fibcall -0.0101%, rexxcps -0.0690%, emptyloop +0.0000%,
  sendloop +0.0000%, spreads at most 0.0001%.

## Plan and tree

- Brief Step 1 says "corpus programs (no ooTest file uses them)" for Sys*Sem: done as corpus
  programs. The brief's park points were the would-be ones; they are now real.

## Witnesses (run counts)

Oracle runs: `orx` = the standard wrapper with `timeout -k 5 20`, from a fresh empty directory,
30 runs each; ours: `rexx-run` release, 30 unswitched and 30 under `REXX_SWITCH_MODE=every`. Each
row below had one distinct output per side (the two uncaught-error programs differ between runs
only by the run directory in the traceback path, normalised by the corpus harness), and ours
equals the oracle's.

| corpus program | oracle | ours | every |
|---|---|---|---|
| lang/event_semaphore_post_reset_wait.rex | 30/30 | 30/30 same | 30/30 same |
| lang/event_semaphore_post_wakes_every_waiter.rex | 30/30 | 30/30 same | 30/30 same |
| lang/event_semaphore_wait_not_a_number.rex | 30/30 | 30/30 same | 30/30 same |
| lang/mutex_semaphore_nesting_and_close.rex | 30/30 | 30/30 same | 30/30 same |
| lang/mutex_semaphore_released_when_its_activity_ends.rex | 30/30 | 30/30 same | 30/30 same |
| lang/mutex_semaphore_end_hands_over_to_a_waiter.rex | 30/30 | 30/30 same | 30/30 same |
| lang/mutex_semaphore_stays_with_the_replier.rex | 30/30 | 30/30 same | 30/30 same |
| lang/mutex_semaphore_timed_acquire.rex | 30/30 | 30/30 same | 30/30 same |
| lang/sys_semaphores_unnamed.rex | 30/30 | 30/30 same | 30/30 same |
| lang/sys_semaphores_across_activities.rex | 30/30 | 30/30 same | 30/30 same |
| lang/sys_wait_event_sem_timeout_not_int.rex | 30/30 | 30/30 same | 30/30 same |

collect_stress and the corpus gate (below) run them as well. Paths read from stdout: the post
wake of two waiters (`a 1 b 1 1`), the end hand-over to a parked waiter (`main acquire 1` after
`main waits`), the REPLY continuation's `cont release 0` while the caller holds, the timed acquire
answering 0 then 1, the polled wait answering 0 when a post lands between polls and 121 after its
polls, the counting value left at 0 after a hand-over (`nothing left 121`).

Crate tests under the switch modes (`src/semaphores/tests.rs`, each unswitched, collecting at every
allocation, and under `EveryOpportunity`): hand-off in park order; a post after the last poll left
for the next wait; a semaphore wait outside the deadlock check; a held mutex not collected.

Oracle probes without a witness (kept in the scratchpad): o5, main ending while holding a mutex a
worker waits on: the oracle hangs 5 of 5 (killed, rc 137), ours waits likewise at program end. o9,
an activity that ends with an untrapped error while holding a mutex: oracle `main acquire0` is 1 in
3 of 5 and 0 in 2 of 5 (the release races the message's completion), so no witness.

ooTest: EventSemaphore TEST_WAIT_CONCURRENT passes on both sides, unswitched and under every (2
assertions). MutexSemaphore TEST_EXCLUSION passes unswitched (13 assertions, as the oracle). Under
every it hangs: see Concerns.

## Mutation evidence

Harness `scratchpad/t13/bin/mut.py`: each mutation applied to a copy-restored file, built with
`--profile mutation`, then every Task 13 corpus program run once against the oracle's stored output
and `cargo test --profile mutation -p rexx-exec --lib semaphores::` run. Control M0 (no change):
nothing red, 4 passed.

| mutation | red |
|---|---|
| M1 post wakes only the first waiter | event_semaphore_post_wakes_every_waiter |
| M2 hand-off to the last waiter | lib a_release_hands_the_mutex_to_its_waiters_in_park_order |
| M3 activity end frees without handing over | mutex_semaphore_end_hands_over_to_a_waiter |
| M4 activity end releases one level | mutex_semaphore_released_when_its_activity_ends, mutex_semaphore_end_hands_over_to_a_waiter |
| M5 no re-entry for the owner | mutex_semaphore_nesting_and_close, ..._released_when_its_activity_ends, ..._end_hands_over_to_a_waiter |
| M6 release ignores the owner | mutex_semaphore_stays_with_the_replier |
| M7 close leaves the mutex open | mutex_semaphore_nesting_and_close |
| M8 poll window ignored | lib a_post_after_the_last_poll_is_left_for_the_next_wait |
| M9 held mutexes unrooted | lib a_held_mutex_is_not_collected (green on the first version of the test; strengthened, then red) |
| M10 timed event wait always posted | event_semaphore_post_reset_wait |
| M11 timed acquire always acquired | mutex_semaphore_timed_acquire |
| M12 counting post both hands over and counts | sys_semaphores_across_activities (green before `nothing left` was added; then red) |
| M13 SysReleaseMutexSem posts at any value | sys_semaphores_unnamed |
| M14 negative timeout waits | sys_semaphores_unnamed |
| M15 TimeSpan not read | event_semaphore_post_reset_wait, mutex_semaphore_nesting_and_close |
| M16 timed Sys*Sem wait ends at its first poll | sys_semaphores_unnamed, sys_semaphores_across_activities |
| M17 cleanupMutexes skipped | mutex_semaphore_released_when_its_activity_ends, mutex_semaphore_end_hands_over_to_a_waiter |
| M18 EventSemaphore reset leaves the post | event_semaphore_post_reset_wait |
| M19 SysResetEventSem leaves the value | sys_semaphores_unnamed |

The table above is from `mut1.txt` (M0-M19) and the rerun `python3 mut.py M0 M9 M12` after the two
witnesses were strengthened.

## Checks (commands and results)

All with `CARGO_TARGET_DIR=scratchpad/t13/target`.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0, no diagnostics.
- `cargo clippy -p rexx-exec --all-targets --features pinning -- -D warnings`: exit 0.
- `cargo test -p rexx-exec --lib`: 937 passed, 0 failed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: exit 0,
  `780 of 780 matching`, 29 passed, 1 ignored.
- the same with `REXX_CORPUS_SWITCH=every`: exit 0, 780 of 780.
- the same in debug with `REXX_CORPUS_SWITCH=every`: exit 0, 780 of 780.
- `memcap 8G cargo test -p rexx-exec --test collect_stress`: exit 0, 36 passed.
- `memcap 8G cargo test --release -p rexx-exec --features pinning --test concurrency_tests
  measured::`: exit 0, 18 passed.
- `REXX_CORPUS_GATE=1 REXX_CRITERION_ONE_TABLE=... memcap 16G cargo test --release -p rexx-exec
  --test concurrency_tests`: **exit 101**, 29 passed, 1 failed:
  `the_s2_rows_of_the_derived_list_in_both_modes` with `an inverted wait or a hang:
  ["base/class/MutexSemaphore.testGroup TEST_EXCLUSION: every"]`. Every other criterion-1 row is
  as recorded before this task; the two rows Task 13 changed are in the gate record.
- `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites` re-derived the table;
  the new send-surface row `native_argument_not_a_number` (now raised by WAIT/ACQUIRE) carries
  `agrees yes 88.902`; then `cargo test -p rexx-exec --test refusal_sites`: 5 passed.
- `cargo test -p rexx-parse --test sourceline_oracle`: 1 passed (files generated with the module
  doc's driver).
- `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`: 23 passed;
  the drifted rows are the six semaphore methods (POST, RESET, ISPOSTED, ACQUIRE, RELEASE answer;
  WAIT on a fresh instance is the P27 refusal where the oracle blocks for ever).
- `cargo test -p rexx-exec --test gate_table_c`: 22 passed. `internal_routines` 2, `closed_phases`
  5, `owners` 6, `loud` 9, `native_entries` 24: all passed.
- Bench "it works": every `bench-programs/*.rex` and `rexxcps.rex` on the release head against the
  base binary: stdout, stderr and rc identical except `heapshape` and `rexxcps` (timings only).

## Concerns

1. **TEST_EXCLUSION hangs under `EveryOpportunity`** (criterion 9: no hang in either mode). Every
   mode runs main at the clause boundary after the worker's `step = 6`, before the worker's last
   `sem~acquire`; main's `acquire(1)` takes the free mutex and main, never ending, never releases
   it; the worker waits for ever and so does program end. The oracle reaches the same state when
   the worker yields there: probe x2 (the test reshaped outside ooTest, with `call SysSleep 0.01`
   after the worker's `step = 6`) is killed 3 of 3 at rc 137 on the oracle and hangs on ours too.
   Ruling requested from the lead (an allowance listing this row, or a criterion-9 record). No
   allowance added.
2. Hand-off order (FIFO) and the event post's wake of a timed waiter whose deadline has passed but
   who is still asleep are this crate's choices where the oracle's pthread order is unspecified;
   only interleaving-dependent, witnessed on the crate alone.
3. A Sys*Sem waiter on a handle closed while it waits stays parked (an untimed one for ever, then
   the P27 refusal or the program-end wait); the oracle's waiter blocks on freed memory.

## Probe x2 (oracle hang under the every-mode interleaving)

```rexx
o = .t~new
o~test_exclusion
::class t
::method test_exclusion
  expose step
  sem = .MutexSemaphore~new
  step = 1
  self~run_test_exclusion(sem)
  guard off when step == 2
  say 'm1' sem~acquire(0)
  say 'm2' sem~acquire(0.05)
  step = 3
  say 'm3' sem~acquire(1)
  step = 4
  guard off when step = 5
  say 'm4' sem~release
  guard off when step == 6
  say 'm5' sem~acquire(1)
::method run_test_exclusion unguarded
  expose step
  use strict arg sem
  reply
  say 'w1' sem~acquire
  step = 2
  guard off when step = 3
  say 'w2' sem~release
  say 'w3' sem~release
  guard off when step = 4
  say 'w4' sem~acquire(0)
  say 'w5' sem~acquire(0.05)
  step = 5
  say 'w6' sem~acquire(-1)
  say 'w7' sem~release
  step = 6
  call SysSleep 0.01
  say 'w8' sem~acquire
```
