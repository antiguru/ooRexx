# Task 9 report: TraceObject fields, UNINIT and program end

Status: DONE. Base a4bde5677. Commit 826b07b45 (code, tests, witnesses, gate record, this report).

## gate_table_c hang (abf33a371 G4)

Reproduced: `concept_and_class_gate_table` alone, release, `REXX_CORPUS_GATE=1`, killed by
`timeout -s KILL 400` while still running. A sweep of every table C probe through `rexx-run` under
`timeout -s KILL 15` hung on exactly one: `corpus/gate-tables/methods/alarm__instance.rex`.

Why: its construction is `.Alarm~new(99999, ...)~~cancel`. `Alarm~cancel` runs
`guard on when timerStarted`; `timerStarted` is set by the alarm's started activity, so the WHEN is
false when main reaches it, and this crate refuses a GUARD WHEN that has to wait (Task 12). Main
ends loud; Task 8's program end then waits for the alarm's activity, parked on a 99999 s timer. The
oracle answers the row (7 lines, rc 0): its cancel waits for `timerStarted`, then cancels. So the
row ends on the oracle and not here, until Task 12. The harness ran it with `Invocation::none()`.

Fix: `gate_tables::run_gate_probe` (tables C, D, native entries) runs every probe with the
oracle's own bound, `ORACLE_DEADLINE` (10 s). The row now reports `diverge-both`, loud construct
"a GUARD that has to wait for another activity ...", owner 6, not gated; the table finishes in
16.7 s (`gated by this run: 0`).

Deadlines elsewhere (ruling R-T9-1 below): every in-process harness whose programs come from files
it does not write (corpus, ooTest, gate tables) now carries a deadline: `support::oracle::RUN_DEADLINE`
(60 s) in `licensed_divergences`, `directive_options`, `bootstrap_install_oracle`,
`builtin_status`, `state_builtin_oracle`, `parse_version_oracle`, `outer_context`, `trace_oracle`,
`trace_indent`, `ir_recorded_oracle`, `collect_stress` (stress runs 600 s); `corpus.rs`,
`method_bodies.rs` and the group runner already used the watchdog. `scheduler/tests.rs`'s `run`
carries one too.

## Step 1: UNINIT ordering (verify)

Recorded in `docs/superpowers/plans/phase-6-gate.md` `## S2`. Oracle, 30 runs each:
`u1` 30/30 `main end|activity end|uninit live`; `u2` 30/30 `activity end|main end|uninit dropped`;
`u3` (`call gc 'force'` in the activity) 30/30 `activity end|main end|uninit dropped 1`; `u4`
(allocation pressure then a `call`) 30/30 `activity loop done|uninit dropped|activity end|main
end`; `u5` (REPLY in a termination UNINIT) 29 with `uninit after reply`, 1 without.

Ours after this task, unswitched and `every`: `u1`, `u2`, `u5` match the oracle's majority; `u3`
runs it at the forced collection (`uninit dropped 2|activity end|main end`), `u4` at the
activity's end (`activity loop done|activity end|uninit dropped|main end`): collection timing, not a
specified observable (memory oorexx-gc-ordering-divergence-licensed).

## Step 2: TraceObject fields

`environment/route.rs` `deliver_trace_line`, in the oracle's put order: TRACELINE, INTERPRETER (1),
THREAD (`activity_number()`, lazy as R-T3-1 requires), INVOCATION (`invocation_of(0)`, 0 with no
activation), STACKFRAME (a `StringTable` of the innermost live level's `StackFrame`, ARGUMENTS
INVOCATION LINE NAME TRACELINE TYPE EXECUTABLE TARGET, `getStackFrameAsStringTable`), on `>I>`
lines CALLERSTACKFRAME (the next level down, else the spawner's), and for a method activation
ATTRIBUTEPOOL 0, ISGUARDED `.false`, SCOPELOCKCOUNT 0, HASSCOPELOCK `.false`, RECEIVER.

The spawner: `Activity::spawner` (rooted), captured at `~start` (`spawn_send`, innermost level of
the starting activity, i.e. the frame that sent `~start`, the oracle's skip-first) and at a REPLY
split (`split_level`, the replying frame, skip-first false), as a `StackFrame` plus executable;
turned into the `StringTable` with THREAD on the first `>I>` that needs it and shared after
(`Activity::setCallerStackFrameAsStringTable`).

## Step 3: program end

`scheduler.rs` `run_started_activities`: runs others until none is left; where activities remain
and nothing is ready or sleeping, `idle_for_good` idles on the timer thread (`idle_until`, a day at
a time) until the run's deadline. No refusal at the end any more (`Loud::unsatisfiable_wait` stays
for waits main makes). Before idling, `idle_until` hands the buffered output to the embedding's
sinks, so `rexx-run` shows main's output while it waits (the oracle writes it as it goes).

UNINIT: readied `UNINIT`s run when a started activity ends (`run_until_park`, `Activity.cpp:249`)
and when main ends (`execute_on`, `:324`); then the wait for every activity; then the termination
sweep; then the activities the sweep started run to their end (resolves the parked
"UNINIT-spawned activities never run" and the queued `2026-10-02-uninit-reply-at-termination.md`);
then the unloaders. `Interp::terminate` is `cfg(test)` now; only tests call it.

Parked item "nested pinned-loop depth guard": confirmed done, `run_round` measures the remaining
stack (`stack_exhausted`) on every nested round (P33, Task 5).

## Carried in from Task 8

- `post_timer` asserts one cancel flag per waiter (`# Panics` documented).
- `timer_post_reads_the_waiters_cancel.rex`: `remainder ran` relabelled `waiter result`; its
  `sourceline_oracle` expectation regenerated with the sanctioned driver (one line differs).

## Other changes

- `method_executable` (`environment/identities.rs`): a `.nil` scope (a method set with
  `setMethod ..., 'float'`) panicked in `class_graph.rs`; it now answers the existing loud refusal.
  Found through `TRACE_TraceObject.testGroup` `test_object_and_scope` (rc 101 on base). Queued:
  `2026-10-02-context-executable-setmethod.md` (the oracle answers the floating method).
- `group_runner::source_test_names` skips `::RESOURCE` bodies (it listed 3 of 10 tests of
  `TRACE_TraceObject`, stopping at the first resource's `::class test`).
- `concurrency_tests.rs` `the_outcome_table_of_the_trace_object_group` (gated): asserts the passing
  set (`TEST_SETMAKESTRING_WITH_METHOD_OBJECT`, `TEST_TRACEOBJECT_OPTION`,
  `TEST_TRACEOBJECT_OPTION_INVALID`); the others: `TEST_CALLER_STACK_FRAME`, `TEST_RECEIVER`,
  `TEST_TRACEOBJECT_COLLECTOR_AND_NOTIFY_CLASS_ATTRIBUTES` differ (the collector, below),
  `TEST_TRACEOBJECT_COLLECTOR`, `TEST_VARIABLE` refused at `DO COUNTER ... OVER`,
  `TEST_CALLER_STACK_FRAME_REPLY_START` refused at GUARD WHEN, `TEST_OBJECT_AND_SCOPE` the
  executable refusal above.
- Plan Task 11 Step 4 now names ATTRIBUTEPOOL.

## Rulings

- R-T9-1: deadlines go on every harness whose program set it does not write, and on the
  scheduler tests; the other in-process sites run fixed inline programs that start no activity and
  stay as they are (a blocked end needs a started activity; `grep -rn 'run_program(' --include=*.rs
  rust/crates` lists the sites) -- cost
  if wrong: a defect in one of those fixed programs hangs its test binary instead of failing it.
- R-T9-2: ATTRIBUTEPOOL reads 0 with the guard entries: the variable pool's number is keyed by
  (object, scope), Task 11's guard table key; written into Task 11's plan text -- cost if wrong: one
  wrong TraceObject value on method lines until Task 11.
- R-T9-3: VARIABLE, and TraceObjects off the routed path (collector, notify, options P/T/S/F on
  the default route), are trace-subsystem work, not activity fields: queued
  `2026-10-02-traceobject-variable-and-collector.md` -- cost if wrong: the TRACE_TraceObject
  collector tests stay red through Task 10's table.
- R-T9-4: EXECUTABLE in a frame table is `.nil` for a native or INTERPRET level and where
  `executable_at` refuses (StackFrame~EXECUTABLE is excluded in class-methods.txt) -- cost if wrong:
  `.nil` where the oracle answers an object, in those frames only.
- R-T9-5: activities an end-of-run UNINIT starts are run to their end (the oracle runs the u5
  continuation 29/30, racing process exit) -- cost if wrong: one more line than the oracle's 1-in-30.

## Witnesses

`corpus/lang/trace_object_activity_fields.rex` (THREAD, frame, guard entries of unguarded methods
with no GUARD, CALLERSTACKFRAME for a routine call, a `~start` and a REPLY continuation) and
`corpus/lang/uninit_after_every_activity.rex`, both in `phase-8.txt`, sourceline expectations
generated. Stability (stdout+stderr+rc hash): oracle 30/30 one hash each; ours unswitched 30/30 and
`REXX_SWITCH_MODE=every` 30/30, the oracle's hash; both pass `collect_stress`. stdout read: every
path the trace witness claims prints (spawner thread 1 on WORK and on the REP continuation, caller
line `reply 5`, `guard 0 0 0 W`). One adjustment while building it: a method whose `TRACE A`
follows `EXPOSE` gets no `>I>` here (queued `2026-10-02-trace-entry-after-expose.md`), so the
witness's REP sets `done` through its attribute.

Red before, base a4bde5677 built from `git archive` in its own target directory with the new tests
copied in:
- `a_reply_in_a_termination_uninit_runs_its_continuation`: left `main end\nuninit before reply\n`.
- `an_uninit_a_collection_readied_runs_when_its_activity_ends`: left `activity
  end\nmain after\nuninit\n`.
- `program_end`: both fail (self-wait: process ended, the refusal; timer: no output seen).
- trace witness on the base binary: 79 lines differ (the sink failed on a `.nil` STACKFRAME and the
  trace went to stdout). The UNINIT witness matches on base: it witnesses the verified ordering,
  which did not change.

## P28 checks (commands run, exit statuses read)

- `cargo fmt --all --check` 0; `cargo clippy -p rexx-exec --all-targets -- -D warnings` 0, with
  `--features pinning` 0 (`Checking rexx-exec` in both).
- `memcap 8G cargo test --release -p rexx-exec --lib`: 926 passed; debug: 926 passed.
- `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`: 742 of 742; with
  `REXX_CORPUS_SWITCH=every`: 742 of 742; debug under every: 742 of 742.
- `collect_stress` (release, gate): 36 passed; again 36 with the corpus (742 of 742) alongside.
- `concurrency_tests` release `--features pinning`, gate: 44 passed (`measured::`, `group_runs`).
- `gate_table_c` gate: 22 passed, finished in 16.7 s; `gate_table_d` 23; `method_bodies` gate 23,
  "regressions this run: 0. other drift from the committed table: 0"; `refusal_sites` 5 (the
  committed `refusal-sites.tsv` re-derives unchanged); `native_entries` 24, `closed_phases` 5,
  `owners` 6, `loud` 9, `program_end` 2 (release and debug); `rexx-parse --test
  sourceline_oracle` 1.
- Bench "it works": every `rust/bench-programs/*.rex` on the release binary against the base's:
  stdout identical except `heapshape`'s two timing figures; `extcall` rc 158 on both (needs its
  library's environment).

## Concerns

- R-T9-1 leaves fixed-program harnesses without a deadline; the lead may want them too.
- The collector/option path (R-T9-3) is what most of TRACE_TraceObject's thread tests read, so
  Task 10's table will show them differing for that, not for THREAD.
- `.superpowers/sdd/queued/2026-10-02-uninit-reply-at-termination.md` is resolved by this task and
  left in place.
