# Task 3 report: activity numbers, errors in started activities, Message accessors

Status: DONE. Commits 82c073c6b, 2cb2f57a1 (code and witnesses), and the commit carrying this report's
gate section. Gates G1-G8 green at 2cb2f57a1.

Base: 9b06ca28a (checked before starting).

## What changed

* **Activity numbers** (`scheduler.rs`, `activity.rs`). `Activity::number` is assigned on first use by
  `Interp::activity_number` from a per-interpreter counter (`Activity::getIdntfr`, lazy as the oracle's is:
  the oracle numbers an OS thread when something first asks). `Scheduler::spawn` numbers the spawner
  (`setCallerStackFrameAsStringTable`, `Activity.cpp:1206`) and gives the new activity the front of
  `Activities::pooled`, a FIFO of ended activities' numbers (`Option<u32>`: a pooled thread that never
  got a number stays unnumbered). An ended activity is pooled while the pool holds no more than
  `MAX_POOLED` = 5 (`ActivityManager.cpp:650`, `items() > MAX_THREAD_POOL_SIZE` refuses), so up to six
  numbers are kept. `.context~thread` answers `activity_number()`.
* **Message natives** (`dispatch/object_protocol.rs`, rows in `dispatch.rs`): `HASRESULT` (the RESULT
  entry is now set only when the send answered a value, and `clear_completion` removes it and the
  condition on a resend, `MessageClass::clearCompletion`), `ERRORCONDITION`, `TARGET`, `MESSAGENAME`,
  `ARGUMENTS` (a copy). `Object~start`'s message now carries TARGET/MESSAGENAME/SCOPE/ARGUMENTS
  entries as `startCommon`'s `new MessageClass` does.
* **Message reuse** (`Error_Execution_message_reuse`, 98.915, new `Raised::message_reuse`): once a
  start has been sent to a message (`Interp::started_messages`, `flagMsgActivated`), send/sendWith/
  start/startWith raise 98.915, after setting the target/arguments as the oracle does. Needed because
  `Object~start` messages now hold a send: without the check, `m~send` on one would silently resend
  where the oracle raises (before this task it was a loud refusal).
* **Errors in started activities** (`scheduler.rs` `run_started`, `condition.rs`, `run.rs`,
  `run/condition.rs`, `error.rs`, `lib.rs`, `dispatch/library.rs`):
  * a failing held or started send pushes its message on `Activity::failed_sends`;
  * the condition object built for that failure is given to each such message as its
    `ERRORCONDITION` entry (`MessageClass::error`): at the SIGNAL trap that takes it
    (`run/condition.rs`), at the started activity's root (`settle_failed_sends`, before the report
    printed at once), at main's untrapped end (`lib.rs`), and where a native frame holds it
    (`hold_native_condition`);
  * `~result` re-raises through `Interp::reraise_kept_condition`: the report echoes the object's
    `TRACEBACK` (read directly with `list_items`, as `Activity::display` calls
    `ListClass::makeArray`), then names the `~result` clause's line through a new
    `FailureSite::Reraised { line, name }`, which echoes nothing and freezes the site stack
    (`seal_site_level` does not seal it, so no level the failure leaves records a line,
    `Activity::reraiseException` adds none); a trap of it takes the kept object with POSITION,
    PROGRAM, PACKAGE, PROPAGATED updated and INSTRUCTION SIGNAL (`trap_reraised_condition_object`).
  * only a `SYNTAX` condition completes a synchronous send's message with an error
    (`RexxActivation.cpp:2470`, `notifyObject->error` for SYNTAX alone); any other condition leaves
    it uncompleted (`record_held`). A started send records whatever ended its activity
    (`MessageDispatcher::handleError`), so `Message~start` now records through `Then::Started` like
    `Object~start` (`Then::Held` is the synchronous send alone).
* `.context~thread` itself: `dispatch/context.rs` `context_thread`.

## Rulings (design choices the spec does not settle)

* **R-T3-1 Numbering is lazy.** The oracle numbers an OS thread the first time `getIdntfr` runs on it
  (`Activity.cpp:108-112`), and the only callers before Task 9 are `.context~thread` and the spawner
  numbering at `~start`. Measured: a started activity that never asks and never starts leaves its
  number unassigned (`context_thread_lazy.rex`: C is 2, not 3, and E reuses the silent thread as 3).
  The pool therefore holds `Option<u32>`. Task 9's TraceObject THREAD must call `activity_number()`.
* **R-T3-2 Pool bound.** `poolActivity` refuses when `items() > 5`, so six numbers are kept; measured
  by `context_thread_pool_bound.rex` (second chain `9 8 7 6 5 4 10 11`).
* **R-T3-3 errorCondition is the interpreter's own condition object.** It answers the same
  `Body::Native` Directory a SIGNAL trap holds (and that `CONDITION('O')` copies to a store
  Directory). Its identity is what the oracle's is (`m~errorCondition == m~errorCondition`, POSITION
  updated by a re-raise, measured), but Directory methods built only for store directories
  (`ITEMS`, `HASINDEX`, `ALLINDEXES`) refuse loudly on it where the oracle answers: loud, not wrong.
* **R-T3-4 The re-raise reads TRACEBACK directly.** `Activity::display` calls
  `ListClass::makeArray` on the object's TRACEBACK without a send, so `reraise_kept_condition` takes
  the lines through `list::list_items` rather than sending MAKEARRAY: no new Rust-to-Rexx recursion
  site, so no PinKind or FRAME_PROBES case.
* **R-T3-5 98.915 is built here.** `Object~start` messages now hold their send, so without
  `checkReuse` a resend would answer where the oracle raises; the reuse check (activated flag only;
  `startPending` implies it) is the smallest change that keeps it from being silently wrong.
* **R-T3-6 Settling points.** The oracle builds the condition object at the raise and notifies the
  message as the condition passes the dispatching frame. This crate builds the object where the
  failure stops, so a failed send's message is given the object at each place a failure can stop:
  the SIGNAL trap, a started activity's root, main's end, a native frame holding it
  (`hold_native_condition`); `clear_failure_levels` (deferred replies, UNINIT) drops the list.

## Witnesses and red-before evidence

Corpus programs (`rust/corpus/lang/`, listed in `phase-8.txt`, SOURCELINE files generated with the
`sourceline_oracle.rs` driver). Each was compared on stdout, stderr and exit status separately with the
oracle from a fresh empty directory, and run 30 times on the oracle (`stab.sh`: distinct
(stdout, stderr, rc) triples). Base = release build of 9b06ca28a (git archive, own target dir, a
`Compiling rexx-exec` line seen).

| program | oracle 30 runs | head | base 9b06ca28a |
|---|---|---|---|
| context_thread_numbers | 30/30 one triple | same | stdout differs (`main 1 D 1 B 1 C 1`) |
| context_thread_sequential | 30/30 | same | stdout differs (all 1) |
| context_thread_pool_bound | 30/30 | same | stdout differs (all 1) |
| context_thread_lazy | 30/30 | same | stdout differs (all 1) |
| message_start_error_condition | 30/30 | same | all three differ (ERRORCONDITION loud, rc 120) |
| message_result_reraise | 30/30 | same | all three differ |
| message_result_relayed | 30/30 | same | all three differ |
| message_accessors | 30/30 | same | all three differ (HASRESULT loud) |
| message_reuse | 30/30 | same | all three differ (TARGET loud) |
| message_send_error_condition | 30/30 | same | all three differ (HASRESULT loud) |
| message_error_condition_after_main | 30/30 | same | all three differ |
| message_send_user_condition | 30/30 | same | all three differ |
| started_numeric_address | 30/30 | same | same: a regression witness only (NUMERIC not inherited and the default ADDRESS already held at the base) |

Stdout read for each: the thread programs print `main 1 D 3 B 2 C 3`, `first 2 / second 2 / main 1`,
`first  2 3 4 5 6 7 8 9 / second 9 8 7 6 5 4 10 11`, `B silent, C 2 / D, E 2 3`; the error programs
print `hasError 1`, the condition's code, POSITION, TRACEBACK size and first line, and the trap's
INSTRUCTION; `message_start_error_condition`'s stderr is the started report (line 21) followed by the
re-raise report repeating the started clause and naming line 15, which is the queued item's shape.

The error witnesses sleep 0.3 s after `~wait` (`SysSleep`): the oracle marks the message complete
before its started thread prints the report, and `reraiseException` writes POSITION into the shared
object, so without the sleep the oracle's own output varies (measured on the scratch probe `err1.rex`,
which waits with `~wait` and no sleep: two distinct triples in 30 runs, the started report naming the
re-raising line 13 in 2 of them instead of its own line 24).

Probes not made corpus programs: `~result` called from a `::requires`d package names that package in
the report (identical to the oracle; two files, which the corpus layout does not take).

Crate tests: `scheduler::tests::the_result_of_a_message_never_sent_is_refused` (ruling P27: the unsent
`~result` refusal, owner none, with `hasResult`/`completed`/`target`/`messageName` answering before
it); `run::tests::message::a_started_method_that_raises_has_an_error_and_reraises_at_result` now
expects the oracle's shape (it named RESULT and the calling line before); `group_runs::
the_outcome_table_of_the_message_group` asserts TEST_SEND and TEST_START pass.

## The queued item

`.superpowers/sdd/queued/2026-10-01-message-start-result-traceback.md` is closed by
`message_start_error_condition.rex` (and the reraise/relayed witnesses): the re-raise report is the
one-frame traceback plus the `~result` line, and the started activity's report is printed at once on
stderr, both identical to the oracle over 30 runs. The file is left in `queued/` for the controller.

## refusal-sites.tsv

Re-derived with `REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test refusal_sites
-- --test-threads=1`; then the plain run is green. Against 9b06ca28a with line numbers masked
(`diff <(git show 9b06ca28a:rust/corpus/refusal-sites.tsv | sed 's/\.rs:[0-9]*/.rs:N/') <(sed ... )`)
the one row change is the new `message_reuse` row (send surface, `agrees yes 98.915`, witness
`message_reuse.rex`); every other change is a moved line number. No generic refusal row existed for
the five Message methods (they refused through `Loud::native_method`'s missing-row path); their
`method-bodies.txt` rows moved from `loud` to `answers rc 0`
(`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`).

## Message table

`task-3-message-table.md` (raw `task-3-message-table.raw.md`): pass 51, refused 17 (Task 2: 49 / 19).
Changed rows: TEST_SEND (was the TARGET refusal) and TEST_START (was the HASRESULT refusal) now pass.
Remaining refusals: REPLY/REPLYWITH (13), MAKEARRAY of Object (2), HALT, NOTIFY. Object table: the 40
START tests all pass, unchanged.

## "It works" bench check

Every `rust/bench-programs/*.rex` on release builds of the base (9b06ca28a, own target dir) and the
first commit's tree: stdout, stderr and exit status identical except `heapshape`, whose stdout
prints timings (identical with digits masked). `extcall` ran with `LD_LIBRARY_PATH=build/lib` on both.

## Gates

Round 1 at 82c073c6b was stopped by me after G2 (G1, G2 exit 0): a probe made during the run
(`userraise.rex`) showed the first commit answering `~errorCondition` for a synchronous send ended by
a USER condition, where the oracle leaves the message uncompleted; fixed in 2cb2f57a1.

Round 2 at 2cb2f57a1 (`S=$W bash $W/p6-gates/gates.sh`): G1 fmt, G2 clippy from an empty target,
G3/G5 builds, G4 release and G6 debug `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace
--no-fail-fast` (corpus 692 of 692 matching in both), G7 clippy `--features pinning`, G8 pinning
self-tests: all exit 0, finished 2026-10-01T20:05:12+02:00, tree clean at the end.

## Concerns

1. `~errorCondition` answers the interpreter's own condition Directory (R-T3-3): `ITEMS`, `HASINDEX`,
   `ALLINDEXES` and other store-only Directory methods refuse loudly on it.
2. A synchronous send failing with SYNTAX whose failure stops somewhere other than the settling
   points of R-T3-6 leaves `~errorCondition` `.nil` while `~hasError` is 1; I found no such path,
   but the list is enumerated by reading, not derived.
3. The oracle's own error-witness output depends on timing (its report is printed after the message
   completes, and the re-raise mutates the shared object); the witnesses hold only with the 0.3 s
   sleep, measured 30/30 on this machine.
4. The first commit message states a count of witnesses ("Twelve"), against the prose rule; it is
   true for that commit (a thirteenth came in the second).
5. Task 9: TraceObject THREAD must use `activity_number()` so it numbers lazily as R-T3-1 records.

## Fix round 1 (review task-3-review.md)

* **I1** (`condition.rs` `build_condition_object_from`): a condition object built from unwound levels
  none of which has a package, with no running program (a started send to a primitive method failing
  at its activity's root) has no PACKAGE or POSITION and an empty PROGRAM, as
  `generateProgramInformation` with no Rexx frame. The started report for such a failure is now
  positionless (`scheduler.rs` `report_started_failure`: `Error 93:  ...` where it printed
  `running FILE line 0`, a pre-existing difference the witness needed fixed). Witness
  `started_primitive_error_condition.rex` (the reviewer's `sn1`).
* **I2** (`dispatch/object_protocol.rs` `record_held`): an `Ok` completion no longer overwrites an
  error a send of the same message recorded during it (`flagResultReturned` and `flagRaiseError` are
  separate flags); `~hasError` is 1 and `~result` re-raises. Witness `message_send_reentered.rex`
  (the reviewer's `re1`).
* **M4**: the pool's size removed from `context_thread_pool_bound.rex`'s comment (SOURCELINE file
  regenerated; still identical on the oracle, 30/30).
* Oracle crash: entry 22 in `rust/corpus/oracle-crashes.txt` (the reviewer's `tb2`), SIGSEGV rc 139
  3 runs of 3 under `memcap 1G timeout -s KILL 20`; cause read at `Activity.cpp:1428`.
* M1 and M2 queued: `.superpowers/sdd/queued/2026-10-01-condition-object-directory-methods.md` and
  `2026-10-01-message-notify-single-slot.md`, each with its probe and both sides' output measured at
  this round's tree. M3 left to Task 6 as ruled.

Witnesses, each compared on stdout, stderr and rc separately and run 30 times on the oracle:

| program | oracle 30 runs | head | 9c9f6e510 (before this round) |
|---|---|---|---|
| started_primitive_error_condition | 30/30 one triple | same | stdout and stderr differ (POSITION 0, PROGRAM REXX, PACKAGE; `line 0`) |
| message_send_reentered | 30/30 | same | stdout differs (`0 1 1 0`, `result ok`) |

Checks before committing: fmt clean; `cargo test --release -p rexx-exec --lib` 880 passed;
`refusal_sites` green with no row change; corpus 694 of 694 matching.
