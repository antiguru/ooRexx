# Coverage spike: which oracle branches the class-construction witnesses never reach

Spike for Phase 9a Tasks 8 and 8b, 2026-10-12. Answer only: nothing in `rust/` or in
`/home/moritz/dev/repos/ooRexx` was touched, and no crate was built.

## 1. Source and build

* **Source.** `git -C /home/moritz/dev/repos/ooRexx archive 4b66b3558 | tar -x` into
  `/tmp/claude-1000/p9/cov/src`, then the oracle checkout's working-tree diff
  (`git -C /home/moritz/dev/repos/ooRexx diff`, `NumberStringClass.cpp` only) applied on top.
  `diff -rq` of the extracted `interpreter/` against the checkout's `interpreter/` printed nothing.
  This is the closest source to the binary in use: its `librexx.so` was linked at 2026-08-05 16:02:20,
  six seconds after `NumberStringClass.cpp` was last written (16:02:14), so the binary includes that
  edit. Its `rexx -v` says `Build date: Jul 30 2026`, the compile-time stamp of a file not recompiled
  since; the incremental build is consistent with this source but not proven equal to it. Section 1's
  validation is the evidence that they agree.
* **Build.** Out of tree in `/tmp/claude-1000/p9/cov/build`:
  `memcap 8G cmake <src> -DCMAKE_BUILD_TYPE=Debug -DCMAKE_C_COMPILER=/usr/bin/cc -DCMAKE_CXX_COMPILER=/usr/bin/c++ -DCMAKE_C_FLAGS="-O0 --coverage" -DCMAKE_CXX_FLAGS="-O0 --coverage" -DCMAKE_C_FLAGS_DEBUG="-g -O0" -DCMAKE_CXX_FLAGS_DEBUG="-g -O0" -DCMAKE_EXE_LINKER_FLAGS=--coverage -DCMAKE_SHARED_LINKER_FLAGS=--coverage -DCMAKE_MODULE_LINKER_FLAGS=--coverage`,
  then `cmake -DORX_ENABLE_IPO=OFF .`, then `memcap 8G make -j4 rexx` (31 s) and `memcap 8G make -j4` (5 s).
  gcc and gcov 16.2.0.
* **Counters.** Building `rexx.img` runs the coverage interpreter, so the build tree's own `.gcda`
  files hold image-build counts and were never read. Every measured run set
  `GCOV_PREFIX=/tmp/claude-1000/p9/cov/gcda/<phase>` and `GCOV_PREFIX_STRIP=0`, so each phase wrote
  its own tree. Merged trees, from `/tmp/claude-1000/p9/cov/gcda` with
  `R=tmp/claude-1000/p9/cov/build/CMakeFiles/rexx.dir`:
  `gcov-tool merge -o m_wg witnesses/$R groups/$R` and `gcov-tool merge -o all m_wg probes/$R`.
  Checked: `RexxClass::enhanced` is called 21, 10 and 43 times in the witness, group and probe trees
  and 74 in `all`.
* **Runs.** Each program ran on the oracle (`/home/moritz/dev/repos/ooRexx/build`) and then on the
  coverage build, from the same freshly emptied directory `/tmp/claude-1000/p9/cov/runs/<phase>/<name>`
  holding a copy of the program and its `.d` fixtures, under `RLIMIT_AS` 1 GiB, stdin `/dev/null`,
  deadline 20 s on the oracle and 60 s on the coverage build (300 s and 900 s for groups), with
  `LD_LIBRARY_PATH` and `PATH` naming the side's own `lib` and `bin`. ooTest groups replicate
  `group_runner.rs`'s `fresh_copy` (framework, the group's directory, `testOORexx.rex`, `worker.rex`,
  `ooTest.frm` with its `rxfuncquery` lines removed, `rxregexp.cls`) and run
  `testOORexx.rex -f <group file> -U -V 2`, one run per whole group rather than one per test.
  `grep -niE 'rxqueue|rxfuncadd|rxfuncdrop|rxfuncquery|\.rexxqueue|rexxmacro'` over the group
  files printed nothing, so no test was excluded. `pgrep -a rxapi` showed the same single daemon
  (pid 14596) before and after every phase; the coverage build used it and started none.
* **Inputs.**
  * (a) witnesses: every path after the `# 9a Task 8` comment of `rust/corpus/phase-9.txt`, plus every
    `rust/corpus/lang/*.rex` whose name matches `subclass|enhanc|copy|metaclass|(^|_)new(_|\.rex)`
    (which leaves out `no_trailing_newline` and `parse_error_newfile`), deduplicated with `sort -u`.
    `wc -l` of the list gives 75.
  * (b) the groups the brief names: `base/class/` Class, Object, String/String, Stem, Method, Routine, Message.
  * (c) `ls /tmp/claude-1000/p9/scout-enh/probes/p*.rex`, which lists 22 files. The `mem*` and `sub*`
    files were left out: they repeat `p*` shapes in loops and the largest takes 1:22 on the `-O2` oracle.
  * a baseline `say 1`, to separate interpreter start-up from what a witness reaches.
* **Kept.** The scripts (`run.py` runs both sides and compares; `gcovsum.py` runs gcov; `funcs.py`,
  `view.py` and `gain.py` read `.gcov` files), the input lists, the reachability probes and the
  per-phase result tables are copied to
  `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/coverage-spike/`.
  `/tmp/claude-1000/p9/cov` itself was deleted after this file was written (section 5).

### Validation

Every program on both sides, stdout, stderr and exit status compared byte for byte; group stdout with
the `File search:`, `Suite construction:`, `Test execution:` and `Total time:` values masked, as
`group_runner.rs` masks them. Each group's stdout differed in exactly one further line,
`Interpreter: REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026` against `... 12 Oct 2026`, the build-date
stamp; with that line also removed every group agrees, and their stderr is identical. Nothing else
differed, so no measured input shows the sources apart. `confirm*` rows are the reachability probes
of section 3; the `confirm` row for `c09` is a first version of that probe, `confirm2` the second.
Group summaries on the oracle: Class 103 tests 314 assertions, Object 149/334, String 2/20,
Stem 52/253, Method 56/103, Routine 60/99, Message 68/279, every one 0 failures and 0 errors.

| phase | program | verdict | oracle rc | coverage rc |
|---|---|---|---|---|
| baseline | baseline.rex | identical | 0 | 0 |
| witnesses | class_abstract_metaclass_after_inherit.rex | identical | 158 | 158 |
| witnesses | class_abstract_metaclass.rex | identical | 158 | 158 |
| witnesses | class_abstract_metaclass_subclass.rex | identical | 158 | 158 |
| witnesses | class_behaviour_snapshot_subclass.rex | identical | 0 | 0 |
| witnesses | class_metaclass_class_method_does_not_donate.rex | identical | 159 | 159 |
| witnesses | class_metaclass_cycle.rex | identical | 158 | 158 |
| witnesses | class_metaclass_not_a_metaclass.rex | identical | 157 | 157 |
| witnesses | class_metaclass_not_found.rex | identical | 158 | 158 |
| witnesses | class_metaclass.rex | identical | 0 | 0 |
| witnesses | class_metaclass_superclass_wins.rex | identical | 159 | 159 |
| witnesses | class_new.rex | identical | 0 | 0 |
| witnesses | class_subclass_cycle.rex | identical | 158 | 158 |
| witnesses | class_subclass_factory.rex | identical | 0 | 0 |
| witnesses | class_subclass_not_found.rex | identical | 158 | 158 |
| witnesses | class_subclass_refusals.rex | identical | 168 | 168 |
| witnesses | class_subclass.rex | identical | 0 | 0 |
| witnesses | class_target_not_a_class.rex | identical | 157 | 157 |
| witnesses | collection_copy.rex | identical | 0 | 0 |
| witnesses | collection_subclasses.rex | identical | 0 | 0 |
| witnesses | condition_object_copy.rex | identical | 0 | 0 |
| witnesses | context_executable_enhanced.rex | identical | 0 | 0 |
| witnesses | context_executable_routine_new.rex | identical | 0 | 0 |
| witnesses | context_executable_run_copy.rex | identical | 0 | 0 |
| witnesses | context_executable_setmethod_copy.rex | identical | 0 | 0 |
| witnesses | define_native_array_subclass.rex | identical | 0 | 0 |
| witnesses | do_with_supplier_subclass.rex | identical | 0 | 0 |
| witnesses | enhanced_constant_method.rex | identical | 0 | 0 |
| witnesses | enhanced_native_method.rex | identical | 0 | 0 |
| witnesses | enhanced_primitive_receivers.rex | identical | 0 | 0 |
| witnesses | enhanced_runs_new.rex | identical | 0 | 0 |
| witnesses | enhanced_scope.rex | identical | 0 | 0 |
| witnesses | enhanced_unset.rex | identical | 0 | 0 |
| witnesses | executable_copies.rex | identical | 0 | 0 |
| witnesses | executable_subclass_new.rex | identical | 0 | 0 |
| witnesses | external_directive_subclass.rex | identical | 0 | 0 |
| witnesses | hash_subclass_init_capacity.rex | identical | 0 | 0 |
| witnesses | message_new_arguments.rex | identical | 0 | 0 |
| witnesses | message_subclass_new.rex | identical | 0 | 0 |
| witnesses | metaclass_new.rex | identical | 0 | 0 |
| witnesses | method_new_bad_types.rex | identical | 0 | 0 |
| witnesses | method_new_object_source.rex | identical | 0 | 0 |
| witnesses | method_new.rex | identical | 0 | 0 |
| witnesses | mixinclass_enhancing_source.rex | identical | 0 | 0 |
| witnesses | new_init_subclasses.rex | identical | 214 | 214 |
| witnesses | object_copy_class_refusal.rex | identical | 163 | 163 |
| witnesses | object_copy_method.rex | identical | 0 | 0 |
| witnesses | object_copy.rex | identical | 0 | 0 |
| witnesses | object_name_set_values.rex | identical | 0 | 0 |
| witnesses | package_new_directive_subclass.rex | identical | 0 | 0 |
| witnesses | package_new_prologue_raise.rex | identical | 213 | 213 |
| witnesses | package_store_tables.rex | identical | 0 | 0 |
| witnesses | package_subclass_new.rex | identical | 0 | 0 |
| witnesses | parse_error_method_new.rex | identical | 246 | 246 |
| witnesses | parse_error_package_new.rex | identical | 221 | 221 |
| witnesses | pointer_buffer_new_refused.rex | identical | 163 | 163 |
| witnesses | rexx_context_new.rex | identical | 163 | 163 |
| witnesses | routine_new_context.rex | identical | 0 | 0 |
| witnesses | routine_new_directive_subclass.rex | identical | 0 | 0 |
| witnesses | routine_new_object_source.rex | identical | 0 | 0 |
| witnesses | routine_new_package_parent.rex | identical | 0 | 0 |
| witnesses | routine_new_parent_required.rex | identical | 0 | 0 |
| witnesses | routines_table_resolution.rex | identical | 0 | 0 |
| witnesses | run_native_items_array_subclass.rex | identical | 0 | 0 |
| witnesses | send_new_init_fails.rex | identical | 214 | 214 |
| witnesses | setmethod_copy_private.rex | identical | 0 | 0 |
| witnesses | setmethod_native_array_subclass.rex | identical | 0 | 0 |
| witnesses | stack_frame_new.rex | identical | 163 | 163 |
| witnesses | stem_subclass_new.rex | identical | 0 | 0 |
| witnesses | string_copy.rex | identical | 0 | 0 |
| witnesses | string_subclass_new.rex | identical | 0 | 0 |
| witnesses | subclass_enhancing_expose.rex | identical | 0 | 0 |
| witnesses | subclass_enhancing_native.rex | identical | 0 | 0 |
| witnesses | subclass_enhancing_source.rex | identical | 0 | 0 |
| witnesses | variable_reference_new.rex | identical | 163 | 163 |
| witnesses | variable_reference_subclass_new.rex | identical | 163 | 163 |
| probes | p10.rex | identical | 0 | 0 |
| probes | p11.rex | identical | 0 | 0 |
| probes | p12.rex | identical | 0 | 0 |
| probes | p13.rex | identical | 0 | 0 |
| probes | p14.rex | identical | 0 | 0 |
| probes | p15.rex | identical | 159 | 159 |
| probes | p16.rex | identical | 159 | 159 |
| probes | p1.rex | identical | 165 | 165 |
| probes | p2.rex | identical | 159 | 159 |
| probes | p3.rex | identical | 159 | 159 |
| probes | p4.rex | identical | 0 | 0 |
| probes | p5b.rex | identical | 0 | 0 |
| probes | p5.rex | identical | 163 | 163 |
| probes | p6.rex | identical | 0 | 0 |
| probes | p7b.rex | identical | 0 | 0 |
| probes | p7.rex | identical | 0 | 0 |
| probes | p8.rex | identical | 0 | 0 |
| probes | p9a.rex | identical | 0 | 0 |
| probes | p9b.rex | identical | 0 | 0 |
| probes | p9c.rex | identical | 0 | 0 |
| probes | p9d.rex | identical | 221 | 221 |
| probes | p9e.rex | identical | 0 | 0 |
| groups | Class | identical after masking the `Interpreter:` build-date line | 0 | 0 |
| groups | Object | identical after masking the `Interpreter:` build-date line | 0 | 0 |
| groups | String | identical after masking the `Interpreter:` build-date line | 0 | 0 |
| groups | Stem | identical after masking the `Interpreter:` build-date line | 0 | 0 |
| groups | Method | identical after masking the `Interpreter:` build-date line | 0 | 0 |
| groups | Routine | identical after masking the `Interpreter:` build-date line | 0 | 0 |
| groups | Message | identical after masking the `Interpreter:` build-date line | 0 | 0 |
| confirm | c01_super_skips_hidden.rex | identical | 0 | 0 |
| confirm | c02_methods_skip_hidden.rex | identical | 0 | 0 |
| confirm | c03_inherit_class_mixin.rex | identical | 0 | 0 |
| confirm | c04_inherit_position.rex | identical | 0 | 0 |
| confirm | c05_uninherit_not_mixin.rex | identical | 0 | 0 |
| confirm | c06_defaultname_no_value.rex | identical | 0 | 0 |
| confirm | c07_define_nil.rex | identical | 0 | 0 |
| confirm | c08_delete_absent.rex | identical | 0 | 0 |
| confirm | c09_primitive_class_new.rex | identical | 0 | 0 |
| confirm | c10_package_new_no_source.rex | identical | 0 | 0 |
| confirm | c11_package_new_context.rex | identical | 0 | 0 |
| confirm | c12_package_new_bad_source.rex | identical | 0 | 0 |
| confirm | c13_package_copy.rex | identical | 0 | 0 |
| confirm | c14_metaclass_uninit_new.rex | identical | 0 | 0 |
| confirm2 | c09_primitive_class_new.rex | identical | 0 | 0 |
| confirm3 | c15_package_new_omitted_context.rex | identical | 0 | 0 |

## 2. Branch coverage per file

Command per file and configuration, run in `/tmp/claude-1000/p9/cov/out/<config>` after copying the
build's `<file>.gcno` beside the configuration's `<file>.gcda`:
`gcov -b -c -m <gcda tree>/interpreter/<dir>/<file>.cpp.gcda`. The gcov columns are its
`Branches executed` and `Taken at least once` lines. At `-O0` gcov also counts every exception edge
(`(throw)` in the `.gcov`) and the edges after a call that never returns, so the last column
recounts from the `.gcov` file with `(throw)` edges dropped (`funcs.py --table`): branch outcomes
taken at least once over outcomes in the file.

| file (`interpreter/`) | witnesses: executed / taken (of N) | witnesses + groups + probes: executed / taken | non-throw taken, before -> after |
|---|---|---|---|
| `classes/ClassClass.cpp` | 80.68% / 48.31% of 590 | 84.07% / 56.61% | 275/439 -> 314/439 |
| `classes/ObjectClass.cpp` | 55.43% / 32.14% of 949 | 67.76% / 44.36% | 295/642 -> 377/642 |
| `classes/StringClass.cpp` | 62.18% / 41.97% of 386 | 71.50% / 49.74% | 160/355 -> 188/355 |
| `behaviour/RexxBehaviour.cpp` | 40.79% / 30.92% of 152 | 40.79% / 30.92% | 47/125 -> 47/125 |
| `behaviour/MethodDictionary.cpp` | 75.19% / 45.11% of 266 | 77.44% / 52.26% | 120/195 -> 139/195 |
| `classes/StemClass.cpp` | 38.07% / 25.38% of 394 | 41.12% / 29.44% | 98/308 -> 112/308 |
| `classes/MethodClass.cpp` | 73.79% / 43.45% of 145 | 83.45% / 57.24% | 60/114 -> 75/114 |
| `classes/RoutineClass.cpp` | 45.45% / 26.14% of 176 | 77.27% / 47.73% | 42/112 -> 75/112 |
| `classes/MessageClass.cpp` | 39.84% / 24.61% of 256 | 84.38% / 58.98% | 58/214 -> 142/214 |
| `classes/PackageClass.cpp` | 32.53% / 21.97% of 1156 | 41.52% / 28.89% | 249/926 -> 329/926 |

The baseline `say 1` alone gives ClassClass 4.07% / 2.54%, ObjectClass 13.28% / 7.06%,
StringClass 31.61% / 20.73%, RexxBehaviour 25.00% / 15.13%: start-up is a small share of the
class-construction files.

**The class-construction functions.** File totals mostly measure methods outside this area. Restricted
to `RexxClass::{copyRexx, defineMethodsRexx, defineMethod, deleteMethod, updateSubClasses,
updateInstanceSubClasses, createClassBehaviour, createInstanceBehaviour, checkUninit,
createMethodDictionary, inherit, uninherit, enhanced, mixinClass, subclass, newRexx,
completeNewObject, processNewArgs, addSubClass}`, `RexxObject::{copy, copyObjectVariables, setMethod,
unsetMethod, newRexx, defineInstanceMethod, deleteInstanceMethod, checkUninit, instanceMethod,
objectName, defaultName}`, `RexxBehaviour::{copy, addInstanceMethod, superScope, addScope,
mergeMethodDictionary, hasInstanceMethods}`, `MethodDictionary::{addMethod, addMethods,
addInstanceMethod, addInstanceMethods, removeInstanceMethod, replaceMethods, findSuperMethod,
getMethods, copy}`, every `::newRexx` of the files in the table above, `MethodClass::newMethodObject` and
`PackageClass::copy` (the regex passed to `funcs.py` names exactly these):
**non-throw outcomes taken 438 of 590 by the witnesses, 490 of 590 with groups and probes.**

Among what the groups add over the witnesses in those functions (`gain.py witnesses all`), each a
behaviour **no corpus witness reaches**:
* `Class~delete` is never called by a witness (`RexxClass::deleteMethod` 0 calls in the witness tree, 10 in `all`).
* `Class~methods` is never called by a witness (`RexxClass::methods`, `ClassClass.cpp:1018`).
* `Class~define` with the source omitted (hide), and with name `UNINIT` (`ClassClass.cpp:840`-`856`).
* `inherit` and `uninherit` refusals on a Rexx-defined class, recursive inherit (self, already inherited, mixin inheriting the receiver),
  inherit with a position that is not a superclass (98.945), uninherit of a non-inherited mixin,
  and an inherited mixin with an UNINIT (`ClassClass.cpp:1290`-`1366`, `:1382`-`:1401`).
* `setMethod` with the method omitted, with option `OBJECT`, and with an invalid option
  (`ObjectClass.cpp:1840`-`1870`).
* `instanceMethods(scope)` listing filtered by scope (`MethodDictionary.cpp:522`-`549`),
  `setMethod` of a name the object already set (`:393`-`:395`, the old entry is removed), and
  `unsetMethod` on an object that never had a `setMethod` (`:362`).

What the scout probes add over witnesses and groups together (`gain.py m_wg all`, run on
`ClassClass.cpp`, `ObjectClass.cpp`, `MethodDictionary.cpp` and `StringClass.cpp`): `.Thing~enhanced`
with no argument (`ClassClass.cpp:1443`-`1445`, 93.901), and `RexxString::numberValue` on a String
that is not of the base class (`StringClass.cpp:411`-`422`, arithmetic on an enhanced String).
Nothing else in those files.

## 3. Unreached branches in the class-construction functions

The residue after `all`, from `funcs.py all/<file>.gcov '<selection>'`: every unexecuted line and
every untaken non-throw outcome in the selected functions, each classified. "Confirmed" means a probe
was run on both builds, both agreed (validation table, `confirm*` rows), and the coverage build's
counters (phases `confirm`, `confirm2`, `confirm3`) show the line executed. The probe files are kept
under the names given.

### Observable from Rexx

| site | behaviour | probe (oracle output) | status |
|---|---|---|---|
| `behaviour/MethodDictionary.cpp:453` | super lookup steps over a hidden (`.nil`) entry in an intermediate class and finds the method further up | `c01`: A defines FOO, B=A~subclass hides FOO with `~define('FOO')`, C=B~subclass defines FOO as `return 'C then' self~foo:super`; `say c~new~foo` prints `C then A foo`, while `b~new~hasMethod('FOO')` is `0` | confirmed |
| `behaviour/MethodDictionary.cpp:519`, `:542` | `~methods` and `~instanceMethods(scope)` skip a hidden entry | `c02`: `b~define('FOO','return 1'); b~define('BAR')`; both listings print only `FOO` | confirmed |
| `classes/PackageClass.cpp:362`-`370` | `Package~copy` answers a deep copy | `c13`: `q = .context~package~copy; say q~class (q~name == p~name) (q == p)` prints `The Package class 1 0` | confirmed |
| `classes/PackageClass.cpp:180`-`186` | `.Package~new(name)` with no source loads the named file as a `::REQUIRES` would, prolog included | `c10`: writes `pk.rex` (a `say` prolog and a public routine), `.package~new('pk.rex')` prints `prolog of pk`, then the routine is in `~routines` | confirmed |
| `classes/PackageClass.cpp:202`-`204` | third `NEW` argument a Method or Routine: its package becomes the parent context | `c11`: `.package~new('x', 'say 1', .routine~new('r','return 1'))` runs, answers a Package | confirmed |
| `classes/PackageClass.cpp:212` | third argument of any other class | `c11`: `93.953 Method argument 3 could not be converted to type Method, Routine, or Package object.` | confirmed |
| `classes/PackageClass.cpp:200` | third argument omitted with more arguments after it: they go to INIT | `c15`: `.P~new('x', 'nop', , 'extra')` on a Package subclass, INIT sees `1 extra` | confirmed |
| `classes/PackageClass.cpp:222`-`224` | source neither string nor array | `c12`: `.package~new('x', .object~new)` gives `93.961 The source argument must be a string or array object.` | confirmed |
| `classes/ClassClass.cpp:1348`-`1353` | `inherit(mixin, position)` with `position` a current superclass inserts after it | `c04`: `c~inherit(m1); c~inherit(m2, m1)`; `c~superclasses` prints `Object M2 M1`, `c~new~who` answers `m2` | confirmed |
| `classes/ClassClass.cpp:1329`-`1331` | inherit of a mixin whose base class the receiver's instances lack (the class behaviour has it) | `c03`: `.object~subclass('C')~inherit(.class~mixinclass('M'))` gives `98.943 Class "The C class" is not a subclass of "The M class" base class "The Class class".` | confirmed |
| `classes/ClassClass.cpp:1391`-`1393` | `uninherit` of a class that is not a mixin | `c05`: `c~uninherit(.object~subclass('Z'))` gives `98.942 Class "The Z class" must be a MIXINCLASS for INHERIT.` | confirmed |
| `classes/ObjectClass.cpp:1716`-`1718` | `objectName` (and so `string`) when DEFAULTNAME answers no value falls back to the built-in default name | `c06`: `::method defaultName` with a bare `return`; `objectName` and `string` both print `a T` | confirmed |
| `classes/ClassClass.cpp:846` | `Class~define(name, .nil)` takes the explicit-`.nil` arm (same result as omitted) | `c07`: define FOO then `define('FOO', .nil)`; `hasMethod('FOO')` is `0` | confirmed |
| `classes/ClassClass.cpp:966` | `Class~delete` of a name never defined: no error, no subclass update | `c08`: `c~delete('NOPE')` then `ok 0` | confirmed |
| `classes/ClassClass.cpp:1806`-`1809` | `Class`'s NEW run with a builtin (primitive) class as receiver gives the new class metaclass `.Class` | not reached by either attempt: a `Message` with scope `.Class` (`c09` first version) gave 93.902; `.class~define('MK', .class~instanceMethod('NEW'))` (`c09`, `confirm2`) gave `98.985 User additions are not allowed to the REXX language classes.` | not reached; no Rexx route found, not shown unreachable |

### Not observable from Rexx

| site | why |
|---|---|
| `ClassClass.cpp:1525`-`1527`, `:1634` outcome "parent has UNINIT", `:1364` same | `PARENT_HAS_UNINIT` is written here and read only by these conditions (`grep -rn 'parentHasUninitDefined\|PARENT_HAS_UNINIT'` over `interpreter/` lists nothing else); `completeNewObject` reads `HAS_UNINIT`, which `checkUninit` sets from the method lookup |
| `ClassClass.cpp:1839`-`1841` | reachable (`c14`, a metaclass with an UNINIT method, `.Meta~new('K')`, confirmed) but it sets a flag `setInitialFlagState` has just preserved as set: no effect of its own |
| `ClassClass.cpp:1117`, `:1165` | the "scope already merged" arm of `createClassBehaviour` / `createInstanceBehaviour` entry; the loop above already skips scopes present, so this needs a class to recurse into itself; no route found |
| `ClassClass.cpp:485`, `:1802`, `:1822`, `ObjectClass.cpp:2638`, `MessageClass.cpp:894`, `MethodClass.cpp:595` | outcomes on `new X(...)` lines: gcov's allocation and constructor edges, not source conditions |
| `ClassClass.cpp:170`, `:524`, `:825`, `:1445`, `:1574`, `:1781`, `ObjectClass.cpp:1870`, `MessageClass.cpp:841`, `:868`, `:872`, `RoutineClass.cpp:527`, `:537`, `:539`, `MethodClass.cpp:585`, `:600`, `:602` | the edge after `reportException`, which never returns; the raising line itself ran |
| `ObjectClass.cpp:1720` | closing brace of an `if` block; no source condition |
| `RexxBehaviour.cpp:404`-`406`, `:632`-`634`, `MethodDictionary.cpp:439`-`441`, `:573`-`581` | null or empty dictionary guards; every runtime behaviour has a dictionary |
| `MethodDictionary.cpp:168` (`method == OREF_NULL`), `:318` | every runtime caller passes `.nil` rather than NULL, and directive method tables hold only methods |
| `MethodDictionary.cpp:251`-`297` (`replaceMethods` overloads), `RexxClass::{defineClassMethod, removeClassMethod, removeSetupMethods, inheritInstanceMethods, buildFinalClassBehaviour, mergeBehaviour, createInstance}`, `MethodDictionary::{hideMethod, setMethodScope}` | image build only: callers are `memory/Setup.cpp`'s macros or nothing (`grep -rnE '(->\|\.)(replaceMethods\|hideMethod\|setMethodScope\|...)\('`); `removeSetupMethods` deletes `DEFINECLASSMETHOD` before any program runs |
| `RexxObject::defineInstanceMethods`, `RexxClass::copyInstanceMethods` | no caller in `interpreter/` (`grep -rn`) |
| `MethodClass.cpp:478` | `newMethodObject` with no Rexx frame on the activity: native API only |
| `MessageClass.cpp:881` | `optionArgument(..., "AI", ...)` has already refused any option but `A` and `I` |

`StringClass::newRexx`, `StemClass::newRexx`, `MethodClass::newRexx`, `RoutineClass::newRexx`,
`RexxObject::copy`, `RexxObject::unsetMethod`, `RexxClass::enhanced` (but the edge after its
93.901), `completeNewObject` and `processNewArgs` have no unreached non-throw outcome in `all`.
`StringClass::newRexx` and `StemClass::newRexx` are fully reached by the witnesses alone.

## 4. Verdict on the method

* **Cost.** Configure at 17:39:50, report written by 17:56, plus reading the brief and the inputs
  before that. Build 36 s plus 7 s configure; every run of every phase under 2 minutes together; gcov
  under a second per file. Nearly all of the time is reading `.gcov` output, deciding which outcomes
  are source conditions, and writing probes. One person-hour is a fair estimate for a repeat on a new
  area, less with the kept scripts.
* **Signal.** The confirmed rows of section 3's first table, observable behaviours that no input
  reaches, each with a one-line probe and the oracle's answer already in hand; one unresolved row; and
  the corpus gaps of section 2 that the groups cover but no witness does.
  Most are adjacent to the tasks rather than inside them (`Class~inherit`, `Package~new`,
  `Package~copy`, super lookup past a hidden method); inside the tasks' own functions the witnesses
  are close to complete.
* **What it cannot see.** Branch coverage of the oracle says which oracle behaviours are exercised,
  not whether the crate matches them. The scout's probes reached only one oracle branch the witnesses
  and groups had not, yet `enhanced-semantics.md` records crate divergences in most of its rules: those
  divergences run on lines the corpus already executes. Paths that differ by data and not by branch
  (a NEW answering a foreign object, the count of INIT sends, a dummy class in `~subclasses`) look
  covered. And a gcov "branch" at `-O0` includes exception, allocation and never-returning-call edges,
  so the raw figures need the filtering above before they mean anything.
* **As a standing gate on spec scouts.** Fit it as a required section of the scout report, not a pass
  or fail threshold: the scout names the oracle functions its area touches, runs the corpus witnesses
  and the matching ooTest groups through the coverage build (kept scripts; the build is a one-time
  minute), and must classify every unreached non-throw outcome in those functions as observable (with
  a probe and the oracle's answer) or not (with the reason). Observable ones become witness rows or a
  stated exclusion in the spec. A percentage gate would reward the wrong thing: the gcov figures move
  with exception edges, and 100% is impossible where image-build helpers share the file.

## 5. Clean-up

`/tmp/claude-1000/p9/cov` (sources, build, counters, run directories) was deleted after this file
was written, with `rm -r /tmp/claude-1000/p9/cov`. The scripts, lists, probes and result tables are
in the scratchpad path of section 1.
