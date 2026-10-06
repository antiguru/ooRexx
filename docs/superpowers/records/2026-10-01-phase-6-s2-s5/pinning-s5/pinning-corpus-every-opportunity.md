programs 787, did not finish []

| park | programs | arrivals |
|---|---|---|
| GuardOn | 9 | 18 |
| GuardWhen | 15 | 26 |
| MessageResult | 52 | 116 |
| MessageWait | 43 | 70 |
| Reply | 33 | 46 |
| SemaphoreWait | 5 | 10 |
| SysSemWait | 4 | 13 |
| SysSleep | 64 | 151 |
| Timer | 10 | 18 |

| wait | park | frames | count |
|---|---|---|---|
| deferred slice | - | - | 71 |
| deferred slice | - | Notification | 8 |
| deferred slice | - | OpExec > Forward | 1 |
| deferred slice | - | OpExec > Interpret | 29 |
| deferred slice | - | OpExec > TraceWrapper | 1 |
| deferred slice | - | OpExec > TreeSend | 1 |
| deferred slice | - | Program | 2 |
| deferred slice | - | TraceWrapper | 3 |
| deferred slice | - | Uninit | 1 |
| pinned | GuardWhen | NativeApiCallback | 2 |
| pinned | MessageResult | TreeSend | 3 |
| pinned | MessageResult | Uninit | 1 |
| pinned | SysSleep | TreeEval | 6 |
| pinned | Timer | OpExec > Interpret | 1 |
| pinned yield | - | Notification | 8 |
| pinned yield | - | OpExec > Interpret | 28 |
| pinned yield | - | OpExec > Interpret > TrapHandler | 1 |
| pinned yield | - | OpExec > TraceWrapper | 1 |
| pinned yield | - | OpExec > TreeSend | 1 |
| pinned yield | - | Program | 2 |
| pinned yield | - | TraceWrapper | 3 |

| program | wait | park | frames | count |
|---|---|---|---|---|
| lang/forward_after_reply.rex | deferred slice | - | OpExec > Forward | 1 |
| lang/started_waited_in_uninit.rex | pinned | MessageResult | Uninit | 1 |
| lang/started_waited_in_uninit.rex | deferred slice | - | Uninit | 1 |
| lang/message_halt_wait.rex | deferred slice | - | - | 8 |
| lang/message_halt_untrapped.rex | deferred slice | - | - | 3 |
| lang/message_halt_returning.rex | deferred slice | - | - | 16 |
| lang/message_halt_loop_step.rex | deferred slice | - | - | 8 |
| lang/pinned_busy_wait_interpret.rex | deferred slice | - | OpExec > Interpret | 2 |
| lang/pinned_busy_wait_interpret.rex | pinned yield | - | OpExec > Interpret | 2 |
| lang/pinned_busy_wait_external.rex | deferred slice | - | Program | 2 |
| lang/pinned_busy_wait_external.rex | pinned yield | - | Program | 2 |
| lang/message_halt_pinned_target.rex | deferred slice | - | - | 3 |
| lang/message_halt_pinned_target.rex | deferred slice | - | OpExec > Interpret | 24 |
| lang/message_halt_pinned_target.rex | pinned yield | - | OpExec > Interpret | 23 |
| lang/message_halt_pinned_target.rex | pinned yield | - | OpExec > Interpret > TrapHandler | 1 |
| lang/main_ends_in_pinned_yield.rex | deferred slice | - | - | 1 |
| lang/main_ends_in_pinned_yield.rex | deferred slice | - | OpExec > Interpret | 2 |
| lang/main_ends_in_pinned_yield.rex | pinned yield | - | OpExec > Interpret | 2 |
| lang/main_fails_in_pinned_yield.rex | deferred slice | - | - | 1 |
| lang/main_fails_in_pinned_yield.rex | deferred slice | - | OpExec > Interpret | 1 |
| lang/main_fails_in_pinned_yield.rex | pinned yield | - | OpExec > Interpret | 1 |
| lang/sleeper_wakes_busy_main.rex | deferred slice | - | - | 1 |
| lang/sleep_in_parse_template_and_when.rex | pinned | MessageResult | TreeSend | 3 |
| lang/sleep_in_parse_template_and_when.rex | pinned | SysSleep | TreeEval | 6 |
| lang/reply_then_wait.rex | deferred slice | - | - | 1 |
| lang/reply_inside_constructs.rex | deferred slice | - | - | 13 |
| lang/reply_continuation_error_trace.rex | deferred slice | - | - | 1 |
| lang/reply_continuation_error_trace.rex | deferred slice | - | TraceWrapper | 1 |
| lang/reply_continuation_error_trace.rex | pinned yield | - | TraceWrapper | 1 |
| lang/reply_continuation_error_stderr.rex | deferred slice | - | - | 1 |
| lang/reply_continuation_failed_send.rex | deferred slice | - | - | 1 |
| lang/reply_split_after_a_trap_at_the_reply.rex | deferred slice | - | - | 1 |
| lang/context_of_another_activity.rex | deferred slice | - | - | 1 |
| lang/context_moved_by_reply.rex | deferred slice | - | - | 1 |
| lang/message_notify.rex | deferred slice | - | Notification | 8 |
| lang/message_notify.rex | pinned yield | - | Notification | 8 |
| lang/ticker_cancelled_by_its_target.rex | deferred slice | - | - | 1 |
| lang/timer_cancel_in_whole_days.rex | deferred slice | - | - | 1 |
| lang/timer_native_arguments.rex | pinned | Timer | OpExec > Interpret | 1 |
| lang/trace_object_activity_fields.rex | deferred slice | - | - | 1 |
| lang/trace_object_activity_fields.rex | deferred slice | - | TraceWrapper | 2 |
| lang/trace_object_activity_fields.rex | pinned yield | - | TraceWrapper | 2 |
| lang/guard_serializes_started_sends.rex | deferred slice | - | - | 3 |
| lang/guard_on_waits_in_an_unguarded_method.rex | deferred slice | - | - | 2 |
| lang/guard_when_trace_object_waiting.rex | deferred slice | - | OpExec > TraceWrapper | 1 |
| lang/guard_when_trace_object_waiting.rex | pinned yield | - | OpExec > TraceWrapper | 1 |
| lang/native_guard_on_when_updated.rex | pinned | GuardWhen | NativeApiCallback | 2 |
| lang/guard_when_unwatches_at_its_end.rex | deferred slice | - | OpExec > TreeSend | 1 |
| lang/guard_when_unwatches_at_its_end.rex | pinned yield | - | OpExec > TreeSend | 1 |
| lang/ticker_fires_on_its_replied_activity.rex | deferred slice | - | - | 2 |
