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

1. **A package copy is collectable now.** Nothing holds a copy by id, so the global root that kept
   copies alive went with `copy_package`. Measured: `w = .WeakReference~new(.context~package~copy)`
   then 3000 allocating passes answers `1` here, as on the oracle (Task 3's binary `0`);
   `.Package~new('src','nop')` still answers `0` (oracle `1`), Task 5's. I told the lead I would keep
   copies rooted unless told otherwise and did not: a root kept only to preserve the leak has no test
   that pins anything but the leak. Restoring it is a `RootSet::add_global` keyed by the copy's
   identity in `object_protocol.rs`'s package arm.
2. **The downcast is a virtual `type_id` call** (`PackageState::downcast_ref`), a few Ir per package
   read. The hot reads are behind latches or take one body; a future reader on a per-clause path
   should take the body once.
3. **startup** is at +0.3219% against the pin.
4. A `Package~addPublicClass` of a REXX-package class name on a copy of the REXX package: the copy's
   `~classes` answers the added class, but a merge from that copy keeps the REXX one (first entry
   wins). Not measured on the oracle.
