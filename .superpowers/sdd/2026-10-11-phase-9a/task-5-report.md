# Phase 9a Task 5 report: the `native_method` batch

Base `6a8c461fd`. Code commit `cc7ae4350`. Status: NEEDS_CONTEXT for one gate question (`method_bodies` and `introspection_arity` rows for a
path-valued answer, below); otherwise DONE_WITH_CONCERNS (items left as refusals under the
mechanical-batch rule, and the `samples/samples` finding).

Probe files are in `/tmp/claude-1000/p9/t5-probes/`, run by `/tmp/claude-1000/p9/t5-tools/both.sh`
(oracle under the standard wrapper from a fresh `mktemp -d`, the crate's `rexx-run` under
`memcap 2G timeout 20`; stdout, stderr and status kept apart). `oracle-crashes.txt` was read first;
no listed program was run. Every witness's stdout was read on both sides.

## Per item

### 1. `RexxInfo~executable`, `~libraryPath` (the `invocable` fall-through)

Probe `info`: oracle rc 0, prints `/home/moritz/dev/repos/ooRexx/build/bin/rexx`,
`/home/moritz/dev/repos/ooRexx/build/lib`, `The File class`. Ours before: rc 120, `method
"EXECUTABLE" of class "RexxInfo" is not implemented (Phase 9)`.

Implemented (`dispatch/rexx_info.rs`): `getRexxExecutable` is `.File~new` over
`current_exe()` canonicalised, `.nil` where that fails; `getRexxLibrary` is `.File~new` over the
executable's own directory plus a separator (this crate's runtime is linked into the executable).
Ours after: rc 0, prints the crate's own paths. `RexxInfo~copy` also added (93.970, scope
`RexxInfo`, item 3).

Witness `lang/rexxinfo_executable_file.rex`. **What is compared**: the paths differ by design, so
the witness prints only `~class`, `~isFile`, `~exists`, `~isDirectory` of each answer and the
`==`/`absolutePath` comparison of two `~executable` answers (`The File class 1 1 0`, `1 1`,
`The File class 1 1`); the brief's `~right(5)` suggestion does not hold because the crate's binary
is `rexx-run`, not `rexx`.

### 2. `StackFrame~executable` (the `invocable` fall-through)

Probes `frame`, `frame2`, `frame3`. Oracle answers for every frame kind; ours refused.

Implemented (`dispatch/context.rs`): a frame built from an activation stores `executable_at(depth)`;
an `INTERPRET` fragment frame (live or captured) stores its owning activation's; a `COMPILE` frame
stores `.nil` (`CompileActivationFrame::executable` is `OREF_NULL`). A native level's frame
(`build_native_level_frame`, `build_native_frame_at`) and a `capture_site_frame` frame store none, and
the reader refuses those with the same `native_method` text. Not mechanical for natives: probe `mid`
shows the oracle's native-frame executable is a `Method`/`Routine` whose `~source` is the calling
program's 19 lines and which is not `.string~method('SUBSTR')`, an identity this crate has no store
for. Building a frame never fails on an executable it cannot answer: the entry is left out.

Found and fixed on the way (a shared-path change, two lines): `executable_at` answered the program's
routine for an `INTERNALCALL` activation inside a `::METHOD` or `::ROUTINE`; the oracle answers the
caller's executable (`frame2`: `INTERNALCALL The Method class 1 1`; ours was `The Routine class 1 0`).
This was a pre-existing `.context~executable` defect, visible here because every frame now reads it.

Witness `lang/stackframe_executable.rex` (program, internal call, method, internal call in a method,
internal call in a `::ROUTINE`, INTERPRET live and captured, COMPILE via `~firstItem`, METHOD frames
of a trapped condition). Pin: `dispatch::tests::a_copy_or_frame_executable_this_crate_does_not_keep_is_loud`
(native frame refuses; its neighbour answers).

### 3. `Object~copy` (`object_protocol.rs`)

Probes `copy2`, `copy3`, `copy6`, `copy7`, `copy8`, `copy_context`, `copy_info`, `ann`:

| receiver | oracle | after |
|---|---|---|
| Method | rc 0, new object, same scope/source/package, own flags, own annotations | implemented |
| StackFrame | rc 0, new object, same type/line/context | implemented (native body clone) |
| RexxContext | rc 163, 93.970, scope `RexxContext` | implemented (`COPY` row on `RexxContext`) |
| RexxInfo | rc 163, 93.970, scope `RexxInfo` | implemented (`COPY` row on `RexxInfo`) |
| Routine | rc 0 | **left refused** |
| Package | rc 0 | **left refused** |

Method copy (`Interp::copy_method_object`, `environment/identities.rs`) reuses `newScope`'s clone
(now `clone_executable`), mints a method identity with `copy_method_identity`, and gives the copy a
fresh annotation table.

**Routine copy left refused (new identity rule).** Implemented and measured, then backed out: with
a copy alive, `.context~executable` inside `r~call` answered the copy for the original
(`copy7`: `(r~call == r)` oracle 1, ours 0) and the package's routine for a copy of a `::ROUTINE`
(`(e~call('x') == e)` oracle 1, ours 0). `routine_object_running` picks any live object over the
same directive; the fix is an activation carrying the `Routine` object it was called through
(`begin_routine` and `executable_at`'s Routine arm), which is a new identity rule.

**Package copy left refused (new store).** Probe `copy6`: the oracle's copy has its own `~local`
and its own routine tables (`q~addRoutine` leaves `p~routines~items` 1 against `q`'s 2;
`PackageClass::copy` deep-copies them). A package object here is its `ProgramId`; a copy needs
per-object tables.

Witness `lang/object_copy_method.rex` (Method copy fields, flags, annotations, define/setMethod/run
of a copy, `.method~new` copy, StackFrame copy, `.context~copy` 93.970); `rexxinfo_executable_file.rex`
carries `.rexxinfo~copy`. Pin for the left refusals: the dispatch test above.
`Loud::unknown_receiver`'s doc no longer says a `Method` copy refuses at `COPY`.

### 4. `OBJECTNAME=` (`object_protocol.rs:466`): **left refused (new store, new identity rule)**

Probe `objname`: oracle rc 0 for `'abc'`, `5`, `.object~new`, an Array, a Stem, a VariableReference;
and `'abc'~objectName` on a *later* literal answers `x`, since the oracle's literals are one object
per package. These receivers have no pool or name slot here; strings of 7 bytes or less and small
integers are values in the handle with no identity at all. A side table keyed by `ObjRef` is a new
store needing collector clearing, and for inline values it would key by value, which is a new identity
rule that diverges for computed strings. Pin unchanged
(`a_conversion_this_phase_does_not_model_is_loud_where_the_ones_it_models_answer`).

### 5. `MAKEARRAY` of an `~UNKNOWN` argument list (`array.rs:700`)

Probes `ua1`-`ua11`. Oracle: `StringHashCollection`, `StemClass` and `VariableReference`
`unknownRexx` all call `arrayArgument(arguments, ARG_TWO)`, i.e. `requestArray` then 98.913 for
`.nil` or a multi-dimensional answer. Ours: refused for anything not already an Array, and answered a
2-D array as if it were one (`ua7`: oracle 98.913, ours `The Array class`, a pre-existing wrong
answer).

Implemented: `array_argument` (`object_protocol.rs`) now goes through `request_array_value` (the
`requestArray` model `run/loops.rs` already had) and raises 98.913/88.913 for no conversion, `.nil`, a
multi-dimensional array, or an answer that is not an array; the three `~UNKNOWN` sites call it.
`unconverted_array_argument` and `lookup_for_refusal` are deleted. This changes the shared
`arrayArgument` path for its other callers (`sendWith`, `startWith`, `run`, `Message~new`,
`callWith`, `Supplier~new`): each mirrors a C++ `arrayArgument` call, and the refusal it replaces was
the only difference. An answer that is not an array is UB on the oracle: a string answer read 98.913
in 5 runs, a `List` answer is SIGSEGV 5 of 5 (`oracle-crashes.txt` entry 34, new); ours raises 98.913
for both.

Witness `lang/unknown_argument_list.rex` (string, MAKEARRAY instance, Stem, StringTable, `sendWith`,
no MAKEARRAY, `.nil`, 2-D array, MAKEARRAY answering `.nil`; the string-answer case is left out
because the oracle's answer there is an accident). Pin rewritten:
`dispatch::tests::an_unknown_sent_by_hand_converts_its_argument_list`.

### 6. A metaclass with its own `NEW` (`class_protocol.rs:774`): **left refused (shared path)**

Probes `meta_new`, `meta2`: oracle `RexxClass::subclass` sends `NEW` to the metaclass, takes the
class it answers (built by `Class~newRexx` as a subclass of `Object`, `INIT` sent once), then
re-parents it (superclass, instance behaviour, scopes) and sends `INIT` a second time (`meta2`:
`new j / init j / init j`). This crate builds classes in one step with the superclass fixed at
definition; `.MyMeta~new('k')` alone already agrees. Re-parenting an existing class needs a new
`ClassRegistry` operation, and crash entry 8 (a `NEW` answering a non-class) needs a guard on the
answer. Not mechanical; the refusal and its pin stay.

## Step 3: the unreached sites

**`SUPPLIER` in `~defineMethods` (`class_protocol.rs:577`): reached.** Probes `dm1`-`dm10`, both
sides: a Table, an Array, a user object answering a Supplier, a Relation, `~subclass`'s class-method
table, `~enhanced`, a Supplier subclass overriding `item`, duplicate names in a Table and a Relation
(last in supplier order wins; supplier order agrees with the oracle's), a `.nil` entry, an object
with no `SUPPLIER` (97.1 both). Oracle rc 0 for all but `dm7`/`dm10`; ours refused each before.

Implemented: `method_table_pairs` (string-keyed tables as before; anything else sends `SUPPLIER`
and walks it with `drain_supplier`, which reads the supplier's own state as
`createMethodDictionary`'s direct calls do, so a subclass's `item` override is not consulted, as
`dm8` shows on the oracle). `supplier_refusal` is deleted. A `SUPPLIER` answer that is not a
supplier is SIGSEGV on the oracle 5 of 5 (`dm7`, entry 35, new); ours refuses with
`Loud::receiver_class`, and that constructor's DEVIATION row now cites entry 35. Witness
`lang/define_methods_supplier.rex`.

**`Message~notify` (`object_protocol.rs:1220`): unreached.** Probes `nt1` (notify with a
`MessageNotification` subclass; again after removing `MESSAGENOTIFICATION` from `.environment` and
`.local`) and `nt2` (from a `::REQUIRES` prolog; a non-notifiable target, 88.914): identical on both
sides, rc 0. The site only fires before the REXX library has installed `MessageNotification`. It now
builds `Loud::environment_symbol(b".MESSAGENOTIFICATION", None)`, no phase, under that constructor's
existing GUARD row, whose record now names `nt1`, `nt2` and `info` (`.rexxinfo~executable`'s `.FILE`
lookup is the same shape).

## Refusal tables

`REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites` re-derived
`refusal-sites.tsv`. Diff beyond line-number shifts in `lib.rs`:

```
-Loud	environment_symbol	body	crates/rexx-exec/src/lib.rs:587	off-send-surface	-	-	-
+Loud	environment_symbol	body+send	crates/rexx-exec/src/lib.rs:586	agrees	no	no route: the send sites ask for .MESSAGENOTIFICATION and .FILE, REXX package classes the library bootstrap installs before any program runs	m~notify(.w~new) over ::class w inherit MessageNotification, after .environment~remove('MESSAGENOTIFICATION') and .local~remove('MESSAGENOTIFICATION') (Phase 9a Task 5 probe nt1), and .rexxinfo~executable (probe info): rc 0 on both
-Loud	native_method	send	...	m~dimensions, 'abc'~copy, .Message~new
+Loud	native_method	send	...	m~dimensions, .context~package~copy, .Message~new
```

(`'abc'~copy` answers and did before.) `refusal-dispositions.tsv`: `environment_symbol` record
extended, `receiver_class` record cites entry 35, header points at this report for Phase 9a Task 5
probe names. `oracle-crashes.txt` entries 34 and 35 added.

`native_method` construction sites after: `dispatch.rs` (`invocable`), `construct.rs:102`,
`class_protocol.rs` (metaclass `NEW`), `hash.rs:215`, `object_protocol.rs` (`OBJECTNAME=`, `COPY`),
`context.rs` (native-frame `EXECUTABLE`), from `grep -rn 'Loud::native_method' rust/crates/rexx-exec/src`.

## Group verdicts (suite harness)

```
REXX_SUITE_TABLE=/tmp/claude-1000/p9/t5-suite/table.tsv REXX_SUITE_GATE=1 \
REXX_SUITE_GROUPS=base/class/Message,base/class/Object,base/class/StackFrame,base/class/RexxContext,samples/samples,doc/rexxref/chapter5/Section1 \
CARGO_TARGET_DIR=/tmp/claude-1000/p9/t5-target CARGO_INCREMENTAL=0 memcap 8G cargo test --release -j 4 -p rexx-exec --test suite
```
Exit 101 (`suite: 6 groups, 5 red`), at `cc7ae4350`. Before is the Task 3 record
(`docs/superpowers/plans/phase-9a-gate.md`, run at `2415e1733`) and `suite-9d-set.txt`.

| group | before | after |
|---|---|---|
| `base/class/Message` | red: TEST_REPLYWITH_NOT_ARRAY refuses | **green** |
| `base/class/Object` | red: TEST_RUN_ARRAY_ARGUMENT refuses; TEST_INSTANCEMETHOD, TEST_RUN, TEST_SETMETHOD, TEST_SETMETHOD_SCOPE, TEST_UNINIT fail | red: the same five fail (all in `suite-9d-set.txt`); TEST_RUN_ARRAY_ARGUMENT passes |
| `base/class/RexxContext` | red: TESTCOPY01 refuses, TESTRS01 fails | red: TESTRS01 fails (in the 9d set); TESTCOPY01 passes |
| `base/class/StackFrame` | red: refusal `method "EXECUTABLE" of class "StackFrame"`; TESTNATIVECODEFRAME fails | red: TESTNATIVECODEFRAME fails (in the 9d set); TESTEXTERNALNATIVECODEFRAME blocked by the native-frame refusal (same text as before) |
| `doc/rexxref/chapter5/Section1` | red: TEST_OBJECT_OBJECTNAMEEQUALS refuses | unchanged (item 4) |
| `samples/samples` | blocked at load, `RexxInfo~executable` | red: loads; all 29 fail |

No test newly fails against the before record: every "not passed" test after is in the before
record's list or `suite-9d-set.txt`, except the 29 `samples/samples` tests, which were not run before.

**`samples/samples` cause, measured.** `FileUtils.cls`'s `locateSamplePrg` finds the samples relative
to `.RexxInfo~executable`'s directory (`../samples` from `bin/`), and `execRexxPrgWithArgs` runs
`.RexxInfo~executable`. The crate's executable is `rexx-run` in the cargo target directory (the
harness's `bin-rexx/rexx` is a symlink, which `canonicalize` resolves), and no `samples/` sits beside
it. Driving `execRexxPrg` by hand over the checkout's `samples/` (`/tmp/claude-1000/p9/t5-samplerun/
d2.rex`) runs `arithmeticEvaluation`, `arrayCallback`, `ccreply`, `constrained`, `delegation`,
`dynamicMethod`, `greply`, `month`, `qdate`, `qtime`, `stack`, `properties`, `usecomp` to rc 0 with no
stderr; `makestring` refuses `SYSTEMPFILENAME` (Phase 9) and `factor` refuses `NEW` of class
`Sample`. The answer itself is right (the running interpreter's path); the harness's layout is what
the group cannot use.

`concurrency_tests.rs`'s recorded `whole_groups` rows for `Message`, `Object` and `RexxContext`
still name the refusals these groups no longer reach; that table is a gate-only check re-derived at
the phase close and was not run or edited here.

## Checks (statuses read unpiped)

* `cargo fmt --all`: exit 0.
* `CARGO_TARGET_DIR=/tmp/claude-1000/p9/t5-target CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0 (log shows `Checking rexx-exec`).
* `... cargo test --release -j 4 -p rexx-exec --test refusal_sites --test refusal_dispositions --test closed_phases --test owners --test loud --test phase_ten_reasons`: exit 0.
* `... cargo test --release -j 4 -p rexx-exec --lib -- dispatch:: scheduler:: context`: exit 0, 238 passed (the new and the two rewritten tests among them, read in the log).
* Suite harness, above: exit 101, six groups judged.
* `CARGO_TARGET_DIR=/tmp/claude-1000/p9/t5-target CARGO_INCREMENTAL=0 memcap 8G cargo test --release -j 4 -p rexx-parse --test sourceline_oracle`: exit 0 (new witnesses' expectation files generated with the module comment's driver).
* `REXX_CORPUS_GATE=1 ... cargo test --release -j 4 -p rexx-exec --test corpus`: exit 0 (the phase-9.txt witnesses included).
* `... cargo test --release -j 4 -p rexx-exec --test coverage`: exit 0 (21 passed).
* `... cargo test --release -j 4 --no-fail-fast -p rexx-exec --test introspection_arity --test method_bodies`: **exit 101**, run with a temporary symlink `rust/target/release/rexx-run` -> this task's binary (the tests hardcode that path; the symlink and the empty directories were removed after). Failures:
  * `method_bodies::no_row_started_diverging_or_stopped_answering`: `RexxInfo executable (instance arm): loud -> diverge [diverge-stdout]` and the same for `libraryPath`. `regressed(Loud, Diverge)` is red in every mode and `REXX_METHOD_BODIES_REFRESH` will not write it.
  * `introspection_arity::the_table_matches_the_three_sides`: the two `send-differs` rows' evidence now names the crate's path instead of its refusal. `REXX_INTROSPECTION_ARITY_REFRESH=1` writes `crate rc0 VALUE /tmp/claude-1000/p9/t5-target/release/rexx-run` (and `.../release` for `libraryPath`), and the test is then green (exit 0, 26 passed); that refresh was **not** committed, because the evidence is the path of whichever binary ran.

## Gate question (NEEDS_CONTEXT)

`.RexxInfo~executable` and `~libraryPath` answer the running interpreter's own path on both sides,
so the zero-argument probe can never agree, and both tables record the path in their evidence. The
oracle's answer is right for this crate only as "its own path". Options, each a change to a gate's
rules rather than a mechanical step: (a) a per-row probe override in `method_bodies.rs` printing a
path-free property (`~class`, `~isFile`), and the same for `introspection_arity`; (b) a licensed
`diverge` verdict for path-valued rows, citing a Deviation; (c) normalise the two interpreters' paths
in the evidence. The code commit stands either way; the two tables are red at `cc7ae4350` until this
is ruled.

## Other findings (not this task's)

* `::annotate method m a='one'` raises 19.923 on both sides, but the crate's message reads
  `found "&1"` where the oracle substitutes `found "="` (probe `ann` before its fix).
* `RexxContext~executable` inside an internal call was wrong before this task (fixed, item 2).

## Concerns

1. Left refused, not mechanical: `OBJECTNAME=` on a value with no pool (new store, identity rule);
   a metaclass with its own `NEW` (new registry operation); Routine copy (activation carrying its
   called object); Package copy (per-object package tables); a native frame's `~executable`.
2. `samples/samples` now loads and fails all 29 tests because the crate's executable has no
   `../samples`; a harness layout question for the controller.
3. `array_argument`'s change reaches every `arrayArgument` caller, not only `~UNKNOWN`.
4. `whole_groups` rows for Message, Object, RexxContext are stale until the close gate re-derives them.
5. `method_bodies` and `introspection_arity` are red on the two `RexxInfo` rows (gate question above).

## Rulings applied

Commit `d2f89f559` on top of `cc7ae4350`.

1. **Path-valued rows.** `PATH_VALUED` in `tests/method_bodies.rs` and `tests/support/arity.rs`
   (new `program_rendering`, `rendering`): the `RexxInfo executable` and `libraryPath` probes print
   `~class ~exists` of the answer. Refreshed with `REXX_INTROSPECTION_ARITY_REFRESH=1
   REXX_METHOD_BODIES_REFRESH=1`: `method-bodies.txt` rows `loud` -> `answers rc 0`,
   `introspection-arity.tsv` rows `send-differs` -> `agree rc0`. Re-run without refresh: exit 0
   (26 and 23 passed). The gate record (`docs/superpowers/plans/phase-9a-gate.md`, "Task 5") says
   what is compared. Both tests hardcode `rust/target/release/rexx-run`; each run used a temporary
   symlink there to this task's binary, removed afterwards with its empty directories.
2. **Crate `bin` layout.** `interpreter_bin` (`tests/support/group_runner.rs`) hard-links
   `rexx-run` to `bin-rexx/rexx` (copy if linking fails; replaced when it is not the current
   binary's inode) and links `<run parent>/samples` to the oracle's `build/samples`. `check_path`
   (`tests/suite.rs`) now compares ELF files by content instead of canonical path; its unit test asserts the
   bytes match, that `rexx` does not resolve into the target directory, and that
   `../samples/factor.rex` exists. `samples/samples` through the harness
   (`REXX_SUITE_GROUPS=samples/samples`): exit 101, red, **27 of 29 pass**; TEST_FACTOR refuses
   `NEW` on a `String` subclass (`unbuilt_new`), TEST_MAKESTRING refuses `SysTempFileName` (Phase 9).
   The suite binary's other 49 tests passed.
3. **`whole_groups` rows.** Added `REXX_WHOLE_GROUPS_ONLY` (comma-separated group files) to
   `each_group_of_the_derived_list_in_one_run_in_both_modes` to run three groups alone. Measured
   (`REXX_CORPUS_GATE=1`, Message, Object, RexxContext): Message whole and derived agree with the
   oracle (rows removed); Object whole is `failure, assertions 252, rc 1, last started
   TEST_UNSETMETHOD_NO_ARG, failing [TEST_INSTANCEMETHOD TEST_RUN TEST_SETMETHOD
   TEST_SETMETHOD_SCOPE]` and RexxContext whole is `failure, assertions 358, rc 1, last started
   TEST_SOURCELINE_LAST_LINE, failing [TESTRS01]` (rows rewritten); their `rest` rows are gone, since
   nothing refuses. Re-run: exit 0. `the_differing_tests_are_read_from_every_key` named the removed
   Message key; its example is now `base/class/RexxContext TESTRS01`. `concurrency_tests` ungated:
   exit 0, 54 passed.
4. The five non-mechanical items stay refusals.

Checks for this commit: `cargo fmt --all --check` exit 0; `cargo clippy -j 4 -p rexx-exec
--all-targets -- -D warnings` exit 0; `--test collection_arity` exit 0 (24 passed);
`--test concurrency_tests` exit 0; whole-groups gate for the three groups exit 0;
`--test introspection_arity --test method_bodies` exit 0; suite on `samples/samples` exit 101 as
above.

## Fix round 1

Commit `cd171c821` on top of `d2f89f559`.

* **I1.** `Supplier~init` (`dispatch/collection/supplier.rs`) no longer converts before validating.
  It calls the new `array_argument_object` (`object_protocol.rs`), which answers the converted array
  itself, rooted; `array_argument` is now that plus `message_argument_slots`. A `MAKEARRAY`
  answering `.nil` now reports the original object (`a KN`) in both positions, as the oracle does.
  `request_array` (`dispatch/array.rs`) had no caller left and is deleted.
* **I2.** Witness `lang/array_argument_callers.rex`, in `phase-9.txt`, adapted from the review's
  `callers.rex`. Five values (an Object, MAKEARRAY answering an array, MAKEARRAY answering `.nil`,
  a string, a 2-D array) go through `sendWith`, `startWith`, `run` (sent from the receiver's own
  method, since `run` is private: the review's `o~run` reached 97.2, not the conversion),
  `Message~new` with `'A'`, `callWith`, and both `Supplier~new` positions. `startWith` sends `POS`,
  so no started activity reports an error, which keeps the deferred error-report difference out.
  Oracle and crate rc 0, stdout and stderr byte-identical; stdout read: the 88.913 `argument
  array` line shows `run` reached the conversion. `lang/unknown_argument_list.rex`'s header now
  says it covers `~UNKNOWN` and one `sendWith`.
* **I3.** `refusal-dispositions.tsv` `unknown_receiver`: "Task 7 probe rc4 and Phase 9a Task 5 probe
  copy8 (a Method copy passed to define, setMethod and run answers)". `refusal-sites.tsv`
  `unknown_receiver`: the Method-copy clause is deleted.
* **Minors.** `REXX_WHOLE_GROUPS_ONLY` asserts every named group is a group file of the derived
  list. `interpreter_bin` panics when the stale `rexx` cannot be removed, and asserts the file is
  absent before the copy fallback. `check_path` compares (dev, ino) with `rexx-run` first and reads
  bytes only for a file that is not the same inode. The `frame_executable` doc names the frames that
  refuse: a native level's, one built from a failure's site, and one whose `executable_at` failed.

Checks, statuses read unpiped, all bounded with `timeout`:

* `cargo fmt --all`; `cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites`: exit 0 (only the edited row moved).
* `--test refusal_sites --test refusal_dispositions --test closed_phases --test owners --test loud
  --test phase_ten_reasons --test concurrency_tests`: exit 0 (concurrency_tests 54 passed).
* `-p rexx-parse --test sourceline_oracle`: exit 0.
* `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0 (29 passed, 1 ignored).
* `--lib -- dispatch::`: exit 0, 114 passed.

Unblocked groups through the harness (`REXX_SUITE_GROUPS=base/class/RexxInfo,base/bif/CHAROUT,
base/class/Stream,base/keyword/ADDRESS,base/source.file/SourceFile,base/keyword/VarRef`): exit 101,
4 red. Against the Task 3 baseline (`docs/superpowers/records/2026-10-11-phase-9a/suite-baseline/`):

| group | baseline | now |
|---|---|---|
| `base/class/RexxInfo` | red: EXECUTABLE, LIBRARYPATH blocked | **green** (TEST_REXXINFO_EXECUTABLE's `~name` is `rexx` through the hard link) |
| `base/bif/CHAROUT` | red: TEST_POSITIONING_1_BIN failed, TEST_STDOUT_RET blocked | red: TEST_POSITIONING_1_BIN failed |
| `base/class/Stream` | red: 11 failed, TEST_FLUSH_ON_CLOSE_XXXXXX blocked | red: the same 11 failed |
| `base/keyword/ADDRESS` | red: TEST_COMMAND_CD blocked (SysGetErrorText), TEST_PATH_COMMAND blocked | red: TEST_COMMAND_CD blocked (SysGetErrorText) |
| `base/source.file/SourceFile` | red: TEST_LOGICALNOT01-08 blocked | **green** |
| `base/keyword/VarRef` | red: 5 failed, TEST_COPY and TEST_UNKNOWN_OK blocked | red: the same 5 failed, TEST_COPY blocked (`COPY` of a VariableReference) |

No test newly fails: every test not passed now was not passed in the baseline. TEST_COMMAND_CD's
refusal now reads `(Phase 9)` where the baseline read `(Phase 10)`, a Task 4 owner change, not this
round's.

Concern: `VariableReference~copy` still refuses (VarRef TEST_COPY); it was not among this task's
Step 1 receivers.
