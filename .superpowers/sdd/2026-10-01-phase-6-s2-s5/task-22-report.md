# Task 22 report: race checking and the S4 close

Base `52b038a80`. Status: DONE_WITH_CONCERNS.

Commits:

- `2e6917afe` The timer waits on a timerfd, so a SysSleep ends on time (a regression fix, found
  in G4/G6).
- `b8ec39593` L-1: a stdin read's set-aside posts fail a test instead of hanging it.
- `0fff45ef8` The S4 section of `phase-6-gate.md`, `## S4 close` of `phase-6-pinning.md`, and
  `rust/tsan.supp`.

## TSan

Command (recorded in `phase-6-gate.md` S4 close, "Criterion 3"), from `rust/`, nightly
`rustc 1.100.0-nightly (4aa1fbcf4 2026-09-08)` with its installed `rust-src`. Nothing was
installed.

```
export CARGO_TARGET_DIR=<own dir> RUSTFLAGS="-Zsanitizer=thread"
export TSAN_OPTIONS="log_path=<dir>/tsan suppressions=$PWD/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1"
T="cargo +nightly test -Zbuild-std --target x86_64-unknown-linux-gnu"
$T -p rexx-api --lib
DEPTH="recursion deep eval_limit max_eval_depth the_stack_span a_stack_within_the_margin \
  nested_pinned_waits_are_bounded notifier_failures_too_many_to_nest the_translator_on_a_pool_thread"
$T -p rexx-exec --lib -- $(for t in $DEPTH; do echo --skip $t; done)
$T -p rexx-exec --test signals --test stdin_contention --test program_end --test concurrency_tests \
  --no-fail-fast -- --skip only_the_interpreter_and_its_waits_take_the_halting_signals
```

The script is `p6-scratch/t22/tsan-final.sh`. The result at `b8ec39593` with `tsan.supp` as
committed was the same in each of 10 runs:

- every command exited 0, and no `tsan.*` log was written;
- rexx-api lib 70 passed;
- rexx-exec lib 986 passed, 16 filtered;
- `concurrency_tests` 32;
- `program_end` 2;
- `signals` 36, 1 filtered;
- `stdin_contention` 1;
- 1127 in all.

I ran the whole rexx-exec lib suite, not only the scheduler modules. Every lib program runs on an
interpreter thread with the timer thread live, so all of it exercises the baton and the timer.

Live controls:
- Before `allocator_may_return_null`, the `hold_buffer` races below produced a log in the
  configured `log_path`. So the logging reaches a file.
- `rexx-run` children honour `log_path`. `sigint_ends_a_parse_pull` with `verbosity=1` left 3
  `Running under ThreadSanitizer` logs: the test binary, plus a run and a rerun of `rexx-run`.
- The TSan `rexx-run` is instrumented: its threads include TSan's background thread (below).

Each report triaged:

1. **Data race, `rexx_api::load::hold_buffer` against `BufferBytes::extend_from_slice` and
   `try_reserve_exact`.** Found in `scheduler::tests::lent`, in
   `a_buffer_changed_in_place_under_a_call_stays_shared` and
   `a_buffer_grown_under_a_call_keeps_the_storage_the_call_holds`. **False positive.** The test
   orders the activity's buffer change before the routine's access through a file: the activity
   writes the file after the change, and `HOLDBUFFER` polls for it. TSan does not model ordering
   through the filesystem. Suppressed with `race:rexx_api::load::hold_buffer`, a test routine only.
2. **Data race in `free` inside glibc `_dl_close_worker`.** It comes from `Library::close` during
   `Interp::terminate`, on two interpreter threads of the same test process. It appeared once, at
   `dispatch::library::tests::a_library_call_answers_the_same_under_a_collection_at_every_allocation`,
   in the second of the first three runs before this suppression. **False positive.** Both sides
   run inside `dlclose` under ld.so's `dl_load_lock`, which ld.so takes internally, where TSan
   does not see it. Suppressed with `race:_dl_close_worker`.
3. **SEGV inside TSan** (`__tsan_func_entry`, `__tsan::CurrentStackId`). Caught under `gdb` with
   `handle_segv=0`:
   - an `eval::tests` depth test ran on an interpreter thread at 65,551 frames, 65,526 of them
     `Plan::note`;
   - `scheduler::tests::pool::callback_recursion_on_a_pool_thread_reaches_the_depth_cap` ran on a
     pool thread at 141,427 frames.

   **Not a race.** TSan's shadow call stack holds 65536 frames. The crash is not deterministic per
   test: `a_native_chain_past_the_eval_limit_runs` and the pool recursion tests passed when run
   alone, and crashed in full runs.

   Excluded, together with every test that measures recursion against the native stack (`DEPTH`).
   TSan's larger frames also change what those tests measure:
   `nested_pinned_waits_are_bounded_by_the_stack_remaining` hit Error 11 at a smaller depth than it
   asserts. The pool's other tests still run under TSan (13 in `scheduler::tests::pool`, less the
   excluded ones).
4. **`only_the_interpreter_and_its_waits_take_the_halting_signals` fails under TSan.** It counts
   one more `rexx-run` thread with the halting signals blocked. **TSan's own background thread.**
   In a TSan `rexx-run`, `/proc/<pid>/task/*/status` shows a second thread named `rexx-run` with
   `SigBlk: fffffffe3ffbea07`. This crate's threads show `0000000000004003`. Excluded.

**loom.** I did not rerun it. The lead's G9 at `52b038a80` reports `15 passed ... finished in
371.56s`, with the registration models under `preemption_bound = Some(5)` (`tests/loom.rs:143`).
`2e6917afe` changes only the shipped `Wake`; the loom `Wake` model is untouched. The loom tests
build at `b8ec39593`: `RUSTFLAGS="--cfg loom" cargo test -p rexx-exec --test loom --no-run`,
exit 0.

## Pinning report

`phase-6-pinning.md` `## S4 close`. At `b8ec39593`:

```
RAYON_NUM_THREADS=4 CARGO_TARGET_DIR=<own dir> memcap 8G cargo test --release -p rexx-exec \
  --features pinning --test concurrency_tests -- measured:: --nocapture --test-threads=1
```

Exit 0, 18 passed, in 120.19 s. The results match the S3 close in both modes:

- **Arrivals per park:** identical to the S3 close.
- **Waits-by-kind tables:** identical in both modes. I checked this with `diff` against the S3
  tables in the file.
- **No parks of the S4 kinds:** no `NativeCall` and no `Command` park.
- **No immovable `REPLY` and no inverted wait.**
- **Outcomes:** the only outcome difference between the modes is `MutexSemaphore`
  TEST_EXCLUSION (P46).

The same run at `52b038a80`'s source gave the same figures.

**The `Input` kind.** The brief says a stdin read reports a pinned park of kind `Input` (fix
round 3). That no longer holds. `4cf934a48` (fix round 5, P69) removed `ParkKind::Input` and
`ParkReason::Input`/`Stdin`. A read of the default input stream keeps the baton and idles on the
inbox, so it is not a park, and the report and the pinning gate have nothing to accept or
classify. The pinning record says so, with the commit.

## S4 section

`phase-6-gate.md` `### S4 close` follows the existing Task 18 S4 material and covers:

- **Background gates:** status file quoted verbatim, sums, and the G4 failure.
- **The regression and its fix:** a table of bisect points.
- **Criteria 1, 2, 3 and 9**, plus the gate-only `outer_context` result.
- **The TSan command:** result and triage table.
- **loom.**
- **Rulings P50-P69:** P66/P68 withdrawn by P69.
- **DEVIATIONS rows 9-13**, beside the Task 18 migration divergences already in the section.
- **Queued notes.**

One line is the lead's: `LEAD: the G4 disposition ... goes here.`

### The G4 failure is a real regression, now fixed

G4 failed at `base/rexxutil/SysSleep` TEST_SLEEP_CONCURRENT. G6 passed it only because both modes
failed it alike: `ours Failures 1`.

I first told the lead it was probably load from my TSan runs. **That was wrong**, and I corrected
it in a second message.

The probe is `call time 'r'; call SysSleep 0.3; say time('e') - 0.3`. Its overshoot:

| tree | overshoot, s |
|---|---|
| `7266ae03c` | 0.0001 |
| `a48312f8e` | 0.0001 |
| `532bf29fa` | 0.016-0.020 |
| `52b038a80` | 0.015-0.024 |
| oracle | 0.0002 |

The cause is the timer's `Wake::wait`. It blocked in a socket read with `SO_RCVTIMEO`, which the
kernel runs on its timer wheel.

The fix in `2e6917afe` arms a timerfd and polls it beside the socket. Its overshoot is 0.00008 to
0.00016 s, and the 4-activity version (4.99, 3, 2.4 and 1.333 s) matches.

A plain `ppoll` timeout was my first attempt. It was 1.3-1.7 ms late, because this environment
runs at niceness 5 and poll slack scales with the timeout. So the bound in the new test is 1 ms.

New tests in `sync.rs`:
- `a_timed_wait_ends_close_to_its_timeout`, best of three 300 ms waits under 1 ms late;
- `a_written_wake_ends_one_wait`.

Mutants, `cargo test --release -p rexx-exec --lib sync::`:

| mutant | result |
|---|---|
| socket read timeout | red, "19.711816ms late" |
| `ppoll` timeout | red, "1.535351ms late" |

After the fix, from `rust/`:

```
REXX_CORPUS_GATE=1 REXX_CRITERION_ONE_TABLE=<file> cargo test --release -p rexx-exec --test concurrency_tests -- group_runs::the_s2_rows
```

- exit 0 in 125.28 s, with no P48 rerun;
- the SysSleep row is `pass` / `same`;
- the other rows are G6's, except `REPLY` TEST_REPLY_TWICE_REPLYASSERT normal `differ: oracle 1,
  ours 0`. That is the oracle's own variation, covered by P41's table.

## Carries

### L-1

`tests/signals.rs` `a_stdin_read_waits_without_spinning` now does three things after writing the
input:
- polls `try_wait` up to `END_WAIT` (20 s);
- kills the process and asserts it ended;
- asserts exit status 0.

Mutants, each run with `cargo test --release -p rexx-exec --test signals --
a_stdin_read_waits_without_spinning`, both reverted by copying back a saved `input.rs`
(`git status` clean after):

| mutant | result |
|---|---|
| M1, `other => drop(other)` at `input.rs:389` | FAILED in 22.51 s: "rexx-run still running 20s after the read's input" |
| M2, `drop(kept)` for `self.timer.requeue(kept)` at `input.rs:357` | FAILED in 22.51 s, same message |

Unmutated, the test passes in 2.51 s.

### Task 19 concern 6

The gate-only `outer_context.rs` tests pass in both G4 and G6:
- `a_kept_outer_context_reaches_its_callers_variables`;
- `a_kept_call_context_used_by_another_activity_answers_or_raises`;
- `a_kept_thread_context_used_by_another_activity_does_nothing`.

`g4-test-release.txt:3405-3407`, 23 passed. G6 also 23 passed. `bggates.sh` runs G4 and G6 with
`REXX_CORPUS_GATE=1` (`p6-gates/bggates.sh:27`, `:31`). These are the 98.983 targets. The result
is recorded in the S4 section.

## Checks

On `b8ec39593`, in `p6-scratch/t22/target`:
- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace --release --no-run`: exit 0. Then `memcap 8G cargo test --workspace
  --release` in the foreground: exit 0, 144 `test result` lines, 3032 passed, 0 failed, 4
  ignored. That is 3030 before plus the 2 `sync` tests.

At `52b038a80` plus the L-1 edit, before the timer fix: 3030 passed, 0 failed.

All exit statuses were taken from `$?` of the unpiped command.

## Concerns

1. **G4 needs a disposition.**
   - G4 at `52b038a80` is red, and nobody has rerun it on the Task 22 head.
   - I ran the failing row's test alone on `b8ec39593`; it passed.
   - My all-core TSan runs overlapped G4: lead's load line 12.68 at start. That did not cause the
     failure: the regression reproduces at 0.3 s on a quiet machine. But G4's other wall-clock
     rows ran under that load.
2. **`2e6917afe` changes code at the stage close.**
   - It touches the timer's wake source, a cross-thread path, and adds rustix features `event`
     and `time` to an existing dependency. No new crate, and `Cargo.lock` is unchanged.
   - TSan is clean over it (the 10 runs are on `b8ec39593`). loom's model does not cover the
     shipped `Wake`, as before.
   - It needs review.
   - Also, nothing in the per-task bar would have caught the regression. It only shows in
     gate-only oracle rows, so Task 21's reviews could not see it.
3. **The TSan run excludes some tests.**
   - It excludes the depth-calibrated tests (`DEPTH`) and the thread-count test. So the pool's
     deep-recursion callback path (`callback_recursion_on_a_pool_thread_reaches_the_depth_cap`)
     is not race-checked at depth. Shallower pool callback tests do run.
   - `concurrency_tests` runs without `REXX_CORPUS_GATE`, so its oracle rows are not under TSan.
4. **Both suppressions are judgements from reading the stacks.** I did not test either by
   building an instrumented glibc or by reworking the `hold_buffer` handshake.
5. **The brief's premise about the `Input` park kind is out of date.** See Pinning report.
6. **A queued note may be missing.** progress.md says "Queued: chars/lines answer 1 after a read
   drained the pipe, oracle 0". I found no file for it under `.superpowers/sdd/queued/`; I
   searched for `drained`. It is not listed among the S4 queued notes.
7. **Scratch.** The directories `target`, `target-tsan`, `target-base`, `target-bisect`,
   `target-loom`, `bisect/` and `base-s3/` under `p6-scratch/t22/` are deleted. Logs and scripts
   (44 MB) remain there.

## Fix round 1

Base `75f8aeda8` (the lead's ledger commit `4f811a962` landed on top while this round ran). Brief
`task-22-fix1-brief.md`, from `task-22-review-timer.md`, `task-22-review-tsan.md` and
`task-22-review-gate.md`. Code commit `c66650b52`; record commit after it. Evidence the record
cites is in `s4-close-evidence/`.

### Code items

1. **T1, macOS.** The timerfd field and its arm/poll are under
   `#[cfg(any(target_os = "linux", target_os = "android"))]`; other unix targets poll the socket
   with the timeout (rustix rounds to whole milliseconds on `poll(2)`), capped at a day since
   `poll(2)` takes a C int. `cargo check --target aarch64-apple-darwin -p rexx-exec`: rc 101, the
   only error E0308 at `rexxutil.rs:154`. Mutant, both cfgs widened to `unix` / `not(unix)`: rc
   101 with E0425, E0432 and E0433 (x3) from `rustix::time` timerfd items beside the E0308.
2. **T2, T4.** The timer-slack sentence is gone; the doc says a wake, the timeout or an
   interrupted poll returns, and the caller recomputes.
3. **T3.** The rustix comment in `rexx-exec/Cargo.toml` names `event` and `time` and says "The
   four features".
4. **S1.** `HOLDBUFFER(buffer, key, mark)` marks `key` held under a static `Mutex`, then waits on
   a `Condvar` (up to 5 s) for `BUFFERCHANGED(key)`; the other activity polls `BUFFERHELD(key)`
   before changing the buffer and calls `BUFFERCHANGED` after. No files. The `hold_buffer` line
   is gone from `rust/tsan.supp`. Results:
   - TSan run at `c66650b52` clean (below), both `lent::a_buffer_*` tests ran.
   - Mutant M-S1, `BUFFERCHANGED` pushes nothing: both tests FAILED (`left: "unawaited\n..."`),
     5.03 s.
   - TSan control, `HOLDBUFFER` sleeping 2 s in place of the lock wait, suppression file as
     committed: exit 66, one `WARNING: ThreadSanitizer: data race`, frame #0 `read_volatile`
     (`s4-close-evidence/tsan/control-no-handshake.*`). So the lock is what orders the pair, and
     logging was live.
   Both files restored by copy and `cmp` after each mutant.
5. **S2.** `tsan.sh` skips the 13 depth tests by exact path (`--exact --skip`), and the signals
   test the same way. Lib result `990 passed; ... 13 filtered out`; the three tests the substring
   caught all ran and passed (`the_high_water_mark_is_the_deepest_the_stack_reached`,
   `a_routines_own_clauses_echo_at_indent_zero_however_deep_the_call_site_is`,
   `notifier_failures_too_many_to_nest_are_each_reported`, in `tsan/lib.txt`).
6. **S3.** `scheduler::tests::pool::bounded_callback_recursion_nests_under_one_lend`: 25 levels of
   `TestSendMessage0` callback recursion beside a started `idle` activity, asserting output
   `bottom\nidled\n`, `exits == 1`, `takes == 1`. Passes in release and under TSan. Mutant M-S3,
   the started activity removed (the call is then a lone one and keeps the baton, P43): FAILED,
   `left: 0, right: 1` on `exits`.

### Record items

7. **G1.** The blocking-operations command was rerun at `c66650b52` with the directory prefix
   stripped by `sed` (now part of the quoted command); the output block is that output, checked
   equal to a fresh run by a script. Every output line's file and line is in the table (checked by
   a script parsing the table against the block, no unclassified line and no stale table line).
   New classes: `input.rs:80` (keeps the baton, read on a pool thread, P69/P60/DEVIATIONS 12-13),
   `timer.rs:454` (the timer's own wait), `sync.rs:83`, `:127` (nonblocking wake I/O),
   `#[cfg(test)]` sites in `scheduler/pool.rs` and `dispatch/library.rs`, the `signal.rs`,
   `sync.rs`, `input.rs` test modules and the new `scheduler/tests/` files.
8. **G2, G3.** The `ADDRESS` row now follows P64 with the DEVIATIONS 12 fallback; `input.rs` has
   its own row per P69.
9. **G5.** Queued list: added `2026-10-05-stdin-chars-after-drain`, dropped
   `2026-10-03-interpret-translation-error-traceback`, and `send-site-cache-self-customization`
   is labelled a design note.
10. **G6.** All re-run at `c66650b52` (or across the trees for the probe) and cited:
    - `overshoot.sh` / `overshoot.rex` / `overshoot.log`: every tree rebuilt from `git archive`
      with a required `Compiling rexx-exec` line. Regression trees 0.011-0.020 s; the others
      under 1 ms. The figures differ from the bisect note's (the S3 and Task 20 trees and
      `c66650b52` read about 0.0004 s on this quiet run, `2e6917afe` 0.0001 s); one run per tree,
      not interleaved, so no ordering among the sub-millisecond rows is claimed.
    - `s2rows.sh` / `s2rows.log` / `s2-table.txt`: exit 0, 125.33 s, no P48 rerun, SysSleep
      TEST_SLEEP_CONCURRENT `pass` / `same` at `s2-table.txt:138`; every other row equals G6's
      table at `52b038a80` (script comparison).
    - `pinning.sh` / `pinning.log` / `pinning-diff.py` / `pinning-diff.log`: exit 0, `18 passed`,
      arrivals equal the S3 close's, no waits-by-kind diff in either mode; control (one count
      altered in a copy of the log) prints the row. Outcomes differ between modes only on
      `MutexSemaphore` TEST_EXCLUSION.
    - `tsan.sh` / `tsan/`: one run, below.
    - `loom.log`.
    - Criterion 2 and `outer_context` now cite G4 and G6 lines of the `52b038a80` logs.
11. **G7.** Dropped "prints 123 lines", "the ten more", "18 `measured::` tests" and "1127 in all";
    the counts left are quoted from a cited log.
12. **G8.** The 13 lib exclusions and the signals one are listed by exact path with a reason
    each, and a sentence says the pool callback recursion runs bounded, not at depth.
13. **G9.** (a) the REPLY cell is attributed to G4 (G6 and this run have `pass`); (b) P67 says it
    holds for one interpreter per process; (c) `tsan.supp` names `"S4 close", "Criterion 3, race
    checking"`.
14. **LEAD lines** untouched, as are criteria 1's `52b038a80` citations.

### Checks

From `rust/`, `CARGO_TARGET_DIR` under `p6-scratch/t22f1/`, at `c66650b52`, each exit taken from
the unpiped command:

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace --release --no-run`: exit 0. Then `memcap 8G cargo test --workspace
  --release`: exit 0, 144 `test result` lines, 3033 passed, 0 failed, 4 ignored (3032 before,
  plus the bounded recursion test). A first attempt without a fresh `--no-run` (mutant restores
  had touched sources) was OOM-killed by memcap while compiling; that is why the bar builds
  first.
- loom: `RUSTFLAGS="--cfg loom" cargo test -p rexx-exec --test loom`, own target dir: exit 0,
  `15 passed; 0 failed`, 358.99 s.
- TSan: `bash s4-close-evidence/tsan.sh <target> <logs>`: `api exit 0`, `lib exit 0`, `int exit
  0`, `no tsan log`; rexx-api 70, rexx-exec lib 990 (13 filtered), `concurrency_tests` 32,
  `program_end` 2, `signals` 36 (1 filtered), `stdin_contention` 1.

### Concerns

1. The S1 control and M-S1 run only the two `lent::a_buffer_*` tests; the TSan run itself is one
   run, not ten.
2. The overshoot rows are one run each in sequence; the sub-millisecond rows moved between this
   run and the bisect note's, as they do run to run.
3. The macOS check shows only that `rexx-exec` type-checks up to the pre-existing E0308; the
   non-Linux wait has not run anywhere.
4. Scratch `p6-scratch/t22f1/` is deleted.
