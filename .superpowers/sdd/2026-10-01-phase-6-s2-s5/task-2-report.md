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
| traceback at once | corpus `started_raises.rex` | no traceback (stderr empty) | oracle-identical |
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
