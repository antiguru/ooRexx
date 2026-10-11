# Scout A: where the ooTest suite stands (Phase 9 scoping)

HEAD `a98238be5` (checked with `git rev-parse --short HEAD`). Run 2026-10-10, 21:53 to 22:09 CEST.
Per-group table: `scout-a-groups.tsv` beside this file. Raw descriptors of every run, the driver
and the analysis script are in `/tmp/claude-1000/p9scope/a/` (scratch, not committed).

## Headline

Of the 374 groups that are not Phase 10's by the roadmap's group rules, 354 have tests on this
host and 352 could be judged (the oracle hangs on two). **275 of the 352 are green** against the
oracle's own run, every one of them also byte-identical on masked stdout. 41 stop first on a loud
refusal, 36 complete with a wrong answer, and none stop on a crash, panic or out-of-memory as their
first cause. One group (`base/class/Package`) hangs at program end, after its tests have run.

At the test level the oracle ran 23,170 tests in the judged groups. 232 tests are blocked by a
loud refusal (one per refusal, each renamed out in turn and the group re-run). **165 of those name
Phase 10 and 67 name Phase 9.** 249 tests complete here with a failure the oracle does not have.
Another 33 are blocked by a refusal raised while the group itself loads (all `RexxInfo~executable`,
see below), and 14 of `Package_Options` were not reached before the 40-iteration cap.

Figures: `python3 scripts/analyse.py results-final.jsonl groups.tsv` (exit 0) prints the bins and
the refusal table; the per-phase split and the unreached counts are from the inline script under
"Commands". Both read `/tmp/claude-1000/p9scope/a/results-final.jsonl`.

## 1. Population and baselines

* **Groups.** `find ootest -name '*.testGroup' | wc -l` gives 409; `find ootest/ooRexx -name
  '*.testGroup' | wc -l` gives 402. The other 7 are `ootest/misc/`, which the framework never
  searches.
* **How `testOORexx.rex` discovers groups.** `testOORexx.rex` adds `framework/` to `PATH` and calls
  `worker.rex`; `worker.rex:71` builds `.ooTestFinder~new(cl~root, ...)` with root
  `ooTest.frm:238`'s `TEST_ROOT "ooRexx"`; `ooTestFinder::findFiles` (`ooTest.frm:2120`) runs
  `SysFileTree(root || "*.testGroup", files., "FOS")`, i.e. recursive. Each file is `call`ed and must
  answer a container (`getContainer`, `ooTest.frm:2094`); a container with no executable tests is
  skipped with a notification. OS-specific groups opt out through `TestGroup~restrictOS`
  (`ooTest.frm:1297`), so the Windows groups run zero tests on Linux on both sides. `-f FILE`
  (single file) also uses `SysFileTree` with `S`, and takes the file only when exactly one matches.
  **That bites two groups under the harness's copy**: `fresh_copy` copies the whole group directory
  with its subdirectories, so `base/rexxutil/SysFileTree` and `samples/samples` each find a second
  same-named file under `platform/windows/` and `samples/windows/`, and both sides run zero tests
  ("No test containers found"). Re-run with the duplicate deleted from the copy; the table holds
  the re-run.
* **Baselines.** The tree has three things that could be called baselines:
  * `.github/known-test-failures/{common,linux,...}.txt`, consumed by the CI workflows
    (`.github/workflows/unix.yml:110`). `common.txt` lists `CHAROUT/TEST_STDOUT_START`,
    `LINEOUT/TEST_LINE_TRANSIENT`, `SYSSLEEP/TEST_SLEEP_CONCURRENT`, `SYSSLEEP/TEST_SLEEP_DURATION`;
    `linux.txt` is empty on purpose. These are the closest thing to "existing baselines".
  * `docs/superpowers/plans/conformance-baseline.md`: the oracle's whole-suite run of 2026-07-27
    (24,372 tests, 2 failures, 3 errors). A record, not a machine-readable baseline.
  * `ootest/Expected.Results`: a 4.2.0 Windows transcript. Not usable.

  None of them is a per-test expected-outcome file for this crate. **The baseline used here is the
  oracle's own run of each group under the same harness.** The oracle fails 4 tests that no
  known-failures file lists: `STREAM.TEST_RELATIVE_FILE_EXISTS`, `..._EXISTS2`,
  `Stream.TEST_RELATIVE_EXISTS2`, `..._EXISTS3`, plus one per socket sample (below). The four
  `common.txt` entries all passed on the oracle here.

## 2. Phase 10's share

Rules applied, in order; the rule is column `rule` of the TSV.

| Rule | Groups |
|---|---|
| Extension group (`extensions/`, roadmap "Why Phase 9's gate is L3-core") | 20: `dateparser/DateFormatter`, `dateparser/DateParser`, `hostemu/hostemu`, `json/json`, `json/json_02`, `platform/unix/ncurses/ncurses`, `platform/unix/rxunixsys/SysUnix`, the six `platform/windows/ole/*`, `platform/windows/oodialog/Basic`, `platform/windows/rxwinsys/Clipboard`, `.../WindowsEventLog`, `rxmath/RxMath`, `rxregexp/rxregexp`, `rxsock/socketClass`, `yaml/yaml` |
| Native-API group, `api_group_partition.rs` `PARTITION` row 10 | 5: `API/classic/CLASSIC`, `API/oo/INVOCATION`, `ProcessInvocation`, `ProcessRexxStart`, `RexxStart` |
| RXAPI daemon / external queues / `rxsubcom` registration (roadmap paragraph) | 3, **marked `10?`**: `utilities/rxapi`, `utilities/rxqueue/rxQueue`, `utilities/rxsubcom/rxsubcom` |

The `10?` rows conflict with row 9's own text, which ships `rxqueue` and `rxsubcom` in Phase 9. The
paragraph puts external queues and `rxsubcom` registration in Phase 10; the rows test the
utilities. Phase 9's spec has to decide which wins. They were run separately, for information: on
the crate `rxapi` refuses (`RexxInfo~executable`, Phase 9), `rxQueue` refuses (`.STDQUE`,
Phase 10) and then fails 2 of 3, and `rxsubcom` fails 7 of 13. Their results are not in the bins.

**Groups that are Phase 10's in part**: each test that `group_runner::reaching_rxapi` derives,
ported to Python and checked against `api_group_tests.rs`'s `RECORDED` (it yields exactly
`FUNCTION.TEST_REXXQUEUE` for `FUNCTION`, nothing for `METHOD` and `CONVERSION`). These tests are
renamed out on both sides, as the harness does.

| Group | Tests renamed out |
|---|---|
| `API/oo/FUNCTION` | `TEST_REXXQUEUE` |
| `base/bif/RXQUEUE` | all 18 tests; the group runs 0 tests on both sides |
| `base/class/RexxQueue` | 47 tests; 1 left (`TEST_BUG_1379`), which refuses here (`rexx_query_queue`, Phase 10) |
| `base/keyword/ADDRESS` | `TEST_REXXQUEUE` |
| `base/rexxutil/Macrospace` | all 39 tests; 0 left |
| `base/rexxutil/platform/unix/SysGetMessage` | `TEST_CATALOG_INVALID`, `TEST_GENCAT` (inferred false positives, not read: neither names a queue or macro in its test name) |
| `base/runtime.objects/environmentEntries` | `TEST_LOCAL_STREAMS_QUEUE_ARRAY` |

**A fourth rule the brief did not list, and the one that matters most.** The roadmap's D11 says
RexxUtil is "subset in Phase 7, rest in Phase 10" (`2026-07-27-rust-rewrite.md:115`), and the crate
encodes that: `rust/crates/rexx-exec/src/internal_routines.rs` has 37 `util(..., "Phase 10")` rows
(`grep -n "Phase 10" rust/crates/rexx-exec/src/internal_routines.rs | wc -l`). Those routines are
exercised by `base/rexxutil/*`, `base/class/File`, `base/keyword/ADDRESS`, `base/bif/LINEOUT` and
others. None of these is an extension, native-API or RXAPI group, so the group rules leave them in
Phase 9's gate. **Phase 9's exclusion list has to carry these tests, or Phase 9 has to implement the
routines.** They are 165 of the 232 refusal-blocked tests.

## 3. How it was run

The mechanism is `support/group_runner.rs` plus `concurrency_tests.rs`'s `whole_groups`, ported to
`/tmp/claude-1000/p9scope/a/scripts/suite.py`:

* **Copy.** For each side, `fresh_copy`: `framework/`, the group's directory (recursive),
  `testOORexx.rex`, `worker.rex`, `ooTest.frm`, and `extensions/rxregexp/rxregexp.cls` beside the
  driver. Then `without_rxfuncquery` deletes each `rxfuncquery(` line of the copied `ooTest.frm`
  with the comment above it and the `versions[` assignment below it (asserting that shape).
  `printSummary`'s `RXFUNCQUERY` therefore never runs on either side. `whole_groups`' `mark_starts`
  rewrites the copy's `OOREXXUNIT.CLS` so each test writes `started NAME` to stderr. The
  `reaching_rxapi` tests are renamed `SKIPPED_...` by `skip`.
  The port differs in two ways: files are read and written as Latin-1, byte-preserving, where the
  Rust reads them lossily as UTF-8; and the `SysFileTree` duplicate deletion above.
* **Invocation.** `testOORexx.rex -f <copy>/ooRexx/<dir>/<G>.testGroup -U -V 2`, stdin `/dev/null`,
  cwd the copy.
  * Oracle: `timeout -k 10 300 /home/moritz/dev/repos/ooRexx/build/bin/rexx ...`, with
    `RLIMIT_AS` 1 GiB set before exec (the `ulimit -v 1048576` wrapper),
    `LD_LIBRARY_PATH=<oracle>/lib`, `PATH=<oracle>/bin:$PATH`.
  * Crate: `memcap 2G timeout -k 10 <max(60, 4 x oracle secs)> rexx-run ...`, built at HEAD with
    `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -p rexx-exec --bin rexx-run` (exit 0) and
    copied to scratch (sha256 `17288b82...306e`) so other scouts' builds could not swap it.
    The environment is the harness's `crate_environment`: `LD_LIBRARY_PATH=<oracle>/lib`, and
    `PATH=<dir with rexx -> rexx-run>:$PATH`. So `rexx` resolves to this crate, and `rexxc`,
    `rxqueue` and `rxsubcom` resolve to nothing, which is the state Phase 9 changes.
    One difference from the harness: the crate runs as a `rexx-run` process, as `whole_groups`'
    sim mode does, not through in-process `Invocation`.
* **Iteration.** When the crate's run is cut short (refusal, crash, timeout, OOM), the last
  `started` test is renamed out and the crate side runs again, until a run completes, 3
  timeouts/OOMs, or 40 iterations. The oracle runs once, whole. A refusal raised before any test
  starts cannot be iterated past.
* **Parallelism.** 8 groups at once (`WORKERS=8`), oracle then crate within each worker.
* **Version.** Both sides answer `parse version` with `REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`.

**Wall time: 364 s** for the main pass over all 402 groups (`end.txt - start.txt`). Plus 0.7 s for
the three `10?` groups, 60 s for the two `SysFileTree`-collision re-runs and 361 s for the
`Package` diagnosis. Two oracle runs used their full 300 s (`samples/scclient`,
`samples/sfclient`). The slowest group that finished on the oracle is `base/bif/TIME`, 93.5 s on
the oracle and 93.6 s here. The oracle's runs sum to 824 s.

### Commands, to repeat as a gate

```sh
cd /home/moritz/dev/repos/ooRexx-rust-rewrite/rust
CARGO_INCREMENTAL=0 memcap 8G cargo build --release -p rexx-exec --bin rexx-run
cp target/release/rexx-run /tmp/claude-1000/p9scope/a/bin/rexx-run
cd /tmp/claude-1000/p9scope/a
OUT=results.jsonl WORKERS=8 timeout 14400 python3 scripts/suite.py            # all groups
OUT=dedupe.jsonl python3 scripts/suite.py base/rexxutil/SysFileTree samples/samples
OUT=p10q.jsonl RUN_P10Q=1 WORKERS=1 python3 scripts/suite.py utilities/rxapi \
    utilities/rxqueue/rxQueue utilities/rxsubcom/rxsubcom
python3 scripts/analyse.py results-final.jsonl groups.tsv
```

`suite.py` now applies the duplicate deletion on every run; the main pass ran before it existed,
which is why the two groups were re-run. A gate built from this should be a Rust harness. This is a
Python port, and it has not been reviewed against the Rust line by line beyond the `FUNCTION` check.

## 4. Classification

### Group bins (first cause)

| Bin | Groups |
|---|---|
| green (same tests ran, same failing set, same rc) | 275 |
| loud refusal | 41 |
| wrong answer | 36 |
| crash / panic | 0 |
| timeout / hang | 0 as a first cause; `Package` hangs after a refusal, below |
| out of memory | 0 |
| framework could not load | 0 by that name; 6 groups refuse while loading (below) |
| no tests on host (both sides run 0) | 20 |
| oracle hangs (skipped) | 2: `samples/scclient`, `samples/sfclient` (each waits for a server, killed at 300 s) |
| not run: Phase 10 by group rule | 28 (25 + the 3 `10?`) |

The "no tests on host" row is the 15 `base/rexxutil/platform/windows/*` groups and
`samples/windows/fileNameDialog_demo`, `base/bif/RXQUEUE` and `base/rexxutil/Macrospace` (every
test renamed out as rxapi), and `doc/rexxref/chapter7/Section4` and
`regressions/bug2061_newline_misalignment` (0 tests on the oracle; not read why).

**Refusals at group load.** `RexxInfo~executable` (Phase 9) is called before any test starts in
`samples/samples` (29 oracle tests), `samples/scserver` and `samples/sfserver` (2 each), and three
Windows sample groups that run 0 tests on the oracle (`samples/windows/adsi`, `samples`, `wmi`).
On the crate the Windows ones refuse where the oracle reports no tests, so they are not green either.

**`base/class/Package`.** It refuses at `TEST_NEW01` (`StringTable~items`) and
`TEST_PACKAGE_LOCAL_GET_EMPTY` (`Directory~items`), both Phase 9. With those renamed out, the run
prints its full summary (77 tests, 2 failures, 1 error) and then **does not exit**, killed at 60 s
(rc 124). Renaming tests out one at a time moved the hang without removing it, and a further re-run
with `TEST_PACKAGE_LOCAL` also left out still hung after its summary
(`/tmp/claude-1000/p9scope/a/package.jsonl`). That makes it a hang at program end, not in a test.
Which test leaves the activity or state behind was not isolated. The TSV's 4 `[timeout]` entries
for this group are artifacts of that iteration, not tests that hang.

### Refusal texts, by tests blocked

Each test that stopped a run, counted once (`analyse.py`'s table):

| Tests | Groups | Refusal | Phase named |
|---|---|---|---|
| 40 | 1 | a package settings write | 10 |
| 23 | 2 | `Object~copy` | 9 |
| 18 | 1 | `SysStemSort` | 10 |
| 14 | 1 | `SysFileSearch` | 10 |
| 12 (+33 at load) | 5 (+6 at load) | `RexxInfo~executable` | 9 |
| 12 | 1 | `SysStemCopy` | 10 |
| 11 | 3 | `StackFrame~executable` | 9 |
| 11 | 2 | `SysSearchPath` | 10 |
| 9 | 2 | `SysTempFileName` | 10 |
| 8 | 5 | `StringTable~items` | 9 |
| 7 | 1 | `SysGetMessage` | 10 |
| 6 | 2 | `SysGetFileDateTime` | 10 |
| 5 | 2 | `Object~makeArray` | 9 |
| 5 each | 1 each | `SysFileCopy`, `SysFileMove`, `SysIsFileDirectory`, `SysFormatMessage`, `SysStemDelete`, `SysStemInsert` | 10 |
| 4 each | 1 each | `SysDumpVariables`, `SysSetFileDateTime` | 10 |
| 3 | 1 | `SysDropFuncs` | 10 |
| 2 each | 1 each | `::ROUTINE EXTERNAL` naming `REGISTERED`; `SysGetMessageX` | 10 |
| 1 each | 1 each | `.STDQUE`; LIBRARY REXX `rexx_query_queue`; `SysGetErrorText` | 10 |
| 1 each | 1 each | `Method~new`, `Routine~new`, `Directory~items`, `RexxContext~copy`, `RexxInfo~libraryPath`, `StringTable~allIndexes`, `String~makeArray`, `Object~objectName=` | 9 |

`Package_Options`' 40 are 40 iterations that each removed one test refusing on the same text, and
14 of its tests were still unreached at the cap. "A package settings write" therefore blocks at
least 54 tests in one group.

**What the Phase 9 refusals mean for scope.** Most of them sit at the start of a test and hide
whatever lies behind it. `Object~copy`'s 23 includes 22 in `SecurityManager`, and
`RexxInfo~executable` gates every `samples` group. These Phase 9 rows are the reason the Phase 9
refusals block more than their own count suggests: 67 tests renamed out plus 33 never reached.

### Wrong answers (tests completing with a failure the oracle does not have)

249 tests in 36 groups; the TSV's `tests_wrong` column names each. The first failure's text per
group was read from `runs/<group>/crate-0.out`. Clusters, by inferred cause (from that one message
per group, not diagnosed):

* **Utilities Phase 9 ships**: `rexxc` (8 errors: `External command "rexxc ..."` fails, rc 2 vs 0),
  `rexx_command` (22: `assertRc` expected -1, got 2; option handling of `rexx` as a command),
  `METHOD.TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA` (rc 127, `rexxc` not found).
* **`::REQUIRES` of a file the test writes at run time**: `directives/REQUIRES` (20),
  `directives/CLASS` (6). `Could not find file "activate.cls" for ::REQUIRES` from
  `runDynamicSource`; the file is created by `.TemporaryTestFile`.
* **Conditions not raised**: `SIGNAL` (8, `SIGNAL ON ANY` not catching ERROR/FAILURE/HALT/...),
  `CONDITION` (7), `LABEL` (9, 47.3 not raised), `OPTIONS` (10, 98.974 not raised),
  `LOSTDIGITS` (10), `environmentEntries` (97.1 not raised).
* **Arithmetic / RexxInteger**: `RexxInteger` (19 of 20).
* **Streams**: `STREAM` (6), `Stream` (11), `CHARS` (4), `LINES` (3), `LINEOUT` (4), `CHAROUT` (1),
  the NOTREADY-on-directory/invalid/nonexistent family.
* **RexxUtil already implemented but wrong**: `SysFileTree` (17 of 26: argument validation, `HUGE`,
  tree options).
* **Collections**: `Array` (5, `insert` with an Array index raises 93.907), `IdentityTable` (6),
  `Queue`, `QueueRGF`, `CircularQueue` (`difference`/`xor`), `Bag`, `collections/directory`,
  `collections/queue`, `CollectionMethods`, `String/comparisonOperators.TEST_NIL`.
* **Parser messages**: `expressions/Literals` (12: 15.2/15.3 whitespace in binary/hex literals),
  `incorrectCharacters` (the message substitutes `&1`/`&2` unfilled).
* **Already recorded in `whole_groups::DIFFERING`** (known, owners queued or outside Phase 6):
  `TRACE` (7), `TRACE_TraceObject` (5), `RAISE` (4), `ATTRIBUTE` (2), `Class` (2), `CONSTANT` (1),
  `Object` (5), `RexxContext.TESTRS01`, `METHOD.TESTDELEGATE`, `Method`/`Routine`
  `TEST_NEW_ARRAY_FROM_FILE`/`TEST_NEW_FILE_COMPILED` (compiled files, i.e. `rexxc`).
* Others, one or two each: `Validate` (3), `VarRef` (5), `ANNOTATE` (2), `StackFrame` (1),
  `SecurityManager` (2), `ARG.TEST_EXTERNAL_CALLS`, `SysFileXXX.TEST_FILEEXISTS_NIL_INVALID`.

No test fails on the oracle and passes here; the 4 oracle failures above fail on both sides
(`final failing also on oracle: 4`).

## 5. What this changes for Phase 9's scope

1. The group-level exclusion rules do not cover D11's RexxUtil remainder. 165 of the 232
   refusal-blocked tests name Phase 10 in non-Phase-10 groups (mostly `base/rexxutil/*`), so the
   exclusion file needs test-level rows derived from the refusal owner, or Phase 9 takes those
   routines.
2. Phase 9's own refusals are few (13 texts), but `Object~copy`, `RexxInfo~executable`,
   `StackFrame~executable` and `StringTable~items` sit at the head of whole groups: 67 tests plus 33
   at group load.
3. Shipping `rexxc` and `rexx`'s command line is directly witnessed: `rexxc` (8), `rexx_command` (22),
   `METHOD`'s `rexxc` test, and `Method`/`Routine`'s compiled-file tests.
4. 249 wrong answers in 36 groups are not refusals, so they are this phase's to fix or record.
   Conditions, streams, `RexxInteger`, `::REQUIRES` of run-time files and collection set operations
   are the large clusters.
5. Row 9 ships `rxqueue`/`rxsubcom` while the L3-core paragraph puts their RXAPI side in Phase 10.
   `utilities/rxQueue` and `utilities/rxsubcom` need an explicit owner. `Package` hangs at exit.
   Two socket-sample groups hang on the oracle itself and need excluding from any gate.
