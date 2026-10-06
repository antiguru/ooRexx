# Task 22 review: criterion 3 (TSan run, suppressions, exclusions)

Reviewer: s4-t22-rev-tsan. HEAD 75f8aeda8 (rust/ identical to b8ec39593 apart from `tsan.supp`).
Nothing in the worktree was changed. Mutations were made only in a `git archive` copy under the
scratch directory, which has been deleted.

## Verdict

**Changes required.** These are minor and do not block. Criterion 3 itself holds: the recorded
command runs clean as written from an empty target dir, and both suppressions are false positives.
What needs changing is the record. The gate text says the excluded tests are all depth-calibrated
tests that fail under TSan. That is false for three of them (F2). The `hold_buffer` suppression
also hides more than the record says (F1).

Counts: 0 critical, 0 important, 3 minor, 3 info.

## Checks run

1. **The recorded command, as written** (`phase-6-gate.md:720-730`), run once from `rust/` with a
   fresh `CARGO_TARGET_DIR`. It built from nothing (`-Zbuild-std`, nightly 4aa1fbcf4). Every command
   exited 0 (`EXIT1=0 EXIT2=0 EXIT3=0`) and the log dir stayed empty. The counts match the record
   exactly: rexx-api 70, rexx-exec lib 986 with 16 filtered, `concurrency_tests` 32, `program_end` 2,
   `signals` 36 with 1 filtered, `stdin_contention` 1. That is 1127 `... ok` lines. I did one run,
   not ten.
2. **Suppressions removed one at a time.**
   - Without `hold_buffer`: `scheduler::tests::lent`, 3 runs. Each run wrote one log with 2 reports
     and exited 66.
   - Without `_dl_close_worker`: the full lib command, 10 runs. Nothing was reported (986 passed and
     no log, every run). I triaged this one from the implementer's preserved log,
     `p6-scratch/t22/tsan-final-2/tsan.1595010`.
3. **Exclusions.**
   - The 16 lib tests the substring skips remove are enumerated in F2.
   - The three non-depth tests among them ran under TSan: 3 runs alone, then 3 full lib runs with
     only the 13 true depth tests skipped. All clean.
   - A shallow form of the pool-callback recursion ran in a scratch copy: 7 runs, clean (F3).
   - The excluded `signals` test ran under TSan (F5).
4. **Coverage.** Every surface criterion 3 names had passing tests in the run:
   - **Baton lend/recall:** `ffi::tests::a_callback_takes_the_baton_*`,
     `scheduler::tests::native::a_callback_during_an_off_baton_call_takes_the_baton`,
     `callbacks::recalls_drained_together_are_each_served`, and 13 `callbacks::` tests in all.
   - **Driver pool and lent memory:** 10 `pool::`, 11 `native::` and 6 `lent::` tests.
   - **Inbox and completions:** `scheduler::tests::the_inbox_answers_posts_in_order_then_nothing`,
     `callbacks::an_abandoned_calls_completion_does_not_complete_the_next_call`,
     `lent::an_abandoned_calls_frame_ends_with_its_completion`.
   - **Timer:** `sync::tests::a_timed_wait_ends_close_to_its_timeout` and
     `a_written_wake_ends_one_wait` (the 2e6917afe tests), `scheduler::tests::sleepers_wake_in_deadline_order_and_overlap`,
     `native::the_timer_is_armed_for_a_call_in_flight`, and `time_support::` alarms.
   - **Signals:** `signal::tests::*` and the 36 tests in `signals.rs`, among them
     `sigint_ends_a_parse_pull`.
   - **stdin:** 7 `input::` tests and `stdin_contention`.

   The timerfd path is live in the TSan binary. Under `strace -f`,
   `scheduler::tests::a_sleeper_wakes_while_main_is_busy` made 1 `timerfd_create` and 6
   `timerfd_settime` calls on the timer thread, with deadlines of about 24 ms.

## Findings

### F1. Minor: the `hold_buffer` suppression hides every race on the routine's call tree, including its API callbacks

- **Where:** `rust/tsan.supp:11`; the record is at `docs/superpowers/plans/phase-6-gate.md:753`.
- **Is the race real?** No. With the suppression removed there are exactly two reports, and both
  are the file-ordered pair:
  - `hold_buffer` `write_volatile` (`rexx-api/src/load.rs:928`) against `BufferBytes::try_reserve_exact`
    under `native_mutable_buffer_append`. This is the grown case.
  - `hold_buffer::{closure#0}` `read_volatile` (`load.rs:926`) against `BufferBytes::extend_from_slice`
    under `replace_buffer_contents` / `native_mutable_buffer_overlay`. This is the overlaid case.

  Both "previous" accesses are the activity's change, which `lent.rs`'s `HELD` / `DONE` order
  before the file `HOLDBUFFER` polls for. The values confirm it: the overlaid test reads
  `abXY...`, the post-overlay bytes.
- **Is the suppression narrow?** No. TSan matches a `race:` pattern against every frame of every
  stack in a report, not just frame 0. I demonstrated this: replacing the pattern with
  `race:rexx_exec::scheduler::pool::work` (frame #38 of the pool-side stack) also suppressed both
  reports (0 logs, against 1 without it).
- **What it hides:** any race in `MutableBufferData`, `MutableBufferLength` or `NewString` that
  `hold_buffer` calls on the pool thread. That is the P50 "callback off the baton" surface, and in
  the two tests that exercise lent buffer storage.
- **Failure scenario:** a regression that lets `MutableBufferData` read island state without
  recalling the baton would show up only in these two tests. The suppression would hide it there.
- **Demonstrated:** the matching breadth, yes. A hidden real race, no: none occurs today.
- **Fix:** make the handshake visible to TSan and delete the suppression. For example, a test
  routine the activity calls after its change could set a `static` `Mutex`/`Condvar` that
  `hold_buffer` waits on in place of the file. If the suppression stays, its comment should say it
  covers the routine's whole call tree.

### F2. Minor: the substring `DEPTH` skips remove three tests that pass under TSan, and the record says otherwise

- **Where:** `phase-6-gate.md:725-727`. The claim is at `:755` and in `task-22-report.md:79-83`.
- **Which tests:** `--skip` matches substrings. The 16 tests filtered out are the 13 depth tests
  plus these 3:
  - `run::tests::routines::a_routines_own_clauses_echo_at_indent_zero_however_deep_the_call_site_is`,
    a one-thread TRACE test caught by `deep`;
  - `ir::compile::tests::the_high_water_mark_is_the_deepest_the_stack_reached`, a compiler test
    caught by `deep`;
  - `scheduler::tests::notifier_failures_too_many_to_nest_are_each_reported`, named in `DEPTH`
    itself. It runs 6000 started activities with failing notifiers.
- **Result under TSan:** all three pass and leave no log, both alone (3 of 3) and inside the full
  lib command with only the other 13 skipped (3 of 3: 989 passed, 13 filtered, exit 0, no log).
- **What the record says:** that the exclusion covers "every test that measures recursion against
  the native stack ... which also fail on TSan's larger frames". For these three that is false.
  The record also drops the largest activity-count scheduler test from the race run.
- **Demonstrated:** yes.
- **Fix:** skip the 13 by exact path (`--exact --skip <full::path>`). Then the lib count becomes
  989, and the sentence at `:755` holds.

### F3. Minor: the pool-callback recursion can run shallow under TSan, and the deep form adds no handoff the shallow one lacks

- **Where:** `rexx-exec/src/scheduler/tests/pool.rs:305-337`, excluded by `phase-6-gate.md:725`.
- **The probe:** I added a scratch test with `DEEP`'s shape bounded to 25 levels:
  `.r~new(25)`, whose `deep` method decrements a counter and calls `send0(self, 'deep')` until it
  reaches 0. A started `idle` activity runs alongside. The test asserts output
  `bottom\nidled\n`, `exits == 1` and `takes == 1`.
- **Result:** 7 TSan runs, 6 passed and 0 logs. The first run had no witness assertion. The next
  three asserted `takes >= 24`, which failed every time (`takes=1`) and still left no log. The last
  three asserted `== 1` and passed.
- **Logging control:** the same binary with an empty suppression file logged the `hold_buffer`
  report in the same process. So logging was live.
- **What the witness shows:** the recursion is one driver exit and one baton take (one lend). The
  nested native calls then run pinned on the lent pool thread. Depth repeats that nest; it does not
  repeat lend/recall. So excluding the 141k-frame test hides no handoff path beyond the nest
  itself.
- **Not checked:** whether another running test already nests a native call under a lend at more
  than one level.
- **Demonstrated:** yes, for the probe's result.
- **Fix:** add a bounded-depth test like the probe, so TSan covers the nest every gate.

### F4. Info: the `_dl_close_worker` suppression is accepted

- **Where:** `rust/tsan.supp:17`, `phase-6-gate.md:754`.
- **Evidence:** the preserved report shows `malloc` at `elf/dl-close.c:360` and `free` at
  `dl-close.c:476`, both inside `_dl_close_worker`. They run on two `rexx-interp` threads, from
  `a_library_call_answers_*` and `a_library_routine_answers_*` under `Mapping::close`
  (`rexx-api/src/load.rs:156`). The pair is internal to ld.so, under `dl_load_lock`, which TSan
  does not see.
- **Reproduction:** 0 of 10 full lib runs without the suppression. The original was a 1-in-3 event.
- **Breadth:** by F1's matching rule, it also covers fini code run from `_dl_close_worker`. The
  libraries closed are the uninstrumented C++ test libraries, so little can hide there.
- **Demonstrated:** no, not reproduced.

### F5. Info: the excluded thread-count test is correctly excluded, and its child runs race-free

- **Where:** `phase-6-gate.md:729`, `:756`.
- **What I ran:** `only_the_interpreter_and_its_waits_take_the_halting_signals` under TSan.
- **Result:** it fails only on one extra `("rexx-run", true)` entry in each of the three samples.
  That entry is TSan's background thread. The `rexx-run` child (command wait, `SysSleep`,
  `parse pull`) wrote no TSan log.
- **Demonstrated:** yes.

### F6. Info: whole integration targets are outside the run, and the record does not list them

- **Where:** `phase-6-gate.md:728`, `:737-742`.
- **What is left out:** only 4 of the rexx-exec integration targets run. The gate text names the
  covered paths, and leaves out only `concurrency_tests`' oracle rows. It does not say that, for
  example, `collect_stress` (REPLY, park and pinned-yield programs), `deadline` and
  `outer_context` are not run.
- **What I ran:** `collect_stress`'s five multi-activity tests under TSan (`reply activity
  pinned_yield`): 5 passed, no log.
- **Fix:** one sentence in the gate text naming the targets that are left out.
- **Demonstrated:** yes, the clean result.

## Claims checked

| claim | result |
|---|---|
| clean, 1127 tests | Confirmed in 1 run; 10/10 not re-run. |
| `hold_buffer` is a false positive | Confirmed. The suppression is wider than stated (F1). |
| `_dl_close_worker` is a false positive | Accepted from the preserved log; not reproduced (F4). |
| Depth tests are excluded for the 65536-frame shadow stack | True for 13; 3 more are excluded with no cause (F2). The SEGVs are in `t22/gdb.log` and `gdb2.log`. |
| The thread-count test counts TSan's thread | Confirmed (F5). |
| Left out: deep recursion and oracle rows | The deep recursion is replaceable by a shallow test (F3). Whole targets are also left out (F6). |
