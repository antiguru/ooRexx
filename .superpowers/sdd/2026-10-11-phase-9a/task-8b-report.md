# Phase 9a Task 8b report: the items Tasks 5 and 6 moved out

Base `8d3aa11d9`. Status: NEEDS_CONTEXT for three items left refused (below); every other item is
done and committed.

Probes ran from fresh `mktemp -d` directories under `/tmp/claude-1000/p9/` through
`/tmp/claude-1000/p9/t8b-tools/both.sh` (oracle under the standard wrapper with `timeout -k 5 20`,
`rexx-run` under `memcap 2G timeout -k 5 20`; stdout, stderr and status kept apart and compared with
`cmp`). Probe files are in `/tmp/claude-1000/p9/t8b-probes/`. `oracle-crashes.txt` was read first;
entry 8 (metaclass `NEW` answering a non-class) was not run. Every witness's stdout was read on both
sides; each witness below is byte-identical on the three descriptors.

## Commits

| sha | item group |
|---|---|
| `2c716b02c` | object names |
| `70d314bac` | metaclass `NEW`, `Class~enhanced` through `NEW`, `SUPER` of `.nil` scope |
| `cce984dd3` | object-name table as a `Vec` (callgrind fix) |
| `ff05ec779` | `Routine`/`VariableReference` copies, native method frame `~executable` |
| `acd4aac65` | `Package~local`, annotation tables, security info directory store-backed |
| `2b1b13583` | `.ROUTINES` resolution |
| `31e88f857` | package-local lookup out of line (callgrind fix) |

## 1. Object names

Probes `g1/objname*.rex`, `ident.rex`, `nilname.rex`. Oracle: a name set on an `Array`, a `Stem` or
a `VariableReference` is read back by `~objectName` (and `~string` for an `Array`), carried by
`~copy`, by `USE ARG` and `EXPOSE`, and used in error inserts; `s. = 'dflt'` resets a stem's name.
For strings the name follows the oracle's object identity: a package's literals are one object
(`'abc'` named at one site is named at every site, including a `::ROUTINE` and a symbol spelling the
same text), the integer cache shares -10..100 (`x = 100` named, then `(99+1)~objectName` is the
name), and `||''` and `~string` answer the receiver; a computed string is a fresh object.
`.nil~objectName=` is 97.1 (its behaviour drops the method).

Mechanism: a side table in `Heap` (`names: Vec<(ObjRef, Box<[u8]>)>`) for bodies with no name field,
pruned after each sweep out of line, copied by `copy_object`. Strings, small integers and `.nil`
stay refused (see Concerns).

Witness `lang/object_name_set_values.rex`. Pin
`a_conversion_this_phase_does_not_model_is_loud_where_the_ones_it_models_answer` gains a long-string
refusal and an `Array` success.

## 2. Metaclass `NEW`, `Class~enhanced`, `SUPER` of `.nil` scope

Rules from `enhanced-semantics.md`; the scout's probes `p1`-`p16` were re-run. All agree except
`p2`/`p3` (`instanceMethods()` ordering, below), `p5`/`p8` (R16, below) and `p9c`/`p9d` (the `"&1"`
insert of 35.1, pre-existing, D6).

Mechanism, as the scout recommended, with one change: **a class is built in place when the
metaclass's `NEW` is `.Class`'s and its instances' `INIT` is `Object`'s** (`builds_in_place`), since
nothing can then tell the clone and its rewiring from an in-place build; otherwise `NEW` is sent to
the metaclass, the answer must be a class `Class~new` just built (`ClassGraph::is_cloned`), and
`ClassGraph::reparent` rewires it, keeping its own dictionaries. Reason: the directive install path
then pays two lookups per `::CLASS`, not a send and a second `INIT`. Both `~subclass`/`~mixinclass`
and `::CLASS` (`install_class`, inside a directive activation when it sends) use `new_subclass`.
`Class~enhanced`: `new_subclass(receiver, "Enhanced Subclass")`, `INIT` on it, then the methods into
its own instance dictionary at scope `.nil` (`ClassGraph::enhance_instance_methods`, `.nil` items
hidden), `check_uninit`, `NEW` sent to it with `args[1..]`, and on an answer the dummy built: the
instance's `class` set to the receiver and `ObjectMethods::enhanced_by` set to the dummy (traced,
keeps it alive); for a class answer (R15) `set_owning_class`. The object-level enhancing layer
(`ObjectMethods::enhanced`, `enhance`, `ObjectMethodWrite::Enhance`) is deleted. An unnamed
enhanced instance renders `enhanced <id>` in `to_text` (`Redirect::EnhancedDefault`), which closes the
error-insert queued item. `SUPER` of a `.nil`-scoped method is the behaviour's owner (instance) or
the class's owning class (class side), `Interp::super_scope_in`. A class built at run time is charged
128 bytes per behaviour name toward the next collection.

R16 (a user `NEW` answering an object not built from the dummy): the answer is handed back
untouched, as the scout recommended; the oracle writes into the shared behaviour (`p5`, `p8` differ:
`o: an ODD` against `o: enhanced ODD`, and the oracle's damage to every later `Odd` and `String`).
This is a divergence from an oracle defect with no licence recorded yet.

A metaclass `NEW` answering anything other than a fresh class is `Loud::receiver_class` ("a
metaclass whose NEW answers something other than a new class"), crash entry 8; the entry's
consequence paragraph and the `receiver_class` disposition record cite it. Pin
`a_metaclass_new_answering_no_new_class_is_loud` (`'x'` and `.array` answers refuse; `forward class
(super)` answers `id k`).

Memory (`/usr/bin/time -v` on the subject, `timeout` outside it), `memmeth.rex` (Method-object
source):

| calls | oracle max RSS kB | crate max RSS kB |
|---|---|---|
| 50000 | 51612 | 104840 |
| 200000 | 121480 | 150684 |

Per call from 50000 to 200000: oracle 0.466 kB, crate 0.306 kB. `sub50000.rex` (`~subclass` loop):
1045452 kB at base, 92224 kB now.

Witnesses `lang/enhanced_runs_new.rex` (R1-R9, R12's 35.1 by code only), `lang/enhanced_primitive_receivers.rex`
(R10, R11, R15, and the queued `b7_enhanced_array`/`b7_enhanced_string`), `lang/metaclass_new.rex`
(R14: user metaclass `NEW`, double `INIT` with the second after reparenting, `::CLASS ... METACLASS`,
`~mixinclass`, metaclass-instance `NEW` for `~new` and `enhanced`). Existing enhanced and metaclass
corpus programs (`grep -rli -e '~enhanced(' -e metaclass corpus/lang/*.rex`) all byte-identical.
Queued `2026-10-10-class-enhanced-skips-new.md` and `2026-10-10-enhanced-object-name-in-messages.md`
closed; `2026-10-12-gc-trigger-blind-to-class-weight.md` has a measurement line appended.

## 3. Copies and native `~executable`

Probes `g3/rcopy.rex`, `vrcopy.rex`, `nframes*.rex`, `pcopy*.rex`. `Routine~copy` is
`copy_method_object` (a REXX-package routine's row is carried); `begin_routine` stores the receiver
in the routine activation's `cold.executable` when it enters, and `executable_at` reads it first, so
`r~call == r`, `r~copy~call == r~copy`, `call e` answers `.routines~e`. `VariableReference~copy`
clones the body (the cell is shared). A native method level's frame answers
`method_executable(scope, name)` (`.Stream~method('CHARIN')`, measured identical).

Left refused: a native **routine** level's executable (oracle: a source-less `Routine` per package
and name, `e~package` the caller's, `e == f` across calls; probe `nframes2.rex`), and `Package~copy`
(see Concerns). Witness `lang/executable_copies.rex`; pin
`a_copy_or_frame_executable_this_crate_does_not_keep_is_loud` now asserts the successes and the two
remaining refusals.

## 4. Store-backed tables

`Package~local` is `hash::new_directory(self, 0)`, read in `.NAME` resolution through
`hash::store_get` behind `package_locals.is_empty()`; annotation tables are `hash::new_string_table`
filled by `directory_put`, and a `Method`/`Routine` copy sends `COPY` to its table; the security
manager's info directory is `hash::new_directory`. `EnvironmentModel::string_table` had no reader
left and is deleted. Probes `g4/tables.rex`, `secmgr.rex`, `localfb.rex` (a package local's
`UNKNOWN` answers a `.NAME`, on both sides). Witness `lang/package_store_tables.rex`. Pin
`a_native_row_on_a_receiver_of_another_class_refuses_its_type`'s `~local~items` refusal is now the
success `0`.

## 5. `.ROUTINES` resolution

Probes `g5/rt1.rex` and the ten mutators in `g5/pin/`. The oracle's `CALL` keeps the routine a site
resolved (`RexxInstructionCall::externalTarget`), so a site that resolved `FOO` before
`.routines~put(.routines~bar, 'FOO')` still runs `foo`; a site resolving afterwards runs `bar`;
`remove('FOO')` then a new `call foo` is 43.1. Mechanism: each `.ROUTINES` mutator marks its
package in `routines_written` (the former refusal sites); a lookup through a marked package
(`find_written_routine`) reads the table in place of the package's `routines` map, and a table entry
resolves by its executable record (`routine_object_resolution`; an entry that is not a routine this
crate built is `Loud::receiver_class`). A write does not bump `routine_generation`, so resolved sites
keep their resolution, as the oracle's do. `Loud::package_routines_write` is deleted. Witness
`lang/routines_table_resolution.rex`; pin renamed
`a_write_to_a_packages_routines_table_moves_call_resolution`, every mutator's answer measured.

## Callgrind

`bench-programs/callgrind.sh -r 1 -j 10 base=<8d3aa11d9 rexx-run> head=<31e88f857 rexx-run>`, two
invocations (`-p` halves), outputs `/tmp/claude-1000/p9/t8b-cg/f1`, `f2`; base and head interleaved
per program; spreads 0.0000 (one round). Worst +0.146%; nothing over +0.5%.

| program | d% | program | d% |
|---|---|---|---|
| alloc | +0.0352 | hashnew | -0.0163 |
| alloc4c | +0.0730 | heapshape | -0.2540 |
| arith | +0.0735 | nop | +0.0009 |
| assign | +0.0004 | parse | +0.1272 |
| compound | +0.0009 | pingguard | +0.0179 |
| decloop | +0.1214 | pingmsg | +0.0606 |
| decrender | +0.0702 | pingsem | +0.0062 |
| dirread | +0.1464 | rexxcps | +0.0839 |
| dispatch | -0.0480 | sayloop | +0.0750 |
| dispatchclass | -0.0506 | sendloop | -0.0718 |
| emptyloop | +0.0010 | startup | +0.1412 |
| extcall | +0.0011 | strings | +0.0824 |
| fibcall | +0.0010 | textnum | +0.0070 |
| fibfunc | +0.0010 | varlookup | +0.0004 |

Over +0.1%, by function (`cgdiff.py` over `callgrind_annotate --inclusive=no`):
* decloop +2.9M, parse +2.1M: `collect_now` +2.46M / +1.64M, i.e. a few Ir per swept slot. A
  control build with the name-pruning call removed measured decloop +0.25% (`t8b-cg/d`), so the
  sweep loop's count moves with unrelated edits; `Body::trace` also rises (+0.37M, +0.25M) from the
  store-backed annotation tables now marked each collection.
* dirread +3.9M: `from_utf8` called the same 600,315 times from `lookup_behaviour` at 6 more Ir per
  call (its word-at-a-time path depends on the name's address), plus `memcmp` +0.97M.
* startup +81k: `install_store` +28k (annotation tables), `MethodDict::slot` +29k and
  `lookup_instance_method` +10k (`builds_in_place` per `::CLASS`), `install_class` +12k.

Two fixes are committed for what the first measurements found: a `NameMap` in `Heap` moved hashbrown's
`fallible_with_capacity` out of line in `NativeObject` rehashes (pingmsg +0.87%; `Vec`: +0.048%),
and reading the package local with `directory_get` moved `directory_get` out of `directory_lookup`
(dirread +0.50%; `store_get`: +0.146%). The `SUPER` compare itself: `dispatch`/`dispatchclass`/`sendloop`
are -0.05%..-0.07%.

## Refusal tables

`REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites` at `2b1b13583`: one row removed
(`package_routines_write`), the rest `lib.rs` line shifts. No constructor added. `receiver_class`'s
disposition record now also cites crash entry 8 and the `.ROUTINES` non-routine entry.
`native_method` sites left: `OBJECTNAME=` (strings, small integers, `.nil`), `COPY` (Package),
`EXECUTABLE` (native routine frame), plus the earlier ones.

## Group verdicts (suite harness)

`REXX_SUITE_TABLE=/tmp/claude-1000/p9/t8b-suite/final/table.tsv REXX_SUITE_GATE=1
REXX_SUITE_GROUPS=base/class/Object,base/class/RexxContext,base/class/StackFrame,base/keyword/VarRef,base/class/Package,base/class/Class,base/directives/ROUTINE,doc/rexxref/chapter5/Section1,base/class/Method,base/class/Routine,base/keyword/CALL
... cargo test --release -j 4 -p rexx-exec --test suite l3_core_suite`: exit 101, 8 of 11 red. Against
`docs/superpowers/records/2026-10-11-phase-9a/suite-baseline/verdicts.tsv` no test newly fails;
newly passing: Object TEST_RUN, TEST_SETMETHOD_SCOPE; Package TEST_PACKAGE_LOCAL_GET_EMPTY;
StackFrame TESTEXTERNALNATIVECODEFRAME; RexxContext TESTCOPY01 and ROUTINE TESTDOTROUTINES, TESTPACKAGE
(Tasks 5 and 6); VarRef TEST_COPY; Section1
TEST_OBJECT_OBJECTNAMEEQUALS (group green). `concurrency_tests`' `whole_groups` rows for Object are
stale (gate-only, re-derived at the phase close).

## Checks at `31e88f857` (statuses read unpiped)

* `cargo fmt --all --check`: 0. `cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: 0 (`rexx-core` last changed at
  `cce984dd3` and `rexx-classes` at `70d314bac`, each linted there: 0).
* `cargo test --release -j 4 --no-fail-fast -p rexx-exec -p rexx-classes -p rexx-core --lib`: 0 (rexx-exec 1085 passed).
* `cargo test --release -j 4 --no-fail-fast -p rexx-classes --tests -p rexx-parse --test sourceline_oracle`: 0.
* `REXX_CORPUS_GATE=1 cargo test --release -j 4 --no-fail-fast -p rexx-exec --test corpus --test collect_stress --test refusal_sites --test refusal_dispositions --test closed_phases --test owners`: 0 (27+1 ignored, 37, 5, 3, 8, 6).
* At `70d314bac`: `--test class_lifetime --test class_weak_reference --test unreachable_classes --test collect_policy --test instance_copy_memory --test gate_table_c --test gate_table_d --test introspection_scopes --test method_bodies --test licensed_divergences --test behaviour_wiring --test native_classes_wiring --test uninit_sweep_order --test loud --test bootstrap_install_oracle`: 0; `--test introspection_arity --test coverage --test trace_oracle --test ir_recorded` (temporary `rust/target/release/rexx-run` symlink, removed): 0.
* `REXX_METHOD_BODIES_REFRESH=1 ... --test method_bodies` at `31e88f857`: 0, `method-bodies.txt` unchanged (its `Class enhanced` row probes zero arguments, 93.901 on both).
* Suite: above. Each per-group check took 5 to 10 minutes under load 2 to 12; the per-task 5-minute budget was exceeded.

## Concerns (NEEDS_CONTEXT)

1. **`OBJECTNAME=` on a string, a small integer, `.nil`: left refused.** A string's name belongs to the
   oracle's object identity (literal interning per package, the -10..100 integer cache, operations
   answering their receiver); this crate's strings of 7 bytes or less and small integers are values,
   and its heap strings follow none of those rules (`objname4.rex`, run while heap strings took names:
   5 of its 14 lines differed). Options: (a) keep refused; (b) a by-value table for inline values, wrong for any
   computed value equal to a named one; (c) intern literals per package and model the integer cache,
   a new identity rule on the literal and arithmetic paths. `.nil` needs its behaviour without
   `OBJECTNAME=` and `COPY` (`.nil~hasMethod('COPY')` is 0 on the oracle, 1 here; pre-existing).
2. **`Package~copy`: left refused.** The oracle's copy has its own `~local`, routine and class tables
   (and public ones), imported packages, settings (`q~options('DIGITS', 20)` leaves `p~digits` 9),
   annotations, sharing only entries and the prolog (`pcopy.rex`, `pcopy2.rex`). A package object is
   its `ProgramId` here. Options: (a) a copy is a new `ProgramId` over the same `Rc<Program>` with
   each per-program map cloned (every keyed map must be found, or the copy answers as an empty
   package); (b) a per-object overlay consulted by each `Package` method's mutable state (the
   `package_of` callers: `grep -c 'package_of(interp, receiver)' rust/crates/rexx-exec/src/dispatch/package.rs`
   is 29 at `31e88f857`). No hot path either way.
3. **Native routine frame `~executable`: left refused.** Needs a `Routine` identity per (package,
   routine name) for builtin library routines, with no source.
4. **R16 divergence unlicensed**: `p5`/`p8` differ because the crate does not reproduce the oracle's
   write into a foreign object's behaviour.
5. `instanceMethods()` and `instanceMethods(.nil)` list in this crate's order (sorted, unique) where
   the oracle lists its method dictionary's hash order with shadowed entries repeated; enhancing names
   now appear in both (pre-existing ordering divergence, `ord.rex`).
6. Annotation tables are built eagerly for every `Method`/`Routine` object, now as stores; measured at most +28k Ir on `startup` and part of `Body::trace` in collections.

## Rulings applied

On top of `cfb8af83f`, one commit per item.

1. **`Package~copy`, option (a): `48326cad0`.** `Interp::copy_package` mints a `ProgramId` over the
   same `Rc<Program>` and destructures every `Interp` field by name with no `..`, so a field added
   later does not compile until the copy decides about it; `routine_generation`, missing from my
   first list, was the compiler's catch. Cloned to the copy: options, initial option settings,
   security manager, routine, public-routine, imported-routine, class, public-class, imported-class
   and namespace maps, the written-`.ROUTINES` mark, imports, parent, compiled-method name, required
   path, untranslated mark. `.ROUTINES` and the package annotation table are `COPY` sends; `.METHODS`,
   `.RESOURCES`, the local directory (when built) and the prolog object are shared, which is
   `RexxObject::copy` plus `PackageClass::deepCopy` (`classes/PackageClass.cpp:362`-`:412`). Sub-structs
   (`activity`, `object_model`, `environment`, `option_writes` beyond its `initial` map) are not
   destructured. The REXX package's copy stays refused (`.object~package~copy`: oracle rc 0); it is
   now the `native_method` row's example in `refusal-sites.tsv` and the refusal
   `a_refusal_after_main_ended_in_a_nested_round_is_reported` uses. Witness `lang/package_copy.rex`
   from `pcopy.rex`/`pcopy2.rex` without `~loadLibrary('rxmath')` (no `rxmath` loads here, on either
   copy or original).
2. **Native routine frame `~executable`: `28bf0c253`.** Probes `nframes4.rex`, `nframes5.rex`,
   `nframes6.rex` (with `~firstItem`: a frames list indexes from 0, so the report's earlier `[1]`
   readings were the second frame). Oracle: one source-less `Routine` per name (`SysSleep` twice and
   as a function answer one object, `SysFileDelete` another), package the REXX package, not
   `.context~package`. `Interp::rexx_routine_executable` keeps one per upcased name, rooted globally,
   for a native routine level whose package is the REXX package. A library routine level still
   refuses (oracle: `e~package` is `.nil`; no library loads here to compare). Witness
   `lang/rexx_routine_frame_executable.rex`.
3. **`.nil`'s behaviour: `78d84b7e6`.** `ObjectModel::nil_behaviour`, a copy of `Object`'s instance
   behaviour with `COPY`, `START` and `OBJECTNAME=` hidden in front, built with the model, as
   `memory/Setup.cpp:1758`-`:1760` does before the library loads. Callgrind against `31e88f857`
   (`-p "dispatch dispatchclass sendloop rexxcps startup"`, `t8b-cg/n1`): +0.0000% on the four send
   programs, startup +0.011%. `.nil~instanceMethods` lists 29 names here and 32 on the oracle, which
   lists the `Object` entries the hidden names shadow (concern 5 above). Witness
   `lang/nil_hidden_methods.rex`.
4. **`whole_groups` Object rows: `946ac3834`.** `REXX_CORPUS_GATE=1
   REXX_WHOLE_GROUPS_ONLY=base/class/Object.testGroup ... --test concurrency_tests
   each_group_of_the_derived_list_in_one_run_in_both_modes`: both modes now `failure, assertions 272,
   rc 1, last started TEST_UNSETMETHOD_NO_ARG, failing [TEST_INSTANCEMETHOD TEST_SETMETHOD]`; rows
   rewritten. The same command failed in 2 of 4 runs before any edit with `bin-rexx/rexx appeared
   while it was replaced` (`interpreter_bin`, two runs linking one directory's binary at once); a link
   that finds the same binary already in place now stands, and 4 of 4 runs pass.
5. `OBJECTNAME=` on strings and small integers, and R16: unchanged, awaiting Moritz.

Checks (statuses unpiped): `cargo fmt --all --check` 0; `cargo clippy -j 4 -p rexx-exec
--all-targets -- -D warnings` 0 (and `-p rexx-classes` at `78d84b7e6`: 0); `--lib` of rexx-exec 0
(1085 passed, after moving the scheduler test's refusal); `-p rexx-parse --test sourceline_oracle` 0;
`REXX_CORPUS_GATE=1 --test corpus --test collect_stress` 0 at `48326cad0` (27+1 ignored, 37);
`--test refusal_sites --test refusal_dispositions` 0; `--test concurrency_tests --test suite` 0
(54, 50); suite groups `base/class/Package,base/class/Object,base/class/StackFrame`: same not-passed
sets as the earlier final run, no test newly failing.
