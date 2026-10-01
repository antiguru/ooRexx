# Task 4 report: slices, the timer thread, the deterministic switch mode, Message~halt

Status: DONE through fix round 2. Commits 50d58865e, dd293bbdb (first round), 8e36bcfdb (fix round 1) and the fix round 2 commit; G1-G8 green at 50d58865e; fix round 1 ran the targeted checks of ruling P28, full gates run by the controller.

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
  it when none is; then serves `SLICE`: cleared where nothing else is ready, answered as
  `Failure::Slice` where the clause `yields` at a pin depth of zero, and otherwise deferred (countdown
  1, counted once in the pinning report); a second `SLICE` that finds the deferred one still pinned
  takes a pinned yield (fix round 1, ruling P29).
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
  `SLICE` (a slice belongs to the activity it was set for), disarms the timer where no other activity
  is ready, and collects under the stress mode. A loop of `run_others` whose next
  ready activity is the one whose slice just ended runs it on.
* **Message~halt** (`dispatch/object_protocol.rs` `native_message_halt`, `Interp::halt_message`).
  False for a message never started and for an activity already asked; true where the request is
  made, and where the started activity has no Rexx frame (not begun, or ended), which drops it
  (`Activity::halt`, `Activity.cpp:2155`). The request is an entry in the target's pending traps
  (`PendingTrap::request`) for its running activation and fragment depth, so the end of the clause
  that activation is running delivers it (`deliver_pending_traps`, `raise_requested_halt`):
  `CALL ON HALT` queues a pending trap (`queued_during_delivery`, so the boundary tripwire does not
  take it for the next clause's own), `SIGNAL ON HALT` raises the condition, and untrapped it is 4.1,
  each failing that clause's boundary, which names it as the site (fix round 1).
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
  `SLICE` alone; `HALT` is a per-activity request (an entry in the target's pending traps, with its
  description), because its only requester names one activity. The interpreter-wide `HALT` bit comes with signals (Task 21).
* **R-T4-2 HALT is raised at the end of the clause it found running, in that clause's
  activation**, where the oracle's `processClauseBoundary` raises it (`RexxActivation.cpp:651`,
  `:4085`), including a `RETURN` that ends its activation (fix round 1: the first commit raised it
  at the start of the next clause, which put a halted `RETURN`'s condition in the caller). Measured by
  `message_halt_wait.rex`, `message_halt_untrapped.rex`, `message_halt_self.rex`,
  `message_halt_returning.rex`, `message_halt_loop_step.rex` and `message_halt_interpret.rex`,
  identical to the oracle 30 runs of 30.
* **R-T4-3 A parked target is not woken by HALT.** `Message~halt` only sets the activation's flag in
  the oracle (`RexxActivation::halt`); a thread waiting on a message takes it when its wait ends
  (measured: `message_halt_wait.rex`). The wake in spec section 4 is for interruptible waits
  (SysSleep under SIGINT); no park of this task is one.
* **R-T4-4 A halt to a started activity with no Rexx frame answers true and is dropped**, as
  `Activity::halt` does with `currentRexxFrame == NULL`: not yet begun, or ended.
* **R-T4-5 A slice with nothing else ready is cleared, not taken**, and the timer is disarmed at the
  switch or countdown visit that finds the ready queue empty. Slices are counted from the timer's tick,
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
the Message start tests under the switch mode (17 pass, the MAKEARRAY refusal, one differing: at
the first commit TEST_HALT_START, after fix round 1 TEST_START, see Fix round 1).

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
(`task-4-message-switched-table.raw.md`): 17 pass, 1 refused (MAKEARRAY), at fix round 1 TEST_START differs and TEST_HALT_START passes (the raw file is fix round 1's).

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

1. The flattened-loop pass counts its header clause at `Op::LoopNext` and hands the proof down; the
   per-pass `Option` of the first commit is gone (fix round 1). Not measured (Task 26 does).
2. The oracle raises a halt in the activation the request found running, a `RETURN` that ends it
   included; it does not lose the flag (the first commit's statement here was false, fix round 1).
3. The timer's slices are counted from its own tick, not from each switch, so a slice can be shorter
   than 24 ms; nothing observable depends on its length beyond "a busy loop yields".
4. TEST_HALT_START differs from the oracle until SysSleep parks (R-T4-10); listed as differing in
   `concurrency_tests.rs` rather than refused.

## Fix round 1 (review task-4-review.md; ruling P28: no gates.sh, targeted runs)

* **I1 HALT at the end of the halted clause** (`scheduler.rs` `halt_message`, `raise_requested_halt`;
  `run/condition.rs` `deliver_pending_traps`; `lib.rs` `PendingTrap::request`). The request is now a
  pending-trap entry for the target's running activation and fragment depth (the oracle's flag on
  `currentRexxFrame`), delivered at the end of the clause that activation is running, where the
  oracle's `processClauseBoundary` raises it; this crate's `Activity::halt` field and `record_entered_clause_site` are
  gone. A halted `RETURN` raises in its own activation (h7: `outer after inner trapped sigl 43` in the
  witness, as the oracle). A flattened loop's header clause failing at its boundary is blamed as the
  header's own failures are (`run/loops.rs`: `END` for a pass that fell through), so POSITION and
  the traceback both name the `END`; a self-halt in `INTERPRET` names the interpreted clause. Report
  Concern 2 corrected: the oracle raises in the returning activation and does not lose the flag.
  Witnesses (identical to the oracle on stdout, stderr with the run directory masked, and status,
  30 runs of 30; identical on this side with and without `REXX_SWITCH_MODE=every`; each differs at
  dd293bbdb, built by `git archive` in its own target dir):
  * `message_halt_returning.rex`: trapped (`inner trapped sigl 43`, no error) and untrapped (4.1,
    POSITION 37, traceback `37 *-* return g~q~result` then `30 *-* call plain g`). At dd293bbdb:
    `trapped error 1`, 4.1 at 29.
  * `message_halt_loop_step.rex`: `do forever`, `do while 1`, `do i = 1`, `do until 0`, empty
    bodies: POSITION and traceback the `END` line each. At dd293bbdb: traceback `do forever`.
  * `message_halt_interpret.rex` (the reviewer's h6): untrapped, the traceback's first line is the
    fragment clause `x = g~m~halt;`. At dd293bbdb: the `INTERPRET` line twice.
  The reviewer's `h2` with a `nop` body names `nop` or `END` with POSITION and traceback agreeing;
  which one depends on where the timer's slice lands (on the oracle it was `END` in the reviewer's
  runs), so it is not a witness.
* **I2 pinned yield (ruling P29)** (`scheduler.rs` `serve_requests`, `pinned_yield`; `pinning.rs`
  `pinned_yield!`, `PinReport::pinned_yields`). A `SLICE` seen while pinned is deferred once (the
  bit cleared, `slice_deferred` set, countdown 1); an unpinned clause that yields takes it as before;
  the next `SLICE` that finds the activity still pinned (pin depth above zero) runs one nested round:
  the activity goes to the back of the ready queue and `run_others` runs the others on its stack,
  each until it parks, ends or its own slice ends, owners below set aside, until the pinned activity
  comes round again. A `SLICE` at a pin depth of zero in a clause that cannot yield stays deferred
  (no pinned yield outside a pinned frame). Witnesses, identical to the oracle 30 runs of 30 and red
  at dd293bbdb (killed at 15 s): `pinned_busy_wait_interpret.rex`, `pinned_busy_wait_handler.rex`,
  `pinned_busy_wait_external.rex` (with `pinned_busy_wait_external.d/bwext.rex`). Crate tests
  `a_pinned_busy_wait_yields_to_the_activity_that_ends_it` (INTERPRET and CALL ON handler, under
  `EveryOpportunity` and the timer, fixed output) and `two_pinned_busy_waiters_both_end` (two
  waiters inside INTERPRET on a third activity's flag: `set / B ended / A ended / main ended` under
  both), both red with the pinned yield removed (60 s deadline). `concurrency_tests.rs`
  `a_pinned_busy_wait_counts_its_pinned_yields` (pinning feature): yields keyed with `Interpret`,
  none for the unpinned loop.
  The reviewer's `h5` hung at this round (main set aside as buried); fix round 2 completes it.
* **Message start tests under `EveryOpportunity`.** With pinned yields, main's pinned test body
  yields to started activities: TEST_HALT_START now passes under the switch mode, and TEST_START
  differs (23 assertions of 45): its started `delayValueReturn` runs its `SysSleep`, which blocks the
  thread, to the end inside main's yields before the test asserts it has not completed.
  `MESSAGE_START_DIFFERING_SWITCHED` lists it. The unswitched Message table is unchanged
  (identical raw rows).
* **M1 timer disarm** (`switch_to`): disarmed at the switch that leaves no other activity ready.
  Test `the_timer_disarms_when_no_other_activity_is_ready` (reads the registry entry), red with the
  disarm removed.
* **M2** `in_counted_clause` takes a `DeadlineCounted`; the flattened-loop step takes a closure that
  supplies it (`Op::LoopNext` hands the proof it counted, the `ITERATE` settle path counts after the
  body's outcome is known, so an escape counts nothing); the loop's entry header and the header after
  an `UNTIL` test count at their own sites. No `Option` on the pass path.
* **M4** the stress test compares `EveryOpportunity` with `AtClause(u64::MAX)` (the same visits, the
  slices' switches extra) and a one-activity program with and without the switch mode (visits only);
  each red with its collect removed.
* **M5 queued**: `.superpowers/sdd/queued/2026-10-01-time-elapsed-loop-hang.md` (`elapsed.rex`:
  oracle ends in about 0.6 s; this crate killed at 10 s).

Runs at the fix round 1 tree: `cargo fmt --all --check`; `cargo clippy -p rexx-exec --all-targets
-- -D warnings`, and with `--features pinning`: clean. `REXX_CORPUS_GATE=1 memcap 8G cargo test
--release -p rexx-exec --test corpus corpus_differential`: 705 of 705; with `REXX_CORPUS_SWITCH=every`
release and debug: 705 of 705. `cargo test -p rexx-exec --lib`: 888 passed. `REXX_CORPUS_GATE=1
cargo test --release -p rexx-exec --test concurrency_tests`: 27 passed (Object START 40 pass).
`--features pinning --test concurrency_tests measured::`: 11 passed. `refusal_sites` (release) and
`rexx-parse --test sourceline_oracle`: green.

## Fix round 2 (ruling P30: buried means Rust frames live below)

* **Main is not buried by its own root loop** (`scheduler.rs`). `Activities::owners` records, per
  loop, whether its owner is buried: a pinned wait or pinned yield entered from Rust frames that stay
  live below (`run_others(true)`), or main waiting at its root driver or at the program's end, whose
  state is all in its record (`run_others(false)`). `next_runnable` sets aside buried owners only, and
  the inverted-wait classification (`blocked`, from `set_aside`) follows the same definition.
* **Main's root continuation lives in its record.** `run_activity_root` records where its driver
  stopped (`root_driven`: `drive_floor` for a park, `sliced` plus the ready queue for a slice,
  `Activity::root_end` for its end) and resumes through `root_step`, which any loop calls when it
  runs main (`run_started` for `MAIN`). Main ending in a nested round records `root_end` (rooted in
  `Activity::object_roots`) and stays in the table; the loop that owns main returns as soon as it
  sees it (`root_ended` after each run), and `run_activity_root` answers it, so the program-end steps
  run there, never in the nested round.
* **Witnesses** (identical to the oracle on stdout, stderr with the run directory masked, and status,
  30 runs of 30; identical on this side with and without `REXX_SWITCH_MODE=every`; each killed at 15 s
  at 8e36bcfdb, built by `git archive` in its own target dir):
  * `message_halt_pinned_target.rex` (the reviewer's h5): main busy-waits for, then halts, a target
    spinning inside `INTERPRET`, for SIGNAL ON, CALL ON and untrapped.
  * `main_ends_in_pinned_yield.rex`: main's body ends (`exit 3`) inside the spinning activity's
    pinned yield; `spin ended` follows, status 3.
  * `main_fails_in_pinned_yield.rex`: main fails inside the pinned yield after starting a second
    activity that fails later; main's report comes first, as on the oracle. With the `root_ended`
    check removed the reports come in the other order (measured), so this is the witness of that
    check; `main_ends_in_pinned_yield` alone passes without it.
* **Crate tests:** `main_ends_inside_a_pinned_yield` (`EveryOpportunity` and the timer, status 3,
  fixed lines) and `a_wait_main_can_end_from_its_root_is_satisfied`; both red when every owner is
  treated as buried (the round-1 definition; 60 s deadline).
* **Classifications re-checked under the corrected definition:**
  * **p15 moved** (`HIDDEN_INVERSION`, `scheduler::tests::an_inverted_pinned_wait_is_refused` and
    `concurrency_tests` `measured::an_inverted_wait_is_counted_with_its_kind_and_frames`). Its main
    parks unpinned at its root on `m0`, so it is no longer buried: the program now completes, rc 0,
    with the oracle's lines (oracle 30 runs: `s3 sent m0 / main got 0 / main sent m2`, then `done`
    and `S2 got 20` in either order, 23 and 7, then `S1 got 10`; this crate prints the majority
    order unswitched). That shape is kept as `a_wait_main_can_end_from_its_root_is_satisfied`
    (lines compared as a set). The inverted witness now makes main wait pinned
    (`interpret "say 'main got' m0~result"`): the oracle completes it (rc 0, the same six lines),
    this crate refuses it as inverted, `s3 sent m0` then the refusal, as p15 did; the pinning test
    counts it as before.
  * **p3** (`BURIED_DEADLOCK`, main waits inside `INTERPRET`): still inverted, unchanged.
  * **The fix-round-1 witness of Task 2** (`an_inversion_the_refusing_loop_set_aside_is_refused_as_inverted`,
    main waits in a sort comparator): still inverted, unchanged.
  * The Task 2 corpus programs (`started_waited_in_a_replied_body.rex`, `started_waited_in_uninit.rex`
    and the rest of `phase-8.txt`) all still match.
* **Concern 2 of fix round 1** (TEST_START under `EveryOpportunity`) accepted and carried to Task 5.

Runs at the fix round 2 tree: `cargo fmt --all --check`; `cargo clippy -p rexx-exec --all-targets
-- -D warnings`, and with `--features pinning`: clean. Corpus (release): 708 of 708; with
`REXX_CORPUS_SWITCH=every`, release and debug: 708 of 708. `cargo test -p rexx-exec --lib`: 890
passed. `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test concurrency_tests`: 27 passed.
`--features pinning --test concurrency_tests measured::`: 11 passed. `refusal_sites` (release),
`rexx-parse --test sourceline_oracle`: green.
