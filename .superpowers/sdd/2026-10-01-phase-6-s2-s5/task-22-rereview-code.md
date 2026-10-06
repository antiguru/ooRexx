# Task 22 fix round 1 re-review: code

Reviewer: s4-t22 code re-review. Diff `4f811a962..540e6a72e` (`review-t22fix1-code.diff`); `540e6a72e`
touches no file under `rust/`, so the code is `c66650b52`'s. Nothing in the checkout was changed.
Builds in `/tmp/claude-1000/p6-rr-code-target`; one `git archive` copy at
`/tmp/claude-1000/p6-rr-code-tree` for the macOS clippy run. Both deleted.

## Verdict

**APPROVED.** Every finding is fixed. Two new Minor findings, neither blocking.

## Verdicts per finding

- **T1 (macOS build): FIXED.**
  - `cargo check --target aarch64-apple-darwin -p rexx-exec` at HEAD: the only error is E0308 at
    `rexxutil.rs:154`, and no diagnostic names `sync.rs`.
  - E0308 stops type checking before borrowck and lints run, so I also checked a copy with that
    one line patched (`from_bits_truncate(mode as _)`):
    `cargo clippy --target aarch64-apple-darwin -p rexx-exec -p rexx-api --all-targets -- -D warnings`
    finished clean. The non-Linux `wait` therefore passes borrowck and clippy, not just name
    resolution.
  - Ms rounding: rustix 1.1.4 (the locked version) uses `poll` with `as_c_int_millis` on Apple and
    OpenBSD, which rounds nanoseconds up (`timespec.rs:202`, `+ 999_999`). Short timeouts therefore
    cannot become a zero-timeout spin. FreeBSD and NetBSD take `ppoll` with the `Timespec` as it is.
  - The day cap is load-bearing. Above `c_int` milliseconds, `as_c_int_millis` returns `None`, and
    rustix returns `EINVAL` (`syscalls.rs:208`). Without the cap, `wait` would return at once and
    the timer thread would spin. 86 400 000 ms fits a C int.
  - Deadline recompute and EINTR: an interrupted or capped `poll` returns, the nonblocking reader
    drains, and `run_timer` recomputes as before. On Linux the next `wait` re-arms the timerfd with
    `settime`, so a stale arm cannot carry over. The Linux arm/poll/read sequence is the reviewed
    one, moved unchanged under the cfg.
  - Not run: the non-Linux wait itself (no macOS host). Implementer concern 3 says the same.
- **T2 (timer-slack sentence): FIXED.** Deleted.
- **T4 (wait doc): FIXED.** `sync.rs` now says "Blocks until a wake, `timeout`, or an interrupted
  poll ... The caller recomputes its deadline on every return."
- **T3 (Cargo.toml rustix comment): FIXED.** It names `event` and `time` as the timer thread's
  `poll` and Linux `timerfd`, and "Both" is gone. See N2 for the replacement wording.
- **S1 (hold_buffer handshake): FIXED.**
  - Ordering: the activity changes the buffer, then `BUFFERCHANGED` pushes the key under
    `BUFFER_HANDSHAKE` and notifies. `await_buffer_changed` sees the key under the same mutex, and
    only then do the volatile read and write run. That gives TSan a happens-before edge.
    `hold_buffer` reads `MutableBufferData`/`Length` before it pushes `held`, so the length is still
    the pre-change storage's, as the SAFETY comment says.
  - Deadlock: none found. No guard is held across an interpreter callback. `held.push` is a
    statement-scoped temporary. The wait releases the mutex. `BUFFERHELD` and `BUFFERCHANGED` lock
    only briefly.
  - State between tests: keys are the per-test temp path (pid plus test name), so tests share no
    key. `held` is always cleared. A `changed` entry outlives its test only when `BUFFERCHANGED`
    arrives after a 5 s timeout, which happens only in a failing run.
  - Suppression: the `hold_buffer` line is gone from `rust/tsan.supp`. `tsan/lib.txt` shows both
    `lent::a_buffer_*` tests `ok` with `no tsan log`. The control log has exactly one
    `WARNING: ThreadSanitizer`, with frame #2 `hold_buffer::{closure#0}` at `load.rs:968`, so the
    lock is what orders the pair.
  - Release run: `lent::a_buffer_*` and the bounded test passed 10 of 10, in about 0.55 s each
    (well under the 5 s timeout).
  - Test-only? Called only from `scheduler/tests/lent.rs`, but compiled into the shipping
    `rexx-api` (see N1).
- **S2 (exact-path skips): FIXED.**
  - `tsan.sh` passes `--exact` and one `--skip` per `DEPTH` name.
  - `tsan/lib.txt`: `990 passed; ... 13 filtered out`. None of the `DEPTH` names appears as a
    `test ... ` line (checked per name). An exact skip filters at most one test, so each name
    matched exactly one test and nothing else was filtered.
  - The three tests the substring used to catch (`the_high_water_mark_is_the_deepest_the_stack_reached`,
    `a_routines_own_clauses_echo_at_indent_zero_however_deep_the_call_site_is`,
    `notifier_failures_too_many_to_nest_are_each_reported`) appear `ok`.
  - The totals reconcile: 986 + 16 before, 990 + 13 now, which is one more test (S3's).
- **S3 (bounded recursion test): FIXED.**
  - `pool.rs` `bounded_callback_recursion_nests_under_one_lend` recurses through
    `.r~new(25)`/`left`. It asserts exit 0, stdout `bottom\nidled\n` (`bottom` proves the bottom
    level was reached), `exits == 1` and `takes == 1`.
  - `exits` and `takes` come from `#[cfg(test)]` thread-locals read on the test's own interpreter
    thread, so tests running in parallel cannot mix their counts.
  - It ran under TSan (`tsan/lib.txt`, `ok`) and passed 11 of 11 in release here.
  - The implementer's mutant (no started activity) failed on `exits`, so the witness is live.

## New findings

### N1. Minor: `BUFFER_HANDSHAKE`/`BUFFER_CHANGED` are process-wide mutable statics in the shipping `rexx-api`

- **Where:** `rust/crates/rexx-api/src/load.rs:892-897`.
- **Issue:** they are not `#[cfg(test)]`. They cannot be, because the callers are in another
  crate's tests. So the library build carries a mutable global that is neither the registry nor
  the timer thread. Before this round, `load.rs` had no such static; its other test routines keep
  state in files and arguments.
- **Mitigation:** the brief proposed exactly this shape ("a `static` Mutex/Condvar"). It is
  const-initialised, allocates nothing until a test routine runs, and no production path reaches
  it.
- **Fix, if the rule is to hold by the letter:** either record it as a test-support exception
  where the globals rule is stated, or gate the test routines behind a `rexx-api` feature that
  `rexx-exec`'s `[dev-dependencies]` enables. The same feature could also gate `hold_c_string` and
  its siblings.

### N2. Minor: count in prose in the rustix comment

- **Where:** `rust/crates/rexx-exec/Cargo.toml:65`, "The four features are leaves here".
- **Issue:** this states a set's size where naming the set is enough, so it goes stale the next
  time a feature is added. That is exactly how "Both" went stale (T3).
- **Fix:** "Each feature is a leaf here -- it gates its module and pulls no further crate."

## Constraint checks

- `unsafe`: new blocks are only in `load.rs` (allowed). `sync.rs`, `lent.rs` and `pool.rs` add none.
- Dependencies: none added. The rustix feature list is unchanged by this round.
- Em-dashes: none in added lines.
- Forward-looking prose: none. The `pool.rs` doc sentence "ThreadSanitizer runs this one, not the
  deep ones" describes `tsan.sh` as it is now.

## Out of scope

- On FreeBSD and NetBSD, rustix uses `ppoll` with a `timespec`. So the comment's reason ("`poll(2)`'s
  timeout is a C int of milliseconds") is true for Apple and OpenBSD only. The cap is harmless
  everywhere, and the comment is not false for the targets it was written for. No change needed.
- Under heavy load, the S3 test depends on `idle` (`SysSleep 0.5`) still being live when the first
  `send0` is made. If it is not, the call keeps the baton and `exits` reads 0. 0.5 s is five times
  `DEEP`'s margin. It held 11 of 11 here and once under TSan. This is noted as a flake surface, not
  a defect.
