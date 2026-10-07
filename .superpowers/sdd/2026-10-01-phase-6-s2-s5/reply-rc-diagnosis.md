# REPLY.testGroup derived/every: rc 1 with every test passing

Gate row (G4 release at b6efbfd53, `g4-test-release.txt:2210-2213`):
listed `pass, assertions 18, rc 0, ...`, observed `pass, assertions 18, rc 1, ...`.

## Verdict

A real, deterministic defect, made visible by load: **after `REPLY`, the
continuation's `TIME('E')` restarts from zero**, because the elapsed clock lives
on the `Activity` and `split_level` does not carry it to the continuation's new
activity. The continuation of `TEST_REPLY_SAME_REPLYASSERT` therefore fails its
last assertion (`time(e) must be > 0, found 0`) in **every** run. Whether that
failure changes the rc depends only on when it lands relative to the program's
end. No perf round causes it: c484f4516 has it too.

## Reproduction

Copy built as `whole_groups::copy` does (fresh_copy of framework +
`ooRexx/base/keyword` + driver files, `rxfuncquery` lines removed,
`rxregexp.cls`, `mark_starts`, `test_reply_interpret` renamed to
`SKIPPED_test_reply_interpret`). Each commit built from a touched `git archive`
(`rust interpreter extensions`) in its own target dir; every build log has
`Compiling rexx-exec`. Run, from the copy:

```
PATH=<bin dir with rexx -> rexx-run>:$PATH LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
REXX_SWITCH_MODE=every timeout -k 5 120 <target>/release/rexx-run \
  <copy>/testOORexx.rex -f <copy>/ooRexx/base/keyword/REPLY.testGroup -U -V 2 </dev/null
```

"Load" = 30 such runs at once (separate copies) plus 40 `timeout 60 yes >/dev/null`
busy loops, on 32 cores.

| commit | condition | rc 1 |
|---|---|---|
| 05aac0c58 (= HEAD code) | serial, idle | 0 / 10 |
| 05aac0c58 | 30 parallel, no busy loops | 0 / 30 |
| 05aac0c58 | load | 3 / 30, 3 / 30, 0 / 30, 0 / 30 |
| c484f4516 (before the perf rounds) | load | 1 / 30, 17 / 30 |
| 05aac0c58 + proposed fix | load | 0 / 90 |
| oracle (instrumented copy) | serial | 0 / 5, no failure recorded |

The rate swings between batches (0 to 17 of 30) with machine noise, so the
rc-1 rate does not separate the commits; ba8f7c581 and 634f591a8 were not run,
since the defect is present before all three perf rounds.

The rc-1 runs' stdout is identical to the rc-0 runs' apart from timings
(`Failures: 0` printed), stderr identical (only `started` lines).

## Mechanism

Instrumented copies (stderr line in `ooTestResult~addFailure`,
`ooTest.frm:994`, and the three counts before `worker.rex:150`'s `return max(...)`):

- 150 of 150 crate runs (05aac0c58 and c484f4516) record
  `addFailure TEST_REPLY_SAME_REPLYASSERT | time(e) must be > 0, found 0`, and
  no other failure.
- rc-0 runs print `counts 0 0 0` **before** that line: the continuation's
  failure lands after `finishTestRun` computed its result; P41 lets it run
  before the program ends, where it changes nothing.
- rc-1 runs print the failure **before** `counts 1 0 0` but after
  `testResult~print`: so `Failures: 0` is printed and `worker.rex:150` returns
  `newFailureCount > 0` = 1. Load slows the main activity enough for the
  continuation (`call SysSleep 0.01`, then the assertion) to get in first.
- With the fix, 0 of 90 runs record any failure.

The test (`ootest/ooRexx/base/keyword/REPLY.testGroup:139-155`) does
`assertSame(0, time("e"))`, `reply .`, ..., `call SysSleep 0.01`,
`assertTrue(time("e") > 0)`. Probe, run from an empty dir:

```
o = .t~new
say "main got" o~m
call SysSleep 0.3
::class t
::method m
  say "before" time("e")
  reply 7
  call SysSleep 0.01
  say "after1" time("e")
  call SysSleep 0.01
  say "after2" time("e")
```

oracle: `after1 0.010893`, `after2 0.021727`; crate (05aac0c58 and c484f4516,
both modes): `after1 0`, `after2 0.010515`; crate + fix: `after1 0.010581`,
`after2 0.021016`.

Code:
- `rust/crates/rexx-exec/src/activity.rs:267,274`: `elapsed_anchor` and
  `pending_elapsed_reset` are fields of `Activity`, `None`/`false` in
  `Activity::new` (`:441-442`).
- `rust/crates/rexx-exec/src/builtin/datetime.rs:863`: `TIME('E')` anchors
  lazily with `get_or_insert(reading)`, so a fresh activity's first reading is 0.
- `rust/crates/rexx-exec/src/ir/drive.rs:2938` `split_level`: builds the
  continuation with `self.new_activity()` (`:2946`) and copies clause state,
  indents and `clause_line_override` (`:2980-2983`), but not the elapsed clock.
- Oracle: the clock is `ActivationSettings::elapsedTime`
  (`interpreter/execution/ActivationSettings.hpp:191`, read by
  `RexxActivation::getElapsed`, `RexxActivation.cpp:3424`), and the replying
  activation, settings included, migrates to the new activity
  (`RexxActivation.cpp:730-744`), so the clock survives `REPLY`.

Not P39/P40/deviation row 21: no continuation EXIT or error reaches the
program end; rc 1 is ooTest's own `newFailureCount > 0`.

## Proposed fix (smallest)

In `split_level`, after `ir/drive.rs:2983`:

```rust
continuation.elapsed_anchor = self.activity.elapsed_anchor;
continuation.pending_elapsed_reset = self.activity.pending_elapsed_reset;
```

Tried in a scratch copy of 05aac0c58: probe matches the oracle, 0 / 90 under
load, no recorded failure. Only `rexx-run` was built; no test suite was run
with it. With it, the two REPLY rows of `DIFFERING` (whole and derived,
`every`) may start agreeing with the oracle; `verdict` accepts a listed row
that agrees, so they would not fail, but they may become removable.

## Related divergence, not fixed by this

The clock is per activity here and per activation in the oracle. A method
invoked after the caller started its clock (probe: `say time("e")`,
`SysSleep 0.05`, then a method's first `time("e")`) reads `0` on the oracle
and `0.050234` here. A full fix moves the clock onto the activation (internal
calls copy it, new method/routine activations start fresh), which also makes
the `REPLY` case follow from the activation migrating. `RANDOM`'s
`random_seed` (`activity.rs:258`) sits on the activity the same way and was
not checked.

## Outcome (ruling P89)

Applied with `random_seed` too. Probe, oracle twice: a method seeds `random(1, 100000, 7)`, replies,
and draws two more; the oracle's continuation draws `72518 13736`, the same as a method drawing
all three without a REPLY. Ours drew `307 77748` and `89491 82052`, a fresh seed each run. Witness
`scheduler::tests::a_reply_continuation_keeps_the_elapsed_clock_and_the_random_seed`: red without
the fix, and red with any one of the three copied fields left out.
