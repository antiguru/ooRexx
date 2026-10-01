# Task 3 report: activity numbers, errors in started activities, Message accessors

Status: IN PROGRESS (written as the work goes).

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
