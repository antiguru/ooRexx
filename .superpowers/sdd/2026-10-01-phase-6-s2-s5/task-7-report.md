# Task 7 report: contexts across activities; Message reply, replyWith, notify

Status: DONE. Base 83c0a6ff1. Code at 0e5715b72; this report and the queued note in the commit after it.

## Design

- **Context objects.** `dispatch/context.rs`'s readers go through `at_context`: the activation is
  looked up on the running activity, else among the idle activities
  (`Interp::idle_context_owner`, scheduler.rs), whose record is swapped into the running one's
  place for the read (`Interp::with_idle_activity`, the same two `mem::swap`s `switch_to` makes,
  with a temps frame pushed and popped on the swapped-in roots so the idle activity's temps are
  left as found); the answer is put on the caller's temps after the swap back. No activity holding
  it is 98.981. Every reader is unchanged otherwise, so `~thread` answers the owning activity's
  number, `~line` its clause, `~variables` its slots and `~stackFrames` its whole stack, as the
  oracle's `activation->getActivity()` does. A REPLY continuation's context needs nothing extra:
  Task 6 moves the activation, `context_object` and `invocation` included.
- **Kept call contexts.** With per-activity thread contexts (Task 2), a call context kept by main's
  native call and used from another activity reached main's innermost call, whose conversion
  state was held, and aborted the process on the `RefCell` borrow. `ffi.rs`'s `variables_of`
  now answers `None` where the innermost call of the context's activity is itself busy, and the
  four context-variable members answer null or do nothing. The interpreter-side token
  (`suspended_caller`) needed no change: a token reaches it only through its own activity's
  thread context.
- **Message natives** (`object_protocol.rs`): `~reply`/`~replyWith` set receiver and arguments as
  `~start` does, check reuse, clear the original's completion, validate the scope and start a copy,
  which they answer (`MessageClass::reply`). `Object~copy` of a `Message` (`Primitive::Message`,
  refused before) drops the result and condition entries and copies the notify array
  (`MessageClass::copy`). `~notify` checks `MessageNotification` through the Rexx package
  (88.901/88.914 as the oracle), appends to an `Array` kept in the message's `NOTIFY` entry, and
  sends `messageComplete` at once where the message has already notified. `MESSAGECOMPLETE` and
  `TRIGGERED` are one resumable native: the held send with a new `Then::Triggered` that records the
  outcome and answers nothing.
- **Notification** (`Interp::notify_parties`): after the waiters are woken, `messageComplete` with
  the message to each party listed when it began, then the message is marked notified
  (`Interp::notified_messages`, cleared with the completion). Called from `record_held`'s success,
  `record_started`'s success and, for a failed send, from `attach_condition` once the condition
  object is on the message, so a notifier reads `~errorCondition` as on the oracle. There the
  unwinding failure's record on the activity (`failure_site(s)`, `failure_frame(s)`, origin,
  propagated/reraised, `reraised_object`) is set aside with its objects on the temps and put back
  after (`set_aside_unwinding`), so Rexx run by a notifier cannot clobber it.
  `attach_condition` now answers a `Result`. Each send is pinned under the new
  `PinKind::Notification` (FRAME_PROBES case added; mutating the kind to `Uninit` reddens
  `a_park_under_each_frame_kind_records_it`).
- `run_started`: a condition a notification raises is reported as the started activity's own
  untrapped failure (it used to be dropped by the `Err(Failure::Raised(_))` arm).

## Rulings

- R-T7-1: an idle activity's context is read by swapping its record in, not by teaching each
  reader to address another activity's arena: the readers stay one code path, and the swap is the
  scheduler's own. Cost if wrong: a reader that parks, switches or runs Rexx would do so as the
  wrong activity; none does, and `with_idle_activity`'s doc says so.
- R-T7-2: a kept call context's variable member called from another activity answers nothing.
  The oracle raises 98.983 (`Activity::validateThread`) against the context's own activity, then
  its started activity ends in 44.1 and the program hangs (3 runs of 3, killed at 20 s), so there
  is no oracle answer to match beyond "nothing answered". Only the four context-variable members
  are covered; the other members of a kept call context go through `activation_of`, which by
  reading (not run) still panics the same way, and belong to S4's per-callback thread check
  (spec 2.4).
- R-T7-3: completion wakes waiters, then notifies (the oracle's order in `sendNotification`); a
  failed send notifies at `attach_condition`, not at `record_held`, so the condition object is
  there as the oracle's `error()` has it.
- R-T7-4: a notifier raising in a started activity is reported once; the oracle re-notifies and
  reports twice. Queued: `.superpowers/sdd/queued/2026-10-02-notifier-failure-in-started-activity.md`.

## Tests, with red-before evidence

- Corpus witnesses (phase-8.txt, sourceline expectations generated from scratch copies with the
  module-comment driver): `context_of_another_activity`, `context_moved_by_reply`, `message_reply`,
  `message_notify`, `message_notify_arguments`, `message_notify_error`, `message_notify_untrapped`.
  Red before, base binary on the same shapes (scratch `ctxa.rex`/`ctxb.rex`): 98.981 at the first
  `c~line`, rc 120 (`ctxa`: stdout empty; `ctxb`: stdout empty); `ne1.rex`/`nt1.rex`: `method
  "NOTIFY" of class "Message" is not implemented (Phase 9)`, rc 120; REPLY/REPLYWITH the same
  refusal (Task 5 table).
- Witness stability, `stab.sh` (30 oracle runs and 30 of ours unswitched and 30 under
  `REXX_SWITCH_MODE=every`, all three descriptors against oracle run 1), every witness 30/30 on
  all three; collect_stress (which runs phase-8.txt) green.
- `scheduler::tests::a_context_reads_an_activation_of_another_live_activity` (unswitched, every,
  collect at every allocation): line, name, variables, digits, thread 2 against main 1, stack
  frames and arguments of a started activity's activation, then 98.981 once it ended.
- `scheduler::tests::a_context_follows_its_activation_to_a_reply_continuation`: a REPLY
  continuation's context read before it has run (unswitched and collect only: under every the
  continuation runs first) and while it waits (all three modes), `~invocation` kept, thread 2.
- `outer_context::a_kept_outer_context_used_by_another_activity_answers_nothing` (`o9e.rex`, plain
  and collect stress): stdout `started null`, `ret`, `end main`, rc 0. Red before: base aborted
  with `RefCell already borrowed` at `rexx-api/src/values.rs:782` (panic in a function that cannot
  unwind).
- `concurrency_tests::group_runs::the_outcome_table_of_the_message_group` asserts TEST_REPLY and
  TEST_NOTIFY pass beside TEST_SEND and TEST_START. Red before: both refused (Task 5 table).
- `scheduler::tests::a_refusal_after_main_ended_in_a_nested_round_is_reported` used
  `Message~replyWith` as its refusal; it now uses `.RexxInfo~executable`, still loud.

## Table and derived files

- Message table: `task-7-message-table.md` (pass 65, refused 3; Task 5: pass 52, refused 16); the
  switched start-test table is unchanged from Task 5's.
- `corpus/method-bodies.txt` refreshed (`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p
  rexx-exec --test method_bodies`): `messageComplete`, `notify`, `reply`, `replyWith`, `triggered`
  went from `loud` to `answers`.
- `refusal-sites.tsv` re-derived (`REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test
  refusal_sites`): no change. The removed refusals were the generic unbound-method ones, which
  have no row; no `Raised`/`Loud` constructor was added.

## Bench "it works"

Head (0e5715b72) and base (83c0a6ff1, archived and built in its own target directory with a
`Compiling rexx-exec` line) on every `bench-programs/*.rex`, release: stdout, stderr and rc
identical for all but `heapshape`, whose stdout is wall-clock timings on both (`build_seconds`,
`gc_pause_seconds`); `extcall` is rc 158 (98.903, `orxfunction` not on the library path) on both.

## P28 checks (at 0e5715b72 unless noted)

- `cargo fmt --all --check`: exit 0.
- `cargo clippy -p rexx-exec -p rexx-api --all-targets -- -D warnings`: exit 0; with
  `-p rexx-exec --features pinning`: exit 0 (run on the tree that was committed, before the commit).
- `memcap 8G cargo test -p rexx-exec --lib`: 906 passed. `cargo test -p rexx-api`: green (run
  before the commit, after the `ffi.rs` change).
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: 729 of 729
  matching; with `REXX_CORPUS_SWITCH=every`: 729 of 729; debug with `REXX_CORPUS_SWITCH=every`:
  729 of 729.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test collect_stress`: 36 passed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test concurrency_tests`: 27
  passed.
- `memcap 8G cargo test -p rexx-exec --features pinning --test concurrency_tests measured::`: 16
  passed.
- `cargo test -p rexx-exec --test refusal_sites`: 5 passed. `cargo test -p rexx-parse --test
  sourceline_oracle`: passed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test outer_context`: 22 passed.
- The corpus (release) alongside collect_stress (release): 729 of 729, and 36 passed.

## Concerns

- A kept call context's members other than the context-variable ones, used from another activity,
  still abort on the borrow (by reading `activation_of`; not run). S4's per-callback check.
- `2026-10-01-message-notify-single-slot.md`: a send made through another message's send marks
  both messages failed, so both now also notify.
- A stray `git checkout -p` ran once with stdin closed; it quit at its first prompt and
  discarded nothing (`git diff --stat` unchanged before and after).

## Fix round 1

Commit d0274573d (review `task-7-review.md`; reviewer probes in scratchpad `rev-t7/`).

- **C1 (ruling P37).** `rexx-api/src/ffi.rs`: the thread table tracks the native call entered last
  on any of its contexts (`Running`); a method or call context whose own call and its activity's
  innermost call are both busy was kept by another activity's call (`Addressed::Elsewhere`).
  `GetContextDigits`/`Fuzz`/`Form` and `GetRoutine` answer from that activity's top frame
  (`Surface::kept_numeric`/`kept_executable`): the oracle's non-blocking members read
  `Activity::getApiContext`, the top frame, cast as a native activation. Every other call/method
  member raises 98.983 in the running call (held, raised when it returns) and answers null or does
  nothing; a thread-context member through a kept thread context raises 98.983 and is served by
  the running call. Native frame numbers are one counter for all activities
  (`Activities::next_native`), so a frame token names its call on one activity
  (`Interp::idle_native_owner`). Test `outer_context::a_kept_call_context_used_by_another_activity_answers_or_raises`
  (`tests/outer_context/outer7.cpp`, the reviewer's forge), every member listed in the review,
  plain and collect stress. Red before: the reviewer's run, all 14 abort at rc 134 (`RefCell already
  borrowed`). The `o9e.rex` test it supersedes is removed.
  Ruling R-T7-5: `GetContextFuzz` answers 0 where the oracle answers 256, which comes from reading
  a Rexx activation's memory as a native activation's; digits 9, form 0 and `a Method` match.
- **I1.** `attach_condition` no longer notifies. A held send's failure notifies at once, in
  `record_held` (condition object built then, `settle_failed_sends`), with the unwinding record set
  aside; a notifier's failure replaces the send's, its record on top and the set-aside one below
  (`notify_failed_send`, `restore_unwinding_below`), so it is trapped and reported as the oracle's
  (e1, e2 identical on all three descriptors). A started activity's failure follows the oracle's two
  handlers (`Interp::end_failed_started`, `Activity.cpp:3423` then `:233`): the message is told,
  and where its notifier fails, that failure replaces the send's, the message is told again and
  both notifier failures are reported; the completion's waiter gets a round after a notifier
  failure (`yield_after_notifier_failure`), where the oracle's activity gives up the kernel lock.
  e4 and n1 (`2026-10-02-notifier-failure-in-started-activity.md`, now resolved) match the oracle on
  all three descriptors, 3 of 3 each; crate tests
  `a_notifier_failing_in_a_started_activity_replaces_its_failure` and
  `a_notifier_failing_on_a_started_success_is_the_sends_failure` (unswitched: the waiter's place is
  a scheduling order). The `scheduler.rs` comment is replaced by `end_failed_started`'s doc.
- **I2.** `Activation::notify_message`, the single `notifyObject`: a held send puts its message
  there (not where a native call is above the activation, nor on an activity with no activation),
  completion clears it, and a `SYNTAX` failure fails and notifies only the message the slot holds.
  n4, n4b, `re2a`, `re2b` match the oracle; queued note marked resolved.
- **M1.** `Interp::unnotified_messages` holds only messages whose notification is running or
  failed; notified is "completed and not in it". 1,000,000 `.message~new('a','length')~send`: max
  RSS 94,728 and 93,640 KB at head against 94,540 and 94,184 KB at base (two runs each,
  `/usr/bin/time -f %M`).
- **M3.** `with_idle_activity` asserts in debug that `read` left the running handle, the ready
  queue's length, the pin depth, the activation stack and the open constructs unchanged.
- Witnesses added (phase-8.txt): `message_notify_single_slot`, `message_notifier_fails`,
  `message_notifier_fails_untrapped`; 30/30 oracle, ours unswitched and every (`stab.sh`).
- Accepted: M2, `~line` of a switched-out activity mid-loop (P32 class).

P28 checks at d0274573d: `cargo fmt --all --check` exit 0; clippy `-p rexx-exec -p rexx-api
--all-targets -D warnings` exit 0, with `--features pinning` exit 0; `memcap 8G cargo test -p
rexx-exec --lib` 908 passed; `cargo test -p rexx-api` green (run before the commit on the same
sources); corpus release unswitched 732 of 732, release every 732 of 732, debug every 732 of 732;
collect_stress (release) 36 passed alongside the corpus (release) 732 of 732; concurrency_tests
(gate, release) 27 passed, Message table unchanged (pass 65, refused 3); pinning `measured::` 16
passed; outer_context (gate) release and debug 22 passed; refusal_sites re-derived, no change;
sourceline_oracle passed; method_bodies (release) passed, no drift. Bench "it works": every
`bench-programs/*.rex` identical to base 83c0a6ff1 except `heapshape`'s timings.
