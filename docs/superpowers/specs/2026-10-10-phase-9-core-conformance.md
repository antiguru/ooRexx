# Phase 9: core conformance

**Status:** draft for Moritz's review, 2026-10-11, after three review rounds
(`.superpowers/sdd/2026-10-10-phase-9-scoping/spec-review.md`, `spec-rereview.md`,
`spec-review-fable.md`). Roadmap row 9
(`docs/superpowers/plans/2026-07-27-rust-rewrite.md:661`) is the exit contract, read through this
document; section 7 lists the roadmap edits this spec makes.

**Inputs.** Four scout reports in `.superpowers/sdd/2026-10-10-phase-9-scoping/`, each at
`a98238be5`: `scout-a-report.md` (the whole ooTest suite on both sides, per group in
`scout-a-groups.tsv`), `scout-b-report.md` (benchmarks), `scout-c-report.md` (the binaries, `.rxo`,
the embedding API), `scout-d-report.md` (inherited refusals, subclass `NEW`, queued items). Moritz's
decisions on them are in `decisions.md` in the same directory, restated as rulings R1-R8 in section 6.
Oracle source paths are relative to `/home/moritz/dev/repos/ooRexx`. "Roadmap" below means the plan
file above, and its decision numbers are written "roadmap D11" to keep them apart from this spec's.

---

## 1. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | **L3-core is judged by the suite's own verdict, against a fixed denominator.** A group is green when every test in its expected set (section 2) ran and passed under `testOORexx.rex`, every test that ran is in the expected set or has a row in one of the files of D2, and the run ends normally. Oracle differences are reported in the gate record and never gate. | Moritz, 2026-10-07 (`oracle-is-a-build-tool`): new gates judge by invariants, and ooTest's assertions are those invariants. Without the denominator, a group that discovers no tests is green. Scout A section 1 met that case: a duplicate file in the copy made `SysFileTree` and `samples/samples` run zero tests on both sides. |
| D2 | **Three files hold every test the gate does not require to pass, one row per test, each row with an owner and a mechanically checked reason.** `phase-9-exclusions.txt` holds Phase 10 rows, and Phase 10 deletes it. `suite-licences.txt` is permanent and holds Deviation, environment and flaky rows. `suite-not-run.txt` holds tests the framework does not run on this host, and is permanent too. Section 2 gives each owner's check and the paths. | Scout A section 2: the group rules miss the rows that matter (165 of the 232 refusal-blocked tests sit in groups the gate counts). An exclusion with no checkable reason is a suite that quietly does not run. Permanent rows cannot be removed by Phase 10 work, so they need their own file. |
| D3 | **A refusal naming Phase 10 stays Phase 10's only with a reason from a closed set**: `Rxapi` (needs the daemon), `Extension` (an extension library), `Row10NativeApi` (a row 10 native-API group) or `Embedding` (R4). The reason is a parameter of each constructor shape that names Phase 10 (`util`, `deferred`, `owned_message`, `gap`, `redirection`, `rexx-inventory`'s `EXCLUDED` table, which gains a column), and a structural test asserts every `"Phase 10"` literal under `src/` carries one; literals in `tests/` are out of its scope. Criterion 1 then shows that no gate group reaches a site without one. 9a classifies every `"Phase 10"` site and relabels the ones that are Phase 9's. | Labels rot, as 6.1 found. Non-macrospace RexxUtil was re-homed by R2 (scout D section 2). "A package settings write" (`lib.rs:664`, `fn package_option_write`) is mislabelled: it is `Package~options` and `~defaultOptions` with a value, and `rexx -o` needs it. |
| D4 | **`rexx` and `rexxc` are binaries of `rexx-exec`**, built directly on `run_program` and the parser. `rexx-run` is removed and its callers switched to `rexx` in a 9c task of its own (section 5). The roadmap's `rexx-cli` crate is not created, and historical records are not rewritten. | Scout C section 1: the tests find `rexx-run` through `CARGO_BIN_EXE_rexx-run`, which cargo sets only for binaries of the same package (inferred from cargo's documentation, not run). `rexx` needs no embedding API (R4). |
| D5 | **A `.rxo` file is the oracle's prefix, then a header the oracle rejects, then the checked source** (R5). Restore is interpreter surface, reached through `rexx FILE`, `.Routine~newFile`, `.Routine~new(name, data)` with source, encoded and binary data, `.Method` file loading, `::REQUIRES`, an external `CALL`, and the native `NewRoutine` and `LoadPackageFromData`. A compiled program answers `SOURCELINE`, `PARSE SOURCE` and trace lines as the oracle's compiled program does, which is measured rather than assumed. An oracle-written `.rxo`, or one whose header this build does not accept, raises `Error 3.903` ("cannot be run by this version"), rc 253. | Scout C section 2: the oracle's payload is a heap dump holding pointers and paths, so neither direction can be compatible. The ooTest groups write and read within one implementation, and they assert the first 29 bytes. A header that fails the oracle's `validate` makes the oracle answer 3.903 rather than crash, and a truncated image does crash it, rc 139. |
| D6 | **Every parse-error secondary message fills its inserts, byte-identical to the oracle** (R6). The raise goes through an API that cannot be called without the inserts its message needs, which is 6.1 R3's condition for a reversal. A committed script re-derives `corpus/errors/parse-errors.tsv` from the oracle's `rexxc` with the secondary message line, and the gate compares this crate's `rexxc` against the committed file. A structural test requires each (major, sub) the raising API can emit to have a TSV row, or a recorded reason that no program reaches it. | `rexx` and `rexxc` print exactly these messages, and the `incorrectCharacters` group fails on them (scout A section 4). The TSV alone would leave any message no row raises unfilled. |
| D7 | **Out of scope:** `rxqueue`, `rxsubcom` and the `utilities/rxapi`, `rxQueue` and `rxsubcom` groups (R1); benchmark parity (R3, R8); the embedding API and what depends on it (R4); `rexximage`; the macrospace routines and `RXFUNCQUERY`; the other platforms (roadmap D-P1). | Moritz's rulings. `rexximage` has no tests (`ootest/ooRexx/utilities/rexximage/` is empty), and roadmap D2 needs no image. `RXFUNCQUERY` is Phase 10's in `rexx-inventory`'s `EXCLUDED` table, and Phase 10 executes the RXAPI decision (roadmap D7). |

---

## 2. The suite instrument (D1, D2)

`tests/suite.rs`, gated by `REXX_SUITE_GATE=1`, runs every group `testOORexx.rex` discovers under
`ootest/ooRexx`, on this crate and on the oracle. It generalises `whole_groups` and scout A's Python
port (`/tmp/claude-1000/p9scope/a/scripts/suite.py`), and reuses `support/group_runner.rs`'s
`fresh_copy`, `without_rxfuncquery`, `reaching_rxapi`, `skip` and `interpreter_bin`. Each property
below comes from scout A section 3 or from the reviews:

* **Copy.** Each group gets a fresh copy per side. `ooTest.frm`'s `rxfuncquery` lines are removed, as
  `without_rxfuncquery` removes them. `reaching_rxapi` tests are renamed out on both sides. A group
  file with the same name in a subdirectory of the copy is deleted (this affects
  `base/rexxutil/SysFileTree` and `samples/samples`). Files are read and written byte for byte:
  `skip`, `read_lossy` and `without_rxfuncquery` change from UTF-8 to byte reads and writes, and
  `whole_groups` runs on the same code.
* **Crate side.** This side is a child process of this crate's `rexx`, reached through a `bin`
  directory in the copy that also holds this crate's `rexxc`. Its `PATH` contains no directory holding
  an oracle binary. The harness asserts on each run that `rexx` resolves inside that `bin`, and that
  `rexxc` resolves nowhere on the crate side's `PATH` until 9c delivers it and inside that `bin` from
  then on. Until 9c, `rexx` is a symlink to `rexx-run`.
* **Bounds.** Oracle runs get 300 s. Crate runs get four times the oracle's time, and at least 60 s.
  Every crate run is under `memcap 2G`. When `command -v memcap` fails, the crate run is under
  `ulimit -v 4194304` instead, and the table says so. Every oracle run is under the `ulimit -v`
  wrapper. Eight groups run at once.
* **Iteration.** A loud refusal, a crash, a timeout or an out-of-memory kill ends a crate run (a
  refusal is rc 120). The harness then renames the last started test out and runs again, as scout A
  did, up to 60 runs per group. The refusal text each iteration captured is recorded against that
  test. A test is "ran" if any iteration started it, and it passed only if it passed in the iteration
  that ran it.
* **What a test is.** A test is its uppercased method name within its group. Ran is the set of names
  in the `started` markers, as `whole_groups`' `mark_starts` writes them. The framework's own
  tests-run figure is reported in the table and is not the denominator.
* **Denominator.** A group's expected set is its test methods, enumerated statically by
  `rexx-extract`'s `extract`, minus the group's rows in all three files. `suite-not-run.txt` lists
  the tests the framework does not run on this host, each with its reason: `restrictOS` (for example
  `bug2061_newline_misalignment.testGroup:41`), a test type the framework skips by default (for
  example `doc/rexxref/chapter7/Section4`'s `DOC_EXAMPLE_NOISE_TEST`), or anything else `extract`
  counts that the framework never starts. The harness checks each reason.
* **Suite revision.** `ootest/` is an untracked svn checkout (revision 13178 at the scouts). The gate
  record and the three files name the revision they were derived at, and the harness asserts it.
* **Output.** A table per group gives tests expected, started, passed, failed and blocked (with the
  refusal text), per side. The table and the oracle-difference report go to a path the gate record
  cites.

**Owners and their checks** (D2). One test reads all three files. Their paths are
`docs/superpowers/plans/phase-9-exclusions.txt` (named by the roadmap), and
`rust/corpus/suite-licences.txt` and `rust/corpus/suite-not-run.txt`. `suite-licences.txt` and
`suite-not-run.txt` are permanent. A row's test may run, fail, or end the run with a refusal; the
iteration above carries the run past it.

* **Phase 10, `Rxapi`.** These are rows renamed out by `reaching_rxapi`. The set of those rows equals
  `reaching_rxapi`'s output over the suite. This puts `SysGetMessage.TEST_CATALOG_INVALID` and
  `TEST_GENCAT` here, through `RXFUNCQUERY`.
* **Phase 10, any other reason.** The test sits in a group the roadmap makes Phase 10's (extensions,
  the row 10 native-API groups, the three `utilities` groups of R1), or the refusal text its
  iteration captured names Phase 10 with a D3 reason.
* **Deviation N.** The row cites a number under `phase-4-exclusions.txt`'s `DEVIATIONS -- permanent,
  chosen`, and the test checks that the number exists there. A Deviation row may be a loud refusal
  (Deviation 20, Deviation 28). A Deviation is added only by Moritz's ruling.
* **Environment.** The row cites a committed oracle run record that shows the test failing or hanging
  on this host. Each row needs Moritz's ruling, recorded in the file. The two socket samples the oracle
  hangs on (`samples/scclient`, `samples/sfclient`) are the first candidates. An environment row whose
  test hangs is renamed out on both sides, as `reaching_rxapi` tests are.
* **Flaky.** The row cites `.github/known-test-failures/common.txt` by entry, and the test is allowed
  to pass or fail. This covers `CHAROUT/TEST_STDOUT_START`, `LINEOUT/TEST_LINE_TRANSIENT` and the
  two `SYSSLEEP` timing tests.

**The four oracle `STREAM`/`Stream` relative-file failures** (scout A section 1) are not in
`common.txt`. 9a finds out whether the harness's copy causes them. A harness cause is fixed.
Otherwise each becomes an environment row, by ruling.

**The existing lists.** `whole_groups` keeps `REST_LEFT_OUT`, `DIFFERING` and
`whole-groups/alone.txt` for sim mode. A test asserts that every test named in a `DIFFERING` key's
failing list, and its `last started` test, is fixed or has a row in one of the three files.

## 3. Refusals and silent divergences (9a, 9d)

**Phase 9's own refusals** (scout D section 1, scout A's refusal table) belong to 9a. Each one is
implemented as the oracle answers, with a corpus program or a group test as its witness:

* **The `Loud::native_method` family.**
  * `RexxInfo~executable` and `~libraryPath`. `~executable` stops `samples/samples`, `scserver`,
    `sfserver` and three Windows sample groups at load.
  * `StackFrame~executable`.
  * `Object~copy` on Method, Routine, Package, RexxContext (93.970) and RexxInfo.
  * `OBJECTNAME=` on a String, a SmallInt and the other receivers that have no variable pool.
  * `MAKEARRAY` on an `~UNKNOWN` argument list that is not an Array.
  * The hash methods on the condition object and on `.methods`: `StringTable~items`, `~allIndexes`,
    and `Directory~items`.
  * A metaclass with its own `NEW`.
  * Two further sites: `SUPPLIER` in `~defineMethods` (`class_protocol.rs:577`) and `Message~notify`
    (`object_protocol.rs:1220`). Each is implemented as the oracle answers if a probe reaches it.
    Otherwise it moves to a refusal constructor that names no phase, with a GUARD row in
    `refusal-dispositions.tsv` naming the probes tried.
* **`NEW` on a subclass of String, Stem, Method, Routine and Message** (scout D section 3).
  * The 6.1 prototype patch no longer applies to `class_protocol.rs`, `construct.rs` and
    `environment.rs`, so it serves as a reference, not a base.
  * `Class~enhanced` running `NEW` is part of this work.
  * So are the queued subclass items: `2026-10-10-subclass-package`, `-subclass-rexxinfo`,
    `2026-09-30-stringtable-subclass-new-arg` and `2026-09-30-hash-subclass-init-capacity`.
* **"A package settings write"** (D3). It blocks at least 54 tests of `Package_Options`.
* **`ReleaseLocalReference`.** On the queued probe (`2026-09-29-release-local-reference-memory`),
  peak RSS at 2,000,000 iterations is at most twice peak RSS at 200,000. Today it is 430 MB against
  the oracle's 20 MB at 2,000,000. The probe's C source is committed, and the test builds it with the
  system `cc` against `api/`'s frozen headers and reads peak RSS from the child's rusage; a test that
  finds no `cc` fails.

**`base/class/Package` hangs at program end** after printing its summary (scout A section 4). 9a
isolates the test that leaves the state behind.

**Silent divergences belong to 9d.** In the TSV's `tests_wrong` column, 249 tests in 49 groups fail
here and pass on the oracle. 9d's set is those 249 minus the 52 that sections 4 and 5 name as 9b
and 9c witnesses: 22 from `rexx_command`, 8 from `rexxc`, `METHOD`'s `rexxc` test, the 4
compiled-file tests in `Method` and `Routine`, and 17 from `SysFileTree`. 9d also owns whichever of
the other 9b and 9c witnesses (`SysFileXXX.TEST_FILEEXISTS_NIL_INVALID`, the `incorrectCharacters`
tests) are still failing when 9b and 9c close. 9a commits the set as a file derived from the TSV. Its clusters:

* conditions not raised: `SIGNAL ON ANY`, `CONDITION`, `LABEL` 47.3, `OPTIONS` 98.974, `LOSTDIGITS`,
  and 97.1 from `environmentEntries`;
* `RexxInteger`;
* streams: the NOTREADY family on directories and invalid names;
* `::REQUIRES` of a file written at run time;
* the collection set operations, and `Array~insert` with an Array index;
* conditions and stack frames under INTERPRET, together with the queued traceback family:
  * `2026-10-10-prologue-frames-missing-loading-levels`
  * `-propagated-condition-extra-traceback-line`
  * `-reply-propagated-condition-traceback-line`
  * `2026-10-03-interpret-translation-error-traceback`
  * `2026-10-02-frame-executable-invocation-and-native-levels`

6.1 landed the INTERPRET traceback line (`59eb57f37`). `2026-10-07-interpret-syntax-traceback-line`'s
remaining half, its inserts, is D6's. Every test in the 9d set is fixed, or it gets a
`suite-licences.txt` row naming a numbered Deviation that Moritz has ruled on.

## 4. RexxUtil (9b)

9b takes the routines `internal_routines.rs` refuses as Phase 10, apart from the seven macrospace
routines, which need the daemon (R2). 9a relabels them from Phase 10 to Phase 9. The routines are
`SysCls`, `SysCreatePipe`, `SysDropFuncs`, `SysDumpVariables`, `SysFileCopy`, `SysFileMove`,
`SysFileSearch`, `SysFork`, `SysFormatMessage`, `SysGetErrorText`, `SysGetFileDateTime`, `SysGetKey`,
`SysGetMessage`, `SysGetMessageX`, `SysIsFileDirectory`, `SysIsFileLink`, `SysLoadFuncs`,
`SysOpenEventSem`, `SysOpenMutexSem`, `SysQueryProcess`, `SysSearchPath`, `SysSetFileDateTime`,
`SysSetPriority`, `SysStemCopy`, `SysStemDelete`, `SysStemInsert`, `SysStemSort`, `SysTempFileName`,
`SysUtilVersion` and `SysWait`.

9b also takes:

* the named-semaphore refusal at `lib.rs:511`;
* whichever other `"Phase 10"` sites 9a's D3 classification gives to Phase 9, such as `handle_set`
  (`dispatch/native.rs:264`) and classic-style libraries (`dispatch/library.rs:838`);
* the implemented routines the suite finds wrong: `SysFileTree` (argument validation, `HUGE`, the
  tree options) and `SysFileXXX.TEST_FILEEXISTS_NIL_INVALID`;
* `LIBRARY rexxutil` as a package name, which answers 98.903 here where the oracle loads it
  (`phase-4-exclusions.txt:6123-6135`, re-owned to Phase 9 on agreement).

9b's gate groups are the ooTest groups that call any of these routines, derived by
`grep -rlwi` over `ootest/ooRexx` for the routine names, plus `SysFileTree` and `SysFileXXX`, and
committed as a file at 9b's start. A routine with no suite witness gets a corpus program checked
against the oracle where its behaviour is observable from a program, and a unit test otherwise (for
example `SysGetKey`, which reads a terminal). A routine that turns out to need the daemon keeps its
Phase 10 label with the `Rxapi` reason. The
oracle source is `interpreter/platform/unix/SysRexxUtil.cpp` and
`interpreter/runtime/RexxUtilCommon.cpp`.

## 5. The binaries (9c)

**Parse-error inserts** (D6) come first, because both binaries print them.

**`rexx`** follows `utilities/rexx/platform/unix/rexx.cpp` (scout C section 1):

* Options are read only before the program name: `-e`/`-E`, `-o`/`-O`, `-od`/`-OD` and `-v`/`-V`. Any
  other `-x` is ignored but counted against `.SYSCARGS`.
* Usage text goes to stderr, rc 255. `-v` prints the version banner.
* The program is found by the interpreter's resolver, the search in `require.rs` made reachable. A
  program that is not found raises 3.901, rc 253.
* `ARG(1)` is the joined words, and `.SYSCARGS` the separate ones. Under `-e`, `.SYSCARGS` is `.nil`.
* The exit status is derived as the oracle derives it.
* It keeps everything `rexx-run.rs`'s `main` does: signal handlers, standard input, the transient
  flags of the real descriptors, sinks, `REXX_SWITCH_MODE` with its `rexx-sim:` stderr line, and the
  8-bit truncation of the exit code.

**`rexxc`** follows `utilities/rexxc/platform/unix/RexxCompiler.cpp`:

* The command line is `rexxc input [output] [-s] [-e]`.
* `-s` silences the banner. With no output file, it only checks syntax. `-e` base64-encodes the
  output.
* An output name equal to the input is refused, rc 254.
* Messages and exit statuses are the oracle's.
* The file is written per D5.

**The `rexx-run` switch** is a task of its own. It covers:

* the test files using `CARGO_BIN_EXE_rexx-run`;
* `tests/support/arity.rs:374`, which runs `target/release/rexx-run` by path;
* the bench tools: `rexx-bench-suite.rs`, `report.rs`, `rexx-bench-band.rs`, `rexx-arms.rs`,
  `child.rs`, `wallclock.sh` and `PINNED.md`;
* the prose sites `rust/CLAUDE.md:38`, `rust/corpus/README.md:295` and `rexx-exec/src/lib.rs:2718-2719`;
* `group_runner.rs`'s sim-mode processes and its `interpreter_bin`;
* an audit of the assertions that rely on rc 2 or on a literal path;
* the perf A/B of `rexx` against `rexx-run` (section 8).

## 6. Rulings

Moritz, 2026-10-10 and 2026-10-11, on the survey and the review (`decisions.md`):

| # | Question | Ruling |
|---|---|---|
| R1 | `rxqueue` and `rxsubcom` | Phase 10's, unless they block Phase 9 work. They are pure RXAPI clients (scout C section 3), and they block none. The only use outside `utilities/` is `RexxQueue.testGroup:600` (`TEST_EXTERNAL`, renamed out as rxapi), and `rxsubcom` appears elsewhere only in comments (`OPTIONS.testGroup:199,222`). |
| R2 | Non-macrospace RexxUtil | Phase 9's. The macrospace routines stay Phase 10's. |
| R3 | "Every benchmark at parity" | Struck from Phase 9: "2x is achievable, 1x will be difficult as-is." A new optional row after 9 targets every axis at 2x or better, with 1x as the stretch goal. |
| R4 | The embedding API | Phase 10's. That covers `RexxCreateInterpreter`, `RexxStart`, the `librexx.so` cdylib and its SONAME, the second-instance row (`phase-4-exclusions.txt:5284`), `layout::REFUSING_MEMBERS`, and the `AttachThread` and `AddCommandEnvironment` panics (`ffi.rs:4187`, `:4229`). `ReleaseLocalReference` stays Phase 9's. |
| R5 | The `.rxo` format | The checked source behind the oracle's prefix: "let's not agree on a stable format yet". The format promises no compatibility, not even between versions of this crate. |
| R6 | Parse-error inserts | Every insert filled, byte-identical. This reverses Phase 3's 2026-07-28 text deferral for secondary messages. |
| R7 | Splitting | Phase 9 has sub-phases, each with its own plan and gate (section 9). |
| R8 | Which phases the parity exemption covers | Phases 9, 10 and 11. Row 9.1 owns parity, and every phase from 9 on carries only the non-regression budget. |

## 7. Roadmap and record edits

**On agreement**, in the commit that marks this spec agreed:

* **Global Constraints, the parity gate (:46).** "Every phase from 9 on carries the non-regression
  budget of the Phase 9 spec section 8 instead, and row 9.1 owns parity" (R8).
* **The parity back-references at :655 and :725**, which cite the gate as ":39", cite ":46".
* **The Phase 4 debt-route paragraph (:726).** It gains a sentence: the closed debt route binds
  through Phase 8, and from Phase 9 on row 9.1 holds the bar.
* **Row 3 (:654).** The note about message text records R6's reversal for parse-error secondary
  messages.
* **Row 9.** L3-core per D1 and section 2. "Every benchmark at parity" is replaced by section 8's
  budget. `rxqueue` and `rxsubcom` are removed, as are the embedding API and the second-instance text.
  Non-macrospace RexxUtil and the D3 audit are added, and the sub-phases are named. "Starting from"
  the prototype patch becomes "with the prototype patch as a reference".
* **The L3-core paragraph (:721).** The exclusions are enumerated per test, in the files of D2, and
  Phase 10 deletes `phase-9-exclusions.txt`.
* **Row 10.** It gains `rxqueue`, `rxsubcom`, the three `utilities` groups, the embedding API and
  R4's list. "L3-full with no exclusions" becomes "no exclusions but `suite-licences.txt` and
  `suite-not-run.txt`".
* **Roadmap D11 (:115).** "Rest in Phase 10" becomes "the macrospace routines in Phase 10, the rest
  in Phase 9".
* **The crate tree (:780).** The `rexx-cli/` line is removed (D4).
* **A new row 9.1, "Performance to 2x (optional)".** It targets every benchmark axis at 2x the
  oracle or better, with 1x as the stretch goal (R3). It measures wall clock, with cycles as the
  stand-in where wall clock is too noisy (Moritz, 2026-10-11). Entry: 9. The row text marks whether
  L3-full waits for it as Moritz's open item. Its inputs:
  * scout B's table;
  * `dirread` (4.84x wall) and `extcall` (2.02x), which no record attributes;
  * `startup`, at 12.9x instructions;
  * `dispatch` and `dispatchclass`, at 1.57x and 1.64x cycles where `paths-to-parity.md`'s source
    had 1.23x. Scout B notes that figure came from a different instrument, and partly lines it up
    with Phase 6's accepted overruns, so it is not a clean A/B.
* **`phase-4-exclusions.txt`.**
  * `:5284` (the second instance) is re-owned to Phase 10.
  * `:6179` is split: `ReleaseLocalReference` stays Phase 9's, and parity goes to Phase 9.1.
  * `:6135` (`LIBRARY rexxutil`) is re-owned to Phase 9. `:5516`, also "OWNER: Phase 10", is
    classified by 9a's D3 pass.

**At delivery**, by the sub-phase named:

* 9a relabels the `"Phase 9"` strings R4 moves to Phase 10 with the `Embedding` reason: the five
  `REFUSING_MEMBERS` rows (`layout.rs:379-383`) and the `ffi.rs:4187` and `:4229` assertion texts.
  It finds them with `closed_phases`' `mentions` over `rexx-api`, since a `grep '"Phase 9"'` misses
  the two inside longer literals.
* 9a corrects `rust/CLAUDE.md:13` ("That separate `-O3` build is gone from this machine"). Scout B
  found a `Release` build at `ooRexx-rust-rewrite/build`: 5.3.0, built 27 Jul 2026, from a source
  three days older than `build/`'s.
* 9c closes `phase-4-exclusions.txt:5542` (`rexxc`), and 9d closes `:6114` (INTERPRET).
* 9d removes `"Phase 9"` from the open-phase vocabularies (`closed_phases`' open set,
  `refusal_dispositions.rs`'s `OPEN_PHASES`, `owners.rs`'s `PHASES` and `SPLIT_TABLE_PHASES`), adds
  `"Phase 9.1"` where a row may name it, and teaches `closed_phases`' `names()` that `Phase 9` is not
  `Phase 9.1`. 9a adds `"Phase 9.1"` to `OPEN_PHASES` already, for its split of `:6179`.

## 8. Performance

The instrument is 6.1's: `rust/bench-programs/callgrind.sh` on **every** program, against a pin.
The budget is +0.5% Ir on every program, measured once at the Phase 9 close (Moritz, 2026-10-11:
drift is accepted as a risk): this crate's `rexx` against `rexx-run` built at `a98238be5`, whose
sha256 9a records. `rust/` is unchanged between `a98238be5` and this spec's commits.

9c's `rexx-run` switch first adds `rexx` beside `rexx-run` in one commit and A/Bs the two binaries
built from that commit, then removes `rexx-run` in the next. The A/B is recorded so that the front
end's share of the close figure is known. It sits inside the budget, not on top of it. A regression found
there gets fix rounds under 6.1's stopping rule: three rounds, then the figures go to Moritz. Wall clock is reported, not
gated.

## 9. Sub-phases

Each sub-phase has its own plan and gate record (`phase-9a-gate.md` and so on). 9a comes first,
because the others are measured with its instrument. 9b and 9c are independent of each other, except that
the `rexx-run` switch changes the harness 9b's gate runs on: it lands only after 9b's gate has run. 9d
closes the phase.

| Sub-phase | Delivers | Its gate shows |
|---|---|---|
| 9a Instrument and refusals | Section 7's 9a edits; the perf pin; `tests/suite.rs`, `phase-9-exclusions.txt`, `suite-licences.txt` and `suite-not-run.txt` with their checks (section 2); the D3 classification, relabels and structural test; the four `STREAM` failures explained; the 9d set file; the Phase 9 refusals of section 3; the `Package` hang. | The suite runs whole, and its table is recorded. Every section 3 refusal's witness passes. The exclusion-file test and the D3 test pass. The denominator check passes on every group with no test blocked by a refusal 9b or 9c owns, and the blocked tests are listed in the table with their refusal text. `Package` ends. The sub-phase close command passes. |
| 9b RexxUtil | Section 4. | The groups section 4 names pass under the suite harness, apart from rows the exclusion test accepts. No `"Phase 10"` site in RexxUtil lacks a D3 reason. The sub-phase close command passes. |
| 9c Binaries | D6's inserts and their scripts, with `parse-errors.tsv`'s new message column and its reader `rexx-parse/tests/errors.rs`; `rexx` and `rexxc` (section 5); the restore surface (D5); the `rexx-run` switch and its A/B; section 7's 9c edit. | Criterion 4. The switch's A/B is recorded. The sub-phase close command passes. |
| 9d Divergences and close | Section 3's 9d set. The exclusion files reduced to Phase 10, Deviation, environment and flaky rows. Section 7's 9d edits. Section 10. | Section 10. |

**Checks** (Moritz, 2026-10-11). Three levels:

* **Per task**, at most 5 minutes in all, as each brief states: `cargo fmt`, `clippy -D warnings`
  on the crates the task touched, the test binaries covering what changed, and the ooTest groups the
  task touches. A task moves on once its review is approved and these are green. Much of the work is
  mechanical: a new RexxUtil routine or a missing native method is one function and one witness, and
  such additions are batched several to a task.
* **Sub-phase close:** what its gate cell names, which includes the sub-phase close command,
  6.1's G4: `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release -j 4 --no-fail-fast`.
  Without the variable, the corpus differential only reports.
* **Phase 9 close:** G1-G9, the TSan run, the whole-suite gate of criterion 1 and `callgrind.sh` on
  every program (section 8).

## 10. Exit criteria

1. **L3-core.** `REXX_SUITE_GATE=1`, over every group `testOORexx.rex` discovers at the closing
   commit, is green per D1, denominator included. Its table and the oracle-difference report are in
   `phase-9-gate.md`.
2. **Exclusions.** `phase-9-exclusions.txt` holds only Phase 10 rows. `suite-licences.txt` holds only
   Deviation, environment and flaky rows, and `suite-not-run.txt` only rows with a section 2 reason. The owner checks of section 2 pass.
3. **Labels.** The D3 structural test passes. `closed_phases` polices Phase 9, and no source string or
   open OWNER row names it.
4. **Binaries.** Under criterion 1's harness, `rexx_command`, `rexxc`,
   `METHOD.TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`, and `Method` and `Routine`'s
   `TEST_NEW_ARRAY_FROM_FILE` and `TEST_NEW_FILE_COMPILED` pass. Their expected sets (section 2)
   are 30 tests for `rexx_command` and 8 for `rexxc`. This
   crate's `rexxc` agrees with the committed `parse-errors.tsv` on every row, and D6's coverage test
   passes. `tests/compiled.rs` witnesses D5: it spawns `CARGO_BIN_EXE_rexxc` and the oracle's
   `rexxc` explicitly, one per side, asserts each compile's rc 0 and the 29-byte prefix, and runs one
   program per restore route (`rexx FILE`, `.Routine~newFile`, `.Routine~new(name, data)` in three
   shapes, `.Method` file loading, `::REQUIRES`, an external `CALL`) and per observable
   (`SOURCELINE`, `PARSE SOURCE`, trace lines) on each side's own output. An oracle-written `.rxo`
   is a committed fixture, and this crate answers 3.903, rc 253, on it. The native routes
   (`NewRoutine`, `LoadPackageFromData`) are witnessed by `METHOD.TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`
   under criterion 1's harness.
5. **Performance** stays within section 8's budget on every program.
6. **Gates.** G1-G9 pass as at the 6.1 close, with `REXX_CORPUS_GATE=1`, and the Phase 6 TSan run is
   repeated.
