# Task 22 review: the timer fix (2e6917afe)

Reviewer: t22rev-timer (opus). Tree: HEAD 75f8aeda8. Build: `CARGO_TARGET_DIR=p6-scratch/t22rev-timer/target`.

## Verdict

**Changes required.** One Medium, two Low, two Info.

The wait is correct, and every behavioural check passed when run. The one blocking item is finding
1: the commit silently adds five compile errors on macOS, with no cfg fallback and no recorded
limitation. Findings 2 and 3 are false or stale prose to delete or fix in the same round.

## Checks run

1. **Wake latency.** Probe: `call time 'r'; call SysSleep 0.3; say time('e') - 0.3`, run from
   an empty dir with release `rexx-run`. Overshoot in seconds, 20 runs per column, interleaved
   per iteration, load average 11-14:

   | | min | median | max | runs > 1 ms |
   |---|---|---|---|---|
   | ours, default | 0.000113 | 0.000146 | 0.001181 | 1 |
   | ours, `REXX_SWITCH_MODE=every` | 0.000104 | 0.000137 | 0.001246 | 1 |
   | oracle | 0.000174 | 0.000231 | 0.003906 | 1 |

   The single runs over 1 ms are load tails, and each side has one. See finding 5.
   - The four-sleeper repro (4.99, 3, 2.4, 1.333 s, started farthest-first) ran 5 times per
     mode: 0 bad, max overshoot 2.9 ms (one 4.99 s row in every mode), all others ≤ 0.53 ms.
   - `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test concurrency_tests
     group_runs::the_s2_rows`, run 3 times: rc 0 each time, 125.2-125.5 s, `1 passed`. That
     test runs the whole `base/rexxutil/SysSleep.testGroup` (TEST_SLEEP_CONCURRENT and
     TEST_SLEEP_DURATION) in both modes, so each test is 3/3 in each mode. I did not do 10
     direct driver runs; the brief allowed 3 gate runs.
2. **Correctness of the new wait** (`sync.rs:87-113`, `timer.rs:439-456`).
   - **Re-arm and disarm.** Every `wait` call `timerfd_settime`s before polling. A `None`
     timeout arms a zero `it_value`, which disarms the timer, and `settime` clears the expiry
     count, so a stale expiry cannot cut the next wait short. A nearer deadline arrives as a
     socket byte, `wait` returns, and `run_timer` recomputes and re-arms. Demonstrated by the
     farthest-first four-sleeper repro above, with 0 bad.
   - **No early wakes.** A relative `CLOCK_MONOTONIC` arm made after `now()` cannot expire
     before the deadline. A past deadline saturates to `ZERO`, which returns before arming. Run
     with SysSleep 0, 0.000001, 0.001 and 0.0105: every row has `e >= d` and `e < d + 2 ms` in
     both modes, and the output is identical to the oracle's.
   - **A socket byte wakes at once.** `a_written_wake_ends_one_wait` passes. The four-sleeper
     repro depends on it too.
   - **Timer exits and respawns without leaking fds (P49).** The `Wake` lives in the static
     `Registry` (`timer.rs:268`), so there is one timerfd per process. Demonstrated with a
     temporary integration test, since deleted. It ran 300 `run_program` calls in one process
     (a `start`ed SysSleep plus a main SysSleep), and after each run waited until no
     `rexx-timer` thread was left in `/proc/self/task`. Result: `fds before 7 after 7, timer
     exits observed 300`.
     - To prove the instrument live, I mutated `wait` to leak one timerfd per call. The same
       test went red: `left: 12, right: 1509`.
   - **EINTR.** `poll`'s error is ignored. Both reads are nonblocking (the reader since this
     commit, the timerfd by `TFD_NONBLOCK`), so `wait` returns and `run_timer` recomputes.
     Demonstrated by a temporary test, since deleted. It installed a `SIGUSR2` handler without
     `SA_RESTART` and `tgkill`ed the `rexx-timer` thread every 7 ms while the program ran
     `do 5; call time 'r'; call SysSleep 0.3; say time('e'); end`. Result: 213 signals reached
     the timer thread, and the five sleeps took 0.300082-0.300123 s. (P67 masks only
     INT/TERM/HUP on the timer thread, which is why `SIGUSR2` was used.)
3. **loom.** `RUSTFLAGS="--cfg loom" cargo test -p rexx-exec --test loom`, separate target dir,
   at 75f8aeda8: `15 passed; 0 failed`, finished in 449.92 s, rc 0. The loom `Wake`
   (`sync.rs:131-173`) is unchanged and still models the shipped source faithfully:
   - A notify before a wait is kept (flag versus socket byte).
   - A wait takes all pending wakes.
   - Timeouts and spurious returns are not modelled. The shipped `wait` returns spuriously on
     EINTR or a timer expiry, as the `SO_RCVTIMEO` version did. `run_timer`'s loop recomputes
     from state on every return, so not modelling it hides nothing new.
4. **The witness goes red on the SO_RCVTIMEO version.** Mutant: the `wait` body replaced by
   `set_nonblocking(false); set_read_timeout(timeout); read`, which is 52b038a80's shape (the
   reader has to go back to blocking for the timeout to apply). Command: `cargo test --release
   -p rexx-exec --lib sync::`. Result: rc 101, `a_timed_wait_ends_close_to_its_timeout` panicked
   with `18.992705ms late`, and `a_written_wake_ends_one_wait` stayed ok. `sync.rs` was restored
   from a byte copy checked against `HEAD` with `cmp`, and `git diff --quiet HEAD -- rust` is
   clean. Unmutated, both tests pass (rc 0, 0.90 s).
5. **Portability.** `cargo check --target aarch64-apple-darwin -p rexx-exec`: rc 101, 6 errors.
   Finding 1.

## Findings

### 1. Medium: macOS gains five compile errors, unrecorded

- **Where:** `rust/crates/rexx-exec/src/sync.rs:66-68` (`timerfd_create`, `TimerfdClockId`,
  `TimerfdFlags`) and `:89` (`Itimerspec`, `TimerfdTimerFlags`, `timerfd_settime`).
- **Demonstrated by running.** `cargo check --target aarch64-apple-darwin -p rexx-exec` reports
  E0432 at `sync.rs:89`, E0433 ×3 at `sync.rs:67-68` and E0425 at `sync.rs:66`, beside the
  existing E0308 at `rexxutil.rs:154`. The Task 21 re-review records E0308 as the only error
  (`task-21-rereview-1.md:44`). Task 21 F9 was fixed to keep `signal.rs` building on Apple and
  the BSDs (`signal.rs:102-116`).
- **Failure scenario:** once E0308 is fixed, macOS still does not build, and nothing in the tree
  says why. rustix's timerfd is Linux-only, and none of these mention it: the commit message,
  the `Cargo.toml` comment, the gate section (`phase-6-gate.md:679`) or DEVIATIONS.
- **Fix:** `#[cfg(any(target_os = "linux", target_os = "android"))]` on the timerfd path, and
  for other unix targets one of:
  - the `poll` timeout (the slack is about 0.1-0.5% of the timeout, measured in the bisect
    record);
  - kqueue `EVFILT_TIMER`;
  - or a recorded limitation in the gate section, plus a `compile_error!` naming it.

### 2. Low: the `wait` doc names a mechanism that does not exist

- **Where:** `sync.rs:84-86`, "The deadline is a timerfd's, which expires within the thread's
  timer slack."
- **Demonstrated by running.** A Python probe (timerfd + poll, 300 ms, 5 waits) gave:
  - `PR_SET_TIMERSLACK` 20 ms: overshoot 0.05 ms in 4 of 5 waits, and 1.7 ms in one under load;
  - the default 50 µs slack: the same figures.

  A timerfd's hrtimer is started with no slack range, so the thread's slack does not apply.
  That is why the fix works where the `poll` timeout did not.
- **Failure scenario:** a reader who believes the sentence would tune `PR_SET_TIMERSLACK` to
  fix lateness, or would treat poll's slack and the timerfd's as the same thing.
- **Fix:** delete the sentence.

### 3. Low: the `Cargo.toml` comment on `rustix` is stale

- **Where:** `rust/crates/rexx-exec/Cargo.toml:61-68`. The comment explains `fs` and `system` and
  says "Both features are leaves here". The line under it now enables
  `["event", "fs", "system", "time"]`.
- **Found by reading.**
- **Failure scenario:** the comment is now false. Its "Both" no longer covers the four features,
  and `event` and `time`, which carry the Linux-only timerfd (finding 1), go unexplained.
- **Fix:** add one clause, as the bisect record's proposed diff did: "`event` and `time` are the
  timer thread's `poll` and Linux `timerfd`". Then fix "Both".

### 4. Info: `wait` returns early on EINTR, contrary to its doc

- **Where:** `sync.rs:84`, "Blocks until a wake or `timeout`", and `:109`, where `poll`'s error is
  discarded.
- **Behaviour:** on EINTR, or on any other `poll` error, `wait` returns early. The
  `SO_RCVTIMEO` version did the same. The only caller, `run_timer`, loops and recomputes, so
  nothing observable follows.
- **Demonstrated by running:** 213 SIGUSR2s gave no early or late sleep (check 2).
- **Theoretical case:** a persistent `poll` error such as ENOMEM would spin the timer thread
  instead of blocking. Not demonstrated.
- No change required.

### 5. Info: under heavy load, the tail looked longer than the oracle's, but not when interleaved

- **Run sequentially at load average 26** (loom plus another reviewer's builds), 20 runs each:
  - ours, default: 5 runs over 1 ms (max 3.3 ms);
  - ours, every: 6 runs over 1 ms (max 2.9 ms);
  - oracle, run after both: 1 run over 1 ms.
- **Interleaved at load 11-14:** 1 run over 1 ms on each side (check 1).
- Ours takes two thread hops (timer thread, then the interpreter's inbox), where the oracle's
  sleeping thread wakes itself, so a longer tail under load is plausible. But the sequential
  runs are not a controlled comparison, and the interleaved runs show no difference. Not a
  defect. All values stay far inside SysSleep's 26 ms.

## Scratch

`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/p6-scratch/t22rev-timer/` is deleted,
including its target dirs.
