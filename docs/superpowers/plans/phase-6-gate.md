# Phase 6 gate record

## S0 and S1

Head `1f9be8ea5` before this record's commits. Base `1754a3b5a`. `S` is
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t11`.

### Performance

Tables, commands and binary hashes: `phase-6-perf.md` `## Task 11`.

- Instructions, running total against base: `fibfunc` +1.88% is over the +0.3% budget; every other
  program is inside it. `fibfunc` over budget accepted by Moritz, 2026-10-01 (P22).
- Wall clock, recorded only, no layout control (P19): `decrender` +5.68%, `sendloop` +10.14% and
  `textnum` +8.79% are over their ±4% bars; `dispatch` +8.94% is inside its 14.91% band.

### Pinning counter

`phase-6-pinning.md` `## S1 close`. Every park Task 3 counted is still reached; the frame chains no
longer contain `TreeSend`. Input to the S2 plan.

### Recursion depth and Error 11

Programs, each run with `$S/bin/NAME/rexx-run`, output `depth 9999 rc 11` on base and on head for
recursion by `CALL`, by function and by send. The three counter programs:

```
n = 0                                   /* call.rex */
signal on syntax name h
call sub
exit
sub:
n = n + 1
call sub
return
h: say 'depth' n 'rc' rc
```

`func.rex` and `send.rex` are the same shape with `.local~n` as the counter, through a `::ROUTINE`
`f` (`return f()`) and a `::METHOD` `m` (`return self~m`).

Cap lifted: a scratch copy of `1f9be8ea5` with `MAX_ACTIVATION_DEPTH` and `MAX_EVAL_DEPTH` at
100,000,000 (`rust/crates/rexx-exec/src/run/call.rs`, `src/eval.rs`), run under
`ulimit -v 4194304`, the program taking its depth as an argument and returning `'bottom'` through
`return f(n - 1)` (function), `return self~m(n - 1)` (send) and `call sub k - 1` (call):

| form | depth | wall s | max RSS KB | result |
|---|---:|---:|---:|---|
| call | 1,000,000 | 1.00 | 1,211,076 | bottom |
| function | 1,000,000 | 0.72 | 1,001,456 | bottom |
| send | 1,000,000 | 0.79 | 1,032,424 | bottom |

The counter programs with an `exit` at depth 4,000,000: `call` reaches it (rc 0, 2.38 s, 3,398,964 KB);
function and send abort with signal 6 (rc 134) after 17-18 s at about 3.3 GB, under the
address-space cap. An `exit` at depth 1,000,000 from the function and send counter programs did not
finish in 300 s; the send counter program with `exit` at depth 10,000, 20,000 and 40,000 takes
0.10 s, 0.22 s and 0.69 s. The `call` counter program takes 0.61 s at 1,000,000. The growth of the
`exit` cases was not investigated.

### Remaining loud refusals that name Phase 6

Derived by the commands below, run from the repository root at `1f9be8ea5`:

```
/bin/grep -a -rn 'Phase 6' rust/crates --include=*.rs | /bin/grep -av '/tests\.rs\|/tests/' | /bin/grep -av ':[0-9]*:\s*//'
/bin/grep -a -rPzoc 'Phase\s+6' rust/crates --include=*.rs        # wrapped occurrences: same files as the first command
/bin/grep -a -c 'Phase 6' rust/corpus/refusal-sites.tsv             # 0: that file records no owner
```

The first command's non-test hits, grouped by where the refusal or its owner is:

| where | refusals |
|---|---|
| `rexx-exec/src/lib.rs:679` | a `GUARD` that has to wait for another activity to make its `WHEN` expression true |
| `rexx-exec/src/lib.rs:688` | `Message~result` on a message whose send has not been made |
| `rexx-exec/src/lib.rs:696` | a `REPLY` inside a `DO`, `SELECT` or `IF` |
| `rexx-exec/src/internal_routines.rs` | `SysCloseEventSem`, `SysCloseMutexSem`, `SysCreateEventSem`, `SysCreateMutexSem`, `SysOpenEventSem`, `SysOpenMutexSem`, `SysPostEventSem`, `SysReleaseMutexSem`, `SysRequestMutexSem`, `SysResetEventSem`, `SysWaitEventSem` |
| `rexx-exec/src/dispatch/native.rs:116` | `alarm_startTimer`, `alarm_stopTimer`, `ticker_createTimer`, `ticker_waitTimer`, `ticker_stopTimer` |
| `rexx-api/src/layout.rs:384` | `MethodContextInterface.SetGuardOnWhenUpdated`, `MethodContextInterface.SetGuardOffWhenUpdated` |

The remaining hits are tests (`rexx-exec/tests/`, `*/tests.rs`, `run/tests/`), comments
(`run.rs:2273`, `handle.rs:50`) and the gate's own phase lists (`closed_phases.rs`,
`internal_routines.rs`, `native/tests.rs`). The Phase 6 corpus witnesses stay listed in
`rust/corpus/phase-8.txt` (P17).

### Inputs to S2

- The driver's Park path is refused in S1 and lands with S2's first real parker (P21).
- `REPLY` ordering: the oracle runs a `REPLY` continuation concurrently with its caller, so its
  output can come before the caller's later output; this crate runs it after the caller.
  Pre-existing.

### Gates

Run at `6a621dbbd` by `.superpowers/sdd/2026-09-29-phase-6-s0-s1/p6-gates/gates.sh`; its result lines, verbatim:

```
6a621dbbd76297395444c504aed11f3e8bab31b8
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
G4 release test exit 0
G4 Compiling lines: 0
G5 debug build (test --no-run) exit 0
G6 debug test exit 0
G6 Compiling lines: 0
G7 clippy --features pinning exit 0
G8 pinning self-tests exit 0
6a621dbbd76297395444c504aed11f3e8bab31b8
finished 2026-10-01T11:22:06+02:00
```

G4 release: 2803 passed, 0 failed. G6 debug: 2805 passed, 0 failed (sums of `test result` lines).

## S2

### UNINIT ordering (spec section 6, the *(verify)* item)

Probe sources: `docs/superpowers/records/2026-10-01-phase-6-s2-s5/uninit-ordering-probes.md`.
Oracle, 30 runs each (t3 and t7 10 each), from a fresh directory; stdout `|`-joined, with its
count.

- `u1`, an object main holds, and a started activity still sleeping when main ends:
  30 `main end|activity end|uninit live`. Termination waits for the activity, then runs the
  `UNINIT`.
- `u2`, an object dropped in a started activity that main waits on: 30
  `activity end|main end|uninit dropped`. Without a collection it waits for termination.
- `u3`, the same with `call gc 'force'` in the activity: 30 `activity end|main end|uninit
  dropped 1`: the forced collection does not run it; termination does, on thread 1.
- `u4`, the same with a loop allocating 200000 arrays, then a `call`: 30 `activity loop
  done|uninit dropped|activity end|main end`. A collection readies it and the next activation
  return runs it, on the started activity.
- `u5`, `REPLY` inside an `UNINIT` run at termination: `main end|uninit before reply`, then
  `uninit after reply` or not: the continuation races process exit, and how often it wins
  depends on machine load.
- `t3`, an `UNINIT` run at termination starts an activity that sleeps 0.2 s, then prints:
  10 `main end|uninit starts|uninit after start`, rc 7 (its `exit 7`). The activity prints
  nothing.
- `t7`, an `UNINIT` run at termination starts an activity that never ends: 10 `main end|uninit
  starts a poller`, rc 0, at once.

Rule (`concurrency/Activity.cpp:249`, `:324`, `runtime/InterpreterInstance.cpp:562-581`):
readied `UNINIT`s run when an activity's dispatch ends, main's included, and termination waits for
every activity before its collection and sweep; it does not wait for the activities the sweep's
`UNINIT`s start. This crate runs readied `UNINIT`s when an activity ends and when main ends, waits
for every activity, then sweeps, and does not run what the sweep started (ruling P39). The
oracle's further run at an activation return (`RexxActivation.cpp:705`) is collection timing,
which is not a specified observable; `u3` and `u4` answer differently here for that reason.
Witnesses: `corpus/lang/uninit_after_every_activity.rex`; the ending activity and `t7`'s shape
are crate-side (`scheduler/tests.rs`).


### Criterion 1, S2 rows in both modes

Command, from `rust/`:

```
REXX_CORPUS_GATE=1 REXX_CRITERION_ONE_TABLE=<file> cargo test --release -p rexx-exec \
  --test concurrency_tests -- group_runs::the_s2 --test-threads=1
```

Each derived-list row naming `REPLY`, `~start`, `Message~reply`, `SysSleep`, `.context~thread` or
a TraceObject field runs one test per run on both sides, with the shipped scheduler and under
`EveryOpportunity`. Task 1's starting table covers `base/class/Message` only; its refusals were
the Message methods of Tasks 2 to 7, and the Message rows pass here apart from the two
`makeArray` rows below. Passing in both modes, by group:

- `base/bif/STREAM.testGroup`: TEST_QUERYDIR_EXISTS
- `base/bif/TIME.testGroup`: TEST_2
- `base/class/DateTime.testGroup`: TEST_ELAPSED1
- `base/class/Message.testGroup`: TEST_START, TEST_REPLY, TEST_NOTIFY, TEST_SUPER_OVERRIDE, TEST_STARTWITH_NO_ARRAY, TEST_STARTWITH_TOO_MANY, TEST_REPLYWITH_NO_ARRAY, TEST_REPLYWITH_TOO_MANY, TEST_START_OVERRIDE_CONTEXT, TEST_STARTWITH_OVERRIDE_CONTEXT, TEST_REPLY_OVERRIDE_CONTEXT, TEST_REPLYWITH_OVERRIDE_CONTEXT, TEST_START_OVERRIDE_NOT_FOUND, TEST_STARTWITH_OVERRIDE_NOT_FOUND, TEST_REPLY_OVERRIDE_NOT_FOUND, TEST_REPLYWITH_OVERRIDE_NOT_FOUND, TEST_START_OVERRIDE_NO_METHOD, TEST_STARTWITH_OVERRIDE_NO_METHOD, TEST_REPLY_OVERRIDE_NO_METHOD, TEST_REPLYWITH_OVERRIDE_NO_METHOD, TEST_START_OVERRIDE_NOT_NON_SCOPE, TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE, TEST_REPLY_OVERRIDE_NOT_NON_SCOPE, TEST_REPLYWITH_OVERRIDE_NOT_NON_SCOPE, TEST_HALT_START, TEST_START_OVERRIDE_FROM_NONSELF, TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS, TEST_START_OVERRIDE_AMONG_MIXINCLASSES, TEST_STARTWITH_OVERRIDE_FROM_NONSELF, TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS, TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES
- `base/class/Object.testGroup`: TESTSTART01, TESTSTARTWITH01, TEST_START_NO_NAME, TEST_START_NO_NAME2, TEST_START_NOT_STRING, TEST_START_NO_METHOD, TEST_START_OVERRIDE, TEST_START_OVERRIDE_CONTEXT, TEST_START_OVERRIDE_EMPTY_ARRAY, TEST_START_OVERRIDE_MISSING_NAME, TEST_START_OVERRIDE_MISSING_SCOPE, TEST_START_OVERRIDE_EXTRA_STUFF, TEST_START_OVERRIDE_NON_STRING_NAME, TEST_START_OVERRIDE_NON_CLASS_SCOPE, TEST_START_OVERRIDE_NON_CLASS_SCOPE2, TEST_START_OVERRIDE_NOT_FOUND, TEST_START_OVERRIDE_NOT_NON_SCOPE, TEST_START_OVERRIDE_NO_METHOD, TEST_STARTWITH_NO_NAME, TEST_STARTWITH_NO_NAME2, TEST_STARTWITH_NOT_STRING, TEST_STARTWITH_NO_METHOD, TEST_STARTWITH_OVERRIDE, TEST_STARTWITH_OVERRIDE_CONTEXT, TEST_STARTWITH_OVERRIDE_EMPTY_ARRAY, TEST_STARTWITH_OVERRIDE_MISSING_NAME, TEST_STARTWITH_OVERRIDE_MISSING_SCOPE, TEST_STARTWITH_OVERRIDE_EXTRA_STUFF, TEST_STARTWITH_OVERRIDE_NON_STRING_NAME, TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE, TEST_STARTWITH_OVERRIDE_NON_CLASS_SCOPE2, TEST_STARTWITH_OVERRIDE_NOT_FOUND, TEST_STARTWITH_OVERRIDE_NOT_NON_SCOPE, TEST_STARTWITH_OVERRIDE_NO_METHOD, TEST_START_OVERRIDE_FROM_NONSELF, TEST_START_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS, TEST_START_OVERRIDE_AMONG_MIXINCLASSES, TEST_STARTWITH_OVERRIDE_FROM_NONSELF, TEST_STARTWITH_OVERRIDE_FROM_NONSELF_METHOD_NOT_IN_SUPERCLASS, TEST_STARTWITH_OVERRIDE_AMONG_MIXINCLASSES
- `base/class/RexxContext.testGroup`: TEST_INTERPRETER_THREAD_INVOCATION
- `base/class/Ticker.testGroup`: TEST_TICKER_TWO_ARGS_STRING_TRIGGER, TEST_TICKER_TWO_ARGS_TIMESPAN_TRIGGER, TEST_TICKER_THREE_ARGS_STRING_TRIGGER, TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER, TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER_MULTIPLE, TEST_TICKER_THREE_ARGS_STRING_TRIGGER_MESSAGE, TEST_CANCEL_TWICE
- `base/directives/ATTRIBUTE.testGroup`: TEST001
- `base/directives/CONSTANT.testGroup`: TEST_CONSTANT_METHOD_PROPERTIES
- `base/keyword/GUARD.testGroup`: TEST_OFF, TEST_UNGUARDED, TEST_ON_OFF
- `base/keyword/REPLY.testGroup`: TEST_REPLY_ROUTINE, TEST_REPLY_PROCEDURE, TEST_REPLY_CALL, TEST_REPLY_PLAIN, TEST_REPLY_STRING, TEST_REPLY_ARRAY, TEST_REPLY_NIL, TEST_REPLY_NOP, TEST_REPLY__CODE_RETURN, TEST_REPLY__CODE_EXIT, TEST_REPLY_CONCURRENT
- `base/keyword/TRACE.testGroup`: TEST_TRACE_REPLY
- `base/rexxutil/SysSleep.testGroup`: TEST_SLEEP_NO_ARG, TEST_SLEEP_TWO_ARGS, TEST_SLEEP_INVALID, TEST_SLEEP_INVALID_NEGATIVE, TEST_SLEEP_INVALID_TOO_LARGE, TEST_SLEEP_DURATION, TEST_SLEEP_CONCURRENT
- `base/special.variables/RESULT_RC_SIGL.testGroup`: TEST_RESULT_WITH_REPLY
- `doc/rexxref/chapter5/Section1.testGroup`: TEST_OBJECT_START

Not passing:

| group | test | normal | every opportunity | owner |
|---|---|---|---|---|
| base/bif/TIME.testGroup | TEST_3 | oracle did not finish | same | oracle run exceeds the 10 s runner deadline |
| base/bif/TIME.testGroup | TEST_4 | rc 1, oracle 8 assertions, ours 5 | same | TIME('E')/TIME('R') value after SysSleep; not traced, same in both modes |
| base/bif/TIME.testGroup | TEST_5 | oracle did not finish | same | oracle run exceeds the 10 s runner deadline |
| base/bif/TIME.testGroup | TEST_8 | oracle did not finish | same | oracle run exceeds the 10 s runner deadline |
| base/bif/TIME.testGroup | TEST_9 | oracle did not finish | same | oracle run exceeds the 10 s runner deadline |
| base/bif/TIME.testGroup | TEST_10 | rc 1, oracle 8 assertions, ours 5 | same | TIME('E')/TIME('R') value after SysSleep; not traced, same in both modes |
| base/bif/TIME.testGroup | TEST_11 | oracle did not finish | same | oracle run exceeds the 10 s runner deadline |
| base/class/Alarm.testGroup | TEST_BASE_ALARM | oracle did not finish | same | S3 (GUARD WHEN, semaphores, Alarm) |
| base/class/EventSemaphore.testGroup | TEST_WAIT_CONCURRENT | refused at GUARD WHEN | same | S3 (GUARD WHEN, semaphores, Alarm) |
| base/class/Message.testGroup | TEST_STARTWITH_NOT_ARRAY | refused: Object~MAKEARRAY | same | Phase 9 (Object~makeArray) |
| base/class/Message.testGroup | TEST_REPLYWITH_NOT_ARRAY | refused: Object~MAKEARRAY | same | Phase 9 (Object~makeArray) |
| base/class/Method.testGroup | TESTDIRECTIVES | refused: DO is not implemented | same | loop DO, not a concurrency feature |
| base/class/MutexSemaphore.testGroup | TEST_EXCLUSION | refused at GUARD WHEN | same | S3 (GUARD WHEN, semaphores, Alarm) |
| base/directives/ATTRIBUTE.testGroup | TESTDELEGATE | rc 1, oracle 122 assertions, ours 95 | same | Method delegate attributes; the row matched on `isGuarded` |
| base/directives/METHOD.testGroup | TESTGUARDEDACCESS | rc 1, oracle 1 assertions, ours 0 | same | unguarded attribute read beside a REPLY activity (`BAD` for 21); S3 by feature, not traced |
| base/directives/METHOD.testGroup | TESTDELEGATE | rc 1, oracle 116 assertions, ours 66 | same | Method delegate attributes; the row matched on `isGuarded` |
| base/keyword/CALL.testGroup | TEST_4 | rc 1, oracle 18 assertions, ours 7 | same | TIME('E')/TIME('R') value after SysSleep; not traced, same in both modes |
| base/keyword/GUARD.testGroup | TEST_ON_DEFAULT | rc 1, oracle 1 assertions, ours 0 | same | S3 (GUARD WHEN, semaphores, Alarm) |
| base/keyword/GUARD.testGroup | TEST_ON | rc 1, oracle 1 assertions, ours 0 | same | S3 (GUARD WHEN, semaphores, Alarm) |
| base/keyword/GUARD.testGroup | TEST_WAIT_SIMPLE_TRIGGER | refused at GUARD WHEN | same | S3 (GUARD WHEN, semaphores, Alarm) |
| base/keyword/GUARD.testGroup | TEST_WAIT_SIMPLE | rc 1, oracle 2 assertions, ours 0 | same | S3 (GUARD WHEN, semaphores, Alarm) |
| base/keyword/GUARD.testGroup | TEST_WAIT_MULTIPLE | refused at GUARD WHEN | same | S3 (GUARD WHEN, semaphores, Alarm) |
| base/keyword/RAISE.testGroup | TEST_RAISE_INSERT_CRLF | rc 1, oracle 1 assertions, ours 0 | same | message text conversion (`?` for non-ASCII), not scheduling |
| base/keyword/REPLY.testGroup | TEST_REPLY_TWICE_REPLYASSERT | pass | rc 0, oracle 0 assertions, ours 1 | S2, mode difference below |
| base/keyword/REPLY.testGroup | TEST_REPLY_RETURN_CODE_REPLYASSERT | pass | rc 0, oracle 0 assertions, ours 1 | S2, mode difference below |
| base/keyword/REPLY.testGroup | TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT | pass | rc 0, oracle 0 assertions, ours 1 | S2, mode difference below |
| base/keyword/REPLY.testGroup | TEST_REPLY_EXIT_CODE_REPLYASSERT | pass | rc 0, oracle 0 assertions, ours 1 | S2, mode difference below |
| base/keyword/REPLY.testGroup | TEST_REPLY_STACK_REPLYASSERT | pass | rc 0, oracle 1 assertions, ours 2 | S2, mode difference below |
| base/keyword/REPLY.testGroup | TEST_REPLY_SAME_REPLYASSERT | pass | rc 0, oracle 6 assertions, ours 5 | S2, mode difference below |
| base/keyword/TRACE_TraceObject.testGroup | TEST_TRACEOBJECT_COLLECTOR | refused: DO is not implemented | same | loop DO, not a concurrency feature |
| base/keyword/TRACE_TraceObject.testGroup | TEST_CALLER_STACK_FRAME_REPLY_START | refused at GUARD WHEN | same | S3 (GUARD WHEN, semaphores, Alarm) |
| regressions/bug2003_guard_when.testGroup | TEST_GUARD_WHEN_1 | fails on both | same | S3 (GUARD WHEN, semaphores, Alarm) |

The rows marked `oracle did not finish` are the oracle's run exceeding the runner's 10 s deadline
(`TIME` TEST_3 takes 13.5 s on the oracle). In process, in both pinning tables, `TIME` TEST_3,
TEST_8 and TEST_9 pass, TEST_5 and TEST_11 fail as TEST_4 and TEST_10 do, and `Alarm`
TEST_BASE_ALARM is refused at the GUARD WHEN, each the same in both modes.

#### Mode differences

Six `REPLY` rows, each asserting after the `REPLY` in the continuation, which races the end of the
program: TEST_REPLY_TWICE_REPLYASSERT, TEST_REPLY_RETURN_CODE_REPLYASSERT,
TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT, TEST_REPLY_EXIT_CODE_REPLYASSERT,
TEST_REPLY_STACK_REPLYASSERT and TEST_REPLY_SAME_REPLYASSERT. `rexx-run` (with `REXX_SWITCH_MODE=every`
for the second), 20 runs each, assertions counted:

| test | ours normal | ours every | oracle |
|---|---|---|---|
| TWICE | 0 x20 | 1 x20 | 0 x16, 1 x4 |
| RETURN_CODE | 0 x20 | 1 x20 | 0 x18, 1 x2 |
| RETURN_CODE_SAME | 0 x20 | 1 x20 | 0 x19, 1 x1 |
| EXIT_CODE | 0 x20 | 1 x20 | 0 x18, 1 x2 |
| STACK | 1 x20 | 2 x20 | 1 x20 |
| SAME | 1 x20 | 5 x20 | 1 x18, 6 x2 |

The shipped scheduler ends the program before the continuation asserts; the oracle does so in most
runs and not in all. Under `EveryOpportunity` the continuation always asserts. For STACK the
oracle's 20 runs never show the continuation's assertion that every opportunity always shows, and
SAME counts 5 where the oracle's runs count 1 or 6 (a race of the `u5` kind). The test
`the_s2_rows_of_the_derived_list_in_both_modes` allows a difference for these six rows only.

Over the whole derived list, the two pinning tables give the same per-test outcome except the
`base/class/MethodArgs` TEST_REQUEST_STRING_* rows (Alarm and Ticker rows, S3): normal refuses each at
the GUARD WHEN; under every opportunity each is refused at `DO is not implemented` except
TEST_REQUEST_STRING_MESSAGE, which passes. Both are loud refusals; which one a program reaches first
depends on the schedule.
