# Phase 9a gate: the ooTest suite instrument

This record holds what Phase 9a measures and pins. Sections are added as the plan's tasks land.

## Perf pin

Phase 9's close measures the interpreter against the binary of the last commit before Phase 9
(`a98238be5`, "Phase 6.1: close the oracle-judged seeded gate item as not wanted"). 9a builds the
pin once and measures nothing against it. The binary lives outside the tree at
`/tmp/claude-1000/p9/pin/rexx-run`; if it is lost, rebuild it from the same commit with the
commands below and compare the sha256.

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

`base/class/Package`'s `TESTRECLAIM01` is blocked by `timeout after 60 s`: the hang at program end
Task 9 isolates.

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

`rust/corpus/suite-9d-set.txt`, derived from this run's `tests.tsv` by the command in its header. It
holds the TSV's `tests_wrong` tests less the 52 witnesses spec section 3 names, and two tests the
TSV does not have: `base/class/Package`'s `TEST_PROLOG`, which scout A's run reached only as a
timeout. `TEST_PACKAGE_FINDPROGRAM_REXXTRY` asks `findProgram` for `mime.cls`, which the oracle
finds in its `bin` on `PATH`; the crate side's `bin` now holds the oracle `bin`'s `*.cls` files, the
test passes on the crate through `REXX_SUITE_GROUPS`, and the set leaves it out, as its header says.

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
TEST_FACTOR refuses `NEW` on a `String` subclass and TEST_MAKESTRING refuses `SysTempFileName`.
