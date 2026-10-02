# Task 11 report: guard locks with deadlock detection

Commits: `9f5c68ed5` (implementation, witnesses, gate cell, refusal table, queued note removed),
`457a292e8` (one criterion-1 allowance), and this report's commit. Base `b62155373`.

## Design decisions

- **Table.** `guards.rs`: `GuardTable` keyed by `GuardKey { object, scope }` (receiver identity and
  method scope): owner `ActivityId`, nesting count, FIFO waiters; plus `waiting`, what each parked
  activity waits on (a guard key or a `MessageId`), and the lazily numbered pools for
  `ATTRIBUTEPOOL`. It lives on `Activities`. `ParkReason::Guard(GuardKey)`.
- **Rexx method entry.** `begin_method` reads `guarded` from the directive (`::METHOD`/`::ATTRIBUTE`
  without `UNGUARDED`; `.Method~new` bodies are guarded; `::CONSTANT` is not) and reserves at once
  when the lock is free or its own. Contended, it records a `GuardWait` for the activation and sets
  the countdown to 1; the method's first clause boundary (`serve_requests` -> `serve_guard_wait`)
  then runs the deadlock check, queues the activity, and ends the slice there (not put back on the
  ready queue; `release` readies it with the lock at count 1). Under a pinned frame the same point
  runs `pinned_wait`. So nothing of the body runs before the lock is held, and no `Op::Clause`
  branch was added (the check is in the cold countdown path).
- **Early `>I>`.** A method whose first instruction past an `EXPOSE` is a label-tracing `TRACE`
  has its entry traced before the reserve in the oracle (`RexxActivation::traceEntry`). Such a
  method (`Plan::traces_entry_early`, computed once per body) defers its reserve to the clause
  boundary after its `>I>`, so the `>I>` TraceObject reads the lock as before the reserve and the
  `>I>` comes before a wait.
- **Release.** `finish_call` releases before `<I<`, as `RexxActivation::termination` does
  (`guard_off_at_end`), unless a `REPLY`'s continuation takes the activation.
- **REPLY.** `spawn_continuation`: nesting count 1 moves the lock to the continuation's handle;
  above 1 it releases one level and the continuation reserves again before it resumes
  (`reserve_for_continuation`, parked if contended). No deadlock check on that reserve (concern 2).
- **Other guarded methods.** Generated getter/setter, `DELEGATE` (lock held only while reading the
  target), and guarded `EXTERNAL` methods (set `guarded_externals` at install) reserve in
  `begin_invoke_other`. Contended, the send is kept on the activity (`GuardedSend`) and parks
  through `park_native`; its resume runs it with the lock and releases. Built-in (CPPCode-like)
  natives never reserve, as the oracle's `CPPCode::run` does not.
- **GUARD ON/OFF.** Needed because the library's `Alarm` and `Ticker` use `GUARD OFF` before
  `REPLY`: with the implicit reserve and a no-op `GUARD OFF`, `an_alarm_fires_on_its_replied_activity`
  failed (`triggered 1 1` before `main 0`). `GUARD OFF` releases; `GUARD ON` reserves, a contended
  one waiting pinned under its `Op::Exec` frame. `GUARD WHEN` with a false expression keeps its
  refusal (Task 12).
- **Deadlock.** `Interp::deadlocks(owner)` follows `waiting` from the owner: a guard key to its
  owner, a message to the activity running it (`root_then` `Started`, or a native tail or park
  recording it). Reaching the running activity raises 98.905. Checked on a contended method
  reserve (blamed on the method's first clause, as the oracle's current instruction then is), a
  contended native reserve, `GUARD ON`, and `Message~wait`/`~result` before parking.
- **TraceObject.** `ATTRIBUTEPOOL` from the table's pool numbers (pruned after each collection),
  `ISGUARDED` the activation's flag, `SCOPELOCKCOUNT` the lock's count, `HASSCOPELOCK` whether the
  activation holds it.
- Pending reserves are a list per activity (`guard_waits`), matched by activation, so a method
  entered from a trace route while an outer method's reserve is pending cannot overwrite it.

## Witnesses

Corpus (`rust/corpus/lang/`, listed in `phase-8.txt`, sourceline expectations generated with the
module's `.Package~new` driver):

| program | shows |
|---|---|
| `reply_seen_by_a_guarded_send.rex` | the queued program, back in the corpus |
| `reply_continuation_parks_holding_the_guard.rex` | the queued note's second shape |
| `guard_serializes_started_sends.rex` | guarded sends from two activities serialize; unguarded interleave (each waits inside for the other) |
| `reply_at_nesting_two_reserves_again.rex` | REPLY at nesting 2: continuation reserves again, runs after the sender's method ends |
| `guarded_getter_waits_for_reply.rex` | TESTGUARDEDACCESS minimal |
| `guard_on_waits_in_an_unguarded_method.rex` | contended `GUARD ON` waits; `GUARD OFF` |
| `guard_deadlock_two_activities.rex` | cycle of guarded sends, 98.905 with the oracle's traceback |
| `message_wait_deadlock.rex` | `Message~wait` closing a cycle, 98.905 trapped |
| `trace_object_guard_fields.rex` | ATTRIBUTEPOOL, ISGUARDED, SCOPELOCKCOUNT, HASSCOPELOCK on `>I>`, clause and `<I<` lines, early-entry and ordinary methods |

Run counts, each program, `stab.sh` (stdout md5, stderr md5, rc per run, from a fresh directory):

- Oracle: 30 of 30 identical for every program (`trace_object_guard_fields.rex` re-run 30 of 30
  after its last edit). The oracle's hashes equal ours for all nine.
- Ours, release `rexx-run` at `9f5c68ed5`'s tree: 30 of 30 identical unswitched and 30 of 30
  under `REXX_SWITCH_MODE=every`, every program.
- Under collect_stress: the gated `collect_stress` run covers the union of `phase-*.txt`, these
  included: 36 passed.
- Paths read from stdout: the deadlock programs print the 98.905 line and traceback (stderr
  `25 *-* expose n` / `22 *-* other~poke` as the oracle), the getter prints `GOOD`, the GUARD ON
  program prints `hold out` before `u locked`.
- Crate-side switch-mode test: `guards::tests::only_an_unguarded_method_interleaves_under_every_switch`.
  It found a defect: the guard slice returned before the switch mode reloaded the countdown, so
  later activities stopped switching; fixed by reloading it.
- No probe crashed the oracle.

Mutations (each built release, all nine witnesses run unswitched and `every`, source restored
from a copy):

| mutation | witnesses red |
|---|---|
| no reserve on method entry | all nine (one only unswitched) |
| no transfer at nesting 1 | the getter, both queued REPLY shapes (unswitched) |
| deadlock check never fires | both deadlock programs |
| GUARD ON/OFF do nothing | `guard_on_waits_in_an_unguarded_method` |
| no reserve for generated/external methods | `guarded_getter_waits_for_reply` |
| no early `>I>` deferral | `trace_object_guard_fields` |
| no countdown reload at the guard slice | none of the corpus; the switch-mode unit test (red before the fix) |

One mutation run (M4) left `run.rs` mutated for the next three because the script had no backup
of it; it was restored by hand, its diff checked, and those three were re-run clean (table above
is the clean runs).

## Carried-in items

1. Guard-transfer witness: both programs in the corpus, matching; the queued note removed with
   `git rm`.
2. TESTGUARDEDACCESS: passes in both modes (criterion-1 table, `pass | same`); the owner cell in
   `phase-6-gate.md` says so. It had no allowance in `group_runs`.
3. TraceObject fields: as above.

Criterion-1 rows that now pass besides: GUARD `TEST_ON_DEFAULT`, `TEST_ON`. GUARD
`TEST_WAIT_SIMPLE` now reaches the GUARD WHEN refusal. `TEST_CALLER_STACK_FRAME_REPLY_START`'s
refusal now depends on the mode (allowance in `457a292e8`; see its message).

## Commands and results

At `9f5c68ed5` unless stated (`long.sh`, statuses written unpiped to `status.txt`):

- `cargo fmt --all --check`: 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: 0. `cargo clippy -p rexx-exec
  --all-targets --features pinning -- -D warnings`: 0 (`Checking rexx-exec` seen after a touch).
- `cargo test -p rexx-exec --lib`: 931 passed.
- `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --release --test corpus`: 752 of 752 matching.
- Same with `REXX_CORPUS_SWITCH=every`: 752 of 752. Debug with `every`: 752 of 752.
- `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --release --test collect_stress`: 36 passed.
- `cargo test -p rexx-exec --features pinning --test concurrency_tests measured::`: 17 passed.
- `refusal_sites`: re-derived with `REXX_REFUSAL_SITES_REFRESH=1` (line moves, the new `deadlock`
  row, `scheduler_inconsistency` now also on the send surface); verdict columns filled; 5 passed.
- `cargo test -p rexx-parse --test sourceline_oracle`: 1 passed. `method_bodies`: 23 passed.
  `gate_table_c`: 22 passed.
- `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test concurrency_tests` at `9f5c68ed5`: 28 passed,
  1 failed (`the_s2_rows_of_the_derived_list_in_both_modes`: TEST_CALLER_STACK_FRAME_REPLY_START
  differing between modes). At `457a292e8`: 29 passed, 0 failed (S2 rows table: TEST_CALLER_STACK_FRAME_REPLY_START `refused: a GUARD that has to wait ... then rexx-exec: DO is not implemented`; TESTGUARDEDACCESS `pass | same`).
- Bench "it works": every `rust/bench-programs/*.rex` on the release head against the base
  release binary (built from a `git archive` of `b62155373` in its own target directory, 20
  `Compiling` lines): stdout and rc identical for all but `heapshape`, which prints timings.

## Named risk for Task 26

Every send to a guarded method (every `::METHOD` by default) now does a hash insert and a removal
in the guard table, plus a few field reads in `begin_method` and `finish_call`. Indicative only
(wall clock, three interleaved runs, not the Task 26 instrument): `dispatch.rex` 1.87 s base
against 2.22-2.25 s head. A `perf` sample of the head puts `GuardTable::try_reserve` at 1.67% self
with most of the rest inlined into `begin_invoke`. Candidates: a lock record kept on the
activation for the uncontended single-owner case, or skipping the table while one activity
exists and materialising held locks at the first spawn.

## Concerns

1. A guarded method with an empty body does not wait for a lock another activity holds (no clause
   boundary to wait at): probe `emp.rex`, oracle `hold in / hold out / after empty`, ours
   `hold in / after empty / hold out`.
2. A REPLY continuation's re-reserve (nesting 2) runs no deadlock check; a cycle through it waits
   until the run's end instead of raising 98.905.
3. `setGuarded`/`setUnguarded` on a `Method` object do not change how its sends reserve; the
   directive decides.
4. A contended `GUARD ON` waits pinned (counted as a pinned `GuardOn` park), not stackless.
5. Deadlock 98.905 from a guarded `EXTERNAL` or generated method carries no method traceback line
   beyond the sender's.

## Fix round 1

Commits `bbb0fcc68` (I1, I2, I3 mechanism, M3, M4, witnesses) and `0daf32063` (per-send cost cut).

- **I1.** `Interp::sends_guarded` (dispatch/executable.rs): the directive's answer, overruled by
  `setGuarded`/`setUnguarded` on the `Method` object whose `installed` is the send's `MethodId`.
  Behind `method_flag_writes.is_empty()`, the search out of line. Used in `begin_method` and for
  generated and external methods. Witnesses `method_set_unguarded_is_not_reserved.rex` (su.rex)
  and `method_set_guarded_is_reserved.rex` (sg.rex).
- **I2.** A body with no clause serves its pending reserve where the driver finds `entry >= len`
  (`ir/drive.rs`, only on that path): parked through the slice at the body's end op, or a pinned
  wait. Witness `guarded_empty_method_waits.rex` (emp.rex).
- **I3 (P43), remedy (a).** `GuardTable` has a `live` flag, set by the first `spawn` and by a
  REPLY's split (`guards_go_live`), which puts into the table the locks the running activity's
  activations hold, the replying activation included. Until then a reserve is the activation's flag
  (set in `begin_method` with the other flags, unless an early or package-traced `>I>` has to come
  first), a release touches nothing, and `SCOPELOCKCOUNT` counts the activations holding the key
  (`guard_count`). The method end calls `guard_off_at_end` only when the table is live or a reserve
  is pending; a traced `<I<` runs it first. `Activation` is 512 bytes again: `forwarded`,
  guarded and reserved share `ActivationFlags`, a const assertion holds the size. Witness
  `guard_held_before_the_first_start.rex`: a lock taken at nesting 2 with one activity, contended
  by a `~start` made inside it.
- **M3.** A pinned wait that fails after a release granted the lock records it on the activation
  (method entry, `GUARD ON`), so the activation's end gives it back, or releases it (generated and
  external sends). Not reached by a test: the failure `run_others` returns there is another
  activity's non-condition failure (a loud refusal or the run's deadline), and both end the program
  (P40), so a kept lock cannot be observed afterwards.
- **M4.** The S2 row is back as measured; an `## S3` entry in `phase-6-gate.md` records
  TESTGUARDEDACCESS, TEST_ON_DEFAULT and TEST_ON passing, TEST_WAIT_SIMPLE at the WHEN refusal,
  and the allowed REPLY_START row.
- M1 and M5 left for Task 12, unchanged.

### Callgrind

`bench-programs/callgrind.sh -r 1 -j 6 -p "<programs>" base=<b> head=<h>`, each binary built
release from a `git archive` of its commit in its own `CARGO_TARGET_DIR` (one `Compiling rexx-exec`
line each), spreads 0.0000%:

| program | base `b62155373` | `bbb0fcc68` | head `0daf32063` |
|---|---|---|---|
| dispatch | 21,014,368,690 | +2.7841% | 21,269,421,031 (+1.2137%) |
| dispatchclass | 15,870,847,733 | +2.9490% | 16,074,884,300 (+1.2856%) |
| sendloop | 14,098,735,164 | +4.0786% | 14,343,767,819 (+1.7380%) |
| fibcall | 8,491,425,101 | +0.3852% | 8,524,130,123 (+0.3852%) |
| fibfunc | 8,291,835,921 | +0.4127% | 8,326,052,376 (+0.4127%) |
| emptyloop, varlookup, alloc, startup | | | within +0.06% |

Review head `14232a817` for comparison: +5.71%, +6.02%, +8.48%. What is left is about 49-51 Ir
per guarded send, the same absolute on all three; `sendloop` sends an empty method, so the same
cost is a larger share there (+1.74%, over the ruling's +1.5%). Most of the fibcall/fibfunc
residue is `drive_levels`, which grew with the empty-body branch (+24 M against the `drive_from`
shift). Both are for Task 26.

### Checks at `0daf32063`

- `cargo fmt --all --check` 0; both clippy runs 0; lib 931 passed.
- Corpus with `REXX_CORPUS_GATE=1`: release 756 of 756, release `every` 756 of 756, debug `every`
  756 of 756.
- `collect_stress` 36 passed; pinning `measured::` 17 passed; `concurrency_tests` with the gate
  29 passed (TESTGUARDEDACCESS, TEST_ON_DEFAULT, TEST_ON `pass | same`).
- `refusal_sites` 5, `sourceline_oracle` 1, `method_bodies` 23, `gate_table_c` 22 passed.
- Witnesses, all thirteen: oracle 30 of 30 each; ours 30 of 30 unswitched and 30 of 30 `every`,
  with the oracle's hashes.
- Bench "it works": head against base, stdout and rc identical but `heapshape` (timings).
