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
`base/class/MethodArgs` TEST_REQUEST_STRING_* rows (Alarm and Ticker rows, owned by S3 because
unswitched they stop at GUARD WHEN): under every opportunity the continuation sets the Alarm's
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

#### Wall-clock rows (ruling P48)

The rows whose outcome on this crate depends on a wall-clock boundary or a sleep's duration are
`concurrency_tests.rs` `WALL_CLOCK`, each with the group lines that make it so. Where one of them
fails its check, the S2 and S3 row tests run its two crate runs once more and the Alarm/Ticker test
runs it once more on both sides; only a second failure fails the test. A rerun prints `P48 rerun:`
on stderr and the row tests' table cell ends `(P48 rerun)`. Other rows stay strict.

The rows below depend on the clock on the oracle only: there a `SysSleep` must outlast another
activity's progress, which this crate's park runs at once. Their "normal against the oracle" cell
can vary between runs; it is recorded and not asserted, and none is in the Alarm or Ticker group.

- `base/keyword/GUARD` TEST_OFF (`GUARD.testGroup:139-143`), TEST_UNGUARDED (`:148-151`),
  TEST_ON_OFF (`:156-160`), TEST_WAIT_SIMPLE_TRIGGER (`:240-246`, `:253-256`)
- `base/class/RexxContext` TEST_INTERPRETER_THREAD_INVOCATION (`RexxContext.testGroup:250-253`)
- `base/keyword/RAISE` TEST_RAISE_INSERT_CRLF (`RAISE.testGroup:481-482`)
- `base/keyword/TRACE` TEST_TRACE_REPLY (`TRACE.testGroup:761-762`)
- `base/directives/METHOD` TESTGUARDEDACCESS (`METHOD.testGroup:342-348`)
- `base/keyword/TRACE_TraceObject` TEST_TRACEOBJECT_COLLECTOR (`TRACE_TraceObject.testGroup:227`),
  TEST_CALLER_STACK_FRAME_REPLY_START (`:713`, `:732`)

### Criterion 2, the framework's ticker

`api_group_tests.rs` `the_framework_ticker_runs_without_dash_u` (gate-only) runs `API/oo`
`CONVERSION` on this crate with and without `-U`: both exit 0, each ticker line carries dots,
and every other line equals the `-U` run's. The differential keeps `-U`, since the dots depend on
wall time (`ooTest.frm:2449-2465`): on `base/class/Ticker`, one run each, this crate printed one
dot after `Searching for test containers` and the oracle two.
Unmodified, `testOORexx.rex -f METHOD.testGroup -V 1` from a scratch copy runs the ticker and the
tests and stops at rc 120 at `routine "RXFUNCQUERY" is not implemented (Phase 10)` only; the L2
rows (`phase-4-exclusions.txt`, the roadmap's row 8, `phase-8-gate.md`) record it, and the Rung
reads `L2 -> 10`.

## S4

### Blocking operations (Task 18)

The command, run from the worktree root at `c66650b52` (Task 22 fix round 1), with the
directory prefix stripped from its output:

```
/bin/grep -a -rn 'std::process\|\.wait(\|std::fs\|fs::\|thread::sleep\|File::\|stdin()\|\.read(\|\.write(' rust/crates/rexx-exec/src \
  | sed 's|^rust/crates/rexx-exec/src/||'
```

Classification, by site (paths under `rust/crates/rexx-exec/src/`):

| Hits | Class | Reason |
|---|---|---|
| `command.rs:644` (`collect`'s child wait) and `:467` (its reads), reached through `Block::stream` from `Scheduler::exit_for_block` and through `Block::wait` at `command.rs:712` | off-baton | an `ADDRESS` command with nothing redirected (P64): the child starts on the baton, a pool thread posts what it writes to the inbox as it arrives and then its return code, and the clause settles `RC` once the activity wakes; no island value crosses (bytes and a return code). Where no pool thread can be reserved, `command.rs:712` waits on the baton, uninterruptible (DEVIATIONS 12) |
| `command.rs:20`, `:395`, `:436`, `:515`, `:541`, `:552`, `:609` | not an operation | types, imports and the child's builder; the spawn itself (fork and exec) runs on the baton |
| `command.rs:310` | stays on the baton | `cd`'s directory test, a local `stat` |
| `input.rs:80` (`read_stdin_chunk`) | keeps the baton | a read of the default input stream: a pool thread reads, and the reader keeps the baton and idles on the inbox (P69, DEVIATIONS 13); not a park; a halt ends it (P60). Where no pool thread can be reserved, the read runs on the baton, uninterruptible (DEVIATIONS 12) |
| `install.rs:651`, `:913`, `:1772`, `lib.rs:2171` | spec 2.1 wrapper (package loaders, `::REQUIRES` and external routine files) | reached from nested Rust frames, so pinned |
| `builtin/platform.rs:91`, `:99`, `:164`, `:169`; `builtin/rexxutil.rs:67`, `:78`, `:96`, `:99`, `:118`, `:121`, `:154`, `:611`, `:612`, `:633`, `:766`; `dispatch/files.rs` (every hit) | stays on the baton | single local filesystem calls (`stat`, `access`, `unlink`, `mkdir`, `rename`, `utimensat`, `readdir`) that wait on no other activity; a recorded divergence where a filesystem stalls (the oracle releases its kernel lock around a native routine) |
| `dispatch/stream.rs` (every hit) | spec 2.1 wrapper (the stream BIFs and methods) | pinned, counted by the pinning report |
| `builtin/numeric.rs:581` | not blocking | the process id, as `RANDOM`'s seed |
| `sync.rs:83`, `:127` | not blocking | the wake socket's nonblocking write (`Wake::notify`) and read |
| `run.rs:1437`, `:1574`, `run/interpret.rs:159`, `parse_template.rs:654` | not I/O | `Interp::read`, a variable read |
| `builtin/convert.rs:996`, `:1005`, `dispatch/library/surface.rs:661`, `input.rs:215`, `input.rs:359`, `builtin/rexxutil.rs:87` | not I/O | writes and reads of memory, `reading_stdin` (a state change) and a doc comment |
| `sync.rs:42`, `scheduler/pool.rs:188`, `timer.rs:454` | not reached from a resumable entry | the baton's and a pool thread's own condition-variable waits, and the timer thread's own wait on its wake source (`Wake::wait`) |
| `scheduler/pool.rs:43` and `dispatch/library.rs:1155` (both `#[cfg(test)]` code), `signal.rs:213`, `:238`, `:270`, `sync.rs:234`, `:251`, `:254`, `input.rs:593` (their `tests` modules), `tests.rs`, `builtin/tests.rs`, `plan/tests.rs`, `run/tests/indent.rs`, `dispatch/library/tests.rs`, `dispatch/native/tests.rs`, `ir/corpus_shape_tests.rs`, `scheduler/tests/pool.rs`, `scheduler/tests/lent.rs`, `scheduler/tests/callbacks.rs`, `bin/rexx-run.rs`, `bin/rexx-ir.rs` | not reached from a resumable entry | tests and binaries |

<details><summary>The command's output</summary>

```
signal.rs:213:        let output = std::process::Command::new(std::env::current_exe().expect("this binary"))
signal.rs:238:                std::thread::sleep(std::time::Duration::from_millis(200));
signal.rs:270:            std::thread::sleep(std::time::Duration::from_secs(1));
install.rs:651:        let Ok(text) = std::fs::read(&resolved) else {
install.rs:913:            match std::fs::metadata(require::normalize(candidate, cwd)) {
install.rs:1772:        let Ok(text) = std::fs::read(&resolved) else {
scheduler/pool.rs:43:        std::thread::sleep(std::time::Duration::from_millis(100));
scheduler/pool.rs:188:                    .wait(mail)
scheduler/tests/pool.rs:105:    std::thread::sleep(Duration::from_millis(200));
scheduler/tests/pool.rs:148:    mark.1 = std::fs::read_to_string(path).map_or(0, |read| read.lines().count());
scheduler/tests/pool.rs:435:    let path = std::env::temp_dir().join(format!("rexx-lent-panic-{}", std::process::id()));
scheduler/tests/pool.rs:436:    let _ = std::fs::remove_file(&path);
scheduler/tests/pool.rs:451:    let written = std::fs::read_to_string(&path).expect("the file");
scheduler/tests/pool.rs:452:    let _ = std::fs::remove_file(&path);
scheduler/tests/pool.rs:469:    let path = std::env::temp_dir().join(format!("rexx-native-panic-{}", std::process::id()));
scheduler/tests/pool.rs:470:    let _ = std::fs::remove_file(&path);
scheduler/tests/pool.rs:485:    let written = std::fs::read_to_string(&path).expect("the file");
scheduler/tests/pool.rs:486:    let _ = std::fs::remove_file(&path);
scheduler/tests/pool.rs:545:    let path = std::env::temp_dir().join(format!("rexx-pool-{}", std::process::id()));
scheduler/tests/pool.rs:546:    let _ = std::fs::remove_file(&path);
scheduler/tests/pool.rs:556:    let _ = std::fs::remove_file(&path);
scheduler/tests/lent.rs:53:    let path = std::env::temp_dir().join(format!("rexx-lent-{}-{test}", std::process::id()));
scheduler/tests/lent.rs:54:    let _ = std::fs::remove_file(&path);
scheduler/tests/lent.rs:65:    let _ = std::fs::remove_file(&path);
scheduler/tests/callbacks.rs:53:    std::thread::sleep(Duration::from_millis(MILLIS));
scheduler/tests/callbacks.rs:233:    let path = std::env::temp_dir().join(format!("rexx-interleaved-{}", std::process::id()));
scheduler/tests/callbacks.rs:234:    let _ = std::fs::remove_file(&path);
scheduler/tests/callbacks.rs:249:    let written = std::fs::read_to_string(&path).unwrap_or_default();
scheduler/tests/callbacks.rs:250:    let _ = std::fs::remove_file(&path);
scheduler/tests/callbacks.rs:309:    let path = std::env::temp_dir().join(format!("rexx-batched-{}", std::process::id()));
scheduler/tests/callbacks.rs:310:    let _ = std::fs::remove_file(&path);
scheduler/tests/callbacks.rs:335:    let _ = std::fs::remove_file(&path);
command.rs:20:use std::process::Stdio;
command.rs:310:    match std::fs::metadata(&path) {
command.rs:395:fn exit_code(status: std::process::ExitStatus) -> i32 {
command.rs:436:    running: std::process::Child,
command.rs:467:        match reader.read(&mut piece) {
command.rs:515:        running: std::process::Child,
command.rs:541:            let mut builder = std::process::Command::new(format!("{SHELL_DIRECTORY}/{shell}"));
command.rs:552:            let mut builder = std::process::Command::new(OsStr::from_bytes(program));
command.rs:609:    mut running: std::process::Child,
command.rs:644:    let rc = match running.wait() {
command.rs:712:                    Err(block) => block.wait(),
timer.rs:454:        registry.wake.wait(timeout);
lib.rs:2171:        let Ok(text) = std::fs::read(&resolved) else {
run.rs:1437:                    let (value, _novalue) = self.read(code, *id);
run.rs:1574:                    let (value, _novalue) = self.read(code, *id);
builtin/numeric.rs:581:    u64::from(nanos) << 32 ^ u64::from(std::process::id())
builtin/platform.rs:91:    let status = std::fs::read_to_string("/proc/self/status").ok()?;
builtin/platform.rs:99:    let passwd = std::fs::read_to_string("/etc/passwd").ok()?;
builtin/platform.rs:164:    if !std::fs::metadata(&candidate).is_ok_and(|meta| meta.is_dir()) {
builtin/platform.rs:169:    let resolved = std::fs::canonicalize(&candidate)
builtin/rexxutil.rs:67:    let found = !path.is_empty() && std::fs::metadata(&path).is_ok();
builtin/rexxutil.rs:78:    let found = !path.is_empty() && std::fs::metadata(&path).is_ok_and(|meta| meta.is_file());
builtin/rexxutil.rs:87:/// would give -- measured, and a port calling `std::fs::remove_file` and
builtin/rexxutil.rs:96:    if path.is_empty() || !rustix::fs::access(path.as_str(), rustix::fs::Access::WRITE_OK).is_ok() {
builtin/rexxutil.rs:99:    let code = match rustix::fs::unlink(path.as_str()) {
builtin/rexxutil.rs:118:    let code = match rustix::fs::unlink(path.as_str()) {
builtin/rexxutil.rs:121:            match rustix::fs::rmdir(path.as_str()) {
builtin/rexxutil.rs:154:    let code = match rustix::fs::mkdir(path.as_str(), rustix::fs::Mode::from_bits_truncate(mode)) {
builtin/rexxutil.rs:611:fn attributes(meta: &std::fs::Metadata) -> Vec<u8> {
builtin/rexxutil.rs:612:    use std::os::unix::fs::PermissionsExt as _;
builtin/rexxutil.rs:633:fn entry_line(options: TreeOptions, path: &str, meta: &std::fs::Metadata) -> Vec<u8> {
builtin/rexxutil.rs:766:    let Ok(entries) = std::fs::read_dir(directory) else {
builtin/tests.rs:204:    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
builtin/convert.rs:996:            Piece::Range(start, length).write(&mut out);
builtin/convert.rs:1005:        piece.write(&mut out);
run/interpret.rs:159:                let (value, _novalue) = self.read(code, *id);
run/tests/indent.rs:218:        for entry in std::fs::read_dir(&directory).expect("a readable corpus directory") {
run/tests/indent.rs:227:            let bytes = std::fs::read(&path).expect("a readable corpus program");
sync.rs:42:    condvar.wait(guard).unwrap_or_else(PoisonError::into_inner)
sync.rs:83:        let _ = (&self.writer).write(&[0]);
sync.rs:127:        let _ = (&self.reader).read(&mut [0u8; 64]);
sync.rs:234:                wake.wait(Some(timeout));
sync.rs:251:        wake.wait(Some(Duration::from_secs(5)));
sync.rs:254:        wake.wait(Some(Duration::from_millis(50)));
parse_template.rs:654:                let (value, novalue) = self.read(code, id);
tests.rs:26:        for entry in std::fs::read_dir(&directory).expect("a readable corpus directory") {
tests.rs:35:            let bytes = std::fs::read(&path).expect("a readable corpus program");
dispatch/library/tests.rs:191:    let text = std::fs::read(&path).expect("the corpus witness is readable");
dispatch/library/tests.rs:478:    let text = std::fs::read(&path).expect("the corpus witness is readable");
dispatch/library/surface.rs:661:            data.write(0, &bytes);
dispatch/files.rs:52:fn reachable(path: &str, mode: rustix::fs::Access) -> bool {
dispatch/files.rs:53:    rustix::fs::access(path, mode).is_ok()
dispatch/files.rs:67:    let there = std::fs::metadata(&path).is_ok();
dispatch/files.rs:79:    let yes = std::fs::metadata(&path).is_ok_and(|meta| meta.is_file());
dispatch/files.rs:91:    let yes = std::fs::metadata(&path).is_ok_and(|meta| meta.is_dir());
dispatch/files.rs:108:    let yes = std::fs::metadata(&path).is_ok() && path.contains("/.");
dispatch/files.rs:120:    let yes = reachable(&path, rustix::fs::Access::READ_OK);
dispatch/files.rs:132:    let yes = reachable(&path, rustix::fs::Access::WRITE_OK);
dispatch/files.rs:150:    let size = std::fs::metadata(&path).map_or(0, |meta| meta.len());
dispatch/files.rs:168:    let Ok(entries) = std::fs::read_dir(&path) else {
dispatch/files.rs:188:    let read = std::fs::metadata(path).and_then(|meta| {
dispatch/files.rs:284:        Ok(since) => rustix::fs::Timespec {
dispatch/files.rs:299:            rustix::fs::Timespec {
dispatch/files.rs:305:    let omit = rustix::fs::Timespec {
dispatch/files.rs:307:        tv_nsec: rustix::fs::UTIME_OMIT,
dispatch/files.rs:310:        Stamp::Modified => rustix::fs::Timestamps {
dispatch/files.rs:314:        Stamp::Accessed => rustix::fs::Timestamps {
dispatch/files.rs:319:    rustix::fs::utimensat(rustix::fs::CWD, path, &times, rustix::fs::AtFlags::empty()).is_ok()
dispatch/files.rs:362:    let made = std::fs::create_dir(&path).is_ok();
dispatch/files.rs:378:    if !reachable(&path, rustix::fs::Access::WRITE_OK) {
dispatch/files.rs:381:    let gone = std::fs::remove_file(&path).is_ok();
dispatch/files.rs:394:    let gone = std::fs::remove_dir(&path).is_ok();
dispatch/files.rs:412:    if from == to || std::fs::symlink_metadata(&to).is_ok() {
dispatch/files.rs:415:    let moved = std::fs::rename(&from, &to).is_ok();
dispatch/files.rs:428:    if let Ok(meta) = std::fs::metadata(&path) {
dispatch/files.rs:429:        use std::os::unix::fs::PermissionsExt as _;
dispatch/files.rs:438:        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(rewritten));
dispatch/files.rs:559:        if std::fs::metadata(&candidate).is_ok_and(|meta| meta.is_file()) {
dispatch/library.rs:1155:            let mut file = std::fs::OpenOptions::new()
dispatch/stream.rs:232:    let exists = std::fs::metadata(&path).is_ok_and(|meta| meta.is_file());
dispatch/stream.rs:251:    match std::fs::metadata(&path) {
dispatch/stream.rs:271:    let modified = std::fs::metadata(&path)
dispatch/stream.rs:332:/// opens on unix and `SysFile::open` closes it again and substitutes this
dispatch/stream.rs:575:    if parsed.read_only && !std::fs::metadata(&path).is_ok_and(|meta| meta.is_file()) {
dispatch/stream.rs:585:    let mut opening = std::fs::OpenOptions::new();
dispatch/stream.rs:587:        opening.read(true);
dispatch/stream.rs:589:        opening.read(true).write(true).create(true);
dispatch/stream.rs:611:    if metadata.as_ref().is_none_or(std::fs::Metadata::is_dir) {
dispatch/stream.rs:684:fn read_from(file: &mut std::fs::File, position: u64, len: usize) -> std::io::Result<Vec<u8>> {
dispatch/stream.rs:690:        match file.read(&mut out[filled..])? {
dispatch/stream.rs:701:/// data (`SysFile::gets`) -- and a final unterminated line is still a line.
dispatch/stream.rs:704:    file: &mut std::fs::File,
dispatch/stream.rs:734:/// The lines from `position` to the end, counted the way `SysFile::countLines`
dispatch/stream.rs:736:fn count_lines_from(file: &mut std::fs::File, position: u64) -> std::io::Result<u64> {
dispatch/stream.rs:758:/// (`SysFile::getStreamTypeInfo`). A regular file is persistent.
dispatch/stream.rs:759:fn is_transient(file: &std::fs::File) -> bool {
dispatch/stream.rs:760:    use std::os::unix::fs::FileTypeExt;
dispatch/stream.rs:766:fn size_of(file: &std::fs::File) -> u64 {
dispatch/stream.rs:778:    let mut opening = std::fs::OpenOptions::new();
dispatch/stream.rs:783:    opening.read(true).write(true).truncate(false);
dispatch/stream.rs:799:        Err(_) if for_write => match std::fs::OpenOptions::new().write(true).open(&path) {
dispatch/stream.rs:809:        Err(_) => match std::fs::OpenOptions::new().read(true).open(&path) {
dispatch/stream.rs:1314:fn write_at(file: &mut std::fs::File, position: u64, bytes: &[u8]) -> std::io::Result<()> {
dispatch/stream.rs:1769:fn count_lines_upto(file: &mut std::fs::File, position: i64) -> std::io::Result<u64> {
dispatch/native/tests.rs:36:            let text = std::fs::read(&path).unwrap_or_else(|e| {
ir/corpus_shape_tests.rs:16:use std::fs;
ir/corpus_shape_tests.rs:493:        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
ir/corpus_shape_tests.rs:509:        let text = fs::read_to_string(&path)
ir/corpus_shape_tests.rs:560:            fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
plan/tests.rs:173:        for entry in std::fs::read_dir(&directory).expect("a readable corpus directory") {
plan/tests.rs:182:            let bytes = std::fs::read(&path).expect("a readable corpus program");
input.rs:80:        match std::io::stdin().lock().read(&mut chunk) {
input.rs:215:            Source::Bytes(cursor) => cursor.read(&mut buffer),
input.rs:359:            self.input.reading_stdin();
input.rs:593:        let count = Dribble(b'z').read(&mut buffer).expect("the source answers");
bin/rexx-ir.rs:15:use std::process::ExitCode;
bin/rexx-ir.rs:31:    let text = match std::fs::read(&path) {
bin/rexx-run.rs:16:use std::process::ExitCode;
bin/rexx-run.rs:40:            use std::os::unix::fs::FileTypeExt;
bin/rexx-run.rs:41:            std::fs::metadata(format!("/proc/self/fd/{fd}"))
bin/rexx-run.rs:76:    let text = match std::fs::read(&path) {
bin/rexx-run.rs:91:    let reported = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone().into());
```

</details>

### Migration divergences (Task 18, spec section 11)

* Successive native calls of one activity may run on different pool threads, so a thread-affine
  extension behaves differently from the oracle, where an activity's calls share its thread.
* Pool threads get the interpreter thread's stack (`INTERPRETER_STACK_BYTES`), not the oracle's
  512 KiB: a callback runs Rexx code there, and the translator's, evaluator's and activation
  depth limits are measured against that size. Recursion through `TestSendMessage0` callbacks
  (`scratchpad/t18rev/p9`, release `rexx-run`) with a second activity alive ends at the
  activation cap, 9999 levels (`e.rex`) and 10000 (`g.rex`), rc 245, as at `837482aa0`; the
  oracle ends at about 7750 on its activity threads. With the pool stack reduced in a test, the
  pool thread's own stack check raises 11.1 first
  (`scheduler/tests/pool.rs` `deep_pinned_recursion_on_a_pool_thread_raises_11`).
* Where no pool thread is free, a native call runs on the thread holding the baton and keeps it,
  so a call that waits for another pool thread's callback waits for it in vain.
* A lone activity's native call keeps the baton (ruling P43), so an activity its callback starts
  runs only once the call returns; a call that then waits for that activity waits in vain
  (`scheduler/tests/pool.rs`, `a_lone_call_keeps_the_baton_after_its_callback_starts_an_activity`).

### S4 close

#### Background gates

Run on `52b038a80` by `.superpowers/sdd/2026-10-01-phase-6-s2-s5/p6-gates/bggates.sh`; status file
`.superpowers/sdd/2026-10-01-phase-6-s2-s5/bg/52b038a80/status.txt`, logs beside it in `logs/`.
Its result lines, verbatim apart from the repository path:

```
52b038a80 started 2026-10-06T00:41:17+02:00
G1 fmt exit 0
G2 clippy exit 0
G3 release build exit 0
load G4 12.68 13.43 6.93 1/2788 888295 2026-10-06T00:45:24+02:00
G4 release test exit 101
G5 debug build exit 0
load G6 11.01 6.71 5.85 2/2779 1057485 2026-10-06T00:59:18+02:00
G6 debug test exit 0
G7 clippy pinning exit 0
G8 pinning self-tests exit 0
G9 loom exit 0
.superpowers/sdd/2026-10-01-phase-6-s2-s5/bg/52b038a80/logs/g4-test-release.txt:2
.superpowers/sdd/2026-10-01-phase-6-s2-s5/bg/52b038a80/logs/g6-test-debug.txt:0
P48 reruns: 2
finished 2026-10-06T01:20:34+02:00
```

G4 release: 3029 passed, 1 failed. G6 debug: 3034 passed, 0 failed (sums of `test result` lines).
G4's failure is `the_s2_rows_of_the_derived_list_in_both_modes`, at `base/rexxutil/SysSleep`
TEST_SLEEP_CONCURRENT, after its P48 rerun (`g4-test-release.txt:2052`, `:2190`). G6 passed the
same row only because both modes failed it alike: its cell is `differ: rc 1: oracle Failures 0,
ours Failures 1` / `same (P48 rerun)` (`g6-criterion-one-table.txt`).

The cause is a regression from `532bf29fa`. The timer thread waited in a socket read with
`SO_RCVTIMEO`, which the kernel runs on its timer wheel, so a `SysSleep` ended late. Measured by `call time 'r'; call SysSleep 0.3; say time('e') - 0.3` repeated
4 times in one program (`overshoot.rex`), with release `rexx-run` built from `git archive` of
each tree, then the oracle, one run each. Script and log, under
`.superpowers/sdd/2026-10-01-phase-6-s2-s5/s4-close-evidence/`:

```
bash overshoot.sh <scratch> 7266ae03c a48312f8e 532bf29fa 52b038a80 2e6917afe c66650b52 > overshoot.log
```

| tree | overshoot, s |
|---|---|
| `7266ae03c` (S3 close) | 0.000800 0.000426 0.000426 0.000411 |
| `a48312f8e` (Task 20) | 0.000578 0.000429 0.000469 0.000203 |
| `532bf29fa` (Task 21) | 0.014161 0.019931 0.019998 0.019978 |
| `52b038a80` | 0.011344 0.019943 0.020057 0.019918 |
| `2e6917afe` | 0.000097 0.000064 0.000081 0.000095 |
| `c66650b52` (Task 22 fix round 1) | 0.000456 0.000418 0.000413 0.000433 |
| oracle | 0.000146 0.000128 0.000128 0.000185 |

`2e6917afe` (Task 22) arms a timerfd for the deadline and polls it beside the wake socket; other
unix targets wait on the poll's timeout (`c66650b52`, which leaves the Linux path as `2e6917afe`'s).
The rows of the S3 close and both fixes are under half a millisecond, against 11 to 20 ms on the
regressed trees. `sync.rs` `a_timed_wait_ends_close_to_its_timeout` bounds the lateness at 1 ms. At `c66650b52`, from `rust/`, `s4-close-evidence/s2rows.sh`:

```
REXX_CORPUS_GATE=1 REXX_CRITERION_ONE_TABLE=<file> cargo test --release -p rexx-exec \
  --test concurrency_tests -- group_runs::the_s2_rows
```

exits 0 with no P48 rerun (`s2rows.log`), and the row is `pass` / `same` (`s2-table.txt:138`).
Every other row's cells are G6's; `base/keyword/REPLY` TEST_REPLY_TWICE_REPLYASSERT's normal cell
is `pass` here and in G6, and `differ: rc 0: oracle 1, ours 0` in G4: the oracle counts 1 in
some runs, as the P41 table above records.

G4 at `52b038a80` is red on the regression above and is not waived. The gates were run again in
full by the same script on `c66650b52`, the head that carries both fixes; status file
`.superpowers/sdd/2026-10-01-phase-6-s2-s5/bg/c66650b52/status.txt`, logs beside it in `logs/`. Its result lines, verbatim apart from the repository
path:

```
c66650b52 started 2026-10-06T11:22:26+02:00
G1 fmt exit 0
G2 clippy exit 0
G3 release build exit 0
load G4 13.46 16.25 10.31 4/2680 2200888 2026-10-06T11:27:01+02:00
G4 release test exit 0
G5 debug build exit 0
load G6 13.62 17.56 15.74 11/2831 2383822 2026-10-06T11:46:52+02:00
G6 debug test exit 0
G7 clippy pinning exit 0
G8 pinning self-tests exit 0
G9 loom exit 0
.superpowers/sdd/2026-10-01-phase-6-s2-s5/bg/c66650b52/logs/g4-test-release.txt:0
.superpowers/sdd/2026-10-01-phase-6-s2-s5/bg/c66650b52/logs/g6-test-debug.txt:0
P48 reruns: 0
finished 2026-10-06T12:08:22+02:00
```

G4 release: 3033 passed, 0 failed. G6 debug: 3037 passed, 0 failed (sums of `test result` lines).
`the_s2_rows_of_the_derived_list_in_both_modes` passes in G4 with no P48 rerun on the SysSleep row
(`g4-test-release.txt:2049`, `g4-criterion-one-table.txt:138`). The criteria below cite this run's
logs; the `52b038a80` run stays as the record of the regression.

#### Criteria

* **Criterion 1.** At `c66650b52`, G4's and G6's tables (`g4-criterion-one-table.txt`,
  `g4-criterion-one-s3-table.txt`, `g4-timer-table.txt` and the G6 ones) equal the S3 close's rows
  above, apart from `SysSleep` TEST_SLEEP_CONCURRENT, which now passes, and the normal cells of
  G4's `REPLY` TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT, TEST_REPLY_EXIT_CODE_REPLYASSERT and
  TEST_REPLY_STACK_REPLYASSERT, which differ from the oracle's run by one assertion at rc 0
  (`g4-criterion-one-table.txt:117-119`; P41). They pass in G6, and
  `diff bg/52b038a80/logs/g4-criterion-one-table.txt bg/c66650b52/logs/g4-criterion-one-table.txt`
  shows the same three rows passing at `52b038a80`. Every Alarm and Ticker test passes in both
  modes in both gate runs.
* **Criterion 2.** `the_framework_ticker_runs_without_dash_u` passes in G4 and in G6
  (`g4-test-release.txt:1663`, `g6-test-debug.txt:1667`); `the_alarm_and_ticker_groups_pass_in_both_modes`
  passes in both (`g4-test-release.txt:2050`, `g6-test-debug.txt:2055`).
* **Criterion 3.** Below: the ThreadSanitizer run is clean, and G9's `loom` passes.
* **Criterion 9.** `phase-6-pinning.md` `## S4 close`: no inverted-wait refusal, no immovable
  `REPLY`, and no hang in either mode over the derived list. The test that ends at the run's
  deadline under every opportunity is `MutexSemaphore` TEST_EXCLUSION, the oracle's own hang
  (P46).
* The gate-only `outer_context.rs` tests (Task 19's kept-context 98.983 programs, against the
  oracle) pass in G4 and in G6: `a_kept_outer_context_reaches_its_callers_variables`,
  `a_kept_call_context_used_by_another_activity_answers_or_raises` and
  `a_kept_thread_context_used_by_another_activity_does_nothing` (`g4-test-release.txt:3252-3254`,
  `g6-test-debug.txt:3257-3259`).

#### Criterion 3, race checking

The ThreadSanitizer run, gate-only, from `rust/` with the installed nightly
(`rustc 1.100.0-nightly (4aa1fbcf4 2026-09-08)`, its `rust-src` component), is
`.superpowers/sdd/2026-10-01-phase-6-s2-s5/s4-close-evidence/tsan.sh`:

```
bash tsan.sh <target dir> <log dir>
```

It runs `cargo +nightly test -Zbuild-std --target x86_64-unknown-linux-gnu` under
`RUSTFLAGS="-Zsanitizer=thread"` and `TSAN_OPTIONS="log_path=<log dir>/tsan
suppressions=$PWD/tsan.supp allocator_may_return_null=1 second_deadlock_stack=1"` on rexx-api's
lib tests, rexx-exec's lib tests less the exclusions below (`--exact --skip <path>`), and the
`signals`, `stdin_contention`, `program_end` and `concurrency_tests` targets less one test.

Clean means every command exits 0 and `<log dir>` holds no `tsan.*` file. `rexx-run`, which the
integration tests start, is built with the same flags, and its children write to `log_path`
too: `sigint_ends_a_parse_pull` run with `verbosity=1` added left a `Running under
ThreadSanitizer` log for the test process and for each `rexx-run` it started.

The covered paths: the baton, the inbox and completions (`scheduler::`, the lib's program
runs, each on an interpreter thread with the timer), the timer and its wake source (`sync::`,
every `SysSleep`, `GUARD WHEN` and slice), the driver pool (`scheduler::tests::pool`, `::native`,
`::callbacks`, `::lent`, rexx-api's `ffi::` callbacks), signals (`signals.rs`, `signal::`) and
the stdin read (`stdin_contention.rs`, `input::`). `concurrency_tests` runs here without
`REXX_CORPUS_GATE`: its rows against the oracle are timing-dependent and run in G4 and G6. The
pool's callback recursion runs bounded, 25 levels
(`scheduler::tests::pool::bounded_callback_recursion_nests_under_one_lend`: one driver exit, one
baton take), not at depth.

Result at `c66650b52`, one run, `tsan.sh` output and the test logs in `s4-close-evidence/tsan/`:
`api exit 0`, `lib exit 0`, `int exit 0`, `no tsan log`. The `test result` lines: rexx-api lib
`70 passed`; rexx-exec lib `990 passed ... 13 filtered out`; `concurrency_tests` `32 passed`,
`program_end` `2 passed`, `signals` `36 passed ... 1 filtered out`, `stdin_contention` `1 passed`.

Excluded, each by exact path:

| test | reason |
|---|---|
| `eval::tests::a_native_chain_past_the_eval_limit_runs` | recursion to a depth limit calibrated to the native stack; TSan enlarges every frame, and its shadow call stack holds 65536 frames; an `eval::tests` depth test crashed TSan under `gdb`, its stack nearly all `Plan::note` |
| `eval::tests::eval_raises_11_1_exactly_one_term_past_max_eval_depth` | the same |
| `eval::tests::eval_survives_exactly_max_eval_depth_terms_and_prints_the_oracles_own_answer` | the same |
| `ir::drive::tests::recursion_by_function_call_keeps_the_native_stack_flat` | recursion to a depth limit calibrated to the native stack; TSan enlarges every frame, and its shadow call stack holds 65536 frames |
| `ir::drive::tests::recursion_by_new_into_init_keeps_the_native_stack_flat` | the same |
| `ir::drive::tests::recursion_by_send_keeps_the_native_stack_flat` | the same |
| `scheduler::tests::a_stack_within_the_margin_is_refused` | a thread stack sized against the stack margin, which TSan's frames change |
| `scheduler::tests::nested_pinned_waits_are_bounded_by_the_stack_remaining` | reaches Error 11 at a smaller depth than it asserts under TSan |
| `scheduler::tests::pool::callback_recursion_on_a_pool_thread_reaches_the_depth_cap` | nested `TestSendMessage0` callbacks crashed TSan under `gdb` past its shadow stack; its bounded form runs |
| `scheduler::tests::pool::deep_pinned_recursion_on_a_pool_thread_raises_11` | recursion to a depth limit calibrated to the native stack; TSan enlarges every frame, and its shadow call stack holds 65536 frames (a pool thread's) |
| `scheduler::tests::pool::the_translator_on_a_pool_thread_has_the_interpreter_threads_stack` | translator nesting to a depth calibrated to a pool thread's stack |
| `scheduler::tests::recursion_is_bounded_by_the_stack_remaining` | recursion to a depth limit calibrated to the native stack; TSan enlarges every frame, and its shadow call stack holds 65536 frames |
| `tests::the_stack_span_does_not_depend_on_what_else_the_program_evaluated` | measures the native stack span, which TSan's frames change |
| `only_the_interpreter_and_its_waits_take_the_halting_signals` (`signals`) | counts the `rexx-run` threads with the halting signals blocked, and TSan adds its own background thread: in a TSan `rexx-run`, `/proc/<pid>/task/*/status` shows a second `rexx-run` thread with nearly every signal blocked beside this crate's thread blocking SIGHUP, SIGINT and SIGTERM |

Reports, each triaged:

| what | where | verdict |
|---|---|---|
| data race, read and write of a mutable buffer's bytes | `rexx_api::load::hold_buffer` on a pool thread against `BufferBytes::extend_from_slice` / `try_reserve_exact` on the holder, `scheduler::tests::lent` `a_buffer_changed_in_place_under_a_call_stays_shared` and `a_buffer_grown_under_a_call_keeps_the_storage_the_call_holds` | false positive at `b8ec39593`: the test ordered the two through a file. Since `c66650b52` it orders them through a lock (`BUFFERHELD`, `BUFFERCHANGED`), and no suppression covers it. Control: with `HOLDBUFFER` sleeping in place of the lock wait, the two tests write one data race report (`tsan/control-no-handshake.*`) |
| data race in `free` | glibc's `_dl_close_worker` under `Library::close` at program end on two interpreter threads (`dispatch::library::tests::a_library_call_answers_the_same_under_a_collection_at_every_allocation` and another test's run); found once | false positive: both run under the dynamic linker's `dl_load_lock`, taken inside ld.so where TSan does not see it. Suppressed, `race:_dl_close_worker` |
| SEGV in `__tsan_func_entry` / `__tsan::CurrentStackId` | the depth tests above; found under `gdb` | TSan's shadow call stack. Not a race. Excluded |

`tsan.supp` holds the one suppression, with its reason.

**loom.** G9 at `c66650b52`: `RUSTFLAGS="--cfg loom" cargo test -p rexx-exec --test loom`,
`test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 401.33s`
(`g9-loom.txt`), the registration models under the P58 preemption bound.

#### Rulings and licensed divergences of S4

Rulings, `.superpowers/sdd/2026-10-01-phase-6-s2-s5/progress.md`:

* P50: no API callback touches island state off the baton; the Conversion split and the
  baton-guarded `Host` accessor landed in Task 17.
* P51: per-task checks are fmt, clippy and `cargo test --workspace --release`; the full gates run
  at stage closes.
* P52: a pool thread runs the native call and the holder lends the baton through the inbox.
* P53: a run ended with a call in flight leaks its `Interp` and leaves its pool threads blocked
  (queued `2026-10-04-aborted-run-leaks-interp`).
* P54: pool threads reserve `INTERPRETER_STACK_BYTES`, as interpreter threads do.
* P55: with the pool at its bound, the fallback call keeps the baton; a call waiting on another
  pool thread's callback then hangs (Migration divergences above).
* P56: the thread-local `CALLING` in rexx-api `ffi.rs`.
* P57: a callback from a thread running no native call, while the holder runs a baton-keeping
  call, waits for it; where the native joins that thread the run hangs, where the oracle aborts
  rc 134.
* P58: the timer thread starts at every interpreter registration; the registration loom models
  run under a preemption bound of 5.
* P59: a signal wakes and halts every sleeper (DEVIATIONS entry 10).
* P60: a halt ends the `ADDRESS` child wait and reads of the default input stream; other
  blocking reads stay uninterruptible (DEVIATIONS entry 12).
* P61: handlers install over `SIG_DFL` and `SIG_IGN` for SIGINT and SIGTERM, over `SIG_DFL` only
  for SIGHUP.
* P62: handlers install at `rexx-run` and the C API's interpreter creation only.
* P63: `ADDRESS` children start with SIGPIPE at its default (DEVIATIONS entry 11).
* P64: every unredirected `ADDRESS` command waits off the baton; without a pool reservation it
  waits inline, uninterruptible (DEVIATIONS entry 12).
* P65: a halt withdraws semaphore waits and not message waits; a message wait whose runner the
  halt does not wake can end in the loud "nothing left to run" refusal.
* P66 and P68: withdrawn by P69.
* P67, as amended: SIGINT, SIGTERM and SIGHUP are blocked in pool, timer and other crate threads
  and unblocked around the child wait and the stdin read; the interpreter thread keeps them. It
  holds for one interpreter per process (queued `2026-10-05-two-interpreters-signal-race`).
* P69: no other activity runs during a read of the default input stream (DEVIATIONS entry 13).

DEVIATIONS rows added in S4 (`phase-4-exclusions.txt`), each with owner none:

* 9, a native call's old MutableBuffer storage outlives a collection (Task 20, spec 2.5);
* 10, a signal halts where the oracle dies or hangs, and wakes what the oracle leaves waiting
  (Task 21, P59-P61);
* 11, an `ADDRESS` command's child starts with SIGPIPE at its default (P63);
* 12, a blocking wait a signal does not end (P60, P64);
* 13, no other activity runs during a read of the default input stream (P69).

The Migration divergences above (Task 18) are the others S4 records.

#### Queued in S4

`.superpowers/sdd/queued/`: `2026-10-04-aborted-run-leaks-interp`,
`2026-10-05-call-on-notready-stdin-eof`, `2026-10-05-stdin-chars-after-drain`,
`2026-10-05-stream-fifo-terminal-seek`, `2026-10-05-two-interpreters-signal-race`; and a design
note, not a defect, `2026-10-04-send-site-cache-self-customization`.

## S5

### Criterion 4, the single-owner audit (Task 23)

The inventory is derived by ruling P71's command, from the repository root at `0ac73b804`; its
output is `docs/superpowers/records/2026-10-01-phase-6-s2-s5/criterion-4-inventory.txt`:

```
/bin/grep -a -rn 'unsafe impl\|thread::spawn\|thread::Builder\|thread::scope\|Arc<\|Mutex<\|Condvar\|Atomic\|thread_local!' rust/crates
```

Every hit falls in a row below (file and line at `0ac73b804`). A plain `static` needs no row: the
compiler requires it to be `Sync`.

| Site | Cross-thread type | Why sound | Fact or test |
|---|---|---|---|
| `rexx-exec/src/island.rs:52` | `Islanded<T>`, for the payloads `IslandPayload` lists (`:31`): `NonNull<Interp>` (`Island`) and `(Box<OffBaton>, ThreadContext)` (`PooledCall::work`, `dispatch/library.rs:1077`), which holds `ObjRef`s and an `Rc` | made and taken only on the baton; moved untouched; `Lent<'b>` bounds the `&mut Interp` | `unsafe impl<T: IslandPayload> Send` (P72): a third payload does not compile (`task-23-report.md`); `island::tests::an_island_value_is_made_only_on_the_baton`, `..._taken_only_on_the_baton`; TSan (criterion 3) |
| `rexx-exec/src/island.rs:130-138` | `Interp`, `RegFrame<'static>` | neither is `Send` | not-`Send` assertion (A3); its control, the same assertion on `u64`, fails E0283 (`task-23-report.md`) |
| `rexx-exec/src/baton.rs:21,23,26` | `Baton<L>`: `Mutex<State>`, `Condvar`, test `AtomicU64` | the holder and lend state is the lock's data; `lent()` answers only to the lendee | `tests/loom.rs` baton models; TSan |
| `rexx-exec/src/timer.rs:64,85,87,178,245,287,321`; `lib.rs:1238`; `dispatch/library.rs:1081-1211` | `Arc<Inbox<Posted>>`, `Arc<InterpBaton>`, `Requests(AtomicU32)` | `Posted` holds no `ObjRef`; request bits publish no data | `require_send::<Posted>()` (`scheduler.rs:171`, A4); `ObjRef` compile_fail (`rexx-core/src/handle.rs:52`, `:57`); `tests/loom.rs` inbox and timer models; `scheduler::tests::a_post_from_another_thread_ends_an_idle` |
| `rexx-exec/src/scheduler/pool.rs:52-83,123,177` | `Arc<Shared>`, `Arc<Mailbox>`, `Job = Box<dyn FnOnce() + Send>` | a job is `Send` by its type; the job kinds (`scheduler.rs:633` native call, `:664` blocking operation, `input.rs:377` stdin) capture `Send` values and `Islanded` | the type of `Job`; `scheduler/tests/pool.rs` |
| `rexx-exec/src/timer.rs:205` | timer thread | touches only the registry (`Mutex<State>`) and inboxes | `static REGISTRY` (`Sync` by the compiler); loom timer models |
| `rexx-exec/src/lib.rs:3151` | interpreter thread | its body is `FnOnce + Send + 'static`; `Interp` is built on it | the closure's bound; A3 |
| `rexx-exec/src/command.rs:618` | pipe drain, `thread::scope` | captures pipes and `&(dyn Fn + Sync)` only | the `Sync` bound |
| `rexx-exec/src/signal.rs:36,40` | `static PENDING: AtomicBool`, `WAKE: AtomicI32` | async-signal-safe flags; carry no data | `signal.rs` unit tests; loom `timer::signal` models |
| `rexx-exec/src/sync.rs` | std or loom re-exports, `Wake` | the shim the loom models compile | `tests/loom.rs` |
| `rexx-api/src/ffi.rs:109,112` | `unsafe impl Send for ValueDescriptor`, `Value` | a word and two ints; pointer members are addresses, dereferenced in `unsafe` at their sites | `require_send::<NativeCall>()`, `<Completion>()` (`invoke.rs:250-254`); `rexx-api/tests/invoke.rs:754-760` moves both across threads |
| `rexx-api/src/ffi.rs:619` | `Requester = Arc<dyn Baton + Send + Sync>` | a foreign thread reaches the interpreter only by taking the baton | `HostRef` debug asserts (`ffi.rs:192`, `:208`, `:1320`) tested by `ffi.rs:6330`; `scheduler/tests/callbacks.rs` |
| `rexx-api/src/ffi.rs:700`, `layout.rs:421`, `load.rs:1518` | non-test `thread_local!`s `CALLING`, `REFUSED`, `HOOK_THREW` | per native call on one OS thread's stack; no `ObjRef` | a `thread_local!` value is not sent; every other `thread_local!` hit is `#[cfg(test)]` or in a test file (`/bin/grep -a -rn -B3 'thread_local!' --include=*.rs rust/crates`) |
| `rexx-api/src/load.rs:113-205,1157-2018` | `Arc<Mapping>` (`Mutex<Option<Library>>`, `AtomicBool`, `AtomicUsize`) | a close refuses while a call is counted in flight; the row outlives its mapping by the `Arc` | compile_fail `load.rs:191`, `:1681`; `load.rs` tests |
| `rexx-api/src/load.rs:746-900` (doc-hidden) | foreign-thread and buffer-handshake test natives (P70) | play an extension's foreign thread | `scheduler/tests/callbacks.rs`, `scheduler/tests/lent.rs` |
| `rexx-parse/src/selector.rs:19,44` | `Selector(Arc<[u8]>)` | immutable bytes | `Arc<[u8]>: Send + Sync` |
| test, tool and probe files: `rexx-api/src/ffi.rs` tests, `rexx-api/src/invoke/tests.rs`, `rexx-api/tests/invoke.rs`, `rexx-exec/src/scheduler/tests*`, `rexx-exec/src/dispatch/library/tests.rs`, `rexx-exec/src/ir/corpus_shape_tests.rs`, `rexx-exec/src/bin/rexx-ir.rs`, `rexx-exec/tests/*`, `rexx-classes/tests/*`, `rexx-parse/tests/deep.rs`, `rexx-parse/examples/depth_probe.rs`; and `#[cfg(test)]` counters in `rexx-exec/src/{lib,scheduler,install,ir/compile,ir/counters,dispatch/library}.rs`, `rexx-num/src/addsub.rs` | test harness, tool and probe threads, test counters | outside the interpreter or compiled only for tests | not in the shipped interpreter |

The static assertions Task 23 adds (the S5 audit's A1-A5, `s5-find-audit.md`):

* A1, `rexx-core/src/frame.rs:90`, `:99`: compile_fail doctests, a `RegFrame` held by a `'static`
  value (E0597) and sent (E0277), with a compiling control holding the register's `ObjRef`.
* A2, `rexx-core/src/body.rs:138`: `require_static::<Body>()`.
* A3, `rexx-exec/src/island.rs:130-138`: `Interp` and `RegFrame<'static>` are not `Send`.
* A4, `rexx-exec/src/scheduler.rs:171`: `require_send::<Posted>()`.
* A5, `rexx-exec/src/island.rs:31`, `:52`: `Islanded`'s payloads sealed (P72).

`cargo test -p rexx-core --doc` at `0ac73b804`: 3 passed, and 5 compile_fail passed.

Two spec sentences are amended to what holds (P73), in
`docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md`: section 5's "refuses an object
handle" adds that object handles reach a pool thread of the same interpreter only inside
`Islanded` (P52); section 2.5's non-test `thread_local!`s add `CALLING` (P56).

### Criterion 5, D3 frame ownership

| Claim | Fact or test |
|---|---|
| no heap object holds a frame | `Body` has no lifetime (`rexx-core/src/body.rs:57`) and is `'static` (A2, `:138`); `RegFrame<'a>` borrows its arena (`frame.rs:117`), cannot outlive it (compile_fail `frame.rs:168`), cannot be held by a `'static` value or sent (A1, `frame.rs:90`, `:99`); `NativeState::Pointer(*mut c_void)` (`body.rs:177`) is a raw pointer outside the lifetime fact and holds C-supplied addresses only (P74) |
| contexts resolve across all activities | `dispatch/context.rs:159` `at_context`, `scheduler.rs:723` `idle_context_owner` |
| live, finished, moved | `scheduler::tests::a_context_reads_an_activation_of_another_live_activity` (live, then 98.981 once finished), `scheduler::tests::a_context_follows_its_activation_to_a_reply_continuation` (moved by REPLY); corpus `context_of_another_activity`, `context_moved_by_reply` |

### Criterion 6, the sharing fraction

The `sharing` feature of `rexx-exec` (forwarding `rexx-core`'s) tags each heap slot with the last
activity that resolved its object through `Heap::get`, `Heap::get_mut` or `Heap::body_text`, or made
it, and counts objects touched by more than one activity; `Outcome::sharing` reports the counts.
`concurrency_tests` `sharing::one_activity_shares_nothing` and
`sharing::an_object_read_by_a_started_activity_is_shared` witness it; the second fails with the
tag switch in `Interp::switch_to` and `Interp::swap_running` removed (`task-23-report.md`).

Off, it costs nothing: release `rexx-run` built without features from `0ac73b804` has the same
`.text` hash as one built from the base with only the static assertions applied, and the same
functions with the same sizes as the base apart from one symbol's name
(`docs/superpowers/records/2026-10-01-phase-6-s2-s5/sharing-off-text-hash.txt`).

At `0ac73b804`, from `rust/`, files in `docs/superpowers/records/2026-10-01-phase-6-s2-s5/`:

```
memcap 8G cargo test --release -p rexx-exec --features sharing --test corpus -- --exact sharing_fraction_over_the_corpus --nocapture
memcap 8G cargo test --release -p rexx-exec --features sharing --test concurrency_tests -- --exact sharing::sharing_fraction_over_the_derived_list --nocapture
```

| run | record | objects | made before the program | shared | shared / objects |
|---|---|---|---|---|---|
| corpus, every program of the differential | `sharing-corpus.md` | 520530 | 214064 | 1088 | 0.21% |
| ooTest, criterion 1's derived list | `sharing-derived.md` | 374510 | 54944 | 790 | 0.21% |

Each record lists its programs or tests with a shared object (corpus) or every test with its
outcome (derived list).

### Criterion 7, the ping-pong benchmark

`rust/bench-programs/pingpong/pingmsg.rex` (a message started and its result awaited),
`pingsem.rex` (two activities alternating through two `EventSemaphore`s) and `pingguard.rex` (two
activities passing a turn through `GUARD ON WHEN`). Recorded, not gated. From `rust/`, `rexx-run` a
release build of `e57dc8315` without features, 32 CPUs:

```
PROGRAMS="pingpong/pingmsg pingpong/pingsem pingpong/pingguard" bash bench-programs/wallclock.sh -r 9 -o OUT -x "pingpong/pingmsg pingpong/pingsem pingpong/pingguard" rexx-run=TARGET/release/rexx-run
```

Load averages 1.18 3.46 6.68 before and 1.42 3.31 6.52 after
(`docs/superpowers/records/2026-10-01-phase-6-s2-s5/pingpong/binaries.txt`); every run exited 0 with
the same stdout on both sides. Medians of 9 interleaved runs, seconds (`pingpong/table.txt`, runs in
`pingpong/wall.tsv`):

| program | rexx-run | oracle |
|---|---|---|
| `pingmsg` | 1.325 | 0.656 |
| `pingsem` | 0.121 | 0.581 |
| `pingguard` | 0.119 | 0.470 |

Each program gave one stdout and rc 0 in 30 runs on each side (`pingpong/thirty-runs.txt`, by
`pingpong/thirty-runs.sh`).
