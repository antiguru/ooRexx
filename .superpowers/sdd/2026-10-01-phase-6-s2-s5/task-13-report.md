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

## P46

Ruling P46 applied at `b5e6e01d1`. `EVERY_FORCES_THE_RACE` in `tests/concurrency_tests.rs` names
MutexSemaphore TEST_EXCLUSION alone. It applies only when the oracle cell is `pass`, the unswitched
run exits 0 without the inverted-wait refusal, and the every run ends at the deadline (status None)
without that refusal. Any other outcome on that row goes through the strict path: the stuck check
and `compare_modes`. Any other row that hangs still fails.

The reproduction is recorded at
`docs/superpowers/records/2026-10-01-phase-6-s2-s5/exclusion-every-hang.rex`: probe x2, with a
header comment. It is TEST_EXCLUSION reshaped outside ooTest, with `call SysSleep 0.01` after the
worker's `step = 6`. On the oracle it hung 3 of 3 (rc 137 under `timeout -k 5 20`) both times it
ran: as the scratchpad probe, and again from the recorded file after the comment was added.

The gate record row reads "every mode forces the test's own race; the oracle hangs in the same
interleaving (P46)".

Gate rerun at `b5e6e01d1`: `REXX_CORPUS_GATE=1 REXX_CRITERION_ONE_TABLE=... memcap 16G cargo test
--release -p rexx-exec --test concurrency_tests` exits 0, 30 passed, 0 failed. Against the
`7451743ce` table, two rows differ:
- the TEST_EXCLUSION row, now the P46 label;
- REPLY TEST_REPLY_TWICE_REPLYASSERT. Its oracle cell reads `differ: rc 0, oracle 1 assertion,
  ours 0`; at `7451743ce` it read `pass`. The oracle's own count varies between runs (P41 class),
  and the test does not gate on that cell.

## Fix round 1 (P47)

Commit `59817d2c8`. Review: `task-13-review.md`. Ruling P47.

### Design

- **Wake, then re-test.** A post or release now only readies waiters. The semaphore's state is not
  handed over. Each woken waiter re-tests its condition before it resumes and parks again where the
  condition no longer holds; a timed waiter parks again until its original deadline.
- **Where the re-test runs.** There are three resume paths:
  - a started activity's resume, in `run_started`;
  - main's root step;
  - the pinned wait in `park_native`, which loops `pinned_wait` and re-tests.
  The re-test (`Interp::retest_semaphore`) records its answer by activity, and the native's resume
  reads it back.
- **EventSemaphore.**
  - A timed wait re-tests the post: `SysSemaphore::wait(t)` loops while `!postedCount`
    (`SysSemaphore.cpp:298-301`).
  - An untimed wait does not re-test: `wait()` tests with `if` (`:250-253`), checked in the
    source. The witness below shows this deterministically (oracle 30/30): after a post and reset
    in one clause, the untimed waiters answer 1 and the timed one 0.
- **MutexSemaphore.** A release frees the lock and readies its first waiter. The waiter takes the
  lock only if it is still free when it runs, so the releaser's next acquire wins (M1).
- **Sys*Sem.**
  - A post adds to the value and readies the first untimed waiter. That waiter is a `sem_wait`,
    which in glibc retries the decrement after its futex wake, so the value is never handed over
    directly.
  - A timed wait (`WaitKind::Poll`) wakes only at its own poll times, t0 + k·100 ms. It tries
    `take_counting` there and answers 121 one period after its last try. No post wakes it. The old
    `takes_until` window is gone.
- **M2.** `SysCreateEventSem(name)` and `SysCreateMutexSem(nonempty name)` answer `''` and create
  nothing when glibc `sem_open` rejects the name with `EINVAL`: nothing is left once the leading
  slashes go, or a slash follows them. Checked on glibc alone with a C program using
  `O_CREAT|O_EXCL`; never run on the oracle; `/dev/shm` held 0 entries before and after. Any other
  name keeps the Phase 10 refusal.
- **M3.** Parked waiters are indexed by activity (`Semaphores::parked`), so `withdraw` touches one
  queue. Held mutexes are indexed by owner (`Semaphores::held`), so an activity's end and the
  collector's roots touch only the mutexes held.
  The reviewer attributed the cost to `withdraw`, but indexing the waiters alone left the probe
  unchanged. Before the fix, q0/q1 measured 0.46/0.81 s (three runs: 0.81, 0.89, 0.78); with
  waiters indexed, 0.46/0.80. perf on 30000 round trips put 20.75% self time in `run_round`, which
  is `release_ended_mutexes` scanning every entry at every activity end. With held mutexes indexed
  by owner, q0/q1 measure 0.46/0.47 and 0.46/0.46.
  Disabling the per-collection `prune` and `object_roots` scans instead did not change q1 (0.89,
  0.94 and 0.87, 0.86), so neither was the cost.

### Witnesses (oracle 30, ours 30 unswitched and 30 under every; one output each, ours = oracle)

- `lang/event_semaphore_timed_wait_retests_the_post.rex` (I1): `pulse 1 0 1 0`, `post 1 1`. The
  post and reset are one clause (`e~~post~reset`). In the reviewer's two-clause form, every mode
  can run the timed waiter between the two clauses and answer 1, a schedule the oracle did not show
  in 30 runs.
- `lang/sys_semaphore_poll_takes_at_its_poll.rex` (I2, the reviewer's h2): `main rel 0 req 0`,
  `main rel2 0`, `w 0 0`.
- `lang/sys_semaphore_post_waits_for_the_next_poll.rex`: a post between two polls stays in the
  semaphore, so main's own try 30 ms later takes it (`main took 0`, `w 0`). Added because
  mutant N4 (a post readies a polling wait) survived every other witness in both modes.
- M1 is not a corpus witness, because the oracle splits on it. The reviewer's two-clause h1 gave
  28/30 for `reacq0 1`; the one-clause form gave 18/30 `main 1 1`, 12/30 `main 1 0`. Ours is
  deterministic in both modes and matches the majority. It is a crate test,
  `a_releaser_takes_the_mutex_again_before_its_waiter`; the minority schedule is recorded under
  P47.
- M2 is a crate test, `a_name_sem_open_rejects_creates_nothing`: `''`, `'///'`, `'a/b'`, `'/'`,
  `'/a/b'` all answer the empty string. The Phase 10 refusal for a valid name stays in
  `run/tests/directives.rs`.
- The 11 earlier corpus witnesses, rerun on the fix build, each give one output 30/30, unswitched
  and every, with the same hashes as before. The reviewer's probes e1, h1, h2, h3, pin1, pin3, pin4,
  pin5, a1, s1, g1 and g2 were rerun against the oracle (3 runs each side).
  - Every one matches the oracle unswitched.
  - Under every, all match except pin3. Its pre-fix binary gives the same line order (`init waits 1`
    before `w posted`), so the difference is not from this round: it is the P44 cadence of a woken
    waiter, and the values agree.
  - e1 two-clause and h1 under every give the oracle-observed minority, as described above.
- ooTest: TEST_WAIT_CONCURRENT passes in both modes (2 assertions). TEST_EXCLUSION passes
  unswitched (13 assertions) and under every is the P46 deadline hang.

### Mutation evidence (`mut2.py`, `mut3.py`; corpus runs compared with stored oracle output)

| mutation | red |
|---|---|
| M0 control | nothing (6 lib tests pass) |
| N1 post readies only the first waiter | event_semaphore_post_wakes_every_waiter, event_semaphore_timed_wait_retests_the_post |
| N2 untimed event wait re-tests the post | event_semaphore_timed_wait_retests_the_post |
| N3 timed event wait keeps a reset post | event_semaphore_timed_wait_retests_the_post |
| N4 a post readies a polling wait | sys_semaphore_post_waits_for_the_next_poll, unswitched and every (green before that witness existed) |
| NM1 release hands the lock to its first waiter | lib a_releaser_takes_the_mutex_again_before_its_waiter, a_release_readies_the_mutex_waiters_in_park_order |
| N6 every name refuses | lib a_name_sem_open_rejects_creates_nothing |
| N7 held mutexes not indexed | mutex_semaphore_released_when_its_activity_ends, ..._end_hands_over_to_a_waiter; lib a_held_mutex_is_not_collected |
| N8 timed Sys*Sem wait has one poll | sys_semaphores_unnamed, sys_semaphores_across_activities, sys_semaphore_poll_takes_at_its_poll |
| N9 a poll takes nothing | sys_semaphores_across_activities, sys_semaphore_poll_takes_at_its_poll |
| N10 release readies the last waiter | lib a_release_readies_the_mutex_waiters_in_park_order |
| N11 post readies without counting | sys_semaphores_unnamed, sys_semaphores_across_activities, sys_semaphore_poll_takes_at_its_poll; lib a_post_after_the_last_poll_is_left_for_the_next_wait |
| M4, M5, M6, M7, M9, M13, M14, M15, M17, M18, M19 (as in the first table) | red as before |
| M10 every woken wait answers 1 | event_semaphore_post_reset_wait, mutex_semaphore_timed_acquire, event_semaphore_timed_wait_retests_the_post |

### Checks at `59817d2c8`

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo clippy -p rexx-exec --all-targets --features pinning -- -D warnings`: exit 0.
- `cargo test -p rexx-exec --lib`: 939 passed.
- `refusal_sites` 5, `gate_table_c` 22, `internal_routines` 2, `closed_phases` 5, `loud` 9,
  `native_entries` 24, `sourceline_oracle` 1, `method_bodies` (release) 23: all passed, with no
  table drift.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: 783 of 783.
  The same under `REXX_CORPUS_SWITCH=every`: 783 of 783. Debug under every: 783 of 783.
- `memcap 8G cargo test -p rexx-exec --test collect_stress`: 36 passed.
- `memcap 8G cargo test --release -p rexx-exec --features pinning --test concurrency_tests
  measured::`: 18 passed.
- `REXX_CORPUS_GATE=1 REXX_CRITERION_ONE_TABLE=... memcap 16G cargo test --release -p rexx-exec
  --test concurrency_tests`: exit 0, 30 passed, 0 failed.
  - The semaphore rows are unchanged: TEST_WAIT_CONCURRENT pass/same, TEST_EXCLUSION pass/P46.
  - Against the P46 table, only two REPLY REPLYASSERT oracle cells moved, both oracle-count
    variance (P41).
- Callgrind, 3 rounds, pre-fix head `18c2a2f47` against `59817d2c8`: dispatch, fibcall, rexxcps,
  emptyloop and sendloop all +0.0000% (at most 172 Ir), spreads 0.0000%.
- Bench "it works" against the pre-fix head: identical except the timing lines of `heapshape`
  and `rexxcps`.

### Recorded (P47)

- **M1 minority schedule.** The oracle's releaser loses the race 2/30 in the two-clause form and
  12/30 in the one-clause form; ours always wins it.
- **Pre-existing, outside Task 13.** `native_array_append` copies the array's slots on every
  append (`collection::append_slot` → `slots_of`/`occupied`). Building the reviewer's 80000-element
  array dominates its probes: a callgrind run of q1 spent 99% of its instructions there and had not
  reached the timed phase when stopped at 600 s.

### Background gate failures on 18c2a2f47 (fix round 1, commit `669b701cb`)

- **dispatch_seam** (G4, G6). `src/dispatch/semaphore.rs` named the seam token without being
  listed in `CLEARANCE_CONSUMERS`. Its natives take `_cleared: Cleared` as a parameter only, as
  `time_support.rs`'s do, so I added it to the list.
  `cargo test -p rexx-exec --test dispatch_seam`: 6 passed, both debug and `--release`.
  `dispatch_seam` is now in my per-task checks.
- **measured::a_park_under_each_frame_kind_records_it** (G8, `Program: {}`). It did not reproduce
  at 18c2a2f47 (15 of 15 passed, archived tree, own target dir) or at `59817d2c8` (45 of 45).
  The cause is in the harness: every `report_of` call rewrote the shared `pinning-probes/extf.rex`
  with a truncating `fs::write` while the parallel probes read it. A probe that reads it empty
  parks nowhere: an empty `extf.rex` gives error 44 and no park, which is exactly `Program: {}`.
  The fix stages the routine under the thread's own name and renames it into place, only where its
  content differs. The test's expectation is unchanged.
  After the fix, `cargo test -p rexx-exec --features pinning --test concurrency_tests --
  measured::a_ measured::an_` passed 10 of 10 (15 each), and `--release ... measured::` passed 18.
  No staged file was left behind.
  Because the race was never reproduced, no run demonstrates the fix; it rests on the mechanism.
- After the change: fmt and both clippy runs are clean.
