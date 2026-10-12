# Phase 9a gate: the ooTest suite instrument

This record holds what Phase 9a measures and pins. Sections are added as the plan's tasks land.

## Perf pin

Phase 9's close measures the interpreter against the binary of the last commit before Phase 9
(`a98238be5`, "Phase 6.1: close the oracle-judged seeded gate item as not wanted"). 9a builds the
pin once and measures nothing against it. The binary was built outside the tree at
`/tmp/claude-1000/p9/pin/rexx-run` and was lost when `/tmp` was wiped by a reboot on 2026-10-12.
Ruling (`progress.md:51`): the Phase 9 close rebuilds it from the same commit with the commands
below and records the new sha256 beside this one.

* sha256: `c97bf40a561cc46a4fa41ac64e7c8a16d42483f3a1869774a1c5e03610e49667`
* size: 58,283,928 bytes
* built 2026-10-11, release profile of `rust/Cargo.toml` at that commit (`debug = true` and its `lto`
  and `codegen-units` settings), `Compiling rexx-exec` present in the build output.

The whole commit is archived, not only `rust/`, because `rexx-inventory`'s build reads files outside
`rust/`. Run from the worktree:

```sh
mkdir -p /tmp/claude-1000/p9/pin/src
git archive a98238be5 | tar -x -C /tmp/claude-1000/p9/pin/src
find /tmp/claude-1000/p9/pin/src -exec touch {} +
cd /tmp/claude-1000/p9/pin/src/rust
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/p9/pin/target \
    memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run
cp /tmp/claude-1000/p9/pin/target/release/rexx-run /tmp/claude-1000/p9/pin/rexx-run
sha256sum /tmp/claude-1000/p9/pin/rexx-run
```

The source and target directories were deleted after the copy.

## Row files

The tests the suite gate does not require to pass, one row per test (`dir/group`, test, owner,
reason), each file headed by the ootest revision it was derived at (13178):

* `docs/superpowers/plans/phase-9-exclusions.txt`: Phase 10, by `reaching_rxapi`, by group, or by
  a captured refusal that names a queue or the function registry.
* `rust/corpus/suite-licences.txt`: the `common.txt` Flaky rows. No Deviation or Environment row
  is seeded; those need Moritz's ruling.
* `rust/corpus/suite-not-run.txt`: `restrictOS` and skipped-test-type rows.

`tests/suite.rs`'s `the_row_files_hold_only_checked_rows` runs each owner's check
(`support/suite_rows.rs`) and asserts every row `derived_rows` finds in the group files is present.
A Phase 10 refusal row's check accepts a refusal naming Phase 10; `suite_rows::refusal_carries` is
where Task 4 requires its D3 reason. Rows per owner and reason at the commit that adds this section:

```sh
cat docs/superpowers/plans/phase-9-exclusions.txt rust/corpus/suite-licences.txt rust/corpus/suite-not-run.txt \
  | grep -v '^#' | awk -F'\t' '{r=$4; if (r ~ /^refusal: /) r="refusal"; if (r ~ /^restrictOS line/) r="restrictOS";
    if (r ~ /^common.txt/) r="common.txt"; print $3" / "r}' | sort | uniq -c
```

| rows | owner / reason |
|---|---|
| 274 | Phase 10 Rxapi / reaching_rxapi |
| 18 | Phase 10 Rxapi / group |
| 4 | Phase 10 Rxapi / refusal |
| 973 | Phase 10 Extension / group |
| 63 | Phase 10 Row10NativeApi / group |
| 4 | Flaky / common.txt |
| 183 | NotRun / restrictOS |
| 1 | NotRun / test type DOC_EXAMPLE_NOISE_TEST |

## Baseline suite run

At `2415e1733`, 2026-10-11, 328 s wall (status file start and end stamps; the test reported 327.61 s):

```sh
cd rust
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=/tmp/claude-1000/p9/t3/run2/table.tsv CARGO_INCREMENTAL=0 \
    memcap 8G cargo test --release -j 4 -p rexx-exec --test suite l3_core_suite -- --nocapture
```

Exit 101: `l3_core_suite` is red, as expected before 9b to 9d. Its outputs are in
`docs/superpowers/records/2026-10-11-phase-9a/suite-baseline/`: `table.tsv` (per group and side),
`verdicts.tsv` (D1 per group with each reason, and the oracle-side denominator check),
`oracle-differences.tsv` and `tests.tsv` (every test's row owner and verdict per side).

| | groups | expected | started | passed |
|---|---|---|---|---|
| oracle | 402 | 23003 | 23935 | 23928 |
| crate | 402 | 23003 | 23421 | 22850 |

"Expected" is the enumeration less the rows; "started" and "passed" count rowed tests too. Of the
expected tests, the crate passed 22472, failed 251, and 280 were blocked (243 by a refusal or a
timeout, 37 by a refusal at load). The oracle-side denominator check is equal on every group.
The crate ran 658 runs, 38 groups more than once (`Package_Options` 55 times).

D1: 324 groups green, 78 red.

* Blocked at load (`RexxInfo~executable`): `samples/samples`, `samples/scclient`,
  `samples/scserver`, `samples/sfclient`, `samples/sfserver`, `samples/windows/adsi`,
  `samples/windows/samples`, `samples/windows/wmi`.
* An expected test not passed: `API/oo/METHOD`, `base/bif/ARG`, `base/bif/CHAROUT`,
  `base/bif/CHARS`, `base/bif/CONDITION`, `base/bif/LINEOUT`, `base/bif/LINES`, `base/bif/STREAM`,
  `base/class/Array`, `base/class/Bag`, `base/class/CircularQueue`, `base/class/Class`,
  `base/class/CollectionMethods`, `base/class/File`, `base/class/IdentityTable`,
  `base/class/Message`, `base/class/Method`, `base/class/Object`, `base/class/Package`,
  `base/class/Package_Options`, `base/class/Queue`, `base/class/QueueRGF`,
  `base/class/RexxContext`, `base/class/RexxInfo`, `base/class/RexxInteger`, `base/class/Routine`,
  `base/class/StackFrame`, `base/class/Stream`, `base/class/String/comparisonOperators`,
  `base/class/Validate`, `base/class/collections/directory`, `base/class/collections/queue`,
  `base/directives/ANNOTATE`, `base/directives/ATTRIBUTE`, `base/directives/CLASS`,
  `base/directives/CONSTANT`, `base/directives/METHOD`, `base/directives/OPTIONS`,
  `base/directives/REQUIRES`, `base/directives/RESOURCE`, `base/directives/ROUTINE`,
  `base/expressions/Literals`, `base/keyword/ADDRESS`, `base/keyword/LABEL`,
  `base/keyword/LOSTDIGITS`, `base/keyword/RAISE`, `base/keyword/SIGNAL`, `base/keyword/TRACE`,
  `base/keyword/TRACE_TraceObject`, `base/keyword/VarRef`, `base/rexxutil/SysDumpVariables`,
  `base/rexxutil/SysFileDateTime`, `base/rexxutil/SysFileSearch`, `base/rexxutil/SysFileTree`,
  `base/rexxutil/SysFileXXX`, `base/rexxutil/SysFormatMessage`, `base/rexxutil/SysSearchPath`,
  `base/rexxutil/SysStemCopy`, `base/rexxutil/SysStemDelete`, `base/rexxutil/SysStemInsert`,
  `base/rexxutil/SysStemSort`, `base/rexxutil/platform/unix/SysGetMessage`,
  `base/rexxutil/platform/unix/tilde`, `base/runtime.objects/environmentEntries`,
  `base/security.manager/SecurityManager`, `base/source.file/SourceFile`,
  `base/source.file/incorrectCharacters`, `doc/rexxref/chapter5/Section1`,
  `utilities/rexx/rexx_command`, `utilities/rexxc/rexxc`.

`base/class/Package`'s `TESTRECLAIM01` is blocked by `timeout after 60 s`: it loops until a package it
dropped is collected, and this crate never collects a package object (Task 9's report).

The run before it, at `717ab354b`, found the enumeration missing what `getTestMethods` runs
(block comments, quoted names with blanks, `::CONSTANT` and `::ATTRIBUTE` methods, class methods)
and the four `STREAM`/`Stream` relative-file failures on the oracle; `2415e1733` fixed both.

**The `STREAM`/`Stream` finding.** The harness's copy causes the four oracle failures. Each test
opens a group file outside its own group's directory by a relative path
(`base/class/Array.testGroup` and `API/oo/CONVERSION.testGroup` from `base/bif`,
`base/bif/STREAM.testGroup` from `base/class`), and the copy held only the group's own directory.
`suite.rs`'s `NEIGHBOURS` copies those files where the checkout has them; the four pass on the oracle
in this run, and `the_relative_path_tests_pass_on_the_oracle_with_their_neighbours_copied` fails
with the copy disabled.

## The socket sample groups run alone

Controller ruling, Task 3 fix round 1: `samples/scclient`, `samples/scserver`, `samples/sfclient`
and `samples/sfserver` run one group at a time, after the others and beside none (`suite.rs`'s
`ALONE`), on both sides, and get no Environment row. Their `TEST_04` binds fixed ports: beside each
other at eight groups at once they failed or hung on the oracle in a different combination in each
of scout A's run and this task's two whole runs (`EADDRINUSE` from a bind in scout A's), and run
alone they pass. The records are `rust/corpus/suite-oracle-runs/samples-*.txt`. Run alone through
`REXX_SUITE_GROUPS` at the fix round's tree, all four `TEST_04` pass on the oracle. No Environment
candidate remains; in the `2415e1733` run no other test without a Phase 10 row failed on the oracle.

## DIFFERING cross-check

`concurrency_tests.rs`'s `every_differing_test_is_fixed_or_has_a_row`, gate-only, reads the per-test
record. Against this run's:

```sh
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=/tmp/claude-1000/p9/t3/run2/table.tsv CARGO_INCREMENTAL=0 \
    memcap 8G cargo test --release -j 4 -p rexx-exec --test concurrency_tests every_differing_test_is_fixed_or_has_a_row
```

Exit 101, red as expected. The tests neither passing nor rowed: `base/bif/STREAM`
TEST_OPEN_WRITE_ONLY_3274050_C, _D, TEST_QUERYFILE_EXISTS_OPENED_01, _03,
TEST_SEEK_CLOSEDFILE_2785896, TEST_SEEK_CLOSEDFILE_2787994; `base/class/Class` TEST_ACTIVATE,
TEST_METHODS; `base/class/Message` TEST_REPLYWITH_NOT_ARRAY; `base/class/Method`
TEST_NEW_ARRAY_FROM_FILE, TEST_NEW_FILE_COMPILED, TEST_NEW_FOUR_ARGS; `base/class/Object`
TEST_INSTANCEMETHOD, TEST_RUN, TEST_RUN_ARRAY_ARGUMENT, TEST_SETMETHOD, TEST_SETMETHOD_SCOPE;
`base/class/RexxContext` TESTCOPY01, TESTRS01; `base/directives/ATTRIBUTE` TESTDELEGATE,
TESTMISPLACEDCLASSMETHOD; `base/directives/CONSTANT` TEST_EXPRESSION_NOVALUE_ERROR;
`base/directives/METHOD` TESTDELEGATE, TESTPACKAGE; `base/keyword/RAISE` TEST_RAISE_INSERT_CRLF,
TEST_RAISE_PROPAGATE, TEST_RAISE_SYNTAX_EXIT_02, TEST_RAISE_SYNTAX_RETURN_02; `base/keyword/TRACE`
TEST_TRACE_DROP, TEST_TRACE_EXIT, TEST_TRACE_EXPOSE, TEST_TRACE_IGNORED,
TEST_TRACE_LABEL_WITH_FORWARD, TEST_TRACE_OTHER_ENTRYPOINT, TEST_TRACE_PROCEDURE;
`base/keyword/TRACE_TraceObject` TEST_CALLER_STACK_FRAME, TEST_CALLER_STACK_FRAME_REPLY_START,
TEST_OBJECT_AND_SCOPE, TEST_TRACEOBJECT_COLLECTOR, TEST_VARIABLE; `doc/rexxref/chapter5/Section1`
TEST_OBJECT_OBJECTNAMEEQUALS.

## The 9d set

`rust/corpus/suite-9d-set.txt` was first derived from this run's `tests.tsv` and is re-derived at
the 9a close from the close run's record, by the command in its header (see "9a close" below).
`TEST_PACKAGE_FINDPROGRAM_REXXTRY` asks `findProgram` for `mime.cls`, which the oracle finds in its
`bin` on `PATH`; the crate side's `bin` holds the oracle `bin`'s `*.cls` files, and the test passes
on the crate in the close run.

## Task 5: path-valued rows and the crate's `bin`

`.RexxInfo~executable` and `~libraryPath` answer the running interpreter's own path, so no value
can match between the two. Their rows in `rust/corpus/method-bodies.txt` and
`rust/corpus/introspection-arity.tsv` compare the answer's class and whether the file it names
exists (`PATH_VALUED` in `tests/method_bodies.rs` and `tests/support/arity.rs`), not the path; the
corpus witness `lang/rexxinfo_executable_file.rex` compares the same properties.

The suite harness's crate-side `bin-rexx/rexx` is a hard link (a copy where a link cannot be made)
to `rexx-run`, with `samples` beside it linked to the oracle's `build/samples`: the test framework
finds the samples at `../samples` from `.RexxInfo~executable`, which resolves a symlink.
`samples/samples` through `REXX_SUITE_GROUPS` at Task 5's ruling commit: red, 27 of 29 pass;
TEST_FACTOR refuses `NEW` on a `String` subclass and TEST_MAKESTRING fails: `makestring.rex`
stops at `rxregexp.cls:48`'s `LIBRARY rxregexp` with 98.903 ("9a close", "The 9d set").

## 9a close

The record's run is close run 3 at `5fd68d060`, by `/tmp/claude-1000/p9/close/run.sh`:

```sh
cd rust
REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=/tmp/claude-1000/p9/close/suite/table.tsv CARGO_INCREMENTAL=0 \
    memcap 8G cargo test --release -j 4 -p rexx-exec --test suite l3_core_suite -- --nocapture
```

Its status file (`/tmp/claude-1000/p9/close-run3/status.txt`): start 2026-10-12T22:36:51Z, workspace
exit 101 at 22:54:53Z, suite exit 101 at 22:59:18Z, `finished`. Logs beside it: `workspace.log`,
`suite.log`.

* Run 1 at `9cd9ebd7c`: workspace exit 101 on `concurrency_tests` (stale `whole_groups` rows for
  `base/class/Method` and `base/directives/METHOD`, and `sim_gate` on Method's compiled-file tests);
  suite 64 of 402 groups red. Fixed by `dbec28ca8`, `18bd99117`, `39b78a0e3` (`task-10-fix-report.md`).
* Run 2 at `fccf0d832`: workspace exit 101 on `sim_gate`'s Object rest part (`TEST_INSTANCEMETHOD`,
  `TEST_SETMETHOD`, which Tasks 5 and 8b unblocked); suite 62 red. Sim rows re-derived by `0ad23af25`,
  `dca9c3ac8`, `5d22dacb0` (`task-10-sim-report.md`).

### The suite runs whole

Run 3's outputs are in `docs/superpowers/records/2026-10-11-phase-9a/suite-close/`, the baseline's
file set; `tests.tsv` carries `# groups all`. Wall 265 s by the status stamps (the test reported
264.57 s), against the baseline's 328 s. Every group ended normally on both sides.

| | groups | expected | started | passed |
|---|---|---|---|---|
| oracle, baseline `2415e1733` | 402 | 23003 | 23935 | 23928 |
| oracle, close `5fd68d060` | 402 | 23003 | 23935 | 23932 |
| crate, baseline | 402 | 23003 | 23421 | 22850 |
| crate, close | 402 | 23003 | 23935 | 23480 |

Of the expected tests, the crate passed 22630, failed 248 and had 125 blocked (124 by a refusal, 1
by a timeout, none at load); the baseline's were 22472, 251 and 280 (242, 1 and 37). The oracle
passed all 23003. The crate ran 544 runs, 23 groups more than once (baseline 658 and 38).

D1: 340 groups green, 62 red (baseline 324 and 78). Red to green: `base/class/CollectionMethods`,
`base/class/Message`, `base/class/Package_Options`, `base/class/RexxInfo`,
`base/class/collections/directory`, `base/directives/ANNOTATE`, `base/directives/METHOD`,
`base/directives/RESOURCE`, `base/directives/ROUTINE`, `base/source.file/SourceFile`,
`doc/rexxref/chapter5/Section1`, `samples/scserver`, `samples/sfserver`, `samples/windows/adsi`,
`samples/windows/samples`, `samples/windows/wmi`. Green to red: none. No test that passed on the
crate in the baseline fails or is blocked in the close run.

```sh
export LC_ALL=C
B=docs/superpowers/records/2026-10-11-phase-9a/suite-baseline
R=docs/superpowers/records/2026-10-11-phase-9a/suite-close
awk -F'\t' '!/^#/ && $1!="group" {g[$2]++; e[$2]+=$3; s[$2]+=$4; p[$2]+=$5; n[$2]+=$10}
    END {for (k in g) print k, g[k], e[k], s[k], p[k], n[k]}' $R/table.tsv
awk -F'\t' '$2=="crate" && $10>1' $R/table.tsv | wc -l
awk -F'\t' '!/^#/ && $1!="group" && $3=="-" {c=$5; if (c ~ /^blocked at load/) c="at load";
    else if (c ~ /^blocked: timeout/) c="timeout"; else if (c ~ /^blocked/) c="refusal"; print c}' $R/tests.tsv | sort | uniq -c
awk -F'\t' '!/^#/ && $1!="group" && $3=="-" {print $4}' $R/tests.tsv | sort | uniq -c
join -t$'\t' <(awk -F'\t' '!/^#/ && $1!="group" {print $1"\t"$2}' $B/verdicts.tsv | sort) \
    <(awk -F'\t' '!/^#/ && $1!="group" {print $1"\t"$2}' $R/verdicts.tsv | sort) | awk -F'\t' '$2!=$3'
join -t$'\t' <(awk -F'\t' '!/^#/ && $1!="group" {print $1"|"$2"\t"$5}' $B/tests.tsv | sort) \
    <(awk -F'\t' '!/^#/ && $1!="group" {print $1"|"$2"\t"$5}' $R/tests.tsv | sort) | awk -F'\t' '$2=="passed" && $3!="passed"'
```

The same commands with `$B` give the baseline figures. The last prints nothing.

### Every section 3 refusal's witness passes

Run 3's workspace log: `corpus_differential ... ok`, "mode: STRICT (the gate)", "1019 of 1019
matching" over the phase subset files through `phase-9.txt`. Every program below is listed in
`rust/corpus/phase-9.txt`, under `rust/corpus/`.

| Section 3 refusal | Witness, or the ruling that moved it |
|---|---|
| `RexxInfo~executable`, `~libraryPath` | `lang/rexxinfo_executable_file.rex`; no group is blocked at load in run 3 |
| `StackFrame~executable` | `lang/stackframe_executable.rex`; the REXX package's native routine frame, `lang/rexx_routine_frame_executable.rex` (Task 8b, `progress.md:70`); a library routine frame's stays refused, 9d (`progress.md:137`) |
| `Object~copy` on Method, RexxContext (93.970) | `lang/object_copy_method.rex` |
| `Object~copy` on RexxInfo | `lang/rexxinfo_executable_file.rex` |
| `Object~copy` on Routine, and `VariableReference~copy` | `lang/executable_copies.rex` (Task 8b, `progress.md:70`, `:80`) |
| `Object~copy` on Package | `lang/package_copy.rex` (Task 8b); the REXX package's own copy stays refused, 9d (`progress.md:137`) |
| `OBJECTNAME=` without a variable pool | `lang/object_name_set_values.rex` (Task 8b); on a String or a SmallInt it stays refused, Deviation 31 (`progress.md:131`) |
| `MAKEARRAY` on a non-Array `~UNKNOWN` list | `lang/unknown_argument_list.rex`, `lang/array_argument_callers.rex` |
| Hash methods on the condition object and `.methods` | `lang/condition_object_hash.rex`, `lang/package_tables_hash.rex`; hash iteration order is Deviation 33 (`progress.md:134`) |
| A metaclass with its own `NEW` | `lang/metaclass_new.rex` (Task 8b); a metaclass `NEW` answering an existing class stays refused under Deviation 32's extended scope (`progress.md:140`, `:143`) |
| `SUPPLIER` in `~defineMethods` | `lang/define_methods_supplier.rex`; a SUPPLIER answer that is not a supplier stays loud, `refusal-dispositions.tsv`'s `receiver_class` DEVIATION row (oracle crash entry 35) |
| `Message~notify` | unreached: `refusal-dispositions.tsv`'s `environment_symbol` GUARD row names probes `nt1`, `nt2` (Task 5) |
| `NEW` on a subclass of String, Stem, Method, Routine, Message | `lang/string_subclass_new.rex`, `lang/stem_subclass_new.rex`, `lang/executable_subclass_new.rex`, `lang/message_subclass_new.rex` |
| `Class~enhanced` running `NEW` | `lang/enhanced_runs_new.rex`, `lang/enhanced_primitive_receivers.rex` (Task 8b, `progress.md:109`); a user `NEW` through `enhanced` answering another object, Deviation 32 |
| Queued `subclass-package` | `lang/package_subclass_new.rex` |
| Queued `subclass-rexxinfo` | `lang/class_target_not_a_class.rex` |
| Queued `stringtable-subclass-new-arg`, `hash-subclass-init-capacity` | `lang/hash_subclass_init_capacity.rex` |
| "A package settings write" | `lang/package_option_writes.rex`, `lang/package_default_options.rex`; `base/class/Package_Options` green in run 3 |
| `ReleaseLocalReference` | `tests/release_local_reference.rs`: 3 passed in run 3's workspace log |
| `base/class/Package` hangs at program end | Task 9: `TESTRECLAIM01`; see `Package` below |

### The row-file test and the D3 test

Run 3's workspace log: `tests/suite.rs` (`suite-a1f154c753a34319`) 50 passed, 0 failed, among them
`the_row_files_hold_only_checked_rows ... ok`; `tests/phase_ten_reasons.rs`
(`phase_ten_reasons-7b0d693935c1fa18`) 5 passed, 0 failed, among them
`no_production_literal_names_phase_10 ... ok`.

### The denominator check

Every red verdict in run 3 is "not passed" only. No verdict names a test not started, a test started
outside the expected set and the rows, an abnormal end, or an uncaptured refusal, and the
oracle-side denominator is `equal` on every group. So started equals expected on every group,
including those with a test blocked by a 9b refusal:

```sh
grep -c 'not started\|outside the expected\|did not end normally\|not captured\|not the one captured\|started on the oracle' $R/verdicts.tsv
awk -F'\t' '!/^#/ && $1!="group" && $4!="equal"' $R/verdicts.tsv | wc -l
```

Both print 0. The blocked expected tests (`$3=="-"` and a `blocked` crate verdict in `tests.tsv`),
each refusal a RexxUtil routine of spec section 4, so 9b's:

| Group | Tests | Refusal (`rexx-exec: `) |
|---|---|---|
| `base/class/File` | TEST_FILE_LASTACCESSED, TEST_FILE_LASTMODIFIED | routine "SYSGETFILEDATETIME" is not implemented (Phase 9) |
| `base/keyword/ADDRESS` | TEST_COMMAND_CD | routine "SYSGETERRORTEXT" is not implemented (Phase 9) |
| `base/rexxutil/platform/unix/SysGetMessage` | TEST_MESSAGE_SET, TEST_X_NO_ARGS | routine "SYSGETMESSAGEX" is not implemented (Phase 9) |
| `base/rexxutil/platform/unix/SysGetMessage` | TEST_MESSAGE, TEST_MESSAGE_INSERTS, TEST_MESSAGE_SURPLUS_INSERTS, TEST_NO_ARGS, TEST_NO_MESSAGE, TEST_REXX_ALL, TEST_REXX_MAJOR | routine "SYSGETMESSAGE" is not implemented (Phase 9) |
| `base/rexxutil/platform/unix/tilde` | TEST_TILDE | routine "SYSTEMPFILENAME" is not implemented (Phase 9) |
| `base/rexxutil/SysDumpVariables` | TEST_LONG, TEST_NONE, TEST_NO_ARGS_TOO_MANY, TEST_SIMPLE_AND_STEM | routine "SYSDUMPVARIABLES" is not implemented (Phase 9) |
| `base/rexxutil/SysFileDateTime` | TEST_GET_NO_ARGS, TEST_GET_OPTION_INVALID, TEST_GET_OPTION_NULL, TEST_GET_THREE_ARGS_TOO_MANY | routine "SYSGETFILEDATETIME" is not implemented (Phase 9) |
| `base/rexxutil/SysFileDateTime` | TEST_SET_DATETIME_INVALID, TEST_SET_FOUR_ARGS_TOO_MANY, TEST_SET_GET_DATETIME, TEST_SET_NO_ARGS | routine "SYSSETFILEDATETIME" is not implemented (Phase 9) |
| `base/rexxutil/SysFileSearch` | TEST_BUFFER_SIZE, TEST_FIVE_ARGS_TOO_MANY, TEST_INVALID_OPTION, TEST_LONG_LINE, TEST_LONG_LINENUMBERS, TEST_NO_ARGS, TEST_NO_FILE, TEST_NO_STEM, TEST_NUL_CHARACTER, TEST_ONE_ARG_TOO_FEW, TEST_STEM_ARRAY_OBJECT, TEST_STEM_ARRAY_ZERO, TEST_STEM_NAME, TEST_TWO_ARGS_TOO_FEW | routine "SYSFILESEARCH" is not implemented (Phase 9) |
| `base/rexxutil/SysFileXXX` | TEST_FILECOPY, TEST_FILECOPY_INVALID, TEST_FILECOPY_NO_ARG, TEST_FILECOPY_ONE_ARG, TEST_FILECOPY_THREE_ARGS | routine "SYSFILECOPY" is not implemented (Phase 9) |
| `base/rexxutil/SysFileXXX` | TEST_FILEMOVE, TEST_FILEMOVE_INVALID, TEST_FILEMOVE_NO_ARG, TEST_FILEMOVE_ONE_ARG, TEST_FILEMOVE_THREE_ARGS | routine "SYSFILEMOVE" is not implemented (Phase 9) |
| `base/rexxutil/SysFileXXX` | TEST_ISFILEDIRECTORY, TEST_ISFILEDIRECTORY_INVALID, TEST_ISFILEDIRECTORY_NO_ARG, TEST_ISFILEDIRECTORY_TWO_ARGS, TEST_MKDIR | routine "SYSISFILEDIRECTORY" is not implemented (Phase 9) |
| `base/rexxutil/SysFileXXX` | TEST_TEMPFILE_EXHAUST, TEST_TEMPFILE_FILLER_LONG, TEST_TEMPFILE_FILLER_NULL, TEST_TEMPFILE_NO_ARG, TEST_TEMPFILE_PATH, TEST_TEMPFILE_SIMPLE, TEST_TEMPFILE_TEMPLATE_FILLER_MISMATCH, TEST_TEMPFILE_THREE_ARGS | routine "SYSTEMPFILENAME" is not implemented (Phase 9) |
| `base/rexxutil/SysFormatMessage` | TEST_LONG_SUBSTITUTIONS, TEST_NO_ARGS_TOO_FEW, TEST_ONE_ARG_TOO_FEW, TEST_SUBSTITUTIONS, TEST_TOO_MANY_SUBS | routine "SYSFORMATMESSAGE" is not implemented (Phase 9) |
| `base/rexxutil/SysSearchPath` | TEST_SEARCHPATH_ABSOLUTE, TEST_SEARCHPATH_ARGS_FOUR, TEST_SEARCHPATH_ARGS_NONE, TEST_SEARCHPATH_ARGS_ONE, TEST_SEARCHPATH_EMPTY, TEST_SEARCHPATH_OPTION_INVALID, TEST_SEARCHPATH_OPTION_NULL, TEST_SEARCHPATH_RELATIVE, TEST_SEARCHPATH_SHELL, TEST_SEARCHPATH_VARIABLE | routine "SYSSEARCHPATH" is not implemented (Phase 9) |
| `base/rexxutil/SysStemCopy` | TEST_BAD_FROM_COUNT, TEST_BAD_TO_COUNT, TEST_COPIES, TEST_COUNT_OUT_OF_RANGE, TEST_FROM_OUT_OF_RANGE, TEST_INVALID_OPTION, TEST_SPARSE_INSERT_SOURCE, TEST_SPARSE_INSERT_TARGET, TEST_SPARSE_OVERLAY_SOURCE, TEST_TO_OUT_OF_RANGE, TEST_UNSET_FROM_COUNT, TEST_UNSET_TO_COUNT | routine "SYSSTEMCOPY" is not implemented (Phase 9) |
| `base/rexxutil/SysStemDelete` | TEST_BAD_COUNT, TEST_DELETES, TEST_OUT_OF_COUNT, TEST_OUT_OF_RANGE, TEST_UNSET_COUNT | routine "SYSSTEMDELETE" is not implemented (Phase 9) |
| `base/rexxutil/SysStemInsert` | TEST_INSERT_EMPTY, TEST_INSERT_OUT_OF_RANGE, TEST_INSET, TEST_STEM_SIZE_BAD_VALUE, TEST_STEM_SIZE_NOT_SET | routine "SYSSTEMINSERT" is not implemented (Phase 9) |
| `base/rexxutil/SysStemSort` | TESTSORTRANGE, TEST_ALREADY_SORTED, TEST_BAD_COUNT, TEST_BAD_DIRECTION_ARG, TEST_CASE_SENSITIVITY, TEST_COMPOUND_NAME, TEST_DESCENDING_SIMPLE, TEST_INVALID_FIRSTCOL_LASTCOL, TEST_INVALID_FIRST_LAST, TEST_INVALID_ORDER, TEST_INVALID_STEM, TEST_OUT_OF_RANGE, TEST_SIMPLE_LETTERS, TEST_SORT, TEST_SORT_COLS, TEST_SPARSE_ARRAY, TEST_TOO_MANY, TEST_UNSET_COUNT | routine "SYSSTEMSORT" is not implemented (Phase 9) |
| `base/security.manager/SecurityManager` | TEST_CALL_HANDLE_NO_RESULT, TEST_CALL_HANDLE_RESULT, TEST_CALL_PASS_NO_ARGS, TEST_REQUIRES_HANDLE, TEST_REQUIRES_HANDLE_SECURITY_MANAGER, TEST_REQUIRES_PASS | routine "SYSDROPFUNCS" is not implemented (Phase 9) |
| `base/security.manager/SecurityManager` | TEST_CALL_PASS_THREE_ARGS | routine "SYSSEARCHPATH" is not implemented (Phase 9) |

The one expected test blocked otherwise is `base/class/Package`'s `TESTRECLAIM01`, below.

### `Package` ends

`suite-close/table.tsv`, `base/class/Package`, crate: 80 expected, 80 started, 77 passed, failed
`TEST_NEW01 TEST_PROLOG` (9d set), blocked `TESTRECLAIM01=timeout after 60 s`, 2 iterations, ended
normally. The oracle passed all 80. This is the amended criterion (Moritz 2026-10-12,
`progress.md:155`): `TESTRECLAIM01` waits for a dropped package to be collected, which
`.superpowers/sdd/queued/2026-10-12-package-as-real-object.md` owns.

### The close command

Exit 101 on one test of one binary: run 3's workspace log ends `error: 1 target failed: -p rexx-exec
--test concurrency_tests`, which reports 53 passed and 1 failed,
`group_runs::the_s2_rows_of_the_derived_list_in_both_modes`, panicking at
`concurrency_tests.rs:2465` with "tests differing between the modes:
["base/rexxutil/SysSleep.testGroup TEST_SLEEP_DURATION"]" after "P48 rerun:
base/rexxutil/SysSleep.testGroup TEST_SLEEP_DURATION: differs: status Some(1) then Some(1)". Every
other test binary in the log reports 0 failed. The run's load was about 20 (`progress.md:171`). The
same test was `ok` in runs 1 and 2 (their workspace logs).

Re-run alone three times at `5fd68d060`, load 1.3 to 2.7: `/tmp/claude-1000/p9/close/s2-1.log`,
`s2-2.log`, `s2-3.log`, each "1 passed; 0 failed ... 53 filtered out", about 125.7 s. Those runs
used the `-p rexx-exec` build of the binary (`concurrency_tests-1f18f7e0d364845e`); the workspace
run used `concurrency_tests-a4ebff9c6b5a89b4`.

Ruling (`progress.md:172`): the close command passes on every check except a timing assertion that
fails only under external load; the test goes to Moritz as a Flaky-row candidate. If wrong, the close
is re-run on a quiet machine.

### The DIFFERING cross-check

Now that a full-run record exists, against it:

```sh
cd rust
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-11-phase-9a/suite-close/table.tsv \
    CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo test --release -j 4 -p rexx-exec --test concurrency_tests \
    every_differing_test_is_fixed_or_has_a_row
```

Exit 101, red as expected before 9c and 9d; the record passed `full_run_record`. Neither passing nor
rowed: `base/bif/STREAM` TEST_OPEN_WRITE_ONLY_3274050_C, _D, TEST_QUERYFILE_EXISTS_OPENED_01, _03,
TEST_SEEK_CLOSEDFILE_2785896, TEST_SEEK_CLOSEDFILE_2787994; `base/class/Class` TEST_ACTIVATE,
TEST_METHODS; `base/class/Method` TEST_NEW_ARRAY_FROM_FILE, TEST_NEW_FILE_COMPILED;
`base/class/Object` TEST_INSTANCEMETHOD, TEST_SETMETHOD; `base/class/RexxContext` TESTRS01;
`base/directives/ATTRIBUTE` TESTMISPLACEDCLASSMETHOD; `base/directives/CONSTANT`
TEST_EXPRESSION_NOVALUE_ERROR; `base/keyword/RAISE` TEST_RAISE_INSERT_CRLF, TEST_RAISE_PROPAGATE,
TEST_RAISE_SYNTAX_EXIT_02, TEST_RAISE_SYNTAX_RETURN_02; `base/keyword/TRACE` TEST_TRACE_DROP,
TEST_TRACE_EXIT, TEST_TRACE_EXPOSE, TEST_TRACE_IGNORED, TEST_TRACE_LABEL_WITH_FORWARD,
TEST_TRACE_OTHER_ENTRYPOINT, TEST_TRACE_PROCEDURE; `base/keyword/TRACE_TraceObject`
TEST_CALLER_STACK_FRAME, TEST_CALLER_STACK_FRAME_REPLY_START, TEST_OBJECT_AND_SCOPE,
TEST_TRACEOBJECT_COLLECTOR, TEST_VARIABLE. Each is in the 9d set except Method's two, which are 9c's
witnesses (spec section 3). The workspace run's `ok` for this test is its early return without
`REXX_SUITE_GATE`.

### The 9d set

`rust/corpus/suite-9d-set.txt` is re-derived from `suite-close/tests.tsv` by the awk command in its
header, unchanged but for its input. Against the baseline derivation it loses the tests 9a fixed
(`CollectionMethods` TEST_COLLECTION_NEW_2_ARGS, `Object` TEST_RUN and TEST_SETMETHOD_SCOPE,
`collections/directory` TEST_19, `ANNOTATE` TEST_ATTRIBUTE_ANNOTATION and TEST_CONSTANT_ANNOTATION,
`ATTRIBUTE` and `METHOD` TESTDELEGATE, `CLASS` TEST_SUBCLASS_NO_CLASS_ENVIRONMENT) and gains tests
that were blocked in the baseline and fail now: `Package` TEST_NEW01, `SecurityManager`
TEST_ENVIRONMENT_HANDLE, TEST_METHOD_HANDLE_RESULT, TEST_STREAM_PASS_BIF_STREAM_OPEN, `samples`
TEST_MAKESTRING, `scclient` TEST_04_SCCLIENT.REX and `sfclient` TEST_04_SFCLIENT.REX. No test reads
the file.

The three samples tests run a sample in a child `rexx` that loads an extension library. Run on the
crate's `rexx-run` from a fresh directory with the oracle `bin`'s class file beside it,
`scclient.rex` stops at `socket.cls:40`, `::requires "rxsock" LIBRARY`, `sfclient.rex` at its own
`::requires 'rxsock' LIBRARY`, and `makestring.rex` at `rxregexp.cls:48`'s `LIBRARY rxregexp`, each
98.903 "Unable to load library". Those libraries are Phase 10 Extension; the tests have no row.

### Environment and Flaky candidates for Moritz

* Environment: none. The oracle passed every expected test in run 3, and its only failures,
  `API/classic/CLASSIC` TEST_51_EXTERNALFUNCTION and TEST_54_EXTERNALFUNCTION and
  `extensions/platform/unix/rxunixsys/SysUnix` TEST_SYSCRYPT, carry Phase 10 rows
  (`awk -F'\t' '!/^#/ && $1!="group" && $4!="passed" && $4!="not started"' $R/tests.tsv`). The socket
  sample groups pass on the oracle run alone (`ALONE`, above).
* Flaky: `base/rexxutil/SysSleep` TEST_SLEEP_DURATION in the close command's S2 list
  (`concurrency_tests`' `the_s2_rows_of_the_derived_list_in_both_modes`), on the evidence under "The
  close command". `suite-licences.txt` already holds its Flaky row for the suite gate (`common.txt:
  SYSSLEEP.testGroup/TEST_SLEEP_DURATION`); the S2 list has no licence.

## Carried to 9b-9d

Deferred minors and queued items from `.superpowers/sdd/2026-10-11-phase-9a/progress.md`, by line:

* `:27` `dispatch/library/tests.rs:177`'s stale `collect_stress` comment.
* `:28` `skip` compares method names through `from_utf8_lossy`.
* `:29` `group_runner.rs:435` `contains` on an empty needle.
* `:37` Task 2 minors (5), (7), (8), (9), (10).
* `:44` Task 3 minors (3), (5), (6).
* `:48` a refusal row on a test that refuses at load reads as not captured.
* `:64` the `ffi.rs` texts' owners unpinned; the structural scan covers a bare "Phase 10" only.
* `:71`, `:86`, `:93`, `:100`, `:102`, `:165` the "&1" inserts (D6).
* `:75` `Method~copy` side tables never pruned; `startWith`'s error header.
* `:76` `handle_set` beyond HANDLE:0/1/2 (owner: the task taking stream refusals).
* `:86` condition-object build cost; `:96` `refuse_routines_write`'s scan; `:128` eager annotation
  tables; `:145` cumulative per-task perf: all for the Phase 9 close callgrind.
* `:51` the perf pin's rebuild at the Phase 9 close.
* `:100` external `CALL` under package `TRACE R`; `Routine~new`'s `>I>` package.
* `:112` queued `2026-10-12-method-source-compile-retention.md`, `2026-10-12-gc-trigger-blind-to-class-weight.md`.
* `:117` the Stem unknown traceback line, in queued `2026-10-10-directory-setmethod-unknown-traceback-line.md`.
* `:118` `rexx-bench`'s runnability test ignores `CARGO_TARGET_DIR`.
* `:119` queued `2026-10-12-buffer-copy-uncharged.md`.
* `:132` `instanceMethods`' repeated shadowed entries (9d).
* `:134` queued `2026-10-12-randomised-hash-iteration.md`.
* `:137` the REXX package's own copy; a library routine frame's `~executable` (9d).
* `:142` Task 8b minor 3, `Routine~call`'s cold block per call.
* `:144` `Package~copy` retention per copy (queued).
* `:150` queued `2026-10-12-package-as-real-object.md` (owns `TESTRECLAIM01`).
* `:165` `.D~new~instanceMethod('M') == .C~method('M')`; `sim-gate.tsv`'s stale rest rows for
  Method and METHOD.
* `:167` `lib.rs:1428`'s double-hyphen dash; `method_object_for`'s `class_owns` on a non-class scope.
* `:169` `debug_ms` and the MutexSemaphore/STREAM `k` not re-derived; REPLY every-mode rows kept.
* `:172` TEST_SLEEP_DURATION, Flaky candidate (above): the S2 list in `concurrency_tests` has no
  licence path (9d).
* This close: `samples` TEST_MAKESTRING, `scclient` TEST_04_SCCLIENT.REX and `sfclient`
  TEST_04_SFCLIENT.REX end in 98.903 for `rxregexp` and `rxsock`, Phase 10 Extension libraries.
  Ruling: loading a Phase 10 extension library becomes a loud refusal carrying
  `Deferral::Extension`, the first item of 9b; the three tests then take Phase 10 Extension rows
  and leave the 9d set.
* This close: `extensions/yaml/yaml`'s
  TEST35_DIRECTIVES and TEST47_MULTILINEFLOWCOLLECTIONS, blocked by "a compiled ... has no driver arm"
  refusals in a Phase 10 Extension group (`suite-close/table.tsv`).
