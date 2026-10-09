# Task 11 report: the unjudged mutants

Base `118d78b5e`. Work commit `8910306e5`. Paths below are under `rust/crates/rexx-exec/`, and line
numbers are at `8910306e5`.

## Sites

Each site was re-derived from scout C's mutant table (`scout-c-report.md` section 4). The edits are
in `/tmp/claude-1000/p61/t11/bin/mutate.py`, one exact-text edit per mutant, asserted unique and
reverted by the inverse edit.

| mutant | edit | site |
|---|---|---|
| M6 | delete `self.timer.requests().clear(SLICE);` | `switch_to`, `src/scheduler.rs:1017` |
| M7 | delete the `self.slice_deferred = false;` after it | `switch_to`, `:1018` |
| M9 | delete `self.activity.when_parked = false;` | `cancel_wait`, `:1186` |
| M10 | delete `self.activities.guards.withdraw(running);` | `cancel_wait` |
| M11 | delete the `sleepers.retain` | `cancel_wait`, its last statement |
| M12 | delete `out.extend(failed_sends.iter().copied());` | `Activity::object_roots`, `src/activity.rs:558` |

The carry's two `sleepers.retain` candidates are not M11's. `:1361` is in the timer post path and
`:1377` is in `wake_semaphore_waiter`. Scout C's M11 is "`cancel_wait` keeps the sleeper", the
`retain` inside `cancel_wait`, which is also the one Task 10 reverted. `failed_sends` is still
rooted, at `activity.rs:558`.

## Instruments (`cfg(test)` only)

* `Interp::note_cancelled_wait` (`src/scheduler.rs:1211`) is called first in `cancel_wait`. It counts
  in the thread-local `CANCELLED_WAITS` what the cancelled wait finds to withdraw from: a `GUARD
  WHEN` park, a guard lock's queue (`Guards::queues`, added in `src/guards.rs`) and the sleepers.
  It reads its counts before any withdrawal, so a mutant deleting a withdrawal leaves the count
  unchanged. Each count is a reach assertion, not the judge.
* `set_native_entry` (`src/environment.rs:1245`) counts every write, and every write to a handle
  `heap.get` resolves to nothing, in the thread-local `NATIVE_ENTRY_WRITES`.

Each counter is a `#[cfg(test)]` statement or item. `cfg(test)` is set only when this crate's own
unit tests are compiled, so no release or integration-test build contains the code.

## Tests

* `src/scheduler/tests/mutants.rs` is a new submodule of `scheduler/tests.rs`. Each run there is on
  an interpreter thread of its own, so the thread-local counts belong to that run.
  * `a_failed_pinned_sleep_leaves_no_deadline_to_end_a_later_wait` (M11) runs the seeded gate's own
    `tests/sim_gate/m11_stale_sleeper.rex` through `include_str!`, under `sim:1,order=fifo,fail=wait:1`,
    `sim:1,uniform:1,fail=wait:1` and `sim:2,uniform:1,fail=wait:1`. It asserts rc 0, stdout
    `caught 11.1` / `result slow done after 1`, empty stderr, and `sleeps == 1`.
  * `a_failed_guard_when_leaves_no_park_for_a_halt_to_end` (M9): a `GUARD WHEN` reached through
    `INTERPRET` fails under `fail=wait:1` and is trapped, then main parks on a started method's
    result while `halt@K` halts both activities. It runs five specs (fifo, uniform:1 at two seeds,
    `pre:2,k=40`, `uniform:0.3`) and asserts `caught 11.1` / `result slow halted` / `main halted`,
    rc 0, and `when_parks == 1`.
  * `a_failed_guard_lock_wait_leaves_no_place_in_the_queue` (M10): `hold` keeps `o`'s guard through a
    2 s sleep, and main's `touch` through `INTERPRET` fails its guard-lock wait under `fail=wait:1`
    and is trapped. After `hold` ends, a third activity sends `touch`. It runs three specs and
    asserts `caught 11.1` / `held` / `later touched`, rc 0, and `guard_waits == 1`.
  * `a_dropped_failed_send_is_written_while_alive` (M12): `.t~new~start('boom')` is dropped at once
    and `boom` divides by zero, under `gc=1` and `gc=0.1` at seeds 1 to 3. It asserts that the failed
    run makes exactly one write more than the same program with a succeeding `boom` (the condition
    reached the message), and no write to a dead handle.
* `tests/concurrency_tests.rs` `measured::a_slice_deferred_before_a_pinned_park_stays_with_its_activity`
  (M7, `--features pinning`) runs under `EveryOpportunity` and needs no sim. One clause starts B
  (`sortWith` on a 3-array with a one-clause comparator) and C (prints three lines), then sorts
  main's array with a comparator whose only clause is `.m~result`. Main defers a slice there and
  parks pinned, and B's first comparator boundary is pinned with C ready. The test asserts stdout
  `c ran 1` / `done` / `c ran 2` / `c ran 3`, rc 0, and these counts:
  * deferred `[TreeSend, Native("SORTWITH"), SortComparator]` 1, main's deferral, the reach;
  * deferred `[Native("SORTWITH"), SortComparator]` 2;
  * pinned yields `[Native("SORTWITH"), SortComparator]` 1.

  With B's comparator at two clauses (scout C's shape) the per-key totals agreed under M7. Only
  the output moved, by one clause. B makes an even number of pinned visits there, and they
  alternate defer/yield in either phase. The one-clause comparator makes the number odd.

## Mutant table

Unmutated, the four lib tests passed, and the M7 test and the debug seeded gate (`sim_gate::`, 11
tests) passed. Each mutant ran under the two commands below, with one `Compiling rexx-exec` line in
each log and the edit reverted afterwards. `git status` was clean after the last revert.

```
REXX_CORPUS_GATE=1 memcap 8G timeout 1500 cargo test -j 4 -p rexx-exec --no-fail-fast --lib scheduler::tests::mutants
REXX_CORPUS_GATE=1 memcap 8G timeout 1500 cargo test -j 4 -p rexx-exec --features pinning --no-fail-fast --test concurrency_tests -- measured::a_slice_deferred_before sim_gate::
```

Driver: `/tmp/claude-1000/p61/t11/bin/runmut.sh`. Logs: `/tmp/claude-1000/p61/t11/mut-<M>-{lib,conc}.txt`.

| mutant | site | judge | unmutated | mutated | verdict |
|---|---|---|---|---|---|
| M6 | `switch_to`, SLICE kept | none (outcome judges) | all green | all green: lib 4/4, conc 11/11 | equivalent (below) |
| M7 | `switch_to`, `slice_deferred` kept | `a_slice_deferred_before_a_pinned_park_stays_with_its_activity` | deferred 2, yields 1 | deferred `[SORTWITH, SortComparator]` 1 where 2 expected (yields 2), conc exit 101; lib green | killed |
| M9 | `cancel_wait`, `when_parked` kept | `a_failed_guard_when_leaves_no_park_for_a_halt_to_end` | `caught 11.1 / result slow halted / main halted`, rc 0 | `sim:1,uniform:1,fail=wait:1,halt@20`: rc 120, `the scheduler found a ready activity holding a park reason`; lib exit 101; seeded gate green | killed |
| M10 | `cancel_wait`, guard-queue entry kept | `a_failed_guard_lock_wait_leaves_no_place_in_the_queue` | `caught 11.1 / held / later touched`, rc 0 | `sim:1,order=fifo,fail=wait:1`: rc 120, two 98.905 deadlocks, `the scheduler found a guard waiter with no wait recorded for its guard`; lib exit 101; seeded gate green | killed |
| M11 | `cancel_wait`, sleeper kept | `a_failed_pinned_sleep_leaves_no_deadline_to_end_a_later_wait`; also `the_seeded_gate` | `result slow done after 1`, rc 0 | `result The NIL object after 0`, rc 0; lib exit 101; the seeded gate red (`sim gate: 1792 runs, ... 12 reds`) | killed |
| M12 | `object_roots`, `failed_sends` dropped | `a_dropped_failed_send_is_written_while_alive` | writes to dead handles 0 at every seed and Q | `sim:1,gc=1`: 1 write to a dead handle, lib exit 101 (probe: 1 at every seed 1-3 and Q in 1, 0.5, 0.1) | killed |

Probe runs before the final tests (`/tmp/claude-1000/p61/t11/probe-*.txt`) showed more:

* M9 under `order=fifo` printed the unmutated output. The halt's stale unpark of main is undone
  by `park_native`'s retest loop, which parks it again, and no switch sees main ready. Main is seen
  ready, and the mutant is killed, only under preemption. At `sim:2,pre:2,k=40` and
  `sim:3,uniform:0.3` (halt@20, halt@60) the mutant answered `result The NIL object` with rc 0 and
  no invariant breach. At uniform:1 every seed was a refusal.
* M10 was killed at every seed and policy tried, fifo included.

### M6: equivalent

The argument is scout C's. In every switch mode, `serve_requests` clears a fresh `SLICE` before it
yields. A `SLICE` still pending at `switch_to` can therefore only come from the timer thread
between countdown visits. The incoming activity then yields at its first visit, and that schedule
is one the timer could produce anyway, so no outcome leaves the legal set. M6 was green under every judge above. Only a slice-accounting instrument could
kill it, and none was built (recommendation 8).

### M12: the UNINIT variant

The brief's second program, a `Message` subclass with `UNINIT`, cannot be built in this crate. The
oracle prints `main` / `end` / `uninit` and rc 0 for `.mymsg~new(.t~new, 'boom')`, where `mymsg`
subclasses `Message` and defines `UNINIT`. This crate refuses it at rc 120: `method "NEW" of class
"MYMSG" is not implemented (Phase 9)`. The other route, `.message~enhanced(d, .t~new, 'boom')`
with `d['UNINIT']`, gets `uninit` from the oracle. Here it raises 93.902 from `Object`'s `INIT`,
rc 163, which is the queued "`Class~enhanced` skipping `NEW`" item (spec T9 Close). `setMethod` on
the message is 97.2 (private) on both. So M12 is judged on the plain dropped send, which the
counter kills.

## Defect found

At base, `tests/concurrency_tests.rs` did not compile under `--features pinning`. `pinning_table`
moved its `Option<SwitchMode>` into a rayon `Fn` closure (E0507 at `:1588`). Task 9 introduced it
by removing `Copy` from `SwitchMode`, and neither the default test run nor clippy compiles that
module. It was reported to the controller and fixed with `mode.clone()` in `8910306e5`. The
module's 19 tests then passed (`REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --features
pinning --no-fail-fast --test concurrency_tests measured::`, exit 0, 19 passed). The scheduler
showed no defect: the unmutated runs answered as asserted under every spec tried.

## Per-task check (at `8910306e5`'s tree)

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets --features pinning,sharing -- -D warnings`
  (the controller's added line): exit 0, no other breakage.
* `memcap 8G cargo test -j 4 --workspace --no-fail-fast` (debug): exit 0, 3126 passed, 0 failed, 4 ignored (`/tmp/claude-1000/p61/t11/ws.txt`).
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test ir_recorded_oracle`:
  exit 0, corpus 29 passed 1 ignored, ir_recorded_oracle 21 passed
  (`/tmp/claude-1000/p61/t11/gate.txt`).
* No `Loud` constructor was added, so `refusal-sites.tsv` needs no re-derivation.
