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

## Fix round 1

Status: **BLOCKED on Important 1.** Importants 2 and 3, the Minor and the oracle-crashes entry are
done. Code at `952e1788d`. Records are under `t6.10/r1/`, and `t6.10/witnesses/` is refreshed.

Commits since `4137247f5`:
* `990cc347d`: the call path is back to the parent's shape (Important 3). The empty REPLY rest is
  filed at release (`Interp::empty_continuation`). The delivery's placeholder call tail keeps the
  native-tail depths apart.
* `727a2190d`: the entry trace after EXPOSE is decided at the TRACE instruction, off the EXPOSE
  path (Important 3).
* `c2cd1b912` .. `3e5c927b3`: the Important 1 attempt. Reverted by `68fd2cebc`, and kept on
  `wip/t610-r1-fallible-trace`.
* `c415009bd`, reverted by `20631fc37`: the Minor's stderr change. The oracle writes stdout (below).
* `ec322fea1`, `baa497e53`: `oracle-crashes.txt` entry 51.
* `85b3ddd06`, `952e1788d`: the yield predicate (Important 2), with the two witnesses and their
  seeded-gate rows.

### Important 1: BLOCKED, no zero-cost fix found

A clause whose trace echo fails still runs its work in the stale-chunk shapes and in `>>>` lines.
`t6.10/r1/important-1/probes/` has the review's shapes and the variants I added (`s1`..`s6`, `m4`,
`m7`), each with the oracle's output and this crate's at `952e1788d`. Every one differs. For
example, `s1` prints `syntax 97.1 c=1` here and `c=0` on the oracle.

Options measured (callgrind, 1 round, against `t610/rexx-parent`, the task's parent `d2e55befa`;
tables in `t6.10/r1/important-1/options/`):

| option | nop | assign | rexxcps | others | review shapes |
|---|---|---|---|---|---|
| Result-returning trace path, `enter_stepped_clause` fallible (`cg-r1c`) | +5.41 | +6.51 | +0.76 | emptyloop +1.61, fibcall +0.83, sayloop +0.50, dispatch +0.41 | fixed |
| Result-returning trace path plus a `TraceStale` op (`cg-r1d`) | +3.22 | +5.95 | +1.82 | fibcall +0.55, emptyloop +0.32 | fixed |
| Kept failures, one shared exit of the region, no `TraceStale` (`cg-e21`) | 0.0000 | 0.0000 | - | - | not fixed: a clause that does not echo under the chunk's setting never looks |
| E21 plus `TraceStale` (`cg-s1`, `3e5c927b3`) | 0.0000 | +0.54 | +1.73 | fibcall +0.35, sayloop +0.17, interpret +0.10, dispatch +0.09 | fixed (all shapes match once package names are masked) |
| `$stale`-guarded check at clause entry (`cg-e22`) | +1.08 | +2.17 | +0.24 | - | fixed |
| Raise at the clause's end (round 0, as committed) | 0 | 0 | 0 | 0 | not fixed |

Not built: at a TRACE site, swapping to a chunk compiled for the new setting. That needs a pc map
between the two chunks and loop frames that survive the swap.

rexxcps pays most under `TraceStale` because its `trace value tracevar` retraces the body, so
every clause there carries the op. `nop` and `assign` move by whole percent with any added exit
from the driver's region. In `cg-e21`, one shared labeled exit for all trace ops brings that to
zero. The cost of S1 is the op itself.

The controller takes this to Moritz. The candidates are S1 at +0.54% assign and +1.73% rexxcps, or
the round 0 behaviour as a ruled divergence.

### Important 2: done

The pinned yield is suppressed while a trace line's `TraceObject` is made, and while an activation
the destination runs holds its receiver's guard (`Interp::delivery_holds_a_guard`, with
`Activity::delivery_floor` set when `LINEOUT` is sent). Otherwise the destination's clauses yield.

The guard part goes beyond the ruling's wording, and it is needed. `990cc347d` checked only the
running activation. There, on some seeds, `trace_reply_continuation` lost its empty rest's
`>I>`/`<I<` with rc 0. The continuation's delivery had failed with "a pinned wait for a guard that
only an activity pinned below it can end is not implemented" (seen with a print in a scratch
build). That refusal was queued as a trace failure on an activation that ran no more clauses
(Concern 9). Checking every activation above the delivery's floor removes it on those seeds.

Witnesses, both `phase-9.txt` lines with `sourceline_oracle` files, seeded-gate rows (k from a
seed-0 fifo run) and oracle sets written under `REXX_SIM_ORACLE_REFRESH=1`:
* `trace_destination_waits_on_thread` (the review's st1): the destination's unguarded `LINEOUT`
  spins until a worker sets a flag.
* `trace_destinations_two_threads` (the review's two1): two started activities trace into a
  destination that bumps a guarded class counter.

`t6.10/witnesses/stability.txt` (at `952e1788d`): each gives the oracle's one outcome over 8 runs,
and the task's same outcome over 8 wall-clock runs and under every listed `REXX_SWITCH_MODE` seed and
policy. Each seeded-gate row agrees on all its seeds (`t6.10/r1/sim/`). The pre-round binary
`f7eacd9b3` hangs on st1 under `sim:3,uniform:0.2` (killed, rc 137). The parent `d2e55befa` passes
both, so these guard round 0's change, not a parent defect (`witnesses/summary.tsv`: red identical).

### Important 3: done

`t6.10/r1/callgrind.txt`: pin, r0 = `7936f3ca1` (`cg-9d/t6.9-r1/rexx-head`) and head =
`952e1788d` in one run, 3 rounds, exit 0, every spread at most 0.0002%.

| program | head vs pin | r0 vs pin | own delta |
|---|---|---|---|
| startup | +0.0449 | +0.0456 | -0.0007 |
| interpret | -2.1745 | -2.1500 | -0.0245 |
| parse | -0.1221 | -0.0858 | -0.0363 |
| churn | +0.1150 | +0.1150 | 0.0000 |
| sayloop | +0.0484 | +0.0490 | -0.0006 |
| nop | -0.0206 | -0.0206 | 0.0000 |
| assign | -0.0103 | -0.0103 | 0.0000 |
| emptyloop | -0.6399 | -0.6399 | 0.0000 |
| rexxcps | +0.4669 | +0.4571 | +0.0098 |
| sendloop | -0.2049 | -0.2049 | 0.0000 |
| dispatch | -0.3162 | -0.3162 | 0.0000 |
| dispatchclass | -3.4916 | -3.4680 | -0.0236 |
| fibcall | -3.2600 | -3.2590 | -0.0010 |
| intcall | -10.1727 | -10.1368 | -0.0359 |
| extcall | -13.2675 | -13.2675 | 0.0000 |

Every own delta is under +0.05%, and every figure against the pin is under +0.5%.

### Minor: the error report's stream

The oracle writes these lines on stdout. `Activity::traceOutput` catches the destination's failure
and falls back to `lineOut`, which is a printf to stdout (`corpus/oracle-crashes.txt` entry 11c).
This crate already leaves them on stdout under entry 11's licence. The stderr change was reverted,
and `queue_trace_failure`'s doc comment now says stdout.

### oracle-crashes.txt

Entry 51 records the untrapped failed delivery. The program is `call r` into a routine that sets
`.traceOutput~destination("")` and runs `trace r`. The oracle exits with SIGSEGV, rc 139, after
writing the trace lines, `routine end` and `not reached` to stdout. Here the result is 97.1 with
rc 159, report lines on stdout under entry 11.

### Checks at `952e1788d`

From `rust/`, `CARGO_INCREMENTAL=0`, `-j 4`, each status read unpiped:
* fmt, clippy (`-p rexx-exec --all-targets -D warnings`), `--lib` (1152 passed), the refusal-table
  tests, `rexx-parse`, and `unsafe_sites`: exit 0 each.
* Corpus gate: exit 0, 1112 of 1112 matching.
* Whole suite: exit 101 as expected, 402 groups, 12 red. `t6.10/r1/suite/changed-rows.txt` is empty
  against round 0's `tests.tsv`. `changed-rows-vs-t6.9-r1.txt` shows the same rows as round 0's
  list.
* Seeded gate (`t6.10/r1/sim/`, `summary.txt`): every row exits 0.
  * These rows agree with their oracle sets on every seed: the new witnesses,
    `trace_object_activity_fields`, `trace_object_native_caller`, `trace_object_guard_fields`,
    `guard_when_traces_each_evaluation`, `guard_when_trace_object_waiting`,
    `reply_continuation_error_trace`, `trace_debug_ignores_trace` and `call_trace_nested`.
  * `REPLY` whole and derived pass on every seed.
  * `trace_reply_continuation` and `TRACE` whole differ on some seeds (Concern 8).

### Concerns (round 1)

8. **Yielding deliveries reorder concurrent trace lines in the simulation mode.** Once a
   destination's clauses yield, a delivery is no longer atomic. A REPLY continuation's lines can
   then land between the replier's `<I<` delivery and its `LINEOUT`.
   * `trace_reply_continuation` differs from the oracle set on some seeds, every one by the same
     reorder: `<I< ONE` after the continuation's `>I> ONETWO`.
   * `TRACE` whole fails `TEST_TRACE_LABEL_WITH_FORWARD` on some seeds, where round 0 failed only
     `TEST_TRACE_EXIT` and `TEST_TRACE_OTHER_ENTRYPOINT`. Its `test_forwarded3` expects the same
     order, and the destination is the test's `ArrayStream`, whose `lineOut` exposes nothing.
     A standalone copy reproduces the reorder under `sim:1,uniform:1` (`t6.10/r1/forwarded3-reorder/`).
   * Wall clock, seed 0 fifo and the whole suite agree with the oracle. I have not shown whether the
     oracle can take this order. Its 8 runs never did.
   * This is the ruled behaviour of Important 2. The witness's expected output now depends on the
     schedule. Its sim rows differ without being red.
9. **A refusal inside a delivery can vanish.** `deliver_trace_line_as` queues any failure,
   including a `Loud` refusal, as a trace failure on the running activation. It is raised at that
   activation's next clause end, and an activation that runs no further clause drops it: the
   empty REPLY rest above, or a `<I<` line. `952e1788d` removes the one way I found to reach this.
   The drop itself stays, and it is part of Important 1's open design.
10. **A guarded destination that waits on another thread hangs.** If the destination's `LINEOUT`
    exposes a variable and then spins on a flag a worker sets, the oracle prints `done 1`. Here
    the run is killed at 10 s, on wall clock and under `sim:3,uniform:0.2`
    (`t6.10/r1/guarded-waiting-destination/`). The method holds its guard, so it takes no pinned
    yield. This is round 0's behaviour, now only for destinations that hold a guard.
11. **Rule slips.** Once I ran a bare `git checkout` (it only printed status; nothing changed), and
    once `git add -A .` in `rust/crates/rexx-exec/src` (only tracked, intended files were changed).
    A failed `cd` also ran one `sed` in the worktree instead of a scratch copy. It changed one line
    of `scheduler.rs`, and I restored it before any commit (`git diff` checked).

Round 0's Concern 6 (no pinned yield during a delivery) is superseded by Important 2.

### Fix round 1, after the Deviation 50 ruling

Moritz ruled Important 1 as Deviation 50 (`4a5e7083d`). The round-0 behaviour stays, and the
chunk swap is queued.

1. **Deviation 50's witness** (`894c0d372`): `undelivered-trace-clause-runs-first` in
   `tests/licensed_divergences.rs`, with its marker line in Deviation 50's row. It is probe `s1`
   with the destination set before `trace r`, so neither side's stderr names a path. The oracle
   prints `syntax 97.1 c=0` (3 runs) and this crate `c=1`, both rc 0 with empty stderr. Changing
   the crate's expected line turns the test red. `licensed_divergences`, `closed_phases`,
   `builtin_status`, `signals`, `api_group_tests`, `keyword_assertions`, `rexxc`,
   `bif_assertions`, `owners` and `coverage` (the tests that read `phase-4-exclusions.txt`) exit
   0.
2. **A refusal a delivery meets is never dropped** (`9a3431a0b`). A refusal queued for the
   running activation is raised at that activation's clause end as before.
   * Queuing one also allocates the activation's cold state, so its pop runs `Interp::end_cold`,
     and `Interp::carry_trace_refusals` moves the refusal from there. With a caller, the caller
     raises it at its own clause end, the way the 97.1 is raised.
   * With no caller in the activity (a started activity's own method, an empty REPLY rest, a
     main body), the refusal goes to `Activity::trace_refusal`. The activity ends with it: the
     scheduler's end of a started or continued activity and the main body's end check that
     slot.
   * The ordinary call path is unchanged. Only a refusal reaches the cold state, and the main
     body's check sits behind `!nested`.
   * Witnesses (`environment/tests.rs`, `t6.10/r1/delivery-refusal/`): a routine's `<I<`, a
     started method's `<I<`, and an empty REPLY rest's `>I>`. The destination's `n`th `LINEOUT`
     sends `ITEMS` to a Stem built without its allocation, which is refused at rc 120. At
     `952e1788d` each printed `after` at rc 0. Now each ends at rc 120 with the refusal on stderr.
     Without the `carry_trace_refusals` call all three tests go red.
   * There is no oracle half: the oracle segfaults on that Stem shape (`oracle-crashes.txt`).
3. **The reorder of Concern 8 was this crate's alone, and is fixed** (`9a3431a0b`). The oracle has
   two ways to hand the kernel to another thread during a delivery:
   * The run loop's check, which runs only after `yieldInstructions` (50) instructions in one
     activation. `instructionCount` is a `RexxActivation` member, zeroed by `clearObject`
     (`RexxActivation.cpp:616`, `RexxActivation.hpp:591`).
   * A native method call, whose `releaseAccess` (dispatch defaults to true) hands the lock to a
     waiting thread (`NativeActivation.cpp:1304`, `Activity.cpp:2204`, `ActivityManager.cpp:782`).

   The TraceObject is a C++ `StringTable` and needs no clauses.
   * `trace_reply_continuation`'s destination SAYs. That goes to `.output`'s `SAY`, which is the
     native `stream_lineout`, so its reorder is one the oracle's scheduler can produce.
   * `test_forwarded3`'s `ArrayStream` `lineOut` runs three clauses and no native method. Nothing
     in the replier's path between its `<I< ONETWO` and `<I< ONE` releases the kernel either. So
     the order this crate printed there (`t6.10/r1/forwarded3-reorder/`, at `952e1788d`) is one
     the oracle cannot produce.

   Fix: inside a delivery, the pinned yield also requires the running activation to have run
   more than 50 clauses in a row while the slice waits (`Activity::delivery_streak`,
   `DELIVERY_YIELD_CLAUSES`). The count starts when the slice falls due, so this switches no
   more often than the oracle could. Everything is on the deferred-slice path.
   * `tests/trace_delivery_schedule.rs` runs `test_forwarded3` under 60 sim modes and requires
     the oracle's output, which is one outcome over 8 runs. With the threshold at 0 it goes red,
     and it failed on 92 of 120 runs in a scratch sweep.
   * st1 and two1 keep their oracle outcome under every listed mode.

   Since this crate now switches less inside deliveries, `trace_reply_continuation` also agrees on
   all its seeds. Concern 8 is closed.

Checks at `9a3431a0b`:
* fmt, clippy, `--lib` (1155 passed), the refusal tables, `rexx-parse`, `unsafe_sites`,
  `licensed_divergences`, `sim_processes` and `trace_delivery_schedule`: exit 0 each.
* Corpus gate: 1112 of 1112.
* Whole suite: 402 groups, 12 red. `changed-rows.txt` against round 0 is empty.
* Seeded gate (`t6.10/r1/sim/summary.txt`): every row exits 0.
  * Every corpus row agrees with its oracle set on every seed.
  * `TRACE` whole fails only `TEST_TRACE_EXIT` and `TEST_TRACE_OTHER_ENTRYPOINT` on every seed,
    as in round 0.
  * `REPLY` passes on every seed.
* `t6.10/witnesses/stability.txt` and `summary.tsv` are refreshed at `9a3431a0b`. Every run of
  the three concurrent witnesses gives the oracle's outcome.
* Concern 10 stands: `t6.10/r1/guarded-waiting-destination/` is still killed at `9a3431a0b`.
* Perf: `t6.10/r1/callgrind.txt` now holds the run at `9a3431a0b` (pin, r0 = `7936f3ca1`, head; 3
  rounds; exit 0; every spread at most 0.0023%). The Important 3 table above was measured at
  `952e1788d`. Own deltas against r0 at `9a3431a0b`:

  | startup | interpret | parse | churn | sayloop | nop | assign | emptyloop |
  |---|---|---|---|---|---|---|---|
  | -0.0053 | -0.0225 | -0.0367 | -0.0009 | -0.0043 | -0.0001 | 0.0000 | 0.0000 |

  | rexxcps | sendloop | dispatch | dispatchclass | fibcall | intcall | extcall |
  |---|---|---|---|---|---|---|
  | +0.0136 | -0.0001 | 0.0000 | -0.0236 | -0.0011 | -0.0367 | -0.0001 |

  Every own delta is under +0.05%, and every figure against the pin is under +0.5% (rexxcps
  +0.4707%).
