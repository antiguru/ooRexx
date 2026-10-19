# Task 6.10 report: trace

Clusters `traceobject-probe-option`, `interactive-trace-setting`, `trace-label-before-procedure`,
`trace-reply-continuation`, `traceoutput-invalid-destination` and `prolog-trace-lines`. Every
cluster test passes except TraceObject `TEST_CALLER_STACK_FRAME_REPLY_START`,
`TEST_OBJECT_AND_SCOPE` and `TEST_TRACEOBJECT_COLLECTOR`, which now fail only on package names,
Task 6.11's `dynamic-package-name` (Concern 1). In the whole suite only the passing tests' rows
change. Status: DONE_WITH_CONCERNS.

## Commits

* `c450b7759` PROCEDURE after a traced label, TRACE in debug, `Package~trace`, prologue and
  install-stub entry lines.
* `f903d1783` a failed trace `LINEOUT` raises in the traced clause.
* `c239cd23f` every trace line is a TraceObject: option `P`, NOTIFY and VARIABLE.
* `e5f54ba1d` `<I<` traced before the call tail pops, EXPOSE keeps the entry trace allowed, an
  empty REPLY rest is spawned.
* `99d36ca61` no pinned yield while a trace line is delivered.
* `f7eacd9b3` the witnesses but the format one, their `sourceline_oracle/` files, `phase-9.txt` lines, and the
  REPLY witness's seeded-gate row and oracle set.
* `306331c9b` options `T`, `S` and `F` format the default trace route.
* `fcb358fd7` the format witness, records, `suite-9d-set.txt`, the closed queued items.
* `7540e967f` a trace line whose TraceObject cannot be made on the default route goes to standard
  error.
* `a5266a662` a stale chunk's failed echo raises at its clause's end (the perf fix, see Perf), and
  the invalid-destination witness follows it.
* The commit after `a5266a662` holds the final records and this report.

## What changed, per cluster

### trace-label-before-procedure

`Activity::procedure_permitted` and `region_procedure_permitted` are the activity's. A trace line
delivered to a Rexx destination runs that destination's clauses, which spent the permission
between `p:` and `procedure`, so `PROCEDURE` was 17.1. `deliver_trace_line_as`
(`environment/route.rs`) saves both and puts them back after the delivery.

### interactive-trace-setting

* `Package~trace` answers `TraceSetting::toString`, with the `?` of a debug setting
  (`dispatch/package.rs`).
* In interactive debug (`inDebug()`, debug on and no pause running) a `TRACE` instruction leaves
  the setting alone and calls no `>I>`: the bare form, a letter, and `TRACE VALUE` that is not a
  number, an invalid one included (it raised 24.1 here). `ignores_trace_instruction`
  (`run/settings.rs`). A numeric `TRACE` is still a skip count, as in `TraceInstruction.cpp`.

### prolog-trace-lines

* A prologue (`Package~new`, `::REQUIRES`) is an `EXTERNALCALL` activation, announced with
  `>I>`/`<I<` under its package's `::OPTIONS TRACE`, an empty one included
  (`invocation_subject` takes `CallType::Requires`; `run_main` traces entry and exit for it).
* `PackageClass::install` installs a package with classes or requires by calling an empty routine,
  which `.Routine~new`, `.Method~new` and their `newFile` forms reach through
  `LanguageParser::installPackage`. `install_executable` (`install.rs`) now runs that empty routine
  (`run_install_stub`, `lib.rs`) after the directives install, when the package has a `::CLASS` or
  `::REQUIRES`. This gives `TEST_OPTIONS_TRACE_ALL_WITH_FLOATING_METHOD` its leading pair of lines.
* The queued item `2026-10-10-prologue-frames-missing-loading-levels.md`: both its probes are now
  identical to the oracle.

### traceoutput-invalid-destination

`Activity::traceOutput` lets a failed `LINEOUT` raise in the traced activation. The delivery
returns `()` everywhere it is called, so the failure is queued as a `PendingTrap` with a new
`trace_failure` field (`queue_trace_failure`). `deliver_pending_traps` raises it at the clause's
boundary, and while it is queued that activation's further lines are dropped. Where the clause's
echo came from the chunk's own `Op::TraceClause`, that op raises it at once, so the clause does not
run. On a stale chunk the clause still runs first (Concern 2). With no running activation the line
is written out, as before.

### traceobject-probe-option

* `RexxActivation::processTraceInfo` makes a TraceObject for every line. `deliver_trace_line_as`
  now does too, on every route, then:
  * appends it to `.TraceObject~notify` where set;
  * keeps it back under the option `P` the object was made with;
  * otherwise sends `LINEOUT` to a destination of the program's own, or writes to standard error.
    There it writes the line under option `N`, and the object's `makeString` under `T`, `S` and
    `F`.
* Without the TraceObject class (the bootstrap), the line goes to standard error as before. A
  failure before the line reached a program's own destination, such as making the object with the
  stack exhausted, also leaves the line to standard error.
* `>V>` and `>=>` lines carry `VARIABLE`, a StringTable of NAME, VALUE (the object) and
  ASSIGNMENT. A compound has `.nil` for NAME and VALUE, as `traceCompoundValue` passes none.
  `trace_variable` and `trace_assignment` take the value object, and every call site passes it.
* Making an object for every line exposed these defects, which a Rexx destination already had:
  * **`NEW` lost its result under a traced `INIT`.** `finish_call` popped the call tail before
    `<I<`. The delivery's own primitive `NEW` then parked its tail at the depth of the outer `NEW`'s
    tail, and `finish_native_tails` drained both. `<I<` is now traced before the pop
    (`run/call.rs`). This closes the queued item `2026-10-02-traced-init-routed-sink-silent-exit.md`.
  * **EXPOSE spent the entry trace.** `RexxActivation::run` keeps `traceEntryAllowed` past an
    EXPOSE, so `expose v` then `trace a` announces `>I>`. `exec_expose` puts the state back to
    pending. This closes `2026-10-02-trace-entry-after-expose.md`.
  * **A pinned yield during a delivery.** Two activities delivering at once met on the
    TraceObject class's guard under a pin: "a pinned wait for a guard that only an activity pinned
    below it can end". `serve_requests` no longer takes a pinned yield while a line is routed.
    The deferred slice is taken at the next boundary.
* Closed queued items: `2026-10-02-traceobject-variable-and-collector.md`, and
  `2026-10-02-triggered-exit-trace-line-unrouted.md`, whose probe is now identical.

### trace-reply-continuation

The oracle spawns the rest of a replied method even when its body ends at the `REPLY`. That rest
announces `>I>` and `<I<` again on its thread (`RexxActivation::run`'s `REPLIED` arm). Here no
split ran, so no continuation existed. `finish_call` now files an empty continuation while the
replier is still the running level (`file_empty_continuation`, `ir/drive.rs`). The scheduler's
resume path ends a rest with no level at once through `finish_send`. This closes
`2026-10-10-reply-continuation-trace-entry.md`.

## Witnesses, red then green

Red is the parent `d2e55befa`. Green is the final code, `a5266a662`. `t6.10/witnesses/run.sh` runs
each from a fresh directory and writes `summary.tsv`. Every witness is in `phase-9.txt` with its
`sourceline_oracle/` file.

| program | red vs oracle | green vs oracle |
|---|---|---|
| `trace_procedure_after_label` | stdout rc differ | identical |
| `trace_interactive_ignored` | stdout stderr rc differ | identical |
| `trace_prologue_entry` | stdout differs | identical |
| `trace_output_invalid_destination` | stdout differs | identical |
| `trace_object_probe_collect` | stdout stderr differ | identical |
| `trace_reply_continuation` | stdout differs | identical |
| `trace_object_format_default_route` | stdout stderr differ | identical |

What each green stdout prints (read):
* `trace_procedure_after_label`: the `trace i` transcript of an internal call to `p: procedure
  expose a` and of a method's `q: procedure`, collected by a Rexx destination; `result 8`.
* `trace_interactive_ignored`: `?C` after `trace ?c` through bare TRACE, `trace off`, an invalid
  value, `trace value 'R'` and `trace ?`; `N` after the builtin; `?E O` from `trace('O')`; `?C
  COMMANDS`, `?E ERROR`, `C COMMANDS` from `Package~trace` and `options('TRACE')`.
* `trace_prologue_entry`: `>I>`/`<I<` (names stripped) for a `Package~new` prologue with code, a
  class-only package, a `.Routine~new` and a `.Method~new` with a class, the calls; nothing for
  `TRACE N` and for an untraced class package.
* `trace_output_invalid_destination`: 97.1 trapped by the caller for each of
  `trace r` then the destination, the destination then an assignment then `trace r`, a method with
  EXPOSE, and a body compiled under `TRACE R` whose `say` does not run. Then `done`.
* `trace_object_probe_collect`: the collected tracelines with each object identical to the one
  NOTIFY got, and the VARIABLE entries of simple, compound, stem, parse, `INIT`'s `use arg` and
  the loop variable.
* `trace_reply_continuation`: the entry and exit pairs of a forwarded REPLY and its empty rest, of
  `REPLY 42` before a continuing FORWARD, and of a REPLY with a rest; `three 42`, `count 0`.
* `trace_object_format_default_route`: on stderr, the `T`, `S` and `F` formats and the plain line
  for `N`; a method's `F` line with its guard fields; stdout `1`.

`trace_reply_continuation` is concurrent (`t6.10/witnesses/stability.txt`):
* the oracle gave one outcome over 8 runs;
* the task gave the same outcome over 8 wall-clock runs, and under 12 simulation schedules
  (`sim:1`, `sim:7`, `sim:42`, `uniform:0.01/0.2/0.5/1`, `pct:3` at k 40 and 200, `pre:1/2/3`).

It is wired into the seeded gate: a `program` row in `corpus/sim-gate.tsv` with k 65 from a seed-0
fifo run, and `corpus/sim-oracle/corpus__lang__trace_reply_continuation.rex.program.tsv` written
under `REXX_SIM_ORACLE_REFRESH=1` (5 runs, one outcome). The row's 70 seeds all agree with it
(`t6.10/sim/trace_reply_continuation.report.txt`). Debug seeds are 0, since no debug build was made.

Before the yield change, preemptive schedules lost the empty rest's lines to the pinned guard wait
above. That is how the simulation mode found it.

## The tests

These leave `suite-9d-set.txt`, and each passes in the whole-suite run:
* `base/directives/OPTIONS` `TEST_OPTIONS_TRACE_ALL_WITH_FLOATING_METHOD`, `TEST_OPTION_SETTINGS`;
* `base/keyword/TRACE` `TEST_TRACE_IGNORED`, `TEST_TRACE_LABEL_WITH_FORWARD`, `TEST_TRACE_PROCEDURE`;
* `base/keyword/TRACE_TraceObject` `TEST_CALLER_STACK_FRAME`, `TEST_VARIABLE`;
* `base/runtime.objects/environmentEntries TEST_ZZZ_TRACEOUTPUT_INVALID`.

These stay in the set: TraceObject `TEST_CALLER_STACK_FRAME_REPLY_START`, `TEST_OBJECT_AND_SCOPE`
and `TEST_TRACEOBJECT_COLLECTOR` (Concern 1).

Queued items closed (first line now reads CLOSED; probes and outputs in `t6.10/queued/`):
* `2026-10-02-traceobject-variable-and-collector.md`
* `2026-10-10-prologue-frames-missing-loading-levels.md`
* `2026-10-10-reply-continuation-trace-entry.md`
* `2026-10-02-trace-entry-after-expose.md`
* `2026-10-02-traced-init-routed-sink-silent-exit.md`
* `2026-10-02-triggered-exit-trace-line-unrouted.md`

Of these, `trace-entry-after-expose`, `traced-init-routed-sink-silent-exit` and
`triggered-exit-trace-line-unrouted` were 9d items with no cluster, closed as a side effect. No
exclusion row is touched.

## Changed rows

`t6.10/suite/changed-rows.txt` (its first line is the command) diffs against
`t6.9/r1/suite/tests.tsv`. Exactly the tests above change, `passed failed` to `passed
passed`. Suite: 402 groups, 12 red (13 at 6.9 r1). An earlier whole-suite run at `7540e967f` also
showed the known `SysFileTree TEST_TREE_DIRECTORY` flake, which passed alone. The final run at
`a5266a662` did not show it.

## Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t610-target`, `CARGO_INCREMENTAL=0`, `-j 4`, each
exit status read unpiped, on `a5266a662`:
* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test --release -p rexx-exec --lib`: exit 0, 1152 passed. At `306331c9b` it was
  exit 101: `nested_pinned_waits_are_bounded_by_the_stack_remaining` saw the 11.1 report on stdout,
  because the TraceObject could not be made with the stack exhausted. `7540e967f` fixes that.
* `memcap 16G cargo test --release -p rexx-exec --test refusal_sites --test owners --test
  closed_phases --test phase_ten_reasons --test representation_refusals --test
  refusal_dispositions --test coverage --test rexxc --test licensed_divergences --test
  stream_handle --test loud`: exit 0. `refusal-sites.tsv` is unchanged.
* `memcap 16G cargo test --release -p rexx-parse`: exit 0 (includes `sourceline_oracle`).
* `memcap 16G cargo test --release -p rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -p rexx-exec --test corpus`: exit 0, 1110 of
  1110 matching.
* `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/t6.10/suite/table.tsv
  memcap 8G cargo test --release -p rexx-exec --test suite l3_core_suite`: exit 101 as expected,
  402 groups, 12 red, 252.8 s.
* The seeded gate (`REXX_CORPUS_GATE=1 REXX_SIM_ONLY=<row> ... --test concurrency_tests
  the_seeded_gate`) on the new row and on these trace-related rows: `trace_object_activity_fields`,
  `trace_object_native_caller`, `trace_object_guard_fields`, `guard_when_traces_each_evaluation`,
  `guard_when_trace_object_waiting`, `reply_continuation_error_trace`, `trace_debug_ignores_trace`,
  `call_trace_nested`. Each was exit 0, with all 70 seeds agreeing with the oracle set
  (`t6.10/sim/`).
  * The TRACE and TRACE_TraceObject group rows differ from their oracle sets on all 70 seeds, on the
    parent too (`t6.10/sim/parent-*.report.txt`).
  * Here every seed fails exactly the tests the suite still fails, where the parent failed more:
    both parent whole rows ended in an `error` outcome, failing the clusters' tests too.

## Perf

`t6.10/callgrind.txt`: pin, r0 = `7936f3ca1` (`cg-9d/t6.9-r1/rexx-head`) and head = `a5266a662`
(`cg-9d/t6.10/rexx-head`) in one run, 3 rounds, exit 0. The ledger's previous figures are
`t6.9/r1/callgrind.txt`'s head column.

| program | head vs pin | previous vs pin | own delta (ledger) | vs r0, same run |
|---|---|---|---|---|
| startup | +0.0452 | +0.0456 | -0.0004 | -0.0004 |
| interpret | -2.1710 | -2.1500 | -0.0210 | -0.0210 |
| parse | -0.1221 | -0.0858 | -0.0363 | -0.0363 |
| churn | +0.1150 | +0.1150 | 0.0000 | 0.0000 |
| sayloop | +0.0487 | +0.0489 | -0.0002 | -0.0003 |

Every own delta is under +0.05%, and every figure against the pin is under +0.5%.

The first full run at `7540e967f` was OVER: sayloop +0.60% and parse +0.48% against the pin, about
ten instructions a loop pass. Callgrind bisection is in `t6.10/perf-bisect/`:
* `c450b7759` was neutral;
* `f903d1783` carried all of it;
* removing the compiled-echo check alone changed nothing;
* removing the stale-chunk abandon removed it. Moving that abandon out of line did not, and
  neither did passing the op counter by reference: any write to the region's start op from the
  clause entry costs the whole amount.

So `a5266a662` raises a stale chunk's failed echo at the clause's end (Concern 2). Trace
construction and every other new path run only while a line is traced.

## Concerns

1. **TraceObject tests wait on `dynamic-package-name` (Task 6.11).**
   `TEST_CALLER_STACK_FRAME_REPLY_START`, `TEST_OBJECT_AND_SCOPE` and `TEST_TRACEOBJECT_COLLECTOR`
   compare tracelines that carry package names, and here a `.Routine~new` or `.Package~new` package
   is named after the creating file. The install stub's routine name is the package's name, so it
   inherits the same defect. Each test's program, run standalone, is identical to the oracle once
   package and routine names are masked (`t6.10/probes/name-only/`). They stay in
   `suite-9d-set.txt`, and should pass when 6.11 lands.
2. **A stale chunk's failed echo runs its clause first.** When the echo of a clause in a chunk
   compiled under another trace setting fails to deliver, the clause's own work runs before the
   97.1. The oracle runs none of it (`t6.10/probes/stale-clause-runs/`: `say 'say ran'` prints
   here). Doing it at the clause's entry costs about 0.5% on every clause (Perf). A chunk compiled
   under the tracing setting abandons the clause correctly. Proposed as a queued item, or a ruling.
3. **An untrapped failed trace delivery is not copied.** With no SYNTAX trap anywhere,
   `raisePropagate` reaches the base native frame, and `traceOutput`'s `catch (NativeActivation *)`
   writes the line and returns. By then `Activity::raiseException` has popped the traced
   activation, which keeps running. The oracle then prints every later line to stderr and
   segfaults at exit (`t6.10/probes/untrapped-trace-failure/`, oracle rc 139). This crate reports
   the untrapped 97.1, rc 159. This is an oracle defect and is not copied or witnessed. It needs a
   ruling only if the controller sees it as a Deviation.
4. **NOPROLOG packages with a class miss the install stub's lines.** `.Package~new` and
   `::REQUIRES` of a NOPROLOG package with classes or requires call the empty install routine on
   the oracle (`runProlog`, then `install()`). Here the package's main body is not empty, and the
   stub needs a body selector of its own (`t6.10/probes/noprolog-install/`, the missing pair). No
   test of the set reaches it. Proposed as a queued item.
5. **Files outside the clusters' list:** `environment/route.rs`, `run/settings.rs`,
   `dispatch/package.rs`, `lib.rs`, `install.rs`, `run.rs`, `ir/drive.rs`, `run/call.rs`,
   `dispatch.rs`, `scheduler.rs`, `run/condition.rs`, `activity.rs`, `eval.rs`,
   `parse_template.rs`, `run/loops.rs`, `command.rs` and `dispatch/library.rs` (these only for the
   new `PendingTrap` field). Each change is the one the measured cause needed, and the suite and
   corpus show no other changed result.
6. **Schedule and cost.** A trace line now runs `TraceObject~new` and its puts on every route, as
   the oracle does. Tracing is slower, and `.TraceObject~counter` now counts every line, where it
   counted only routed ones. A slice that falls due during a delivery is taken at the next clause
   boundary rather than by a pinned yield. That is a legal schedule, but a scheduling decision the
   controller may want to confirm.
7. A Rexx destination whose own methods are traced (its class in a traced package) has those lines
   written to stderr here. The oracle's run of that shape segfaulted, so it was not witnessed.
