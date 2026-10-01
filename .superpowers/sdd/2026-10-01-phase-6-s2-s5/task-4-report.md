# Task 4 report: slices, the timer thread, the deterministic switch mode, Message~halt

Status: DONE. Commits 50d58865e (code, witnesses, records) and the commit carrying this report's gate section. Gates G1-G8 green at 50d58865e.

Base: dcf9fd554 (checked before starting).

## What changed

* **Request word and timer** (new `timer.rs`). `Requests` is an `AtomicU32` with the `SLICE` bit;
  `Registration` is an interpreter's entry in the live-interpreter registry (a `static` mutex-guarded
  list plus condvar), removed on drop. The timer thread is spawned once per process on the first
  arm (`Once`); it sets `SLICE` in every armed interpreter whose 24 ms
  (`ActivityManager::timeSliceLength`, `ActivityManager.hpp:359`) slice has run out, re-arming it for
  the next slice, and waits on the condvar with no deadline when nothing is armed. The registry and the
  timer are the only process-global state (R3).
* **The countdown** (`clause.rs`). `count_clause_against_deadline(yields)` and the cold
  `countdown_reached(yields)` keep the hot path unchanged (the decrement and the zero test); the cold
  path checks the deadline, reloads 1024 and calls `Interp::serve_requests` (`scheduler.rs`), which:
  collects under the stress mode; under a switch mode reloads the countdown to 1, counts the clause and
  sets `SLICE` where the mode says; otherwise arms the timer when another activity is ready and disarms
  it when none is; raises a pending `HALT`; then serves `SLICE`: cleared where nothing else is ready,
  deferred (countdown 1, counted once in the pinning report) where the clause does not `yield` or the
  pin depth is above zero, and otherwise answered as `Failure::Slice`.
* **The SLICE exit** (`ir/drive.rs`). The count moved to the top of `open_clause!` (after the
  granting instance's hand-off, before anything the clause's opening does), so `Failure::Slice` turns
  into `Exit::Slice(pc)` with nothing of the clause begun. `drive_from` parks the level as a park does
  and answers `Driven::Sliced { floor, at }`; `DriveStart::Sliced` resumes it through
  `Next::Sliced(at)`, which enters as `Next::Body` does (`grant_for`) and gives the countdown back the
  one clause the slice already counted.
* **The loop step** (`run/loops.rs`, `ir/drive.rs` `Op::LoopNext`). A flattened loop's pass header is a
  clause counted inside `in_clause`, which never yields; an empty-body busy loop therefore never
  reached an `Op::Clause` and never switched (measured: `do while \self~done; end` hung forever under
  the timer and under `EveryOpportunity`). `Op::LoopNext` now counts the pass's first header clause
  itself, with `yields` = `TOP`, and hands the proof down (`in_counted_clause`); a slice there is
  `Exit::Slice(pc)` at the `LoopNext` op, which resumes by running that op afresh.
* **Scheduler** (`scheduler.rs`). `Scheduler::yield_at_slice` puts the running activity at the back of
  the ready queue. `run_started` (through `started_driven`) records `Activity::sliced` and yields;
  `run_activity_root` yields main and runs the others until it is ready again. `switch_to` clears
  `SLICE` (a slice belongs to the activity it was set for), lowers the countdown to 1 for an incoming
  activity with a halt pending, and collects under the stress mode. A loop of `run_others` whose next
  ready activity is the one whose slice just ended runs it on.
* **Message~halt** (`dispatch/object_protocol.rs` `native_message_halt`, `Interp::halt_message`,
  `Activity::halt`). False for a message never started and for an activity already asked; true where
  the request is made, and where the started activity has no Rexx frame (not begun, or ended), which
  drops it (`Activity::halt`, `Activity.cpp:2155`). The target applies it at its next countdown visit
  (`raise_requested_halt`): `CALL ON HALT` queues a pending trap (`queued_during_delivery`, so the
  boundary tripwire does not take it for the clause's own), `SIGNAL ON HALT` raises the condition, and
  untrapped it is 4.1, each with the clause the request found running as the site
  (`record_entered_clause_site`).
* **Switch mode** (`invocation.rs`): `SwitchMode::{AtClause(k), EveryOpportunity}` and
  `Invocation::with_switch_mode`, installed after the library bootstrap so `AtClause` counts the
  program's clauses alone; `rexx-run` reads `REXX_SWITCH_MODE` (`every`, `at:K`); the corpus harness
  reads `REXX_CORPUS_SWITCH=every`; the group runner's `SwitchMode::EveryOpportunity` maps to it.

## Rulings

* **R-T4-1 The inbox lands with its first poster.** Nothing posts to an interpreter from another
  thread in this task: the timer sets `SLICE` in the request word, which needs no queue, and
  `Message~halt` is a request from inside the interpreter. An `Inbox` type, an `INBOX` bit and a
  post method with no caller would be dead code a seam may not carry (P20, P21's precedent); they come
  with the first poster (Task 5's sleeper wake or Task 17's completions). The request word carries
  `SLICE` alone; `HALT` is a per-activity request (`Activity::halt`, with its description), because
  its only requester names one activity. The interpreter-wide `HALT` bit comes with signals (Task 21).
* **R-T4-2 HALT is raised at the start of the clause after the one it found running.** The oracle
  raises it in `processClauseBoundary` after the instruction ends (`RexxActivation.cpp:651`,
  `:4085`); the request here is served at the next countdown visit, before the next clause opens, so
  the clause state still names the halted clause: `SIGL` (SIGNAL ON), `POSITION`, the traceback line
  (recorded by `record_entered_clause_site`) and the CALL ON handler's timing (after the next clause,
  as the oracle's queued trap is) all match. Measured by `message_halt_wait.rex`,
  `message_halt_untrapped.rex` and `message_halt_self.rex`, identical to the oracle 30 runs of 30.
* **R-T4-3 A parked target is not woken by HALT.** `Message~halt` only sets the activation's flag in
  the oracle (`RexxActivation::halt`); a thread waiting on a message takes it when its wait ends
  (measured: `message_halt_wait.rex`). The wake in spec section 4 is for interruptible waits
  (SysSleep under SIGINT); no park of this task is one.
* **R-T4-4 A halt to a started activity with no Rexx frame answers true and is dropped**, as
  `Activity::halt` does with `currentRexxFrame == NULL`: not yet begun, or ended.
* **R-T4-5 A slice with nothing else ready is cleared, not taken**, and the timer is disarmed at the
  next countdown visit that finds the ready queue empty. Slices are counted from the timer's tick,
  not from the switch.
* **R-T4-6 The clause a slice ended at is not counted twice.** It was counted when the slice ended;
  `Next::Sliced` gives the countdown one back. Without it `EveryOpportunity` livelocked: each
  resumed activity yielded again at the same clause (measured, `inter.rex` hung).
* **R-T4-7 The loop step yields.** See "The loop step" above; the busy-loop test is the witness
  (`do while \self~done; end`).
* **R-T4-8 TRACE-toggle and raise requests are not built**: no caller in this task (P20).
* **R-T4-9 `AtClause(k)` fires once**, at the k-th clause counted after the library bootstrap; under
  either switch mode the countdown reloads to 1 and the timer is never armed.
* **R-T4-10 `TEST_HALT_START` differs from the oracle until SysSleep parks.** Its `syssleep .5`
  blocks the thread, so the halt finds the started activity before its first clause and is dropped
  (R-T4-4); the test fails on this side with 2 assertions run of 21. `concurrency_tests.rs` lists it in
  `MESSAGE_START_DIFFERING`; Task 5 runs it through the runner.
* **R-T4-11 A slice at a pin depth of zero in a driver that cannot park** is a pinning-truthfulness
  defect: `debug_assert!` in `drive_from`, and in release the driver runs on from the same clause.
  The debug corpus under `EveryOpportunity` never reached it.

## Tests and red-before evidence

Crate tests (`scheduler/tests.rs`), all green; each mutation below was applied to the finished tree
by a script, the scheduler tests run, and the file restored byte for byte (`cmp`):

| test | mutation that reddens it |
|---|---|
| `started_activities_interleave_under_every_opportunity` (fixed expected `a 1 b 1 a 2 b 2 a 3 b 3 done`, three runs; plus the unswitched order) | `serve_requests` answers `Ok` instead of `Failure::Slice`; `Op::LoopNext` counts with `yields` false |
| `a_busy_loop_yields_to_the_activity_that_ends_it` (no mode, real time, 60 s deadline) | **the timer never armed** (`arm` replaced by `disarm`): the run hit its deadline, exit 121 after 60.01 s; also no slice taken; also `Op::LoopNext` not yielding |
| `a_slice_inside_a_sort_comparator_waits_for_the_sort` (`AtClause(4..=12)`, the comparator's clauses: the other activity's line after the sort's; `AtClause(3)`: before it) | the deferred slice dropped (`SLICE` cleared when pinned): the other activity ran after `sorted` |
| `message_halt_in_the_shape_of_test_halt_start` (`EveryOpportunity`; `0 / 1 / 0 1 1 / 4.1 [] / ... / HALT [HALT Test]`, two 4.1 reports) | the request not stored; no slice taken |
| `the_stress_mode_collects_at_every_visit_and_switch` (collections switched minus unswitched, measured 52 against 10) | the collect at the countdown visit removed; the collect at the switch removed |

`concurrency_tests.rs` (`--features pinning`): `a_slice_deferred_inside_a_sort_comparator_is_counted_once`
(one deferred slice, keyed with `SortComparator`); red with the count removed (0) and with the
once-per-slice guard removed (6). `group_runs::the_message_start_tests_under_every_opportunity` runs
the Message start tests under the switch mode (17 pass, the MAKEARRAY refusal, TEST_HALT_START
differing).

Before the loop-step change the busy loop hung under the timer and under `EveryOpportunity` alike
(rexx-run, killed at 30 s); the oracle prints `A ended`.

Corpus witnesses (`rust/corpus/lang/`, listed in `phase-8.txt`, SOURCELINE files generated with the
`sourceline_oracle.rs` driver), each identical to the oracle on stdout, stderr (run directory masked)
and exit status, 30 runs of 30 on the oracle, and identical on this side with and without
`REXX_SWITCH_MODE=every`:

* `message_halt_wait.rex`: SIGNAL ON HALT (`HALT stop signalled SIGNAL sigl 36`, the waiting
  clause) and CALL ON HALT (`called next`, then `handler sigl 48 stop called CALL`, then `called
  after`), a second halt while pending answering 0, before start 0, after the end 1.
* `message_halt_untrapped.rex`: 4.1 reported at `26 *-* g~q~wait`, `hasError` 1.
* `message_halt_self.rex`: a started activity halting its own message, trapped (`self 18`) and
  untrapped (4.1 at line 25).

Oracle crash entry 23 (`oracle-crashes.txt`): halting again after the untrapped halt ended the
activity, SIGSEGV in 5 runs of 10.

## Corpus under EveryOpportunity

`REXX_CORPUS_GATE=1 REXX_CORPUS_SWITCH=every memcap 8G cargo test [--release] -p rexx-exec --test
corpus corpus_differential`: 699 of 699 matching, release and debug (the debug run is the one the
pinning `debug_assert!` would fire in). `REXX_CORPUS_SWITCH=bogus` panics on the first program, so the
knob is read. `object_start.rex` and `send_resumable_entries.rex` are among the matching.

## refusal-sites.tsv

Re-derived with `REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test refusal_sites
-- --test-threads=1`; the plain run is green. Against dcf9fd554 with line numbers masked
(`diff <(git show HEAD:rust/corpus/refusal-sites.tsv | sed 's/\.rs:[0-9]*/.rs:N/') <(sed
's/\.rs:[0-9]*/.rs:N/' rust/corpus/refusal-sites.tsv)`): no difference. No generic refusal row
existed for Message `HALT` (it refused through the missing-row path); `method-bodies.txt`'s `halt`
row moved from `loud` to `answers rc 0` (`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p
rexx-exec --test method_bodies`).

## Message table

`task-4-message-table.md` (raw `task-4-message-table.raw.md`): pass 51, refused 16, differ 1 (Task 3:
51 / 17). The one changed row: TEST_HALT_START, refused at Task 3, now differs (R-T4-10). Object
table: the 40 START tests pass, unchanged. Start tests under `EveryOpportunity`
(`task-4-message-switched-table.raw.md`): 17 pass, 1 refused (MAKEARRAY), TEST_HALT_START differs.

## "It works" bench check

Every `rust/bench-programs/*.rex` on release builds of the base (dcf9fd554 by `git archive`, own
target dir, `Compiling rexx-exec` seen) and this tree: stdout, stderr and exit status identical,
except `heapshape`, whose stdout prints timings (identical with digits masked). `extcall` ran with
`LD_LIBRARY_PATH=build/lib` on both.

## Gates

At 50d58865e (`S=$W bash $W/p6-gates/gates.sh`): G1 fmt, G2 clippy from an empty target, G3/G5
builds, G4 release and G6 debug `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast`
(corpus 699 of 699 matching in both, no `FAILED` line in either log), G7 clippy `--features pinning`,
G8 pinning self-tests (9 passed, `a_slice_deferred_inside_a_sort_comparator_is_counted_once` among
them): all exit 0, finished 2026-10-01T22:37:48+02:00, tree clean at the end. The assertion-table
report in G4 (report mode, not gated) reads 4247 of 4259; not compared against the base.

## Concerns

1. `Op::LoopNext` now counts the pass header itself and hands an `Option<DeadlineCounted>` down, so
   the flattened-loop pass path carries one more well-predicted branch than before; no per-task
   performance gate measures it (Task 26 does).
2. A halt request is served at the next countdown visit, so a request made to an activity whose
   halted clause is the last of its activation is raised in the caller's next clause; the oracle's
   flag lives on the activation and is lost with it. Not witnessed either way.
3. The timer's slices are counted from its own tick, not from each switch, so a slice can be shorter
   than 24 ms; nothing observable depends on its length beyond "a busy loop yields".
4. TEST_HALT_START differs from the oracle until SysSleep parks (R-T4-10); listed as differing in
   `concurrency_tests.rs` rather than refused.
