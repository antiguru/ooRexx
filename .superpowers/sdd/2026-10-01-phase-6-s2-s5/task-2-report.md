# Task 2 report: activity table, scheduler, started messages, pin depth

Status: DONE_WITH_CONCERNS. Implementer s2-t2, base cd35a48ad (checked `git rev-parse HEAD` first).

Commits: 1a7f030cb (the task), 9326d7e4c (SOURCELINE expectations for the new corpus programs), and the
commit carrying this report's final form.

## What landed

- `scheduler.rs`: `ActivityId` (offset into the table), `MessageId`, `ParkReason::MessageResult(MessageId)`,
  `StartedSend`, `Activities` (idle activities boxed with their `ActivityRoots`, a FIFO ready queue, waiters per
  message in park order, retired thread contexts), the `Scheduler` trait implemented for `Interp` with exactly
  `spawn`, `run_until_park`, `park`, `unpark`, each with its first caller. `switch_to` swaps `Interp::activity` and
  `RootSet`'s running `ActivityRoots` with the idle record, only after a driver has returned.
  `run_activity_root` (main's root driver: a park runs the others from main's frames until main is woken),
  `run_started` (a started activity: its first send, or the rest of its wake), `run_started_activities`
  (program end), `message_completed` (wakes all waiters at once, in park order).
- `activity.rs`: `pin_depth`, `native_park`, `first_send`, `root_then`, `drive_floor`, all classified in
  `object_roots`' exhaustive destructure.
- `pinning.rs`: `pinned!`, `pin_enter!`, `pin_leave!` count `pin_depth` in every build; the `pinning` feature adds
  the kind stack. `PinKind::native` always answers a kind (no `Run` native parks any more).
- `ir/drive.rs`: `drive_from(DriveStart, parkable)` with `Driven::{Ended, Parked(floor)}`; the park branch sits on
  the `Exit::Parked` path (call entry), not in an op arm; `resume_woken`/`deliver_woken_send` are cold and out of
  line. A non-root driver turns a park into a refusal delivered at the send. `exec_suspends` no longer calls
  `spawn` (see ruling R4).
- `dispatch.rs`: `NativePark` (reason, continuation, receiver, the `Then`s of the primitive methods between it and
  the driver), `park_native` (reads the pin depth: refuses `Loud::pinned_park` above zero),
  `resume_native_park`; `complete_send`/`complete_native` and the non-top `run_send_op` refuse a park that reached
  them (`refuse_unrooted_park`, an internal-inconsistency refusal, scheduler.rs).
- `object_protocol.rs`: `Object~start`/`~startWith` spawn; `Message~start`/`~startWith` (new, `Run` natives);
  `Message~result`/`~wait` are `Begin` natives that park until the send completes; `record_started`/`record_held`
  wake the message's waiters.
- `lib.rs`: `Interp::activities` replaces `scheduler: SingleActivity`; `Interp::object_roots` roots every idle
  activity's `Activity::object_roots` and `ActivityRoots`; main's root uses `run_activity_root`; `execute` runs
  every started activity before and after the deferred replies; `Loud::unsent_message_result` removed,
  `Loud::unsatisfiable_wait` and `Loud::pinned_park` added (owner none).

## Rulings (design choices the spec does not settle)

- R1. The running activity stays the field `Interp::activity` (every `self.activity.` site unchanged, one load);
  `Interp::activities` holds the others boxed. The brief's "`Interp::activities` (running inline, parked boxed)" is
  met by the pair.
- R2. A native's park travels the send path as `Started::Entered` with the park on `Activity::native_park`, so no
  hot op arm changed (`Started` keeps two variants). The driver checks `native_park` only where a call op parked
  its region (`Exit::Parked`). `begin_send_op` lends the argument stack only for a real entry.
- R3. A primitive method between a parking native and the driver (Message~send of a `RESULT` message) puts its
  `Then` on the park, not on `native_tails`: a tail at the same depth as the enclosing method's own tails would be
  applied to the wrong answer.
- R4. `exec_suspends`' Split arm keeps the continuation on the current activity without calling `spawn` (its only
  producer is test-only; `a_split_exec_goes_on_with_its_flow` unchanged). `spawn` takes the started send.
- R5. Main runs the other activities from its own Rust frames while parked; started activities hold no Rust frames
  while parked. Interim program end: started activities run to completion before the deferred replies and again
  after them (a replied body can start one; probe `m_reply_start.rex` agrees with the oracle).
- R6. A wait nothing left to run can end (an unsent message, a deadlock) is `Loud::unsatisfiable_wait`, owner
  none, where the oracle blocks for ever (oracle-crashes.txt, the unsent-message block, now names it). At program
  end the blocked activities are abandoned (contexts retired) and the refusal reported once.
- R7. An untrapped condition in a started send now writes its traceback on that activity at once (spec section 6,
  `MessageDispatcher.cpp:64-72`); base swallowed it. Corpus `started_raises.rex` agrees with the oracle.
- R8. Three refusal layers for a park off the root: the pin depth in the native (`pinned_park`, witnessed by
  mutation below), and `refuse_unrooted_park` in the Rust-stack consumers and the non-root driver, an internal
  inconsistency (`op_not_driven`) that fires only if a pinned frame went uncounted.
- R9. `~result` woken after a later send of the same message cleared its outcome answers `.nil`.

## Tests added, with red-before evidence

Base evidence: the probes below run on the base binary (debug, cd35a48ad) before any edit; outputs in the
scratchpad `t2/probes/` + `t2/run/`.

| test | where | base | now |
|---|---|---|---|
| started method waits on `m~result`, main sends later | corpus `started_waits_for_a_later_send.rex` | Loud unsent `Message~result` (Phase 6) rc 120 | `3`, oracle-identical |
| two waiters on one message | corpus `message_two_waiters.rex` | same Loud, rc 120 | `3 3`, oracle-identical |
| `~wait` then `~completed` | corpus `message_wait_then_completed.rex` | Loud `WAIT` (Phase 9) rc 120 | `1 0`/`xyz`, oracle-identical |
| `Message~start`/`~startWith` | corpus `message_start.rex` | Loud `START` (Phase 9) rc 120 | oracle-identical |
| wait inside a stackless callee level | corpus `started_waits_inside_a_call.rex` | Loud unsent (sync start reaches the unsent message) | oracle-identical |
| started message whose send is `RESULT` | corpus `started_result_message.rex` | Loud unsent | oracle-identical |
| traceback at once | corpus `started_raises.rex` | Loud `WAIT` of `Message` (Phase 9), rc 120 (corrected in fix round 2) | oracle-identical |
| unwaited SAY (order fixed by the oracle) | corpus `started_unwaited_says.rex` | `started` (not red; witness only) | oracle-identical |
| SAY after main's last line | `scheduler::tests::a_started_method_nothing_waits_on_runs_after_mains_last_line` | base order `started`/`main done` | `main done`/`started` |
| waiters wake in park order | `scheduler::tests::a_completed_message_wakes_its_waiters_in_the_order_they_parked` | base: unsent refusal | passes |
| pinned wait refused | `scheduler::tests::a_wait_under_a_pinned_frame_is_refused`, `a_wait_under_each_pinning_frame_is_refused_as_pinned` (Interpret, NestedLoop, LoopHeader, TrapHandler, Unknown, Operator, Program) | n/a (new refusal) | mutation: depth check disabled -> both red, every kind answers the unrooted refusal instead |
| unsatisfiable wait at program end | `scheduler::tests::a_started_wait_nothing_can_end_is_refused_at_the_programs_end` | n/a | passes |
| context kept after the activity ends | `scheduler::tests::an_ended_activitys_thread_context_is_kept` (a native, `orxmethod TestInterpreterVersion`, reads the instance through the started activity's context) | n/a | mutation: retire push removed -> red (`left: 0 right: 1`) |
| collect-stress | `collect_stress::a_parked_activitys_registers_survive_the_activity_that_wakes_it` (values > 7 bytes in registers of the parked level and the level below, the waker allocates) | base: `Message~start` refused | passes; mutation: idle roots removed -> red; idle `ActivityRoots` alone removed -> red |

The brief's "later call through the cached pointer" half is not made red: a call through a freed context is
undefined behaviour that may still answer, so the red witness is the interpreter's hold on the context (the
`retired` mutation), and the native call on the started activity shows the instance read works.

Changed existing tests: `dispatch::tests` (unsent `~result` refusal text), `ir::drive::tests::a_send_op_runs_its_method_on_the_drivers_own_frame`
(8 stackless entries: `Object~start`'s body is now its activity's root), `run::tests::message::a_started_method_that_raises...`
(`m~wait` before sampling, which is interleaving-dependent; expected stderr now has the started activity's own
traceback first; the re-raise report still differs from the oracle's, see concerns).

## refusal-sites.tsv

Re-derived with `REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test refusal_sites -- --test-threads=1`.
Rows: `unsent_message_result` gone; `unsatisfiable_wait` (body, off the send surface); `pinned_park` (send,
`diverges yes "pins its activity"`, witness: `m~result` in a `sortWith` comparator over a message a started
activity sends; oracle prints 3 and sorts, probe `j2_pinned_sent.rex`); `reply_inside_construct` line moved.
`method-bodies.txt` refreshed (`REXX_METHOD_BODIES_REFRESH=1`): Message result/wait now the unsatisfiable-wait
refusal, start/startWith answer.

## Message table delta

`task-2-message-table.md` (raw `task-2-message-table.raw.md`): pass 39, refused 29 (Task 1: 35 / 33).
START/STARTWITH/WAIT generic refusals are all gone. 4 rows now pass; 11 rows now refuse as a pinned wait; 1 row
(`TEST_STARTWITH_NOT_ARRAY`) moved to the `Object~MAKEARRAY` refusal. The pinned rows: every Message test body runs
under a frame the ooTest framework opens (the S1 pinning report lists every `MessageResult` arrival under
`TreeSend`), so their `~result` meets the pin depth. Task 5's pinned wait is the owner.

## "It works" bench check

All `rust/bench-programs/*.rex` on release builds of base (git archive cd35a48ad, own target dir) and this tree:
stdout, stderr and exit status identical for every program; `heapshape` prints timings (shape identical);
`extcall` run with `LD_LIBRARY_PATH=build/lib` on both (identical `3000000`).

## Gates

Round 1 at 1a7f030cb (`S=$W bash $W/p6-gates/gates.sh`): G1, G2, G3, G5, G7, G8 exit 0; G4 and G6 exit 101 with one
failure each, `rexx-parse --test sourceline_oracle`: the new corpus programs had no `SOURCELINE` expectation files.
Generated with the module comment's oracle script (scratchpad `t2/srcl/srclines.rex`; oracle counts equal each
file's line count), committed in the fix commit. While G8 ran, those eight untracked files were written into the
tree (rexx-parse test data, not read by G8); recorded here as a breach of the frozen-tree rule.

Round 2 at 9326d7e4c: G1-G8 all exit 0, finished 2026-10-01T16:24:39+02:00, tree clean at the end (status file
`gates/status.txt`; round 1's is kept as `gates-round1/status.txt`).

## Concerns

1. **ooTest rows that passed at S1 now refuse, until Task 5.** Every ooTest test body runs under a pinning frame
   (measured with the pinning build on Message `TEST_START`: the park's frames are `[Program]`, the nested program
   the driver chain enters). A synchronous `~start` used to complete before `~result` read it; now `~result` parks,
   meets the pin depth and refuses. Measured on this tree: all 14 `base/class/Object.testGroup` tests the S1 pinning
   report lists as `pass` with a `MessageResult` arrival (`TESTSTART01`, `TESTSTARTWITH01`, the `_NO_METHOD`,
   `_OVERRIDE`, `_OVERRIDE_CONTEXT`, `_OVERRIDE_NOT_FOUND`, `_OVERRIDE_FROM_NONSELF`, `_OVERRIDE_AMONG_MIXINCLASSES`
   rows of both `start` and `startWith`) end rc 120 with the pinned refusal; enumeration command:
   `grep -E '^\| [^|]+ \| pass, rc 0 \| (MessageResult|MessageWait) \|' docs/superpowers/plans/phase-6-pinning.md`.
   No gate covers them. The Message group's 11 pinned rows are the same cause. Task 5's pinned wait is the owner.
   The S1 doc's per-test table shows no `Program` frame anywhere, which disagrees with today's measurement; not
   investigated.
2. The re-raise report of `~result` over a failed started send differs from the oracle's (the oracle repeats the
   started method's failing clause; this crate names `RESULT` and the calling clause). Base differed too, differently.
3. Guard locks are absent (S3), so two started methods on one object run where the oracle deadlocks (probe
   `k_chain.rex`: oracle 98.905, this crate answers).
4. Performance is unmeasured (per the global constraints): the pin-depth increment and decrement at every pinned
   site in every build, the `native_park` test on `Exit::Parked` and in `begin_send_op`'s lend, and `drive_from`'s
   new prologue are Task 26's named risks.
5. A park refused after `Message~result` issued its `MessageId` leaves that id in the message-id table until the
   message completes (no waiter is registered, so nothing wakes wrongly).

## Fix round 1 (ruling P23: the pinned wait lands now)

Status: DONE_WITH_CONCERNS (see the round's concerns at the end).

### What changed

- `park_native` (dispatch.rs): a park reached with the pin depth above zero no longer refuses. It runs
  `Interp::pinned_wait(reason)` on the pinned activity's own stack and then the native's continuation half
  (`resume(self, receiver)`), answering `NativeStarted::Ran`, so the callers above it (the primitive methods
  between it and the pinning frame) finish synchronously as before. The receiver is rooted as a temp across the
  wait.
- `pinned_wait` (scheduler.rs): registers the waiter (`park`), runs `run_others`, and answers `None` on the wake.
  `run_others` now keeps a stack of loop owners (`Activities::owners`): every activity whose loop is on the Rust
  stack (main's root wait, the program-end drain, each pinned wait). A ready owner other than the loop's own is
  set aside ("buried": its continuation is below this loop) and put back at the front of the ready queue when the
  loop returns, so its own loop resumes it. When nothing runnable is ready: if a buried owner is ready the wait is
  inverted and refused, `Loud::inverted_wait` ("a pinned wait for a message's completion that only an activity
  pinned below it can end is not implemented", owner none), counted by the pinning build in
  `PinReport::inverted` keyed by park kind and pinned frames; otherwise `Loud::unsatisfiable_wait`, as at the
  root. Nothing is ever in flight in S2 (no inbox before S4), so "nothing in flight" holds trivially. A refused or
  failed wait withdraws the activity from the waiters (`cancel_wait`), at the root too.
- `Loud::pinned_park` is removed (no producer left).

### Rulings this round

- R10. The refusal names the park kind ("a message's completion"), not the pinned re-entry kind: the kind stack
  exists only in the `pinning` build, and a message that differs between builds would fork the tests. The pinning
  report keys the count by both (`ParkKind`, pinned frames).
- R11. "Inverted" is decided by the ready queue: nothing this loop can run is ready and an owner buried below it is.
  Nothing ready and nothing buried is a plain deadlock, refused as the root's unsatisfiable wait.
- R12. Withdrawn in fix round 2 (its 3.6 GB figure was the memory cap, not a property of the tests).

### How no RegFrame or arena borrow of the pinned activity is live across its swap (P14, D-U4)

The pinned activity's Rust frames below the wait do hold `RegFrame`s (the drive levels between the scheduler and
the pinning frame). What the swap moves is not the arena: `switch_to` swaps the `ActivityRoots` value, whose
`frames` field is an `Rc<FrameArena>` handle; the `FrameArena` itself, its block list and its blocks stay in the
`Rc` allocation and never move (frame.rs property 1). Each live `RegFrame<'a>` borrows the arena through the
driver's own local `Rc` clone (`let arena = self.roots.activity().frames()` in `drive_from`), not through `self`
or `self.roots`, so the borrow checker already rules out a `RegFrame` borrowing anything `switch_to` moves, and the
clone keeps the allocation alive whatever the swap does. During the nested loop nothing reserves, releases or
parks in the pinned arena: every other activity drives its own arena (one `Rc<FrameArena>` per `ActivityRoots`,
D-U4), and the collector reads the pinned arena only through `Activities::object_roots` (`FrameArena::iter`, a
shared read, the same read a collection makes while the activity runs). The inline `Activity` swap is sound for the
reason P13 gives: nothing holds a Rust reference into it across a call that takes `&mut Interp`. The pinned
activity is swapped back in before `pinned_wait` returns, so every pinned frame resumes against its own
`Activity` and `ActivityRoots`.

### Tests this round

| test | evidence |
|---|---|
| `scheduler::tests::a_pinned_wait_runs_the_activity_it_waits_on` (sort comparator waits on a message a started activity sends; `3`, `sorted 1,2`, as the oracle, probe `j2_pinned_sent.rex`) | nested loop mutated off (`pinned_wait` skips `run_others`): red |
| `scheduler::tests::a_pinned_wait_completes_under_each_pinning_frame` (Interpret, NestedLoop, LoopHeader, TrapHandler, Unknown, Operator, Program) | nested loop off: red |
| `scheduler::tests::an_inverted_pinned_wait_is_refused` (probe `p_inverted.rex`; the oracle prints `3`, `main sorted`, `2`, `done`) | nested loop off: red (wrong refusal); buried-owner check off: red |
| `scheduler::tests::a_pinned_wait_nothing_can_end_is_refused` | |
| `measured::an_inverted_wait_is_counted_with_its_frames` (pinning build, G8) | |
| `collect_stress::a_pinned_activitys_registers_survive_the_activity_its_wait_runs` | idle `ActivityRoots` rooting removed: red |
| `group_runs::the_outcome_table_of_the_message_group` now asserts every START test passes except `TEST_START` (`HASRESULT`), `TEST_HALT_START` (`HALT`), `TEST_STARTWITH_NOT_ARRAY` (`MAKEARRAY`), each held to its refusal | nested loop off: red |
| `group_runs::the_outcome_table_of_the_object_group` (new): every Object START test passes | nested loop off: red, listing `TESTSTART01`, `TESTSTARTWITH01` and the others |

Superseded: `a_wait_under_a_pinned_frame_is_refused` and `a_wait_under_each_pinning_frame_is_refused_as_pinned`
(the refusal they held is gone).

### Tables

`task-2-message-table.md` (fresh): pass 49, refused 19 (Task 1: 35 / 33). Every refusal names a Phase 9 method;
none is a wait. `task-2-object-table.md` (new): the 40 Object START tests, all pass, including the 14 the
S1 pinning report lists as passing with a `MessageResult` arrival.

### refusal-sites.tsv

Re-derived: `pinned_park` gone, `inverted_wait` added (body surface: constructed in scheduler.rs), a line moved.

### Bench

Not re-run this round: the change is confined to `park_native`'s pinned branch and the scheduler loop, neither on
a path a bench program reaches (no bench program starts an activity).

### Concerns this round

1. The pinned wait has no slice deferral (Task 4) and no inbox (S4): a pinned waiter whose wait an off-baton
   completion would end does not exist yet.
2. A buried owner that is ready waits for its own loop; if the inner loop's activities then block for ever, the
   refusal is the inverted one even where the oracle would also have deadlocked; the oracle's 98.905 deadlock
   detection is later work.
3. The Object gate run omits the two UNINIT tests: they allocate until the memory cap stops them, on the base too.
4. Concerns 2-5 of the first round stand.

### Gates this round

At 66368f3a7: G1-G8 all exit 0, finished 2026-10-01T17:02:14+02:00, tree clean; both `group_runs` table tests
ran and passed in G4 (release) and G6 (debug). Commits this round: 66368f3a7 (the fix), and the commit carrying
this section.

## Fix round 2 (review task-2-review.md)

### I1: the replied body and UNINIT are pinned frames

`Interp::resume_reply` runs its body under `pinned!(PinKind::DeferredReply)` and `Interp::run_one_uninit` sends
`UNINIT` under `pinned!(PinKind::Uninit)`, both new kinds, each with a `FRAME_PROBES` case (`SysSleep` in a replied
body, in an `UNINIT`) and a row in `phase-6-pinning.md`'s frame table (whose stale `Native` row, naming
`RESULT`/`WAIT` as excepted run halves, is corrected too). A `~result`/`~wait` there is now a pinned wait. Corpus
witnesses `started_waited_in_a_replied_body.rex` (the review's `e2_reply`) and `started_waited_in_uninit.rex`
(`e4_uninit_wait`), oracle-identical, listed in `phase-8.txt`, SOURCELINE files generated. Red before: with the
two `pinned!` wrappers removed, both end rc 120 with the unrooted-park refusal (run on a rebuilt `rexx-run`). The
review's `e3`, `e5`, `e6` are oracle-identical too. The unrooted-park refusal and its siblings no longer go through
`op_not_driven`: `Loud::scheduler_inconsistency` renders "the scheduler found a wait outside every root driver and
pinned frame", "... a woken activity with no wait recorded", "... a wait parked at an op that is not a send".

### I2: inverted is classified over the whole loop stack

Set-aside owners live on `Activities::set_aside`, each tagged with the stack index of the loop that set it aside,
and go back to the ready queue when that loop returns. A loop with nothing to run reports `inverted` iff a loop
strictly enclosing it set a ready activity aside; otherwise the wait is the unsatisfiable one. The label, the
`PinReport::inverted` count and the refusal all follow that one decision. `~wait` now parks with its own reason,
`ParkReason::MessageWait`, so its inverted wait counts under `ParkKind::MessageWait`.

- R13. "Enclosing" is read strictly, as the ruling's two witnesses require: in `p15_hidden_inversion` main was set
  aside by `s1`'s loop, which encloses the refusing `s2` loop (inverted, counted); in `p3_buried_deadlock` main was
  set aside by the refusing loop itself (not inverted: "nothing left", not counted).

Concern on R13: the two shapes differ only in which loop happened to pop the woken owner, not in whether the wait
could end. My fix-round-1 witness (main pinned in a comparator waits on `m1`; a started activity sends `m1`, then
waits pinned on `m2`, which main sends after its comparator returns; the oracle completes) has `p3`'s shape and is
now refused as "nothing left to run" although main is ready below it. It is no longer a test; `p3` is.

Witnesses: `scheduler::tests::an_inverted_pinned_wait_is_refused` (p15), `a_deadlock_with_a_buried_activity_ready_is_not_an_inverted_wait`
(p3); pinning build `measured::an_inverted_wait_is_counted_with_its_kind_and_frames` (p15 with `~result` counts
once under `MessageResult`, with `~wait` once under `MessageWait`, both with `Interpret` among the frames; p3
counts nothing). Mutation: classifying per loop again (`*by == level`) turns both scheduler tests red.

### Minors

- M1: the 3.6 GB sentence in concurrency_tests.rs now says the UNINIT tests allocate until the cap stops them, on
  the base as here (the reviewer's measurement); R12 withdrawn; the Object table's note corrected.
- M2: scheduler.rs module doc says a switch happens where a driver has exited or, for a pinned wait, inside the
  Rust frames that pin the outgoing activity; `pinned_wait`'s doc states the R13 rule.
- M3: see I1.
- M5: see I2.
- M6: `an_ended_activitys_thread_context_is_kept` now also calls `InterpreterVersion` through the retired
  context's instance after the started activity has ended (`ThreadContext::interpreter_version_through_instance`,
  a `#[doc(hidden)]` rexx-api member with its `SAFETY` note in ffi.rs), answering 328448.
- M7: `execute` reports every failure the program-end drains answer, in order, through one
  `report_late_failures` shared with the deferred replies (a refusal on its own line and in the status, a condition
  as its traceback, a deadline left to the guard below); `run_started_to_end` keeps draining after a failing
  activity, so none is left unrun.
- M8: the `group_runs` module doc names both tables.
- M9: the `started_raises.rex` base evidence corrected in the first table (base: the `WAIT` refusal, rc 120).
- M4 (nested pinned-loop depth guard): not this round, per the ruling.

### refusal-sites.tsv

Re-derived: `scheduler_inconsistency` added (body+ir, off the send surface); `inverted_wait` and
`reply_inside_construct` lines moved.

### Gates this round

At 458fa0546: G1-G8 all exit 0, finished 2026-10-01T18:00:23+02:00, tree clean. Commits this round: 458fa0546
(the fix), and the commit carrying this section.

## Fix round 3 (ruling P25)

`Interp::blocked` (scheduler.rs) now answers `inverted` iff `Activities::set_aside` is non-empty: some activity is
ready but its frames lie below a loop on the stack, whichever loop (the refusing one included) set it aside. With
nothing set aside and nothing to run, no activity is ready anywhere (nothing is in flight before S4), and the
refusal is "a wait that nothing left to run can end". Label, refusal and `PinReport::inverted` count follow that
one decision, which reads only the stack state. R13 is superseded.

### Oracle evidence (memcap 1G, `timeout -s KILL 20`, fresh directories)

| witness | oracle | here |
|---|---|---|
| `p15_hidden_inversion.rex` (review) | completes, rc 0 in 0.05 s, six lines | inverted refusal, counted |
| fix-round-1 witness (`t2/probes/p_inverted.rex`: main pinned in a comparator on `m1`; a started activity sends `m1`, waits pinned on `m2`, which main sends after its comparator) | completes, rc 0 in 0.02 s | inverted refusal, counted |
| `p3_buried_deadlock.rex` (review) | prints `main got 10`, `main waits m3`, then hangs (killed at 20 s, rc 137) | **inverted** refusal, counted |

`p3` lands in a different class than the review and the ruling's last sentence give it ("oracle hangs -> nothing
left"). At its refusal the stack state is the fix-round-1 witness's: main is ready, set aside below the refusing
`s1` loop. The two differ only in what main does after it runs (in `p3` it prints two lines and then waits on a
message nothing sends; in the witness it sends `m2`), which no function of the stack state can see. Under P25's
first sentence "nothing left to run" would be false for `p3`: main is ready and the oracle runs it (its two lines),
so the label is inverted. Recorded rather than special-cased; the decision is the controller's if the oracle's
eventual hang should count.

### Tests

- `scheduler::tests::an_inverted_pinned_wait_is_refused` (p15), `an_inversion_the_refusing_loop_set_aside_is_refused_as_inverted`
  (the restored fix-round-1 witness), `a_deadlock_with_a_buried_activity_ready_is_refused_as_inverted` (p3),
  `a_pinned_wait_nothing_can_end_is_refused` (nothing ready anywhere).
- Pinning build `measured::an_inverted_wait_is_counted_with_its_kind_and_frames`: p15 counted once under
  `MessageResult` and once (with `~wait`) under `MessageWait`; p3 counted once under `(MessageResult, [OpExec,
  Interpret])`; a pinned wait on an unsent message with nothing started counts nothing.
- Mutations: per-loop classification (`by == level`) turns the p15 test red; round 2's strictly-enclosing rule
  (`by < level`) turns the restored witness and the p3 test red.

UNINIT-spawned activities at termination: ledgered for Task 9, not changed.
