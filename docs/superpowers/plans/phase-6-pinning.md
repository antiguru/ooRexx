# Phase 6 pinning

Exit criterion 1's test list (spec `2026-09-29-phase-6-concurrency-design.md` section 9) and the
would-be pinned parks those tests reach on today's code (section 11).

## Commands

From `rust/`. The derivation and its check against the two blocks below:

```
cargo test -p rexx-exec --test concurrency_tests
```

The derivation over another root, printed rather than checked:

```
REXX_CONCURRENCY_ROOT=<dir> cargo test -p rexx-exec --test concurrency_tests \
  the_derived_list_is_the_committed_one -- --nocapture
```

The measured table, written to `<target>/tmp/pinning-table.md`:

```
cargo test --release -p rexx-exec --features pinning --test concurrency_tests \
  pinned_parks_over_the_derived_list -- --nocapture
```

A row is a test method whose body, or a `::METHOD`, `::ROUTINE` or `::RESOURCE` of its group file
it reaches by name (a class named reaches its `INIT`; a class method `ACTIVATE` or `INIT` is reached by
every test of the file), uses a feature the criterion names; comments
and string contents are ignored except for the TraceObject entry names. `Message~reply` is matched
beside `~start`.

## Derived list

```text
base/bif/STREAM.testGroup TEST_QUERYDIR_EXISTS SysSleep
base/bif/TIME.testGroup TEST_2 SysSleep
base/bif/TIME.testGroup TEST_3 SysSleep
base/bif/TIME.testGroup TEST_4 SysSleep
base/bif/TIME.testGroup TEST_5 SysSleep
base/bif/TIME.testGroup TEST_8 SysSleep
base/bif/TIME.testGroup TEST_9 SysSleep
base/bif/TIME.testGroup TEST_10 SysSleep
base/bif/TIME.testGroup TEST_11 SysSleep
base/class/Alarm.testGroup TEST_BASE_ALARM Alarm,GUARD,REPLY,SysSleep
base/class/Alarm.testGroup TEST_ALARM_NO_TIME Alarm
base/class/Alarm.testGroup TEST_ALARM_NO_TARGET Alarm
base/class/Alarm.testGroup TEST_ALARM_BAD_TIME Alarm
base/class/Alarm.testGroup TEST_ALARM_NEGATIVE_TIME Alarm
base/class/Alarm.testGroup TEST_ALARM_BAD_TARGET Alarm
base/class/Class.testGroup TEST_SUBCLASSES Alarm
base/class/DateTime.testGroup TEST_ELAPSED1 SysSleep
base/class/EventSemaphore.testGroup TEST_NEW_ONE_ARG semaphore class
base/class/EventSemaphore.testGroup TEST_ISPOSTED_ONE_ARG semaphore class
base/class/EventSemaphore.testGroup TEST_POST_ONE_ARG semaphore class
base/class/EventSemaphore.testGroup TEST_RESET_ONE_ARG semaphore class
base/class/EventSemaphore.testGroup TEST_POST_RESET semaphore class
base/class/EventSemaphore.testGroup TEST_WAIT_TWO_ARGS semaphore class
base/class/EventSemaphore.testGroup TEST_WAIT_NUMBER semaphore class
base/class/EventSemaphore.testGroup TEST_WAIT_SIMPLE semaphore class
base/class/EventSemaphore.testGroup TEST_WAIT_CONCURRENT GUARD,SysSleep,semaphore class,~start
base/class/Message.testGroup TEST_START ~start
base/class/Message.testGroup TEST_REPLY Message~reply
base/class/Message.testGroup TEST_NOTIFY Message~reply,~start
base/class/Message.testGroup TEST_SUPER_OVERRIDE Message~reply,~start
base/class/Message.testGroup TEST_STARTWITH_NO_ARRAY ~start
base/class/Message.testGroup TEST_STARTWITH_NOT_ARRAY ~start
base/class/Message.testGroup TEST_STARTWITH_TOO_MANY ~start
base/class/Message.testGroup TEST_REPLYWITH_NO_ARRAY Message~reply
base/class/Message.testGroup TEST_REPLYWITH_NOT_ARRAY Message~reply
base/class/Message.testGroup TEST_REPLYWITH_TOO_MANY Message~reply
base/class/Message.testGroup TEST_START_OVERRIDE_CONTEXT ~start
base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_CONTEXT ~start
base/class/Message.testGroup TEST_REPLY_OVERRIDE_CONTEXT Message~reply
base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_CONTEXT Message~reply
base/class/Message.testGroup TEST_START_OVERRIDE_NOT_FOUND ~start
base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NOT_FOUND ~start
base/class/Message.testGroup TEST_REPLY_OVERRIDE_NOT_FOUND Message~reply
base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NOT_FOUND Message~reply
base/class/Message.testGroup TEST_START_OVERRIDE_NO_METHOD ~start
base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NO_METHOD ~start
base/class/Message.testGroup TEST_REPLY_OVERRIDE_NO_METHOD Message~reply
base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NO_METHOD Message~reply
base/class/Message.testGroup TEST_START_OVERRIDE_NOT_NON_SCOPE ~start
base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE ~start
base/class/Message.testGroup TEST_REPLY_OVERRIDE_NOT_NON_SCOPE Message~reply
base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NOT_NON_SCOPE Message~reply
base/class/Message.testGroup TEST_HALT_START SysSleep,~start
base/class/Message.testGroup TEST_START_OVERRIDE_FROM_NONSELF ~start
base/class/Message.testGroup TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS ~start
base/class/Message.testGroup TEST_START_OVERRIDE_AMONG_MIXINCLASSES ~start
base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF ~start
base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS ~start
base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES ~start
base/class/Method.testGroup TESTDIRECTIVES TraceObject field
base/class/MethodArgs.testGroup TEST_REQUEST_STRING_CLASS Alarm,Ticker
base/class/MethodArgs.testGroup TEST_REQUEST_STRING_OBJECT Alarm,Ticker
base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STRING Alarm,Ticker
base/class/MethodArgs.testGroup TEST_REQUEST_STRING_METHOD Alarm,Ticker
base/class/MethodArgs.testGroup TEST_REQUEST_STRING_ROUTINE Alarm,Ticker
base/class/MethodArgs.testGroup TEST_REQUEST_STRING_PACKAGE Alarm,Ticker
base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MESSAGE Alarm,Ticker
base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STREAM Alarm,Ticker
base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MUTABLEBUFFER Alarm,Ticker
base/class/MethodArgs.testGroup TEST_REQUEST_STRING_FILE Alarm,Ticker
base/class/MutexSemaphore.testGroup TEST_NEW_ONE_ARG semaphore class
base/class/MutexSemaphore.testGroup TEST_ACQUIRE_TWO_ARGS semaphore class
base/class/MutexSemaphore.testGroup TEST_ACQUIRE_NUMBER semaphore class
base/class/MutexSemaphore.testGroup TEST_RELEASE_ONE_ARG semaphore class
base/class/MutexSemaphore.testGroup TEST_ACQUIRE_ACQUIRE_SIMPLE semaphore class
base/class/MutexSemaphore.testGroup TEST_ACQUIRE_RELEASE_SIMPLE semaphore class
base/class/MutexSemaphore.testGroup TEST_ACQUIRE_RELEASE_NESTED semaphore class
base/class/MutexSemaphore.testGroup TEST_EXCLUSION GUARD,REPLY,semaphore class
base/class/Object.testGroup TESTSTART01 ~start
base/class/Object.testGroup TESTSTARTWITH01 ~start
base/class/Object.testGroup TEST_START_NO_NAME ~start
base/class/Object.testGroup TEST_START_NO_NAME2 ~start
base/class/Object.testGroup TEST_START_NOT_STRING ~start
base/class/Object.testGroup TEST_START_NO_METHOD SysSleep,~start
base/class/Object.testGroup TEST_START_OVERRIDE ~start
base/class/Object.testGroup TEST_START_OVERRIDE_CONTEXT ~start
base/class/Object.testGroup TEST_START_OVERRIDE_EMPTY_ARRAY ~start
base/class/Object.testGroup TEST_START_OVERRIDE_MISSING_NAME ~start
base/class/Object.testGroup TEST_START_OVERRIDE_MISSING_SCOPE ~start
base/class/Object.testGroup TEST_START_OVERRIDE_EXTRA_STUFF ~start
base/class/Object.testGroup TEST_START_OVERRIDE_NON_STRING_NAME ~start
base/class/Object.testGroup TEST_START_OVERRIDE_NON_CLASS_SCOPE ~start
base/class/Object.testGroup TEST_START_OVERRIDE_NON_CLASS_SCOPE2 ~start
base/class/Object.testGroup TEST_START_OVERRIDE_NOT_FOUND SysSleep,~start
base/class/Object.testGroup TEST_START_OVERRIDE_NOT_NON_SCOPE ~start
base/class/Object.testGroup TEST_START_OVERRIDE_NO_METHOD ~start
base/class/Object.testGroup TEST_STARTWITH_NO_NAME ~start
base/class/Object.testGroup TEST_STARTWITH_NO_NAME2 ~start
base/class/Object.testGroup TEST_STARTWITH_NOT_STRING ~start
base/class/Object.testGroup TEST_STARTWITH_NO_METHOD SysSleep,~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_CONTEXT ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_EMPTY_ARRAY ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_MISSING_NAME ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_MISSING_SCOPE ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_EXTRA_STUFF ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_STRING_NAME ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE2 ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NOT_FOUND ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NO_METHOD ~start
base/class/Object.testGroup TEST_START_OVERRIDE_FROM_NONSELF ~start
base/class/Object.testGroup TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS ~start
base/class/Object.testGroup TEST_START_OVERRIDE_AMONG_MIXINCLASSES ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS ~start
base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES ~start
base/class/RexxContext.testGroup TEST_INTERPRETER_THREAD_INVOCATION .context~thread,REPLY,SysSleep,TraceObject field,~start
base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_CANCEL Ticker
base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_TRIGGER SysSleep,Ticker
base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_CANCEL Ticker
base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_TRIGGER SysSleep,Ticker
base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_CANCEL Ticker
base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER SysSleep,Ticker
base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_CANCEL Ticker
base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER SysSleep,Ticker
base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER_MULTIPLE SysSleep,Ticker
base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER_MESSAGE SysSleep,Ticker
base/class/Ticker.testGroup TEST_TICKER_NO_ARGS Ticker
base/class/Ticker.testGroup TEST_TICKER_NO_TIME Ticker
base/class/Ticker.testGroup TEST_TICKER_NO_TARGET Ticker
base/class/Ticker.testGroup TEST_TICKER_BAD_TIME Ticker
base/class/Ticker.testGroup TEST_TICKER_NEGATIVE_TIME Ticker
base/class/Ticker.testGroup TEST_TICKER_ALARM_TIME Ticker
base/class/Ticker.testGroup TEST_TICKER_DATETIME_TIME Ticker
base/class/Ticker.testGroup TEST_TICKER_NEGATIVE_TIMESPAN_TIME Ticker
base/class/Ticker.testGroup TEST_TICKER_BAD_TARGET Ticker
base/class/Ticker.testGroup TEST_TICKER_MESSAGE_NOTIFICATION_TARGET Ticker
base/class/Ticker.testGroup TEST_TICKER_FOUR_ARGS Ticker
base/class/Ticker.testGroup TEST_ATTACHMENT_TOO_MANY_ARGS Ticker
base/class/Ticker.testGroup TEST_CANCEL_TOO_MANY_ARGS Ticker
base/class/Ticker.testGroup TEST_CANCELED_TOO_MANY_ARGS Ticker
base/class/Ticker.testGroup TEST_CANCELLED_TOO_MANY_ARGS Ticker
base/class/Ticker.testGroup TEST_CANCEL_IMMEDIATELY Ticker
base/class/Ticker.testGroup TEST_CANCEL_TWICE SysSleep,Ticker
base/class/Ticker.testGroup TEST_INTERVAL_TOO_MANY_ARGS Ticker
base/class/Ticker.testGroup TEST_TICKER_INTERVAL_ZERO Ticker
base/class/Ticker.testGroup TEST_TICKER_NEW_INTERVAL_TIMESPAN Ticker
base/directives/ATTRIBUTE.testGroup TEST001 TraceObject field
base/directives/ATTRIBUTE.testGroup TESTDELEGATE TraceObject field
base/directives/CONSTANT.testGroup TEST_CONSTANT_METHOD_PROPERTIES TraceObject field
base/directives/METHOD.testGroup TESTGUARDEDACCESS REPLY,SysSleep
base/directives/METHOD.testGroup TESTDELEGATE TraceObject field
base/keyword/CALL.testGroup TEST_4 SysSleep
base/keyword/GUARD.testGroup TEST_WHEN_NOVALUE GUARD
base/keyword/GUARD.testGroup TEST_WHEN_NOT_BOOLEAN GUARD
base/keyword/GUARD.testGroup TEST_ON_OFF_CONSECUTIVE GUARD
base/keyword/GUARD.testGroup TEST_ON_DEFAULT GUARD,REPLY,SysSleep
base/keyword/GUARD.testGroup TEST_ON GUARD,REPLY,SysSleep
base/keyword/GUARD.testGroup TEST_OFF GUARD,REPLY,SysSleep
base/keyword/GUARD.testGroup TEST_UNGUARDED GUARD,REPLY,SysSleep
base/keyword/GUARD.testGroup TEST_ON_OFF GUARD,REPLY,SysSleep
base/keyword/GUARD.testGroup TEST_WHEN_SINGLE_NO_WAIT GUARD
base/keyword/GUARD.testGroup TEST_WHEN_USE_LOCAL_NO_WAIT GUARD
base/keyword/GUARD.testGroup TEST_WHEN_SINGLE_UNINITIALIZED_NO_WAIT GUARD
base/keyword/GUARD.testGroup TEST_WHEN_MULTIPLE_NO_WAIT GUARD
base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE_TRIGGER GUARD,REPLY,SysSleep
base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE GUARD,REPLY,SysSleep
base/keyword/GUARD.testGroup TEST_WAIT_MULTIPLE GUARD,REPLY,SysSleep
base/keyword/RAISE.testGroup TEST_RAISE_INSERT_CRLF REPLY,SysSleep
base/keyword/REPLY.testGroup TEST_REPLY_ROUTINE REPLY
base/keyword/REPLY.testGroup TEST_REPLY_PROCEDURE REPLY
base/keyword/REPLY.testGroup TEST_REPLY_CALL REPLY
base/keyword/REPLY.testGroup TEST_REPLY_TWICE_REPLYASSERT REPLY
base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_REPLYASSERT REPLY
base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT REPLY
base/keyword/REPLY.testGroup TEST_REPLY_EXIT_CODE_REPLYASSERT REPLY
base/keyword/REPLY.testGroup TEST_REPLY_STACK_REPLYASSERT REPLY
base/keyword/REPLY.testGroup TEST_REPLY_PLAIN REPLY
base/keyword/REPLY.testGroup TEST_REPLY_STRING REPLY
base/keyword/REPLY.testGroup TEST_REPLY_ARRAY REPLY
base/keyword/REPLY.testGroup TEST_REPLY_NIL REPLY
base/keyword/REPLY.testGroup TEST_REPLY_NOP REPLY
base/keyword/REPLY.testGroup TEST_REPLY__CODE_RETURN REPLY
base/keyword/REPLY.testGroup TEST_REPLY__CODE_EXIT REPLY
base/keyword/REPLY.testGroup TEST_REPLY_SAME_REPLYASSERT REPLY,SysSleep
base/keyword/REPLY.testGroup TEST_REPLY_CONCURRENT REPLY,SysSleep
base/keyword/TRACE.testGroup TEST_TRACE_GUARD GUARD
base/keyword/TRACE.testGroup TEST_TRACE_REPLY Message~reply,REPLY,SysSleep
base/keyword/TRACE_TraceObject.testGroup TEST_TRACEOBJECT_COLLECTOR REPLY,SysSleep,TraceObject field
base/keyword/TRACE_TraceObject.testGroup TEST_CALLER_STACK_FRAME_REPLY_START GUARD,REPLY,SysSleep,~start
base/rexxutil/SysSleep.testGroup TEST_SLEEP_NO_ARG SysSleep
base/rexxutil/SysSleep.testGroup TEST_SLEEP_TWO_ARGS SysSleep
base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID SysSleep
base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID_NEGATIVE SysSleep
base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID_TOO_LARGE SysSleep
base/rexxutil/SysSleep.testGroup TEST_SLEEP_DURATION SysSleep
base/rexxutil/SysSleep.testGroup TEST_SLEEP_CONCURRENT ~start
base/special.variables/RESULT_RC_SIGL.testGroup TEST_RESULT_WITH_REPLY REPLY
doc/rexxref/chapter5/Section1.testGroup TEST_OBJECT_START SysSleep,~start
regressions/bug2003_guard_when.testGroup TEST_GUARD_WHEN_1 GUARD,Message~reply
```

## Exclusions

```text
base/class/RexxQueue.testGroup TEST_DELETE_WHILE_LINEIN -- RexxQueue, Phase 10
base/class/RexxQueue.testGroup TEST_LINEIN_WAITS_PULL_DOESNT -- RexxQueue, Phase 10
extensions/rxsock/socketClass.testGroup TEST_005 -- extension Phase 10 recompiles
extensions/rxsock/socketClass.testGroup TEST_006 -- extension Phase 10 recompiles
extensions/rxsock/socketClass.testGroup TEST_007 -- extension Phase 10 recompiles
samples/scclient.testGroup TEST_04_SCCLIENT.REX -- sample Phase 10 recompiles
samples/scserver.testGroup TEST_04_SCSERVER.REX -- sample Phase 10 recompiles
samples/sfclient.testGroup TEST_04_SFCLIENT.REX -- sample Phase 10 recompiles
samples/sfserver.testGroup TEST_04_SFSERVER.REX -- sample Phase 10 recompiles
```

## Counter

Feature `pinning` of `rexx-exec`, off by default; `src/pinning.rs`. Park points:

| park | site |
|---|---|
| GuardOn, GuardWhen | `Interp::exec_guard` |
| Reply | `Interp::exec_reply` |
| MessageResult | `native_message_result` |
| MessageWait, SemaphoreWait | `Interp::invocable`'s refusal, for `Message~WAIT`, `EventSemaphore~WAIT`, `MutexSemaphore~ACQUIRE` |
| SysSemWait | `Interp::resolve_routine_call`'s refusal, for `SysWaitEventSem`, `SysRequestMutexSem` |
| SysSleep | `builtin::rexxutil::sleep` |
| Timer | `Interp::invoke`'s deferred-entry refusal, for `alarm_startTimer`, `ticker_waitTimer` |

The guarded-method reservation at a send is not counted. Parks inside a child process a test runs
as a command are not counted.

Pinned frames:

| frame | site |
|---|---|
| SortComparator | `sort_by`'s `merge_sort` |
| Conversion | `Interp::required_string_dispatch`'s `STRING` send, `send_make_string`, `request_array_for_over` |
| Unknown | `Interp::unknown_or_nomethod` |
| Forward | `Interp::exec_forward`'s send |
| Delegate | `Interp::send_to_delegate` |
| Operator | `Interp::send_operator` |
| TrapHandler | `deliver_one_pending_trap` |
| NativeApiCallback | `run_library_method`, `run_library_routine`, `run_command_handler` |
| LibraryEntry | `run_package_hook` |
| StreamWrapper | `builtin/stream.rs`'s sends |
| PullWrapper | `Interp::linein_line` |
| OutputWrapper | `Interp::say_evaluated`'s routed send |
| TraceWrapper | `deliver_trace_line` |
| RedirectWrapper | `Interp::io_context` at `ADDRESS`, `IoContext::finish_lines` |
| LoopHeader | the flat loop's `WHILE` and `UNTIL` |
| NestedLoop | `Op::LoopRun`'s `run_loop_with_header` |
| Interpret | `run_fragment`'s callers |
| TreeEval | `Op::EvalExpr`; a call's non-leaf argument in `run/call.rs` |
| TreeSend | `Op::Message` |
| OpExec | `Op::Exec` |
| Program | `Interp::run_loaded` under a running activation |
| DeferredReply | `Interp::resume_reply` |
| Uninit | `Interp::run_one_uninit`'s send |
| Native | `Interp::invoke`'s run-half native and implemented-external arms; the begin halves, `RESULT` and `WAIT` among them, push none |

## Measured

At the commit that last wrote this section; each test run alone with `-U -V 2 -t`, `ooTest.frm`'s
`rxfuncquery` probes removed. Arrival counts of busy-wait loops depend on timing. "refused at" is
the first `rexx-exec:` line of stderr, the deadline report included.

| park | frames | tests | arrivals |
|---|---|---|---|
| GuardOn | TreeSend > OpExec | 6 | 8 |
| GuardWhen | OpExec | 1 | 1 |
| GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 10 | 10 |
| GuardWhen | TreeSend > OpExec | 12 | 21 |
| Reply | OpExec | 2 | 3 |
| Reply | TreeEval > OpExec > TreeEval > OpExec | 10 | 10 |
| Reply | TreeSend > OpExec | 29 | 32 |
| Reply | TreeSend > TreeEval > OpExec | 2 | 2 |
| Reply | TreeSend > TreeEval > TreeSend > OpExec | 1 | 1 |
| MessageResult | TreeSend | 10 | 24 |
| MessageResult | TreeSend > TreeEval | 4 | 14 |
| MessageWait | TreeSend | 1 | 1 |
| SemaphoreWait | TreeSend | 7 | 7 |
| SemaphoreWait | TreeSend > TreeEval | 1 | 1 |
| SysSleep | - | 3 | 28 |
| SysSleep | TreeSend | 26 | 1449 |
| SysSleep | TreeSend > TreeEval | 1 | 1 |
| Timer | TreeSend | 11 | 11 |

arrivals 1624, with a frame other than TreeEval, TreeSend or OpExec 0

| test | outcome | park | frames | arrivals |
|---|---|---|---|---|
| base/bif/STREAM.testGroup TEST_QUERYDIR_EXISTS | pass, rc 0 | no park reached in-process | | |
| base/bif/TIME.testGroup TEST_2 | pass, rc 0 | SysSleep | TreeSend | 3 |
| base/bif/TIME.testGroup TEST_3 | pass, rc 0 | SysSleep | TreeSend | 5 |
| base/bif/TIME.testGroup TEST_4 | failure, rc 1 | SysSleep | TreeSend | 3 |
| base/bif/TIME.testGroup TEST_5 | failure, rc 1 | SysSleep | TreeSend | 5 |
| base/bif/TIME.testGroup TEST_8 | pass, rc 0 | SysSleep | TreeSend | 4 |
| base/bif/TIME.testGroup TEST_9 | pass, rc 0 | SysSleep | TreeSend | 6 |
| base/bif/TIME.testGroup TEST_10 | failure, rc 1 | SysSleep | TreeSend | 3 |
| base/bif/TIME.testGroup TEST_11 | failure, rc 1 | SysSleep | TreeSend | 5 |
| base/class/Alarm.testGroup TEST_BASE_ALARM | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeSend > OpExec | 1 |
| base/class/Alarm.testGroup TEST_BASE_ALARM | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeSend > TreeEval > OpExec | 1 |
| base/class/Alarm.testGroup TEST_BASE_ALARM | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeSend > OpExec | 1 |
| base/class/Alarm.testGroup TEST_BASE_ALARM | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | SysSleep | - | 1 |
| base/class/Alarm.testGroup TEST_BASE_ALARM | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/Alarm.testGroup TEST_ALARM_NO_TIME | pass, rc 0 | no park reached in-process | | |
| base/class/Alarm.testGroup TEST_ALARM_NO_TARGET | pass, rc 0 | no park reached in-process | | |
| base/class/Alarm.testGroup TEST_ALARM_BAD_TIME | pass, rc 0 | no park reached in-process | | |
| base/class/Alarm.testGroup TEST_ALARM_NEGATIVE_TIME | pass, rc 0 | no park reached in-process | | |
| base/class/Alarm.testGroup TEST_ALARM_BAD_TARGET | pass, rc 0 | no park reached in-process | | |
| base/class/Class.testGroup TEST_SUBCLASSES | pass, rc 0 | no park reached in-process | | |
| base/class/DateTime.testGroup TEST_ELAPSED1 | pass, rc 0 | SysSleep | TreeSend | 1 |
| base/class/EventSemaphore.testGroup TEST_NEW_ONE_ARG | pass, rc 0 | no park reached in-process | | |
| base/class/EventSemaphore.testGroup TEST_ISPOSTED_ONE_ARG | refused at method "ISPOSTED" of class "EventSemaphore" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/EventSemaphore.testGroup TEST_POST_ONE_ARG | refused at method "POST" of class "EventSemaphore" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/EventSemaphore.testGroup TEST_RESET_ONE_ARG | refused at method "RESET" of class "EventSemaphore" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/EventSemaphore.testGroup TEST_POST_RESET | refused at method "ISPOSTED" of class "EventSemaphore" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/EventSemaphore.testGroup TEST_WAIT_TWO_ARGS | refused at method "WAIT" of class "EventSemaphore" is not implemented (Phase 9) | SemaphoreWait | TreeSend | 1 |
| base/class/EventSemaphore.testGroup TEST_WAIT_NUMBER | refused at method "WAIT" of class "EventSemaphore" is not implemented (Phase 9) | SemaphoreWait | TreeSend | 1 |
| base/class/EventSemaphore.testGroup TEST_WAIT_SIMPLE | refused at method "WAIT" of class "EventSemaphore" is not implemented (Phase 9) | SemaphoreWait | TreeSend | 1 |
| base/class/EventSemaphore.testGroup TEST_WAIT_CONCURRENT | refused at method "WAIT" of class "EventSemaphore" is not implemented (Phase 9) | SemaphoreWait | TreeSend > TreeEval | 1 |
| base/class/Message.testGroup TEST_START | refused at method "START" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLY | refused at method "REPLY" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_NOTIFY | refused at method "NOTIFY" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_SUPER_OVERRIDE | refused at method "REPLY" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_STARTWITH_NO_ARRAY | refused at method "STARTWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_STARTWITH_NOT_ARRAY | refused at method "STARTWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_STARTWITH_TOO_MANY | refused at method "STARTWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLYWITH_NO_ARRAY | refused at method "REPLYWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLYWITH_NOT_ARRAY | refused at method "REPLYWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLYWITH_TOO_MANY | refused at method "REPLYWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_START_OVERRIDE_CONTEXT | refused at method "START" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_CONTEXT | refused at method "STARTWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_CONTEXT | refused at method "REPLY" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_CONTEXT | refused at method "REPLYWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_START_OVERRIDE_NOT_FOUND | refused at method "START" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NOT_FOUND | refused at method "STARTWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_NOT_FOUND | refused at method "REPLY" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NOT_FOUND | refused at method "REPLYWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_START_OVERRIDE_NO_METHOD | refused at method "START" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NO_METHOD | refused at method "STARTWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_NO_METHOD | refused at method "REPLY" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NO_METHOD | refused at method "REPLYWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_START_OVERRIDE_NOT_NON_SCOPE | refused at method "START" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE | refused at method "STARTWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLY_OVERRIDE_NOT_NON_SCOPE | refused at method "REPLY" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_REPLYWITH_OVERRIDE_NOT_NON_SCOPE | refused at method "REPLYWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_HALT_START | refused at method "HALT" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_START_OVERRIDE_FROM_NONSELF | refused at method "START" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | no park reached in-process | | |
| base/class/Message.testGroup TEST_START_OVERRIDE_AMONG_MIXINCLASSES | refused at method "START" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF | refused at method "STARTWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | no park reached in-process | | |
| base/class/Message.testGroup TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES | refused at method "STARTWITH" of class "Message" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/Method.testGroup TESTDIRECTIVES | refused at DO is not implemented | no park reached in-process | | |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_CLASS | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_CLASS | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeEval > OpExec > TreeEval > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_CLASS | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_OBJECT | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_OBJECT | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeEval > OpExec > TreeEval > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_OBJECT | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STRING | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STRING | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeEval > OpExec > TreeEval > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STRING | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_METHOD | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_METHOD | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeEval > OpExec > TreeEval > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_METHOD | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_ROUTINE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_ROUTINE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeEval > OpExec > TreeEval > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_ROUTINE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_PACKAGE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_PACKAGE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeEval > OpExec > TreeEval > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_PACKAGE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MESSAGE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MESSAGE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeEval > OpExec > TreeEval > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MESSAGE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STREAM | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STREAM | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeEval > OpExec > TreeEval > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_STREAM | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MUTABLEBUFFER | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MUTABLEBUFFER | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeEval > OpExec > TreeEval > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_MUTABLEBUFFER | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_FILE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeEval > OpExec > TreeSend > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_FILE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeEval > OpExec > TreeEval > OpExec | 1 |
| base/class/MethodArgs.testGroup TEST_REQUEST_STRING_FILE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Timer | TreeSend | 1 |
| base/class/MutexSemaphore.testGroup TEST_NEW_ONE_ARG | pass, rc 0 | no park reached in-process | | |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_TWO_ARGS | refused at method "ACQUIRE" of class "MutexSemaphore" is not implemented (Phase 9) | SemaphoreWait | TreeSend | 1 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_NUMBER | refused at method "ACQUIRE" of class "MutexSemaphore" is not implemented (Phase 9) | SemaphoreWait | TreeSend | 1 |
| base/class/MutexSemaphore.testGroup TEST_RELEASE_ONE_ARG | refused at method "RELEASE" of class "MutexSemaphore" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_ACQUIRE_SIMPLE | refused at method "ACQUIRE" of class "MutexSemaphore" is not implemented (Phase 9) | SemaphoreWait | TreeSend | 1 |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_RELEASE_SIMPLE | refused at method "RELEASE" of class "MutexSemaphore" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/MutexSemaphore.testGroup TEST_ACQUIRE_RELEASE_NESTED | refused at method "RELEASE" of class "MutexSemaphore" is not implemented (Phase 9) | no park reached in-process | | |
| base/class/MutexSemaphore.testGroup TEST_EXCLUSION | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeSend > OpExec | 1 |
| base/class/MutexSemaphore.testGroup TEST_EXCLUSION | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeSend > OpExec | 1 |
| base/class/MutexSemaphore.testGroup TEST_EXCLUSION | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | SemaphoreWait | TreeSend | 1 |
| base/class/Object.testGroup TESTSTART01 | pass, rc 0 | MessageResult | TreeSend | 1 |
| base/class/Object.testGroup TESTSTARTWITH01 | pass, rc 0 | MessageResult | TreeSend | 1 |
| base/class/Object.testGroup TEST_START_NO_NAME | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_NO_NAME2 | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_NOT_STRING | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_NO_METHOD | pass, rc 0 | MessageResult | TreeSend | 1 |
| base/class/Object.testGroup TEST_START_OVERRIDE | pass, rc 0 | MessageResult | TreeSend > TreeEval | 6 |
| base/class/Object.testGroup TEST_START_OVERRIDE_CONTEXT | pass, rc 0 | MessageResult | TreeSend | 1 |
| base/class/Object.testGroup TEST_START_OVERRIDE_EMPTY_ARRAY | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_OVERRIDE_MISSING_NAME | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_OVERRIDE_MISSING_SCOPE | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_OVERRIDE_EXTRA_STUFF | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_OVERRIDE_NON_STRING_NAME | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_OVERRIDE_NON_CLASS_SCOPE | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_OVERRIDE_NON_CLASS_SCOPE2 | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_OVERRIDE_NOT_FOUND | pass, rc 0 | MessageResult | TreeSend > TreeEval | 1 |
| base/class/Object.testGroup TEST_START_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_OVERRIDE_NO_METHOD | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_NO_NAME | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_NO_NAME2 | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_NOT_STRING | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_NO_METHOD | pass, rc 0 | MessageResult | TreeSend | 1 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE | pass, rc 0 | MessageResult | TreeSend > TreeEval | 6 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_CONTEXT | pass, rc 0 | MessageResult | TreeSend | 1 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_EMPTY_ARRAY | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_MISSING_NAME | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_MISSING_SCOPE | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_EXTRA_STUFF | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_STRING_NAME | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE2 | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NOT_FOUND | pass, rc 0 | MessageResult | TreeSend > TreeEval | 1 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_NO_METHOD | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_OVERRIDE_FROM_NONSELF | pass, rc 0 | MessageResult | TreeSend | 4 |
| base/class/Object.testGroup TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_START_OVERRIDE_AMONG_MIXINCLASSES | pass, rc 0 | MessageResult | TreeSend | 5 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF | pass, rc 0 | MessageResult | TreeSend | 4 |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS | pass, rc 0 | no park reached in-process | | |
| base/class/Object.testGroup TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES | pass, rc 0 | MessageResult | TreeSend | 5 |
| base/class/RexxContext.testGroup TEST_INTERPRETER_THREAD_INVOCATION | failure, rc 1 | Reply | TreeSend > TreeEval > TreeSend > OpExec | 1 |
| base/class/RexxContext.testGroup TEST_INTERPRETER_THREAD_INVOCATION | failure, rc 1 | SysSleep | TreeSend > TreeEval | 1 |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_CANCEL | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_TRIGGER | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_CANCEL | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_TRIGGER | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_CANCEL | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_CANCEL | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER_MULTIPLE | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER_MESSAGE | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_NO_ARGS | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_NO_TIME | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_NO_TARGET | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_BAD_TIME | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_NEGATIVE_TIME | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_ALARM_TIME | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_DATETIME_TIME | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_NEGATIVE_TIMESPAN_TIME | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_BAD_TARGET | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_MESSAGE_NOTIFICATION_TARGET | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_FOUR_ARGS | pass, rc 0 | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_ATTACHMENT_TOO_MANY_ARGS | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_CANCEL_TOO_MANY_ARGS | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_CANCELED_TOO_MANY_ARGS | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_CANCELLED_TOO_MANY_ARGS | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_CANCEL_IMMEDIATELY | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_CANCEL_TWICE | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_INTERVAL_TOO_MANY_ARGS | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_INTERVAL_ZERO | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/class/Ticker.testGroup TEST_TICKER_NEW_INTERVAL_TIMESPAN | refused at the LIBRARY REXX entry point "ticker_createTimer" is not implemented (Phase 6) | no park reached in-process | | |
| base/directives/ATTRIBUTE.testGroup TEST001 | pass, rc 0 | no park reached in-process | | |
| base/directives/ATTRIBUTE.testGroup TESTDELEGATE | failure, rc 1 | no park reached in-process | | |
| base/directives/CONSTANT.testGroup TEST_CONSTANT_METHOD_PROPERTIES | pass, rc 0 | no park reached in-process | | |
| base/directives/METHOD.testGroup TESTGUARDEDACCESS | refused at the run exceeded its deadline | Reply | TreeSend > OpExec | 2 |
| base/directives/METHOD.testGroup TESTGUARDEDACCESS | refused at the run exceeded its deadline | SysSleep | TreeSend | 628 |
| base/directives/METHOD.testGroup TESTDELEGATE | failure, rc 1 | no park reached in-process | | |
| base/keyword/CALL.testGroup TEST_4 | failure, rc 1 | SysSleep | TreeSend | 1 |
| base/keyword/GUARD.testGroup TEST_WHEN_NOVALUE | pass, rc 0 | GuardWhen | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_WHEN_NOT_BOOLEAN | pass, rc 0 | GuardWhen | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_ON_OFF_CONSECUTIVE | pass, rc 0 | GuardOn | TreeSend > OpExec | 2 |
| base/keyword/GUARD.testGroup TEST_ON_DEFAULT | pass, rc 0 | GuardOn | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_ON_DEFAULT | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_ON_DEFAULT | pass, rc 0 | SysSleep | TreeSend | 1 |
| base/keyword/GUARD.testGroup TEST_ON | pass, rc 0 | GuardOn | TreeSend > OpExec | 2 |
| base/keyword/GUARD.testGroup TEST_ON | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_ON | pass, rc 0 | SysSleep | TreeSend | 1 |
| base/keyword/GUARD.testGroup TEST_OFF | failure, rc 1 | GuardOn | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_OFF | failure, rc 1 | Reply | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_OFF | failure, rc 1 | SysSleep | TreeSend | 1 |
| base/keyword/GUARD.testGroup TEST_UNGUARDED | failure, rc 1 | Reply | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_UNGUARDED | failure, rc 1 | SysSleep | TreeSend | 1 |
| base/keyword/GUARD.testGroup TEST_ON_OFF | failure, rc 1 | GuardOn | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_ON_OFF | failure, rc 1 | Reply | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_ON_OFF | failure, rc 1 | SysSleep | TreeSend | 1 |
| base/keyword/GUARD.testGroup TEST_WHEN_SINGLE_NO_WAIT | pass, rc 0 | GuardWhen | TreeSend > OpExec | 4 |
| base/keyword/GUARD.testGroup TEST_WHEN_USE_LOCAL_NO_WAIT | refused at USE LOCAL in a ::METHOD body is not implemented (Phase 5) | no park reached in-process | | |
| base/keyword/GUARD.testGroup TEST_WHEN_SINGLE_UNINITIALIZED_NO_WAIT | pass, rc 0 | GuardWhen | TreeSend > OpExec | 2 |
| base/keyword/GUARD.testGroup TEST_WHEN_MULTIPLE_NO_WAIT | pass, rc 0 | GuardWhen | TreeSend > OpExec | 6 |
| base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE_TRIGGER | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE_TRIGGER | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE_TRIGGER | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | SysSleep | - | 26 |
| base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_WAIT_SIMPLE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | SysSleep | TreeSend | 1 |
| base/keyword/GUARD.testGroup TEST_WAIT_MULTIPLE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_WAIT_MULTIPLE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_WAIT_MULTIPLE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeSend > OpExec | 1 |
| base/keyword/GUARD.testGroup TEST_WAIT_MULTIPLE | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | SysSleep | TreeSend | 1 |
| base/keyword/RAISE.testGroup TEST_RAISE_INSERT_CRLF | failure, rc 1 | Reply | TreeSend > OpExec | 1 |
| base/keyword/RAISE.testGroup TEST_RAISE_INSERT_CRLF | failure, rc 1 | SysSleep | TreeSend | 10 |
| base/keyword/REPLY.testGroup TEST_REPLY_ROUTINE | pass, rc 0 | no park reached in-process | | |
| base/keyword/REPLY.testGroup TEST_REPLY_PROCEDURE | pass, rc 0 | no park reached in-process | | |
| base/keyword/REPLY.testGroup TEST_REPLY_CALL | pass, rc 0 | no park reached in-process | | |
| base/keyword/REPLY.testGroup TEST_REPLY_TWICE_REPLYASSERT | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_TWICE_REPLYASSERT | pass, rc 0 | Reply | OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_REPLYASSERT | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_EXIT_CODE_REPLYASSERT | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_STACK_REPLYASSERT | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_PLAIN | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_STRING | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_ARRAY | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_NIL | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_NOP | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY__CODE_RETURN | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY__CODE_EXIT | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_SAME_REPLYASSERT | pass, rc 0 | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_SAME_REPLYASSERT | pass, rc 0 | SysSleep | - | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_CONCURRENT | refused at the run exceeded its deadline | Reply | TreeSend > OpExec | 1 |
| base/keyword/REPLY.testGroup TEST_REPLY_CONCURRENT | refused at the run exceeded its deadline | SysSleep | TreeSend | 741 |
| base/keyword/TRACE.testGroup TEST_TRACE_GUARD | pass, rc 0 | GuardOn | TreeSend > OpExec | 1 |
| base/keyword/TRACE.testGroup TEST_TRACE_GUARD | pass, rc 0 | GuardWhen | TreeSend > OpExec | 1 |
| base/keyword/TRACE.testGroup TEST_TRACE_REPLY | failure, rc 1 | Reply | TreeSend > OpExec | 1 |
| base/keyword/TRACE.testGroup TEST_TRACE_REPLY | failure, rc 1 | SysSleep | TreeSend | 10 |
| base/keyword/TRACE_TraceObject.testGroup TEST_TRACEOBJECT_COLLECTOR | refused at DO is not implemented | Reply | TreeSend > TreeEval > OpExec | 1 |
| base/keyword/TRACE_TraceObject.testGroup TEST_TRACEOBJECT_COLLECTOR | refused at DO is not implemented | Reply | OpExec | 2 |
| base/keyword/TRACE_TraceObject.testGroup TEST_TRACEOBJECT_COLLECTOR | refused at DO is not implemented | SysSleep | TreeSend | 1 |
| base/keyword/TRACE_TraceObject.testGroup TEST_CALLER_STACK_FRAME_REPLY_START | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | GuardWhen | TreeSend > OpExec | 1 |
| base/keyword/TRACE_TraceObject.testGroup TEST_CALLER_STACK_FRAME_REPLY_START | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | Reply | TreeSend > OpExec | 1 |
| base/keyword/TRACE_TraceObject.testGroup TEST_CALLER_STACK_FRAME_REPLY_START | refused at a GUARD that has to wait for another activity to make its WHEN expression true is not implemented (Phase 6) | SysSleep | TreeSend | 2 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_NO_ARG | pass, rc 0 | no park reached in-process | | |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_TWO_ARGS | pass, rc 0 | no park reached in-process | | |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID | pass, rc 0 | no park reached in-process | | |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID_NEGATIVE | pass, rc 0 | no park reached in-process | | |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_INVALID_TOO_LARGE | pass, rc 0 | no park reached in-process | | |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_DURATION | pass, rc 0 | SysSleep | TreeSend | 9 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_CONCURRENT | refused at method "WAIT" of class "Message" is not implemented (Phase 9) | MessageWait | TreeSend | 1 |
| base/rexxutil/SysSleep.testGroup TEST_SLEEP_CONCURRENT | refused at method "WAIT" of class "Message" is not implemented (Phase 9) | SysSleep | TreeSend | 4 |
| base/special.variables/RESULT_RC_SIGL.testGroup TEST_RESULT_WITH_REPLY | pass, rc 0 | Reply | TreeSend > OpExec | 3 |
| doc/rexxref/chapter5/Section1.testGroup TEST_OBJECT_START | refused at method "START" of class "Message" is not implemented (Phase 9) | SysSleep | TreeSend | 1 |
| regressions/bug2003_guard_when.testGroup TEST_GUARD_WHEN_1 | error, rc 2 | no park reached in-process | | |

## S1 close

At `1f9be8ea5`, from `rust/`:

```
CARGO_TARGET_DIR=$S/tgt/pin cargo test --release -p rexx-exec --features pinning --test concurrency_tests -- measured:: --nocapture
```

Exit 0, eight tests passed. The table is `$S/tgt/pin/tmp/pinning-table.md`. Per park, tests and
arrivals, beside `## Measured` (the Task 3 figures, summed over its frame rows):

| park | S1 tests | S1 arrivals | Task 3 arrivals |
|---|---:|---:|---:|
| GuardOn | 6 | 8 | 8 |
| GuardWhen | 22 | 32 | 32 |
| Reply | 42 | 48 | 48 |
| MessageResult | 14 | 38 | 38 |
| MessageWait | 1 | 1 | 1 |
| SemaphoreWait | 8 | 8 | 8 |
| SysSleep | 30 | 1476 | 1478 |
| Timer | 11 | 11 | 11 |

Total arrivals 1622 (Task 3: 1624), with a frame other than TreeEval, TreeSend or OpExec 0. The
frames column of the S1 table lists `OpExec` for GuardOn, GuardWhen and Reply (`TreeEval > OpExec`
for one Reply arrival) and `-` for every other park; Task 3's rows had `TreeSend` in their frame
chains. Every park Task 3 counted is still reached. `SysSleep` differs by two arrivals; busy-wait arrival
counts depend on timing.

## S2 close

At the Task 10 tree, from `rust/`:

```
RAYON_NUM_THREADS=4 cargo test --release -p rexx-exec --features pinning --test concurrency_tests \
  -- measured::pinned_parks --nocapture --test-threads=1
```

Exit 0, both tests passed. The tables are `<target>/tmp/pinning-table.md` (the shipped scheduler)
and `<target>/tmp/pinning-table-every-opportunity.md` (a switch at every clause boundary). Arrivals
per park, beside the S1 close table above:

| park | S1 close | S2 normal | S2 every opportunity |
|---|---:|---:|---:|
| GuardOn | 8 | 27 | 59 |
| GuardWhen | 32 | 31 | 31 |
| Reply | 48 | 67 | 77 |
| MessageResult | 38 | 96 | 96 |
| MessageWait | 1 | 11 | 11 |
| SemaphoreWait | 8 | 6 | 6 |
| SysSleep | 1476 | 135 | 134 |
| Timer | 11 | 19 | 52 |

Every park S1 counted is reached in both modes. `SysSleep` arrivals are 135 against 1476; the S1
close noted that busy-wait arrival counts depend on timing. The waits by kind, normal:

| wait | park | frames | count |
|---|---|---|---|
| deferred slice | - | Program | 3 |
| pinned | MessageResult | Program | 76 |
| pinned | MessageResult | Program > TreeSend | 18 |
| pinned | MessageWait | Program | 8 |
| pinned | SysSleep | Program | 100 |
| pinned yield | - | Program | 1 |

Every opportunity:

| wait | park | frames | count |
|---|---|---|---|
| deferred slice | - | - | 31 |
| deferred slice | - | Notification | 8 |
| deferred slice | - | Program | 326 |
| deferred slice | - | Program > OpExec | 140 |
| deferred slice | - | Program > TreeEval > TreeSend | 7 |
| deferred slice | - | TraceWrapper | 697 |
| deferred slice | - | TraceWrapper > Unknown | 80 |
| deferred slice | - | TraceWrapper > Unknown > OpExec > Forward | 33 |
| deferred slice | - | TraceWrapper > Unknown > OpExec > Forward > Conversion | 7 |
| inverted yield | - | Notification | 6 |
| inverted yield | - | TraceWrapper | 697 |
| inverted yield | - | TraceWrapper > Unknown | 43 |
| inverted yield | - | TraceWrapper > Unknown > OpExec > Forward | 70 |
| inverted yield | - | TraceWrapper > Unknown > OpExec > Forward > Conversion | 7 |
| late wake | MessageResult | Program | 456 |
| late wake | MessageWait | Program | 234 |
| pinned | MessageResult | Program | 75 |
| pinned | MessageResult | Program > TreeSend | 18 |
| pinned | MessageWait | Program | 8 |
| pinned | SysSleep | Program | 99 |
| pinned yield | - | Notification | 6 |
| pinned yield | - | Program | 260 |
| pinned yield | - | Program > OpExec | 140 |
| pinned yield | - | Program > TreeEval > TreeSend | 7 |
| pinned yield | - | TraceWrapper | 697 |
| pinned yield | - | TraceWrapper > Unknown | 43 |
| pinned yield | - | TraceWrapper > Unknown > OpExec > Forward | 70 |
| pinned yield | - | TraceWrapper > Unknown > OpExec > Forward > Conversion | 7 |

No immovable `REPLY` is counted in either mode. Under every opportunity the late wakes are
`MessageResult` and `MessageWait`; the deferred slices and pinned yields have the same frame
chains, and the inverted yields are those under `TraceWrapper` and `Notification`.
