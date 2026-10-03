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
  --test concurrency_tests -- group_runs:: --test-threads=1
```

Each derived-list row naming `REPLY`, `~start`, `Message~reply`, `SysSleep`, `.context~thread` or
a TraceObject field runs one test per run. The normal column compares the shipped scheduler with
the oracle, one oracle run. The every-opportunity column compares this crate's two runs with each
other: masked stdout, stderr and exit status. Task 1's starting table covers `base/class/Message`
only; its refusals were the Message methods of Tasks 2 to 7 and `Object~makeArray`, and the Message
rows pass here apart from the `makeArray` rows below. Passing, and the same in both modes, by group:

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

| group | test | normal against the oracle | every against normal | owner |
|---|---|---|---|---|
| base/bif/TIME.testGroup | TEST_3 | oracle did not finish | same | oracle run exceeds the 10 s runner deadline; passes in process (the pinning tables, and one driver run at rc 0) |
| base/bif/TIME.testGroup | TEST_4 | rc 1, oracle 8 assertions, ours 5 | same apart from elapsed values | elapsed-clock defect, pre-existing (fails at S1 close 1a81353e3); queued 2026-10-02-elapsed-clock-per-routine-and-reset |
| base/bif/TIME.testGroup | TEST_5 | oracle did not finish | same apart from elapsed values | oracle run exceeds the 10 s runner deadline; fails in process like TEST_4 (same elapsed-clock cause) |
| base/bif/TIME.testGroup | TEST_8 | oracle did not finish | same | oracle run exceeds the 10 s runner deadline; passes in process (the pinning tables, and one driver run at rc 0) |
| base/bif/TIME.testGroup | TEST_9 | oracle did not finish | same | oracle run exceeds the 10 s runner deadline; passes in process (the pinning tables, and one driver run at rc 0) |
| base/bif/TIME.testGroup | TEST_10 | rc 1, oracle 8 assertions, ours 5 | same apart from elapsed values | elapsed-clock defect, pre-existing (fails at S1 close 1a81353e3); queued 2026-10-02-elapsed-clock-per-routine-and-reset |
| base/bif/TIME.testGroup | TEST_11 | oracle did not finish | same apart from elapsed values | oracle run exceeds the 10 s runner deadline; fails in process like TEST_4 (same elapsed-clock cause) |
| base/class/Alarm.testGroup | TEST_BASE_ALARM | oracle did not finish | same | S3; passes in both modes at the S3 close |
| base/class/EventSemaphore.testGroup | TEST_WAIT_CONCURRENT | refused at GUARD WHEN | same | S3; passes in both modes at the S3 close |
| base/class/Message.testGroup | TEST_STARTWITH_NOT_ARRAY | refused: Object~MAKEARRAY | same | Phase 9 (Object~makeArray) |
| base/class/Message.testGroup | TEST_REPLYWITH_NOT_ARRAY | refused: Object~MAKEARRAY | same | Phase 9 (Object~makeArray) |
| base/class/Method.testGroup | TESTDIRECTIVES | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/class/MutexSemaphore.testGroup | TEST_EXCLUSION | refused at GUARD WHEN | same | S3; passes at the S3 close, every mode as P46 |
| base/directives/ATTRIBUTE.testGroup | TESTDELEGATE | rc 1, oracle 122 assertions, ours 95 | same | Method delegate attributes; the row matched on `isGuarded`, outside Phase 6 |
| base/directives/METHOD.testGroup | TESTGUARDEDACCESS | rc 1, oracle 1 assertions, ours 0 | same | Task 11: a guarded attribute read does not wait for the lock a REPLY continuation holds (`BAD` where the oracle gives `GOOD`, 10 of 10) |
| base/directives/METHOD.testGroup | TESTDELEGATE | rc 1, oracle 116 assertions, ours 66 | same | Method delegate attributes; the row matched on `isGuarded`, outside Phase 6 |
| base/keyword/CALL.testGroup | TEST_4 | rc 1, oracle 18 assertions, ours 7 | same | elapsed-clock defect, pre-existing (fails at S1 close 1a81353e3); queued 2026-10-02-elapsed-clock-per-routine-and-reset |
| base/keyword/GUARD.testGroup | TEST_ON_DEFAULT | rc 1, oracle 1 assertions, ours 0 | same | S3; passes in both modes at the S3 close |
| base/keyword/GUARD.testGroup | TEST_ON | rc 1, oracle 1 assertions, ours 0 | same | S3; passes in both modes at the S3 close |
| base/keyword/GUARD.testGroup | TEST_WAIT_SIMPLE_TRIGGER | refused at GUARD WHEN | same | S3; passes in both modes at the S3 close |
| base/keyword/GUARD.testGroup | TEST_WAIT_SIMPLE | rc 1, oracle 2 assertions, ours 0 | same | S3; passes in both modes at the S3 close |
| base/keyword/GUARD.testGroup | TEST_WAIT_MULTIPLE | refused at GUARD WHEN | same | S3; passes in both modes at the S3 close |
| base/keyword/RAISE.testGroup | TEST_RAISE_INSERT_CRLF | rc 1, oracle 1 assertions, ours 0 | same | message text conversion (`?` for non-ASCII); fails at S1 close too, outside Phase 6 |
| base/keyword/REPLY.testGroup | TEST_REPLY_TWICE_REPLYASSERT | pass | assertions 0 then 1 | S2, mode difference below (P41) |
| base/keyword/REPLY.testGroup | TEST_REPLY_RETURN_CODE_REPLYASSERT | pass | assertions 0 then 1 | S2, mode difference below (P41) |
| base/keyword/REPLY.testGroup | TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT | pass | assertions 0 then 1 | S2, mode difference below (P41) |
| base/keyword/REPLY.testGroup | TEST_REPLY_EXIT_CODE_REPLYASSERT | rc 0, oracle 1 assertions, ours 0 | assertions 0 then 1 | S2, mode difference below (P41) |
| base/keyword/REPLY.testGroup | TEST_REPLY_STACK_REPLYASSERT | pass | assertions 1 then 2 | S2, mode difference below (P41) |
| base/keyword/REPLY.testGroup | TEST_REPLY_SAME_REPLYASSERT | pass | assertions 1 then 5 | S2, mode difference below (P41) |
| base/keyword/TRACE_TraceObject.testGroup | TEST_TRACEOBJECT_COLLECTOR | refused: DO is not implemented | trace lines on stderr differ | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/keyword/TRACE_TraceObject.testGroup | TEST_CALLER_STACK_FRAME_REPLY_START | refused at GUARD WHEN | trace lines on stderr differ | refused at DO in both modes at the S3 close; queued 2026-10-02-do-with-over-refusal |
| regressions/bug2003_guard_when.testGroup | TEST_GUARD_WHEN_1 | fails on both | same | fails on both sides alike at the S3 close |

The rows marked `oracle did not finish` are the oracle's run exceeding the runner's 10 s deadline
(`TIME` TEST_3 takes 13.5 s on the oracle); this crate's two runs are still compared. `Alarm`
TEST_BASE_ALARM is refused at the GUARD WHEN in process in both modes.

The check against inverted-wait refusals matches the crate's message (`pinned below it can end`,
`lib.rs` `inverted_wait`). To show it live, its text was temporarily replaced by `a GUARD that has
to wait`, a refusal these rows do reach: the test failed naming those rows in both modes, and the
text was restored. With the real text no row is stuck in either mode, so no inverted-wait refusal
occurs in these rows.

#### Mode differences

Ruling P41: the differences in the `*_REPLYASSERT` rows below are a licensed scheduling
divergence of the P32/P36 class. Each asserts after the `REPLY`, in the continuation, which races
the end of the program. Ours is deterministic: the shipped scheduler ends the program first and
every opportunity runs the continuation first. The oracle varies. Assertions counted, ours 20 runs
per mode through `rexx-run` (`REXX_SWITCH_MODE=every` for the second), the oracle as many runs as
shown, each at rc 0:

| test | ours normal | ours every | oracle |
|---|---|---|---|
| TWICE | 0 x20 | 1 x20 | 0 x22, 1 x8 (30 runs) |
| RETURN_CODE | 0 x20 | 1 x20 | 0 x26, 1 x4 (30) |
| RETURN_CODE_SAME | 0 x20 | 1 x20 | 0 x24, 1 x6 (30) |
| EXIT_CODE | 0 x20 | 1 x20 | 0 x21, 1 x9 (30) |
| STACK | 1 x20 | 2 x20 | 1 x42, 2 x8 (50) |
| SAME | 1 x20 | 5 x20 | 1 x28, 6 x2 (30); 1 x81, 6 x19 (100 more) |

Every count of ours is one the oracle also gives, except SAME's 5, which the oracle did not give in
the runs above. The oracle does interleave a `REPLY` continuation with its sender clause by clause:
for a sender of three `say`s and a continuation of five `say`s, a SysSleep and a `say`, 50 oracle
runs gave 40 sequential, 9 alternating and 1 continuation first, while ours alternates only under
every opportunity. The test allows a difference between the modes in these rows only when both runs
end at rc 0 with the same stderr and the same stdout apart from the `Assertions:` line.

Further differences are allowed by row:

- `TIME` TEST_4, TEST_5, TEST_10 and TEST_11 print an elapsed time in their failure output, which
  differs between any two runs. Compared without the `[failure]`, `Expected:`, `Actual:` and
  `Message:` lines they agree.
- `TEST_TRACEOBJECT_COLLECTOR` and `TEST_CALLER_STACK_FRAME_REPLY_START` are refused at rc 120 in
  both modes with the same `rexx-exec: ` refusal line and the same stdout; the trace lines on stderr
  differ. In the first the lines of
  `reply` and the next clause swap places; in the second every opportunity prints one more line, a
  `>I> Method "M_S"` entry, before the refusal ends the run. Not covered by P41.

Over the whole derived list, the two pinning tables give the same per-test outcome except the
`base/class/MethodArgs` TEST_REQUEST_STRING_* rows (Alarm and Ticker rows; at the S3 close
TEST_REQUEST_STRING_MESSAGE passes and the others are refused at DO WITH ... OVER in both modes,
queued `2026-10-02-do-with-over-refusal`; at the S2 close unswitched they stopped at GUARD WHEN): under every opportunity the continuation sets the Alarm's
`timerStarted` first, the GUARD WHEN is satisfied, and each reaches the DO WITH ... OVER refusal
instead, except TEST_REQUEST_STRING_MESSAGE, which passes. Both are loud refusals. Probe `a2.rex`,
which stops before the DO, matches the oracle under every opportunity 10 of 10 (queued
`2026-10-02-do-with-over-refusal`).

## S3

### Criterion 1 rows after Task 11

Measured by `the_s2_rows_of_the_derived_list_in_both_modes` (`REXX_CRITERION_ONE_TABLE`) at
`457a292e8`; the S2 table above is the S2 close and stays as measured.

| group | test | normal against the oracle | every against normal |
|---|---|---|---|
| base/directives/METHOD.testGroup | TESTGUARDEDACCESS | pass | same |
| base/keyword/GUARD.testGroup | TEST_ON_DEFAULT | pass | same |
| base/keyword/GUARD.testGroup | TEST_ON | pass | same |
| base/keyword/GUARD.testGroup | TEST_WAIT_SIMPLE | refused at GUARD WHEN | same |
| base/keyword/TRACE_TraceObject.testGroup | TEST_CALLER_STACK_FRAME_REPLY_START | refused at GUARD WHEN | refused at DO (allowed, `457a292e8`) |

### Criterion 1 rows after Task 12

Measured by `the_s2_rows_of_the_derived_list_in_both_modes` (`REXX_CRITERION_ONE_TABLE`) at
`c80ad0eab`, the rows whose cells Task 12 changed. The `457a292e8` allowance is gone; the P42
trace allowance keeps `TEST_TRACEOBJECT_COLLECTOR` only.

| group | test | normal against the oracle | every against normal | owner |
|---|---|---|---|---|
| base/class/EventSemaphore.testGroup | TEST_WAIT_CONCURRENT | refused: EventSemaphore `WAIT` | refused: `WAIT` then `POST` (allowed, `c80ad0eab`) | Task 13 |
| base/class/MutexSemaphore.testGroup | TEST_EXCLUSION | refused: MutexSemaphore `ACQUIRE` | same | Task 13 |
| base/keyword/GUARD.testGroup | TEST_WAIT_SIMPLE_TRIGGER | pass | same | - |
| base/keyword/GUARD.testGroup | TEST_WAIT_SIMPLE | pass | same | - |
| base/keyword/GUARD.testGroup | TEST_WAIT_MULTIPLE | pass | same | - |
| base/keyword/TRACE_TraceObject.testGroup | TEST_CALLER_STACK_FRAME_REPLY_START | refused: DO is not implemented | same | queued 2026-10-02-do-with-over-refusal |

`EventSemaphore` TEST_WAIT_CONCURRENT: the worker's store wakes main's `GUARD OFF WHEN`, and which
unbuilt method is refused first, the worker's `WAIT` or main's `POST`, follows the schedule. The
allowance holds only while both refusals name an `EventSemaphore` method.

`Alarm` TEST_BASE_ALARM is still `oracle did not finish` / same. In process it now passes its
`GUARD WHEN` waits and is refused at `a compiled call op does not name a call of its own body`,
a defect present at `217f33a2c` in a single activity: `self~assertTrue(1, 'a' d)` with `d` a
`.DateTime` (an argument concatenating an object whose `STRING` is Rexx).

The `base/class/MethodArgs` TEST_REQUEST_STRING_* rows (pinning tables, both modes) are no longer
refused at GUARD WHEN: TEST_REQUEST_STRING_MESSAGE passes and the others are refused at the DO
WITH ... OVER refusal in both modes (queued 2026-10-02-do-with-over-refusal).

### Criterion 1 rows after Task 13

Measured by `the_s2_rows_of_the_derived_list_in_both_modes` (`REXX_CRITERION_ONE_TABLE`) at
`b5e6e01d1`, the rows whose cells Task 13 changed, with the P46 allowance applied. The `c80ad0eab`
allowance is gone.

| group | test | normal against the oracle | every against normal | owner |
|---|---|---|---|---|
| base/class/EventSemaphore.testGroup | TEST_WAIT_CONCURRENT | pass | same | - |
| base/class/MutexSemaphore.testGroup | TEST_EXCLUSION | pass | every mode forces the test's own race; the oracle hangs in the same interleaving (P46) | - |

`MutexSemaphore` TEST_EXCLUSION under every opportunity: main runs between the worker's `step = 6`
and its last `acquire`, takes the mutex with `acquire(1)` and, never ending, never releases it, so
the worker waits for ever and the program's end waits for the worker. The oracle hangs the same
way when the worker yields there (`call SysSleep 0.01` after `step = 6`: 3 of 3 killed, rc 137, under `timeout -k 5 20`; `docs/superpowers/records/2026-10-01-phase-6-s2-s5/exclusion-every-hang.rex`).

### S3 close

Measured at `7266ae03c` from `rust/`; the S3 rows test at the Task 14 tree:

```
REXX_CORPUS_GATE=1 REXX_CRITERION_ONE_TABLE=<file> cargo test --release -p rexx-exec \
  --test concurrency_tests -- --test-threads=1
REXX_CORPUS_GATE=1 REXX_CRITERION_ONE_S3_TABLE=<file> cargo test --release -p rexx-exec \
  --test concurrency_tests -- group_runs::the_s3_rows
```

The runner's oracle deadline is 30 s per test (`group_runner.rs`, `ORACLE_TEST_DEADLINE`), so the
rows the S2 table marks `oracle did not finish` are compared: `TIME` TEST_3, TEST_8 and TEST_9
and `Alarm` TEST_BASE_ALARM pass, and `TIME` TEST_5 and TEST_11 differ as TEST_4 does.
`the_s3_rows_of_the_derived_list_in_both_modes` runs each row naming a GUARD, a semaphore class,
`Sys*Sem`, Alarm or Ticker and no S2 feature, as the S2 test runs the rest. Every row of either
test not below passes and is the same in both modes.

S2 rows:

| group | test | normal against the oracle | every against normal | owner |
|---|---|---|---|---|
| base/bif/TIME.testGroup | TEST_4 | differ: rc 1: oracle 8, ours 5 | same apart from elapsed values | elapsed-clock defect, pre-existing; queued 2026-10-02-elapsed-clock-per-routine-and-reset |
| base/bif/TIME.testGroup | TEST_5 | differ: rc 1: oracle 12, ours 7 | same apart from elapsed values | elapsed-clock defect, pre-existing; queued 2026-10-02-elapsed-clock-per-routine-and-reset |
| base/bif/TIME.testGroup | TEST_10 | differ: rc 1: oracle 8, ours 5 | same apart from elapsed values | elapsed-clock defect, pre-existing; queued 2026-10-02-elapsed-clock-per-routine-and-reset |
| base/bif/TIME.testGroup | TEST_11 | differ: rc 1: oracle 12, ours 7 | same apart from elapsed values | elapsed-clock defect, pre-existing; queued 2026-10-02-elapsed-clock-per-routine-and-reset |
| base/class/Message.testGroup | TEST_STARTWITH_NOT_ARRAY | refused: method "MAKEARRAY" of class "Object" is not implemented (Phase 9) | same | Phase 9 (Object~makeArray) |
| base/class/Message.testGroup | TEST_REPLYWITH_NOT_ARRAY | refused: method "MAKEARRAY" of class "Object" is not implemented (Phase 9) | same | Phase 9 (Object~makeArray) |
| base/class/Method.testGroup | TESTDIRECTIVES | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/class/MutexSemaphore.testGroup | TEST_EXCLUSION | pass | every mode forces the test's own race; the oracle hangs in the same interleaving (P46) | -; P46 |
| base/directives/ATTRIBUTE.testGroup | TESTDELEGATE | differ: rc 1: oracle 122, ours 95 | same | Method delegate attributes; the row matched on `isGuarded`, outside Phase 6 |
| base/directives/METHOD.testGroup | TESTDELEGATE | differ: rc 1: oracle 116, ours 66 | same | Method delegate attributes; the row matched on `isGuarded`, outside Phase 6 |
| base/keyword/CALL.testGroup | TEST_4 | differ: rc 1: oracle 18, ours 7 | same | elapsed-clock defect, pre-existing; queued 2026-10-02-elapsed-clock-per-routine-and-reset |
| base/keyword/RAISE.testGroup | TEST_RAISE_INSERT_CRLF | differ: rc 1: oracle 1, ours 0 | same | message text conversion (`?` for non-ASCII), outside Phase 6 |
| base/keyword/REPLY.testGroup | TEST_REPLY_TWICE_REPLYASSERT | pass | assertions 0 then 1 | -; P41 |
| base/keyword/REPLY.testGroup | TEST_REPLY_RETURN_CODE_REPLYASSERT | pass | assertions 0 then 1 | -; P41 |
| base/keyword/REPLY.testGroup | TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT | pass | assertions 0 then 1 | -; P41 |
| base/keyword/REPLY.testGroup | TEST_REPLY_EXIT_CODE_REPLYASSERT | pass | assertions 0 then 1 | -; P41 |
| base/keyword/REPLY.testGroup | TEST_REPLY_STACK_REPLYASSERT | pass | assertions 1 then 2 | -; P41 |
| base/keyword/REPLY.testGroup | TEST_REPLY_SAME_REPLYASSERT | pass | assertions 1 then 6 | -; P41 |
| base/keyword/TRACE_TraceObject.testGroup | TEST_TRACEOBJECT_COLLECTOR | refused: DO is not implemented | trace lines on stderr differ | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/keyword/TRACE_TraceObject.testGroup | TEST_CALLER_STACK_FRAME_REPLY_START | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| regressions/bug2003_guard_when.testGroup | TEST_GUARD_WHEN_1 | fails on both | same | -; fails on the oracle the same way |

S3 rows:

| group | test | normal against the oracle | every against normal | owner |
|---|---|---|---|---|
| base/class/MethodArgs.testGroup | TEST_REQUEST_STRING_CLASS | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/class/MethodArgs.testGroup | TEST_REQUEST_STRING_OBJECT | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/class/MethodArgs.testGroup | TEST_REQUEST_STRING_STRING | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/class/MethodArgs.testGroup | TEST_REQUEST_STRING_METHOD | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/class/MethodArgs.testGroup | TEST_REQUEST_STRING_ROUTINE | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/class/MethodArgs.testGroup | TEST_REQUEST_STRING_PACKAGE | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/class/MethodArgs.testGroup | TEST_REQUEST_STRING_STREAM | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/class/MethodArgs.testGroup | TEST_REQUEST_STRING_MUTABLEBUFFER | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/class/MethodArgs.testGroup | TEST_REQUEST_STRING_FILE | refused: DO is not implemented | same | DO WITH / DO COUNTER refusal, outside Phase 6; queued 2026-10-02-do-with-over-refusal |
| base/keyword/GUARD.testGroup | TEST_WHEN_USE_LOCAL_NO_WAIT | refused: USE LOCAL in a ::METHOD body is not implemented (Phase 5) | same | `USE LOCAL` in a method body, outside Phase 6 |

`TEST_REPLY_SAME_REPLYASSERT` under every opportunity now counts 6 assertions, where the S2 close
counted 5; 6 is a count the oracle gives (the P41 table above), so every count of ours in those
rows is now one the oracle also gives.

`the_alarm_and_ticker_groups_pass_in_both_modes`: every test of `base/class/Alarm` and
`base/class/Ticker` passes against the oracle with the shipped scheduler and under every
opportunity (gate-only, `concurrency_tests.rs`). In process, `Alarm` TEST_BASE_ALARM passed its
`GUARD WHEN` waits from Task 12 on and then refused at `a compiled call op does not name a call of
its own body`; that was a nested driver's clauses emptying the caller's pushed call arguments
(fixed, `Interp::drive`; witness `corpus/lang/call_argument_concatenates_a_rexx_string_method.rex`).

Witnesses, stdout and exit status the same in every run (oracle under `timeout -k 5 20`, this
crate's `rexx-run` unswitched and with `REXX_SWITCH_MODE=every`, 30 runs each; one run each
under `collect_stress`, which runs `phase-8.txt`):

- `alarm_cancel_waits_for_the_timer.rex`: a cancel sent at once waits in `guard on when
  timerStarted` until the replied activity has started the timer, then cancels it.
- `alarm_message_target.rex`: an Alarm whose target is a Message sends it when it fires
  (`msgobj~triggered(self)`, `CoreClasses.orx:1565`), on the replied activity.
- `ticker_fires_on_its_replied_activity.rex`: a Ticker triggers on the activity its `REPLY` made
  until main cancels it.
- `call_argument_concatenates_a_rexx_string_method.rex`, single activity.

Review Focus 2, the uncancelled Ticker: `scheduler/tests.rs`
`an_uncancelled_ticker_keeps_the_programs_end_waiting` (killed by the run's deadline). The oracle
on the same program: `main done`, `ticked`, killed at 3 s, rc 137, 3 of 3. This crate's
`rexx-run` used 2 clock ticks (20 ms) of CPU in 3 s on it.

### Criterion 2, the framework's ticker

`api_group_tests.rs` `the_framework_ticker_runs_without_dash_u` (gate-only) runs `API/oo`
`CONVERSION` on this crate with and without `-U`: both exit 0, the ticker's two lines carry dots,
and every other line equals the `-U` run's. The differential keeps `-U`, since the dots depend on
wall time (`ooTest.frm:2449-2465`): on `base/class/Ticker`, one run each, this crate printed one
dot after `Searching for test containers` and the oracle two.
Unmodified, `testOORexx.rex -f METHOD.testGroup -V 1` from a scratch copy runs the ticker and the
tests and stops at rc 120 at `routine "RXFUNCQUERY" is not implemented (Phase 10)` only; the L2
rows (`phase-4-exclusions.txt`, the roadmap's row 8, `phase-8-gate.md`) record it, and the Rung
reads `L2 -> 10`.
