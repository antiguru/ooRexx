# 2026-10-02-do-with-over-refusal

Found by the Phase 6 S2-S5 Task 10 review (a). Not fixed; owned by no Phase 6 stage.

`DO WITH INDEX ... ITEM ... OVER` and `DO COUNTER ... OVER` are refused with `DO is not
implemented` (`run/tests/loops.rs:196-228`). Criterion-1 rows that reach it: `Method` TESTDIRECTIVES
(`do with index ... item ... over`, `do counter ... over`), `TRACE_TraceObject`
TEST_TRACEOBJECT_COLLECTOR (`do counter`), and the `MethodArgs` TEST_REQUEST_STRING_* rows, whose
`checkRequestString` runs `do with index instance item hasMakeString over ...`.

The MethodArgs rows differ between switch modes only in which refusal comes first. `activate
class` builds an `Alarm` and calls `cancel` at once; `Alarm~cancel` runs `guard on when
timerStarted`, which init sets in the continuation after its `REPLY`.

- Unswitched, main runs first after the REPLY, `timerStarted` is false, and the GUARD WHEN refusal
  (S3) is reached.
- Under `EveryOpportunity` the continuation runs first, the GUARD WHEN is satisfied, and main
  reaches the DO WITH refusal.

Probe `a2.rex`:

    a = .Alarm~new(1, .Message~new(.nil, "hashCode"))
    a~cancel
    say 'cancelled'

Unswitched: GUARD WHEN refusal, rc 120. `REXX_SWITCH_MODE=every`: `cancelled`, rc 0, 10 of 10 runs;
the oracle prints `cancelled`, rc 0. Adding `do with index i item v over .array~of(1); end; say 'end'`
gives under every `cancelled`, `DO is not implemented`, rc 120; the oracle prints `cancelled`,
`end`, rc 0.
