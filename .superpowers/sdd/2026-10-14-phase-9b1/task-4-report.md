# Phase 9b1 Task 4 report: Package is a heap object holding its state

## Step 1: how the Package body refers to its program

**Choice: the body holds the `ProgramId`, inside an erased payload.** `rexx-core` gains a
`PackageFields` trait (`trace`, `clone_fields`, `Any` as a supertrait) and a `PackageState` box over
it. A plain package keeps it in `NativeObject`'s `Bound` slot (boxed, so every other native object
stays 104 bytes), and a `Package` subclass instance keeps it as `NativeState::Package`.
`rexx-exec`'s `PackageData` (`src/package_state.rs`) is the one implementation: its `program:
Option<ProgramId>` names the compiled program, `None` for the REXX package, and every copy holds the
same id. `Interp::programs` stays the strong holder, and Task 6 owns its lifetime (prune at sweep by
the ids live package, method and routine objects hold).

The state has to be erased whatever the program handle is: its fields are `rexx-exec` types
(`MergedRoutine`, `PackageOptions`, `ProgramId`). So the three options differ only in what the
erased payload carries:

| option | what the body holds | why not |
|---|---|---|
| ProgramId (chosen) | `Option<ProgramId>` | |
| `Rc<Program>` in the payload | the handle itself | The call path reads `programs[id]` from the activation's id either way. A held `Rc` makes the table a second holder that Task 6 must make weak, and a weak upgrade per call costs on the path that has the headroom to lose. |
| move the handle type | `rexx-core` depends on `rexx-parse`, or `Program` moves | A crate-graph change for a field `rexx-core` never reads. |

The hot path is unchanged in shape: the activation keeps `program_id` and `Rc<Program>`, so
`call.rs`, `dispatch.rs` and `lib.rs`'s `Rc::clone(&self.programs[..])` read what they read before.
The per-entry read that moves is `options_of` (`start_from_package`, the requires prolog check):
`Interp::program_packages[id]`, then the heap slot, then a downcast, where it was a
`FxHashMap<ProgramId, PackageOptions>` probe. `program_packages` replaces `package_objects`
(`Package -> ObjRef`); `object_packages` is gone, the body's own `program` field answers it.

**Measured** with the options, initial options and package identity moved (the rest still in maps),
from `rust/`:

```
timeout 600 bench-programs/callgrind.sh -B 0.5 -j 8 -p "startup pkglocal fibcall extcall" \
    -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-4-step1 \
    pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t4a
```

exit 0. Records in `docs/superpowers/records/2026-10-14-phase-9b1/task-4/step1-callgrind/`.

| program | pin | head | whole d% | -libc d% |
|---|---|---|---|---|
| startup | 119095470 | 119445463 | +0.2939 | +0.4184 |
| pkglocal | 4185405401 | 4091785511 | -2.2368 | -3.7204 |
| fibcall | 9109404880 | 8805403334 | -3.3372 | -2.2911 |
| extcall | 8975966670 | 7785339747 | -13.2646 | -1.7880 |

Against Task 3's head (`task-3-fix1`), startup moved +7,304 Ir (119438159 to 119445463). All four
are inside +0.5% of the pin.

## Status

DONE_WITH_CONCERNS. Commits: `4a5b8b571` (step 1), `fc0266803` (step 2), `7f26fbd71` (step 3),
`b2ae5bd48` (step 4, the per-entry cost), `cd01d7fd6` (records and two stale docs, no code).

## Step 2: the state in the body

`PackageData` (`rust/crates/rexx-exec/src/package_state.rs`) holds: program id, options and initial
options, security manager, `routines`, `routines_written`, public and merged routines, classes,
public and merged classes, namespaces, local, imports, parent, the main `Routine` object, the
`.METHODS`/`.ROUTINES`/`.RESOURCES` tables and the constant values. Every one of those maps is gone
from `Interp`, and so are `package_objects`, `object_packages`, `package_parents` and `parented`.
`Interp::program_packages` (a `Vec` by `ProgramId`, replacing `package_objects`) answers a
program's own package; `rexx_package` answers the REXX package. `which_package` reads the body's
`program` field.

Decisions:

- **Package methods read the receiver's own body**, so a copy answers from its own fields. Imports,
  namespaces and the parent hold package objects, not `Package` values: `p2~addPackage(q)` imports
  the object, and a package built with a copy as its context resolves through that copy.
- **A program with directives builds its package before installing** (`install_directives`), so no
  install-time write allocates. A program with none builds it only when asked, which keeps
  `a_program_with_no_send_and_no_class_never_builds_the_object_model` true.
- **The library's classes are also the REXX package's own fields** (`record_package_class`), the
  first library program's declaration of a name standing. The REXX package's class lookups read
  that one body instead of one body per library program (step 4).
- **Latches replace the emptiness tests the maps gave for free**: `security_armed`,
  `routines_written`, `routines_tables`, `package_locals`, `options_armed`. Each only ever sets.
- **Left on `Interp`, with the reason**: `required_packages` is the requires cache (the oracle's
  `PackageManager`), which Task 5 makes weak; `required_paths`, `compiled_method_names`,
  `untranslated` and `library_programs` are facts about a program, not mutable package state, and
  every copy shares the program. The package annotation table stays in `Interp::annotations` for
  the original and in the object's own annotation slot (as before) for a copy.
- **The `.ROUTINES`-table write check** (`routines_table_package`) looks only at program packages: no
  code runs in a copy, so no `.ROUTINES` reaches a copy's table.
- The exhaustive-destructuring `copy_package` functions on `Activity`, `EnvironmentModel` and
  `ObjectModel` are deleted: a copy is the body's clone and reads nothing else.

### GC: the traces that replace roots

`PackageData::trace` replaces the global roots under `package_local_root_key`,
`package_table_root_key`, `program_routine_root_key` and `constant_root_key`, and the
`object_roots` entries for security managers, classes and public and merged routines. `routines`
stays untraced, as Task 3 left it. New test
`the_objects_a_package_holds_survive_collect_on_every_allocation` (`tests/collect_stress.rs`; oracle
measured with the same program, rc 0, identical stdout). Its `::CONSTANT` half uses a package a
routine built: install temps of the main program stay on the base frame for the whole run, so a
constant of the main program is held without the trace and would not have gone red.

Mutation, each one trace deleted from `package_state.rs` and restored from a saved copy (`git diff
--stat` empty after each), command
`CARGO_INCREMENTAL=0 timeout 500 memcap 8G cargo test -j 4 --release -p rexx-exec --test collect_stress -- the_objects_a_package_holds an_added`:

| trace deleted | red test |
|---|---|
| `local` | the_objects_a_package_holds_survive_collect_on_every_allocation |
| `tables` | the_objects_a_package_holds_survive_collect_on_every_allocation |
| `main_routine` | the_objects_a_package_holds_survive_collect_on_every_allocation |
| `constants` | the_objects_a_package_holds_survive_collect_on_every_allocation |
| `security_manager` | the_objects_a_package_holds_survive_collect_on_every_allocation |
| `merged_public_routines` | an_added_routine_object_survives_in_its_importer |
| `public_routines` | an_added_public_routine_object_survives_its_table_entry |

The classes, namespaces, imports and parent traces cannot go red here: classes are never collected
(D59) and every package object is still a global root. They become load-bearing in Task 5.

## Step 3: copy

`Package~copy` takes the generic body clone (`object_protocol.rs`, `Primitive::Package` now maps to
`BehaviourId::OBJECT`), then `duplicate_package_tables` gives the copy its own `.ROUTINES` table and
annotation table, which is `PackageClass::deepCopy`'s list. `build_package_shares` first builds the
main `Routine` object and the three tables on the original so the copy shares them. A copy holds the
original's `ProgramId`. A subclass instance keeps its state in `NativeState::Package`
(`native_as_instance` moves it there), so it copies the same way.

`checkRexxPackage` refuses only the REXX package's own object (`Interp::is_the_rexx_package`): the
oracle's test is `isInternalCode()`, which is `isOldSpace()` (`classes/PackageClass.hpp:154`), and a
copy is new space. Measured: the oracle's REXX copy takes `addClass` and `addRoutine`, and the REXX
package itself refuses with 98.984.

Witness `rust/corpus/lang/package_copy_mutated.rex` (listed in `corpus/phase-9.txt`, sourceline
expectation generated with the module driver of `sourceline_oracle.rs`). Oracle first, from a fresh
empty directory: rc 0, stderr empty, stdout

```
copy Package 0 1
routine 1 1
routines 0 1
class 0 1
findclass 1 1
public 0 1
imports 0 1 1
imported 0 1
manager 1
reverse 1 0
mine 1 1
copy2 1 1
subclass MYPKG s 0 1 1
rexx REXX 0 Package 1 1
rexx copy 1 0 1 1 0
rexx addClass 98.984
```

The crate at `7f26fbd71`: identical stdout, empty stderr, rc 0. The Task 3 binary
(`rexx-run-9b1-t3f`) stops at the `subclass` line with rc 120, `a message send to a package object
this crate did not build`. L5 and L6's own snippets (scout-leaks) give `MYPKG`/`y` and `REXX`, rc 0,
on both sides. The witness's manager line shows only that the original is untouched: no code runs in
a copy, so its manager is not observable.

## Step 4: refusals, budget, suite

`representation_refusals.rs`: both `Task 4` rows and the `Task 4` owner are gone. The refusal for a
receiver that is not a package (a Package method run on another object, which the oracle reads
through the wrong layout, `oracle-crashes.txt` entry 33, Deviation 28) is now `a message send to a
value that is not a package`, the hash collections' wording. `refusal-sites.tsv` re-derived
(`REXX_REFUSAL_SITES_REFRESH=1`; line moves only) and its `native_method` row's example list drops
`.object~package~copy`, which answers now. `scheduler/tests.rs`'s
`a_refusal_after_main_ended_in_a_nested_round_is_reported` used `.object~package~copy` as its
refusal; it uses `rxqueue('G')` (Phase 10 Rxapi) instead.

`collect_stress`'s `NO_ALLOCATION_PROGRAMS` loses the programs with directives whose install
refuses (duplicate class, missing external library and the like): each now allocates its package
before the refusal, as the oracle's compile builds its `PackageClass`.

### Budget

The first full measurement at `7f26fbd71` was over: sendloop +3.9449%, dispatch +2.4218%, pkglocal
+0.9167% (`docs/superpowers/records/2026-10-14-phase-9b1/task-4/callgrind-first/`). Causes, from
`callgrind_annotate` diffs against the pin: `start_from_package` was no longer inlined and a
`memcpy` of the method activation appeared in `begin_invoke` (about 76 Ir a send); the REXX
package's class search downcast one body per library program on every `.NAME` miss. Fixes in
`b2ae5bd48`: `PackageStart`, taken before the activation is built and gated by `options_armed`
(the `begin_invoke` memcpy is gone in the next profile); the library classes in the REXX body;
`#[inline]` on `effective_security_manager` and `rexx_package_class`, `#[inline(never)]` on the
uncached search.

Final, binary `rexx-run-9b1-t4f` built at `b2ae5bd48` (`cd01d7fd6` changes two doc comments only),
three rounds, from `rust/`, each `timeout 590 bench-programs/callgrind.sh -B 0.5 -j 12 -p "<set>" -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-4-final-<a|b|c> pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t4f`,
exit 0 for each of the three (records in `task-4/callgrind/{a,b,c}/`):

| program | whole d% | -libc d% |
|---|---|---|
| intcall | -10.7938 | -9.9478 |
| fibcall | -3.4489 | -2.2909 |
| fibfunc | -3.4975 | -2.3377 |
| extcall | -13.2641 | -1.7877 |
| dispatch | -0.4731 | -0.3131 |
| dispatchclass | -3.8437 | -2.5535 |
| sendloop | -0.4428 | -0.4685 |
| varlookup | +0.0031 | +0.0019 |
| pkglocal | -6.8709 | -9.0802 |
| startup | +0.3219 | +0.4453 |
| alloc | -1.6253 | -1.9517 |
| hashnew | -0.0510 | -0.1730 |
| alloc4c | -0.1007 | -0.1182 |
| arith | +0.1387 | +0.1499 |
| assign | +0.0022 | +0.0014 |
| compound | +0.2282 | +0.0029 |
| decloop | +0.2157 | +0.2173 |
| decrender | -0.2197 | -0.2391 |
| dirread | -1.6802 | -1.5994 |
| emptyloop | +0.0052 | +0.0033 |
| nop | +0.0043 | +0.0028 |
| parse | -0.0580 | -0.0737 |
| sayloop | +0.2160 | +0.2372 |
| strings | -0.2223 | -0.3930 |
| textnum | -0.1300 | -0.1517 |
| heapshape | -0.3882 | -0.4168 |
| rexxcps | -0.0216 | -0.0179 |
| pingguard | +0.0619 | -0.1514 |
| pingmsg | -1.2100 | -2.8357 |
| pingsem | -0.1995 | -0.3020 |

Least headroom: startup +0.3219% (Task 3 left it at +0.2876%; the REXX package and its library
class tables are now built during the bootstrap).

### Suite

Groups: `base/class/Package`, `base/directives/REQUIRES`, and every group whose name contains
`Package`, `ROUTINE` or `METHOD`, from
`find ootest/ooRexx -name '*.testGroup' | sed 's|^ootest/ooRexx/||; s|\.testGroup$||' | grep -E 'Package|ROUTINE|METHOD|REQUIRES'`
(run from the repository root): `API/oo/METHOD`, `base/class/Package`, `base/class/Package_Options`,
`base/directives/METHOD`, `base/directives/REQUIRES`, `base/directives/ROUTINE`. Added the
case-insensitive matches `base/class/Method`, `base/class/MethodArgs`, `base/class/Routine`.

```
REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Package,base/class/Package_Options,base/directives/REQUIRES,API/oo/METHOD,base/directives/METHOD,base/directives/ROUTINE,base/class/Method,base/class/MethodArgs,base/class/Routine REXX_SUITE_TABLE=<records>/suite/table.tsv CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite -- --nocapture
```

Exit 101 (5 of 9 groups red), at `b2ae5bd48`. The per-test rows of these groups in
`task-4/suite/tests.tsv` are byte-identical to the same groups' rows in
`docs/superpowers/records/2026-10-13-phase-9b/close2/suite/tests.tsv` (`diff` of the sorted rows,
exit 0). TESTRECLAIM01 is still red, Task 5's.

## Checks at `cd01d7fd6` (outputs in the session scratchpad, statuses read unpiped)

| command (from `rust/`) | exit | result |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings` | 0 | rexx-api, rexx-classes and rexx-bench touched first so they re-lint against the new `rexx-core` |
| `CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo test -j 4 --release -p rexx-exec --lib` | 0 | 1121 passed |
| `CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo test -j 4 --release -p rexx-core` | 0 | |
| `... cargo test -j 4 --release -p rexx-exec --test refusal_sites --test representation_refusals --test owners --test closed_phases --test phase_ten_reasons --test directive_options --test package_requires --test introspection_scopes` | 0 | |
| `... cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` | 0 | |
| `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo test -j 4 --release --no-fail-fast -p rexx-exec --test corpus --test collect_stress` | 0 | collect_stress 40 passed; corpus 29 passed 1 ignored, `1055 of 1055 matching` |

## Concerns

1. Resolved by the lead's ruling, see "L7's copy half" below.
2. **The downcast is a virtual `type_id` call** (`PackageState::downcast_ref`), a few Ir per package
   read. The hot reads are behind latches or take one body; a future reader on a per-clause path
   should take the body once.
3. **startup** is at +0.3219% against the pin.
4. Fixed in fix round 1 (I3), measured on the oracle.

## L7's copy half (ruling (a), copies unrooted)

Task 4 delivers this part of L7: a package copy nothing holds is collected, and a weak reference to
it clears. Nothing holds a copy by id, so no root replaces the one `copy_package` gave each copy.
Original packages stay global roots, as before; Task 5 keeps them.

Oracle first, from a fresh empty directory, the program of the new test with a loop of
`z = copies('y', 100000); q = .array~new(100)` in place of the filler: at 100 and 300 passes rc 0,
`dropped 1` / `held 1` (at 30 passes `dropped 0`: the oracle had not collected yet, and when it
collects is not specified). The crate's release binary at those counts answers `dropped 0`, its
collector not yet due. That timing difference falls under the GC-ordering licence, so the witness is
a `collect_stress` test, as with `corpus/lang/weak_reference_value.rex`'s clearing half:
`a_dropped_package_copy_is_collected` asserts `dropped 1` / `held 1` under
collect-on-every-allocation. Mutation: a global root added for each copy in
`object_protocol.rs`'s package arm turns it red; restored from a saved copy, `git diff --stat` empty.

The commands, from `rust/`, all exit 0: `cargo fmt --all --check`;
`CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`;
`REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo test -j 4 --release -p rexx-exec --test collect_stress`,
41 passed.

## Fix round 1

Commit `5194b8428`. The review is `task-4-review.md`.

**I1. A REXX package copy's added `Routine` object.** I followed the oracle's model, in which a
package's routines table holds whatever `~addRoutine` filed. `build_package_string_table` now
builds a table for a package with no program when a write asks for one, so the copy's `.ROUTINES`
table holds the object and the body's `tables` trace reaches it. `routines` stays untraced, and its
comment now states the invariant this rests on. I did not take the alternative, tracing `routines`:
that would also hold an entry a written `.ROUTINES` table has replaced, which is the shape
`an_added_public_routine_object_survives_its_table_entry` exists to keep untraced.

**I2. Lookups on a REXX package copy.** `package_public_class_of` for a package with no program now
reads its own public classes, then the registry, then what it merged. That matches the oracle,
where the copy's installed public classes (the REXX package's, copied) come before
`mergedPublicClasses`. `installed_class_of` keeps its own classes and then the registry.

**I3. A merge from a REXX package copy.** I read the oracle's rule in `PackageClass::mergeRequired`
(`classes/PackageClass.cpp:693-745`). The source's `installedPublicClasses` merges first, then its
`mergedPublicClasses`, and each `merge` adds only names the target lacks. `addInstalledClass` uses
`setEntry` (`:1401-1419`), so a redefined name replaces the old one within the source's own table.
`package_class_table` now builds that one table: the registry's entries minus every name the
package's own fields hold, then the package's own. `merge_package`'s first-wins order then takes
the copy's class.

The oracle was measured first, from fresh empty directories. The reviewer's three probes give
`weak 1` / `found Routine 1 0`, `importedClasses 1 1` / `findClass 1 1`, and `List` / `1 1`; the
crate agrees on all three. Witness `rust/corpus/lang/package_rexx_copy_lookups.rex` (listed in
`corpus/phase-9.txt`, with its sourceline expectation from the module driver). On the oracle it
gives rc 0, empty stderr, and:

```
imported 1 1
find 1 1
own first 1 1
redefined 1 1 1 1
importer List 1 1
original 1 0
```

The crate at `5194b8428` matches it byte for byte. The binary before the fix round
(`rexx-run-9b1-t4f`) answers `find 0 0` and `importer Array 1 0`.

**I4 and I1 witnesses** (`tests/collect_stress.rs`, under collect-on-every-allocation; each program
was measured on the oracle with a 3000-pass `copies('y', 100000)` loop in place of the filler, rc 0,
same lines):

| test | expects | goes red when |
|---|---|---|
| `a_rexx_package_copys_added_routine_survives` | `weak 1` / `found Routine 1 0` | a package with no program builds no table (I1 reverted) |
| `a_package_copy_held_as_an_import_survives` | `weak 1 1` / `import 1 1` / `ns 1` | `imports` untraced |
| `a_package_copy_held_as_a_parent_survives` | `weak 1` / `parent jmark` | `parent` untraced |

Each mutant was applied by an exact edit and restored from a saved copy, and `cmp` shows the file
identical afterwards. The parent test's copy is a copy of a source package that declares the
routine. My first version followed the reviewer's probe and filed the routine with `~addRoutine` on
the copy, and it stayed green with the parent trace deleted: after `j~addRoutine(name,
.routine~new(...))` the copy survives being dropped, on the crate and also on the oracle (`weak 1`
after 3000 passes). So that shape cannot isolate the parent trace.

**Minor.** `is_rexx_package` returns `Result`. A value that is not a package is refused as `a
message send to a value that is not a package`, not read as the REXX package.
`package_find_public_class` propagates the refusal.

**Checks at `5194b8428`**, from `rust/`, statuses read unpiped: `cargo fmt --all --check` 0;
`CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo clippy -j 4 -p rexx-core -p rexx-exec --all-targets -- -D warnings`
0; `... cargo test -j 4 --release -p rexx-exec --lib` 0 (1121 passed); the refusal, owners, phase
and package test binaries 0; `... -p rexx-parse --test sourceline_oracle` 0;
`REXX_CORPUS_GATE=1 ... cargo test -j 4 --release --no-fail-fast -p rexx-exec --test corpus --test collect_stress`
0 (collect_stress 44 passed; corpus `1056 of 1056 matching`, 29 passed, 1 ignored).

**Budget**: `timeout 590 bench-programs/callgrind.sh -B 0.5 -j 12 -p "startup pkglocal" -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-4-fix1 pin=.../rexx-run head=.../rexx-run-9b1-t4g`
(built at `5194b8428`), exit 0, three rounds. Records are in `task-4/fix1-callgrind/`.

| program | whole d% | -libc d% |
|---|---|---|
| startup | +0.3229 | +0.4476 |
| pkglocal | -6.8709 | -9.0802 |
