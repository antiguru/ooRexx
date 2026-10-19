# Task 6.11 report: classes, methods and packages

Clusters `dynamic-package-name`, `method-object-identity`, `setmethod-instancemethod-kind`,
`method-package-prolog`, `class-activate-order`, `uninit-define-on-superclass` and
`native-stack-frames`. Every cluster test passes, and so do the three TraceObject tests Task 6.10
left on package names. In the whole suite only those rows change. Status: DONE_WITH_CONCERNS
(Concern 1, the raw glibc figures of the perf check).

## Commits

* `58902b630` the fixes of every cluster.
* `3d5768bf7` the witnesses, their `sourceline_oracle/` files and `phase-9.txt` lines,
  `suite-9d-set.txt`, the first records.
* `a0be77243` `checkUninit` on a cascade only when the change touches `UNINIT` (the unconditional
  call cost about 300k Ir at startup over the library's defines).
* `a22a686d0` the install order resolved in place, with no allocation beyond its own list.
* The commit after `a22a686d0` holds the final records and this report.

## What changed, per cluster

### dynamic-package-name

`>I>`/`<I<` lines named a package by `Interp::package_path`, which knows only files. They now use
`program_display_name` (`run/settings.rs`), the name `Package~name` and `PARSE SOURCE` already
answer, so a `.Routine~new`, `.Method~new` or `.Package~new` body, a prologue of `.Package~new`,
and a routine that package defines are announced under the name they were built with. A
prologue's routine name takes the same name. This closes `TEST_TRACE_EXIT`,
`TEST_TRACE_OTHER_ENTRYPOINT`, OPTIONS `TEST_TRACE_LABELS` and the three TraceObject tests
(`TEST_CALLER_STACK_FRAME_REPLY_START`, `TEST_OBJECT_AND_SCOPE`, `TEST_TRACEOBJECT_COLLECTOR`). The
`t6.8/probes/p3` shape (`.Method~new("", ...)` announcing `""`) is the same path.

### class-activate-order

`class_install_order` (`directives.rs`) was a depth-first walk, so `::class class1 subclass class3`
put `class3, class1` ahead of `class2`. It now follows `LanguageParser::resolveDependencies`
(`parser/LanguageParser.cpp:1801`): each round takes the first class in source order whose
`SUBCLASS`, `METACLASS` and `INHERIT` targets declared in the file are installed, and a cycle blames
the first class left. `resolve_in_place` does the rounds over the class list itself with a bitset.
Cycles (`a`/`b` mutual, a class naming itself, a cycle behind an independent class) answer as
before and as the oracle.

### method-object-identity

* A `Method` object reached through a behaviour was keyed by the receiver's class
  (`introspection.rs`, `method_object_for`), so `.sub~methods` and `.sub~new~instanceMethod` gave
  objects other than `.base~method`. It is keyed by the defining class, as `Class~method` keys it.
* `Class~methods` and `Object~instanceMethods` listed one entry per name. The oracle's
  `MethodDictionary::getMethods` (`behaviour/MethodDictionary.cpp:507`) lists every defined entry,
  the overridden ones and the ones behind a hiding entry included. `MethodDict::defined_entries`
  (rexx-classes) lists them, and `behaviour_entries` reads it. This was `TEST_METHODS`' failure:
  `.WaterVehicle~methods` lacked Object's `INIT`, which `Vehicle` overrides.

### setmethod-instancemethod-kind

`instanceMethod` asked only the class behaviour, so a `setMethod` name answered `.nil`. It now reads
the object's own dictionary first: the object `setMethod` installed, or `.nil` for a hidden name.
`instanceMethods` lists the object's own entries filtered by the scope like the behaviour's, each
as the object `setMethod` installed; an `OBJECT`-scoped one is listed under its class scope and not
under `.nil` (`t6.11/probes/p9`, identical to the oracle).

### uninit-define-on-superclass

`ClassGraph::update_instance_sub_classes` now runs `check_uninit` on each class it rebuilds, as
`RexxClass::updateInstanceSubClasses` (`ClassClass.cpp:1071`) does, when the change touches
`UNINIT`; no other change can make `UNINIT` resolve where it did not.

### method-package-prolog

`Package~prolog` answers `.nil` for the package of a `Method` or `Routine` built from source, whose
main section became that executable (`generateMethod`/`generateRoutine` leave `initCode` null,
`LanguageParser.cpp:599`, `:633`). Such a package is the one holding a synthetic directive.

### native-stack-frames

`sortWith` and `stableSortWith` run their comparator under a level of their own
(`Interp::in_method_level`, `Activity::method_levels`, `LiveLevel::Method`), which
`.context~stackFrames`, a condition's `STACKFRAMES` and a trace line's caller frame list as a
`METHOD` frame: the comparator as its arguments, the receiver as target, `Array~method(name)` as
executable, no line, the `Compiled method` line as traceback. The two names no longer share one
native function, so each level carries its own name. The level costs nothing outside those two
methods.

## Witnesses

Corpus programs in `rust/corpus/phase-9.txt`, each with its `sourceline_oracle/` file from that
test's driver (run on a scratch copy). `t6.11/witnesses/summary.tsv`, by `t6.11/witnesses/run.sh`,
red is the parent binary `t6.10-r3/rexx-head` (code identical to `55681370e`), green
`t6.11/rexx-head` (`a22a686d0`):

| program | parent | task |
|---|---|---|
| `trace_compiled_package_name` | stderr differs | identical |
| `class_activate_dependency_order` | stdout differs | identical |
| `method_object_identity_through_subclass` | stdout differs | identical |
| `setmethod_instance_method` | stdout, stderr, rc differ (97.1 at line 8) | identical |
| `uninit_defined_on_superclass_later` | stdout differs (no uninit) | identical |
| `package_prolog_of_compiled_executable` | stdout differs | identical |
| `stackframes_sortwith_comparator` | stdout, stderr, rc differ | identical |

Read stdout, each path printing: the `>I>`/`<I<` pairs of `.Routine~new`, both `.Method~new`
forms, the `.Package~new` prologue and its routine; the order `b c a e d g f` (subclass, inherit
and metaclass dependencies); identity through subclass, instance and scoped supplier, and the
entry counts for an overridden `INIT` and `BAR`; `instanceMethod` after a string, an array, a
Method and a hide, and an own entry beside the class's; `uninit ran for Second` after the first
collection and nothing after the second; `.nil` for the three executable packages and `Routine` for
the two packages; the `SORTWITH` and `STABLESORTWITH` frames and the condition's frames. Collection
output prints counts and memberships only.

## The tests

`suite-9d-set.txt` loses `base/class/Class` `TEST_ACTIVATE`, `TEST_METHODS`; `base/class/Object`
`TEST_INSTANCEMETHOD`, `TEST_SETMETHOD`, `TEST_UNINIT`; `base/class/Package TEST_PROLOG`;
`base/class/StackFrame TESTNATIVECODEFRAME`; `base/directives/OPTIONS TEST_TRACE_LABELS`;
`base/keyword/TRACE` `TEST_TRACE_EXIT`, `TEST_TRACE_OTHER_ENTRYPOINT`; and
`base/keyword/TRACE_TraceObject` `TEST_CALLER_STACK_FRAME_REPLY_START`, `TEST_OBJECT_AND_SCOPE`,
`TEST_TRACEOBJECT_COLLECTOR`.

## Queued items

None of the four the brief names closes; each probe still differs from the oracle on this task's
build (`t6.11/queued/`):
* `2026-10-19-context-name-during-install.md`: `q1`, `[9 ::CONSTANT]` against `[9 ]`.
* `2026-10-18-install-error-names-caller-file.md`: `q2`, `Error 41 running x` against the main
  file's path.
* `2026-10-17-routine-call-frame-name.md`: `q4`, `ROUTINE MYNAME` against `ROUTINE CALL`.
* `2026-10-17-newmethod-parent-package.md`: the native-API path was not touched.

The cluster's own parents stay open in part: `2026-10-14-native-method-frames.md` is closed for
`SORTWITH` (its named test passes), and `Routine~call`'s `CALL` frame is still absent;
`2026-10-02-frame-executable-invocation-and-native-levels.md`'s fexe3 still lacks `METHOD SEND`
(`t6.11/queued/fexe3`) and fexe2 is untouched. Those are native levels entered through a call tail
rather than run on this Rust stack, and are not widened here.

## Suite delta

`t6.11/suite/` at `58902b630`: 402 groups, 5 red. Against `t6.10/r3/suite/tests.tsv`
(`t6.11/suite/changed-rows.txt`) exactly the thirteen tests above change, each `failed` to `passed`
with no row. `t6.11/suite2/` at `a22a686d0`: 6 red; against `t6.11/suite/` the only changed row is
the known flake SysFileTree `TEST_TREE_DIRECTORY` (`t6.11/suite2/changed-rows.txt`).

## Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t611-target`, `CARGO_INCREMENTAL=0`, each exit status
read unpiped, on `a22a686d0`:

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec -p rexx-classes --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1155 passed.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test owners --test
  closed_phases --test phase_ten_reasons --test representation_refusals --test
  refusal_dispositions --test coverage --test rexxc --test licensed_divergences`: exit 0. No `Loud`
  site added or removed.
* `memcap 16G cargo test -j 4 --release -p rexx-parse -p rexx-classes`: exit 0 (includes
  `sourceline_oracle`).
* `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1121 of 1121 matching.
* `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t6.11/suite2/table.tsv memcap 8G cargo test -j 4
  --release -p rexx-exec --test suite l3_core_suite`: exit 101 as expected, 402 groups, 6 red,
  362 s. The `t6.11/suite/` run at `58902b630`: exit 101, 5 red, 255 s.

## Perf

`t6.11/callgrind.txt`: pin, parent (`t6.10-r3/rexx-head`) and head (`a22a686d0`) in one run, 3
rounds. Exit 1 is the script's `SPREAD pingguard parent 0.0121%` line, the parent's own spread.
Own delta is head d% minus `t6.10/r3/callgrind.txt`'s head figure; the -libc column is head
-libc d% minus parent -libc d%, same run.

| program | vs pin | t6.10 r3 vs pin | own delta (ledger) | own delta -libc |
|---|---|---|---|---|
| startup | +0.0605 | +0.0420 | +0.0185 | +0.0053 |
| interpret | -1.7701 | -2.1721 | +0.4020 | +0.0002 |
| parse | -0.1211 | -0.1224 | +0.0013 | +0.0002 |
| churn | +0.1189 | +0.1140 | +0.0049 | +0.0007 |
| sayloop | +0.0448 | +0.0458 | -0.0010 | +0.0030 |
| rexxcps | +0.4772 | +0.4784 | -0.0012 | 0.0000 |
| sendloop | +0.0693 | -0.2050 | +0.2743 | 0.0000 |
| dispatch | -0.1352 | -0.3162 | +0.1810 | 0.0000 |

Every other program's own delta is under +0.01% raw and -libc, except pingmsg at +0.1526 raw
and -0.0999 -libc, and pingguard and pingsem, which are below zero on both.

## Concerns

1. **interpret's raw own delta is +0.40%, over the +0.05% budget, and sendloop's +0.27% and
   dispatch's +0.18% are past it too; each is glibc.** The -libc deltas are at most +0.0053% on
   every program. The glibc counts move with the heap's state, not the code under test: the parent
   binary itself, unchanged, measured `interpret` at 4115983694 and 4135197304 Ir and `sendloop` at
   14555027297 and 14625033731 Ir (+0.47% and +0.48%) in runs that differed only in the `-o`
   directory and program list, with -libc equal to under 0.0001% (`t6.11/perf-layout.txt`). In the
   intermediate builds the same programs swung the other way: one build had rexxcps +0.0246 own
   and interpret +0.49, another sendloop -0.55 and interpret +0.66, with -libc unchanged. The
   sendloop excess was traced to `Activity::invocation_name`'s `bcmp` (18 to 26 Ir per send, the
   same calls), the interpret one to `malloc`. Taking a call path's raw libc figure as its cost
   rewards the binary that lands a lucky heap; I took the -libc figure as the measurement and need a
   ruling on whether that stands.
2. `Class~methods`/`instanceMethods` now list every scope's entry. Where `setMethod` installs one
   name twice under different scopes (`FLOAT` then `OBJECT`) the oracle keeps both entries and this
   crate's own dictionary keeps one name; not measured.
3. The `sortWith` level's scope is found by `lookup(receiver, name)` when it is pushed, so a
   subclass that overrides `SORTWITH` and forwards to `super` reports the subclass's scope in the
   frame's traceback line. Not measured against the oracle.
