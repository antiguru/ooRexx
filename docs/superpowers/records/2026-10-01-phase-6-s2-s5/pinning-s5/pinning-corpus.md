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
| SysSleep | 64 | 158 |
| Timer | 10 | 18 |

| wait | park | frames | count |
|---|---|---|---|
| deferred slice | - | - | 2 |
| deferred slice | - | OpExec > Interpret | 6 |
| deferred slice | - | Program | 1 |
| pinned | GuardWhen | NativeApiCallback | 2 |
| pinned | MessageResult | TreeSend | 3 |
| pinned | MessageResult | Uninit | 1 |
| pinned | SysSleep | TreeEval | 6 |
| pinned | Timer | OpExec > Interpret | 1 |
| pinned yield | - | OpExec > Interpret | 6 |
| pinned yield | - | Program | 1 |

| program | wait | park | frames | count |
|---|---|---|---|---|
| lang/started_waited_in_uninit.rex | pinned | MessageResult | Uninit | 1 |
| lang/message_halt_wait.rex | deferred slice | - | - | 2 |
| lang/pinned_busy_wait_interpret.rex | deferred slice | - | OpExec > Interpret | 1 |
| lang/pinned_busy_wait_interpret.rex | pinned yield | - | OpExec > Interpret | 1 |
| lang/pinned_busy_wait_external.rex | deferred slice | - | Program | 1 |
| lang/pinned_busy_wait_external.rex | pinned yield | - | Program | 1 |
| lang/message_halt_pinned_target.rex | deferred slice | - | OpExec > Interpret | 3 |
| lang/message_halt_pinned_target.rex | pinned yield | - | OpExec > Interpret | 3 |
| lang/main_ends_in_pinned_yield.rex | deferred slice | - | OpExec > Interpret | 1 |
| lang/main_ends_in_pinned_yield.rex | pinned yield | - | OpExec > Interpret | 1 |
| lang/main_fails_in_pinned_yield.rex | deferred slice | - | OpExec > Interpret | 1 |
| lang/main_fails_in_pinned_yield.rex | pinned yield | - | OpExec > Interpret | 1 |
| lang/sleep_in_parse_template_and_when.rex | pinned | MessageResult | TreeSend | 3 |
| lang/sleep_in_parse_template_and_when.rex | pinned | SysSleep | TreeEval | 6 |
| lang/timer_native_arguments.rex | pinned | Timer | OpExec > Interpret | 1 |
| lang/native_guard_on_when_updated.rex | pinned | GuardWhen | NativeApiCallback | 2 |
