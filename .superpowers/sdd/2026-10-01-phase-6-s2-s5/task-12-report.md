# Task 12 report: GUARD ON/OFF/WHEN parking and the variable barrier

Commits, base `217f33a2c`: `e1cc6f7a3` (implementation, witnesses, refusal table), `8b4b06927`
(barrier on the heap object, no switch on a notify, hot arm restored), `604907355` (P42 row
dropped), `c80ad0eab` (EventSemaphore schedule allowance), `1e2432ea2` (three witnesses the
mutation round asked for), and this report's commit.

## Design decisions

- **Park arm (P21).** `exec_guard` answers `ExecOutcome::Park(reason)`. `exec_suspends` decides:
  at a root driver's level (`top`) with nothing pinned it records the park
  (`park_instruction`, a `NativePark` whose resume answers nothing) and the region parks through
  the shared `'entered` path with `Deliver::Exec`; `resume_woken` resumes the region at the
  `Op::Exec` itself (`parked.at - 1`), which runs the instruction again. Anywhere else it is a
  pinned wait and the instruction runs again in place. A failed wait drops the instruction's
  state (`abandon_guard_exec`), keeping a lock a release granted.
- **Instruction state.** `Activity::guard_exec` (`GuardExec`: activation, instruction index,
  `Reserve` or `When { reacquire }`, the watched variables). A re-run continues from the wait:
  after `Reserve` the lock is held; after `When` it reserves again where it held the lock
  (`RexxActivation::guardWait`), then re-evaluates.
- **M1.** A contended `GUARD ON` parks through the same path (`set_guard_state`'s pinned wait is
  gone; `guard_on`/`guard_off` replace it).
- **What WHEN watches.** The parser records the exposed variables the expression names on
  `Guard::watched` (the walk the 99.913 check already made, now collecting: simple names, stems,
  a compound's stem and variable tails, nothing from a compound spelled earlier). At run time each
  name is looked up in the activation's exposures.
- **Barrier.** One test of `Object::watched`, a bool on the heap object (it fits the existing
  padding), on the four store/drop paths every object variable write goes through
  (`set_exposed_variable`, `clear_exposed_variable`, `set_pool_variable`, `clear_pool_variable`:
  the pool path, attribute setters, `SetObjectVariable`, the timer natives, variable references).
  It is tested before the store so the store stays a tail call; a watched object takes the cold
  `store_watched`. The flag is never cleared (sticky). The watch lists are on the guard table
  keyed by owner, pruned with the pool numbers after a collection. A first version kept them in
  `ScopePools`; its extra field changed `Body`'s discriminant encoding and cost every instance
  match (dispatch +0.47%, rexxcps +0.96% Ir), so it moved.
- **Notify.** A store or drop of a watched variable posts each current watcher
  (`Activity::guard_posted`, the oracle's `guardSem`); a watcher parked in its wait is readied.
  **No switch is asked for** (departure from spec section 6's "switch request at the next clause
  boundary"): with one, `TEST_BASE_ALARM`'s waiter ran before the notifier's next clause and read
  `TRIGGERTIME` unset (41.1), where the oracle's shape `alarm/a4.rex` answers 30 of 30; the
  oracle's `yieldControl` hands its kernel lock only to an activity already queued for it, and a
  woken guard waiter is not yet (probe `wake.rex`: oracle `a woke` 30 of 30, now ours too). So a
  store with no watchers is not observable beyond the cold path, and "DROP notifies only while
  watchers remain" holds trivially.
- **Posted while evaluating.** The flag is cleared before each evaluation; a false answer with the
  flag set releases and re-evaluates at once instead of parking (`GuardInstruction.cpp:167-185`).
- **Stem elements.** Spec section 6 says a compound element store of an exposed stem notifies.
  Measured false: a WHEN on `a.1` watches the stem variable `A.`; `a.1 = 5` leaves it parked and
  `a. = 5` wakes it (oracle 30 of 30, `guard_when_stem_element_store.rex`).
- **Trace.** The clause is echoed once; each evaluation traces its intermediates and `>K> "WHEN"`.
  That line's `TraceObject` carries `ISWAITING` (true where the value starts with `0`), after
  `RECEIVER` as the oracle puts it.
- **Native API.** `SetGuardOn`/`SetGuardOff` now act on the library method frame's own lock
  (`NativeFrame::reserved`, set when the send reserved it); the frame releases at its end only
  where it still holds it. `SetGuardOn/OffWhenUpdated` watch the variable, set the lock state,
  wait pinned (release, `GuardWhen`, reserve again), and answer the value. The watch is withdrawn
  after the wait; the oracle keeps it, which only affects a later yield, while a kept entry here
  could post an unrelated activity that reuses the handle. Layout owner rows removed.
- **Blocked WHEN.** Main alone blocked is the wait nothing left can end (P27, loud); a started
  activity blocked keeps the program's end waiting until the run's deadline, as the oracle's does
  (`a_blocked_guard_when_keeps_the_programs_end_waiting`).
- **99.913** is the parser's (pre-existing); translation errors are reported loudly here in
  general (pre-existing, e.g. 99.907), so no differential witness carries it.

## Witnesses

Corpus, `rust/corpus/lang/`, listed in `phase-8.txt`. Oracle 30 runs from a fresh directory, ours
3 runs unswitched and 3 under `REXX_SWITCH_MODE=every`, all one output each and equal to the
oracle's (stdout and status; stderr compared too), command `t12/cmp.sh FILE 30 3`:

| program | oracle | shows |
|---|---|---|
| `guard_on_lock_passes_between_activities.rex` | 30/30 | M1, `inv.rex`: rc 0 where base refused rc 120 |
| `guard_when_waits_for_a_store.rex` | 30/30 | WHEN gives up the lock, each store wakes it |
| `guard_when_stem_element_store.rex` | 30/30 | element store does not wake, stem store does |
| `guard_when_attribute_setter.rex` | 30/30 | a generated setter wakes it |
| `guard_when_drop_wakes.rex` | 30/30 | DROP wakes; GUARD OFF WHEN returns without the lock |
| `guard_when_traces_each_evaluation.rex` | 30/30 | trace of each re-evaluation, stderr strict |
| `guard_when_trace_object_waiting.rex` | 30/30 | `ISWAITING` 1 then 0, absent on other lines |
| `guard_when_replies_take_turns.rex` | 30/30 | bug 2003 shape, `n 0`..`n 9` |
| `guard_when_posted_while_evaluating.rex` | 30/30 | single activity; store during evaluation |
| `guard_when_holds_the_lock_again.rex` | 30/30 | woken WHEN holds the lock |
| `guard_when_unwatches_at_its_end.rex` | 30/30 | finished WHEN stops watching |
| `native_guard_on_when_updated.rex` | 30/30 | both native WhenUpdated members |
| `native_guard_off_keeps_the_count.rex` | 30/30 | native SetGuardOff gives back its own level |

collect_stress runs all of them (it reads every phase subset file). No oracle crashed.

Crate-side: `scheduler/tests.rs` `a_blocked_guard_when_keeps_the_programs_end_waiting` (deadline
300 ms, `DEADLINE_EXIT`) and `a_store_wakes_a_parked_watcher_without_a_switch` (oracle 30 of 30
for each of its four programs); `ir/drive/tests.rs` `a_parked_exec_runs_its_instruction_once_woken`
(scripted park, stackless and pinned, the instruction runs once after waking); pinning
`measured::a_guard_when_is_a_pinned_wait_only_under_a_pinned_frame`; `rexx-parse`
`a_guard_when_watches_the_exposed_variables_it_names`; `concurrency_tests`
`the_outcome_table_of_the_guard_group_in_both_modes` (GUARD.testGroup `TEST_WAIT_MULTIPLE`,
`TEST_WAIT_SIMPLE`, `TEST_WAIT_SIMPLE_TRIGGER` and the rest pass in both modes; the remaining rows
are refusals of the test's own INTERPRET'd translation errors and of `USE LOCAL`).

Gate table C `methods/alarm__instance.rex`: `agree=7`, `loud=no` (`gate_table_c` log at
`604907355`); the owner is phase 6, enforced at phase close, so no file changes.

## Mutation evidence

`t12/mutate.sh` on a `git archive` copy of `604907355`, profile `mutation`, each witness run once
unswitched against the oracle's captured output; control (a comment added) green on all 11.

| mutation | witness | result |
|---|---|---|
| exposed store skips the barrier | waits_for_a_store | red, loud unsatisfiable wait |
| exposed drop skips the barrier | drop_wakes | red |
| pool store skips the barrier | attribute_setter | red; the alarm gate row hangs (timeout 124) |
| notify posts nobody | waits_for_a_store, drop_wakes | red |
| posted flag ignored | posted_while_evaluating | red |
| no reacquire after a WHEN wake | holds_the_lock_again | red (green on every earlier witness) |
| WHEN keeps the lock while waiting | waits_for_a_store | red |
| park always pinned | lock_passes_between_activities | red, inverted wait rc 120 |
| region resumes past the Exec op | waits_for_a_store, traces_each_evaluation | red |
| no unwatch at the end | unwatches_at_its_end | red (green on every earlier witness) |
| no watch registered | waits_for_a_store | red |
| no ISWAITING | trace_object_waiting | red |
| native frame releases at end regardless | native_guard_off_keeps_the_count | red (green before the started send was added) |
| native WhenUpdated does not wait | native_guard_on_when_updated | red |

## Carried items

- **M1** closed (above).
- **M5.** The `457a292e8` allowance is deleted. `TEST_CALLER_STACK_FRAME_REPLY_START` now reaches
  the DO refusal in both modes with the same stderr, so it left the P42 list (`604907355`).
  `TEST_TRACEOBJECT_COLLECTOR` was never refused at GUARD WHEN; it stays under P42.
  `EventSemaphore` TEST_WAIT_CONCURRENT now refuses `WAIT` unswitched and `POST` under every
  opportunity (the worker's store wakes main): allowed by row only while both name an
  EventSemaphore method (`c80ad0eab`, Task 13).
- Gate plan: "Criterion 1 rows after Task 12" added, MethodArgs TEST_REQUEST_STRING_* included.

## Checks

At `604907355` (`t12/checks.sh`, statuses in `t12/checks2-status.txt`): `cargo test -p rexx-exec
--lib` 933 passed; corpus `--release` gated 0, gated `REXX_CORPUS_SWITCH=every` 0, debug gated every
0; `collect_stress` 36 passed; `refusal_sites` 5; `sourceline_oracle` 1; `method_bodies` 23;
`gate_table_c` 22; pinning `measured::` 18. `concurrency_tests` with `REXX_CORPUS_GATE=1`: failed on
the EventSemaphore row (above); after `c80ad0eab`, `-- group_runs:: --test-threads=1` 7 passed,
exit 0. The first run (at `e1cc6f7a3`, under callgrind load) also showed SysSleep
TEST_SLEEP_DURATION differing between modes; unloaded at `604907355` it is the same.
`cargo fmt --all --check` 0; clippy `--workspace --all-targets -D warnings` 0 and `-p rexx-exec
--all-targets --features pinning` 0. At `1e2432ea2`, for the added witnesses: corpus `--release` gated 0,
gated every 0, debug gated every 0, `collect_stress` 36 passed, `sourceline_oracle` 1
(`t12/checks4-status.txt`).

## Performance

Callgrind, `bench-programs/callgrind.sh -r 3`, libc subtracted, base `217f33a2c` and head
`604907355` each built from `git archive` into its own target directory (one `Compiling rexx-exec`
line each), spread 0.0000% everywhere:

| program | base | head | delta |
|---|---|---|---|
| assign | 19483200362 | 18983200723 | -2.5663% |
| compound | 8982437974 | 8927435148 | -0.6123% |
| dispatch | 21269420847 | 21259421041 | -0.0470% |
| dispatchclass | 16074884238 | 16058884292 | -0.0995% |
| sendloop | 14343767635 | 14343767827 | +0.0000% |
| fibcall | 8524130047 | 8494900754 | -0.3429% |
| fibfunc | 8326052300 | 8322613677 | -0.0413% |
| rexxcps | 17849147034 | 17812930247 | -0.2029% |

`assign`'s gain is `ops_loop_steady` codegen (−3 Ir per clause), not a semantic change. A scratch
program storing two exposed variables 200,000 times (`t12/cg/objstore.rex`): 266070299 -> 266270160
(+0.5 Ir per store, +0.075%); `set_exposed_variable` itself is +5 Ir per call on dispatch, the
barrier's load, test and branch. `exec_procedure` is pinned out of line: without that the inliner
moved it into `exec_flow` and fibcall was +2.4%.

## Concerns

1. The notify asks for no switch, a departure from spec section 6's text, on the measurements
   above. Cost if wrong: a woken waiter runs at the notifier's next wait, end or slice rather
   than its next clause.
2. Pre-existing defect: `self~assertTrue(1, 'a' d)` with `d` a `.DateTime` refuses with `a
   compiled call op does not name a call of its own body` at `217f33a2c` too (single activity,
   `t12/alarm/b1.rex`). `TEST_BASE_ALARM` now reaches it in process. Not fixed here; needs an
   owner.
3. The spec's compound-element sentence is false on the oracle (above); the spec should drop it.
