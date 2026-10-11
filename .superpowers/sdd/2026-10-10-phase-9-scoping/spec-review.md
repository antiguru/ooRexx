# Review of the Phase 9 spec draft

Spec: `docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md` (untracked), reviewed at HEAD `a98238be5`.
Read-only: grep and read only, nothing run.
Line numbers are the draft's.
"R:" is `docs/superpowers/plans/2026-07-27-rust-rewrite.md`.

Verdict: not ready.
The four blockers follow.
* The roadmap's Global Constraints parity gate still binds Phase 9.
* The L3-core gate has no pinned denominator.
* The crate side of the harness can run the oracle's `rexxc`.
* Two of the three exclusion owners have no reason test.

## Blockers

### 1. Striking parity from row 9 does not lift the Global Constraints parity gate

* **Spec:** :137 (R3), :147 and :152-156 (section 7).
* **Evidence:** R:47 says "No phase from 2 onward closes with a Rust subsystem slower than its C++ counterpart on the Phase 0 benchmark suite … This is the rule everywhere unless a phase says otherwise."
  Scout B (section 3a) measures 15 of 21 loop axes slower, plus `rexxcps`.
  Section 7 edits row 9 but not R:47.
  It also leaves R:726, the closed debt route for `arith`, `strings` and `rexxcps`, three axes still over.
  `phase-4-exclusions.txt:6179` is "OWNER: Phase 9 (benchmark and resource parity)" and the spec gives it no new owner.
  `decisions.md` strikes parity "from Phase 9" only. It says nothing about R:47 or about Phases 10 and 11.
* **Fix:** add the R:47 and R:726 amendments to section 7, worded as "Phase 9 says otherwise: section 8's budget replaces parity for this phase".
  Ask Moritz whether the same exemption applies to rows 10 and 11, and record the answer.
  Split row 6179 into ReleaseLocalReference (Phase 9) and parity.
  Give parity an owner that `closed_phases` will accept (see finding 18).

### 2. The L3-core gate (D1, criterion 1) has no pinned denominator

* **Spec:** :20, :182.
* **Evidence:** D1 counts a group green when "every test not in `phase-9-exclusions.txt` passes … and the run ends normally".
  A group whose framework discovers no tests on the crate side meets that.
  So does a group that discovers fewer tests than it has.
  The failure mode has already happened: scout A section 1 records `SysFileTree` and `samples/samples` running "zero tests" on both sides because of a duplicate file in the copy.
  Scout A's own green bin (section 4) requires "same tests ran". The spec dropped that.
* **Fix:** in criterion 1, require that per group the set of tests the crate ran equals the group file's test methods, enumerated statically from `::method test*`, minus that group's exclusion rows.
  Enumerating statically keeps the check free of oracle judgement.
  List the groups that legitimately run zero tests on this host in a committed file, each with its reason (scout A's 20).

### 3. The crate side of the harness can run the oracle's `rexxc`, so criterion 4 can pass with the wrong binary

* **Spec:** :32-45, :124-125, :188.
* **Evidence:** `group_runner.rs:469` `crate_environment` builds PATH as `once(bin).chain(env::var_os("PATH")…)`.
  The caller's PATH is inherited.
  Scout C section 2: "With the oracle's `bin` on PATH, both sides run the oracle's `rexxc`."
  If `rexxc` is missing from the crate's `bin`, or misnamed, the `rexxc` group, `METHOD.TEST_REXXC_…` and the Method and Routine compiled-file tests run the C++ compiler.
  The crate then only has to restore what the oracle wrote, which D5 says cannot happen.
  The tests would therefore fail, but they would fail for the wrong reason.
  The reverse also holds: if this crate's `rexx` invoked `rexxc`, it would reach the oracle's.
* **Fix:** in section 2, require that the crate side's PATH holds this crate's `rexx` and `rexxc` and no directory containing an oracle binary.
  Have the harness assert, per run, that `rexxc` resolves inside the copy's `bin`.

### 4. Two of the three exclusion owners have no reason test, so they swallow real failures

* **Spec:** :21, :94-95, :184-185.
* **Evidence:** D2 defines a reason test only for Phase 10 rows ("refuses with a Phase 10 text or sits in a Phase 10 group").
  Criterion 2 requires "its reason test passes" for every row, including "a numbered Deviation or environment".
  Neither of those has a defined test.
  A Deviation row is checkable only as "the number exists". The `phase-4-exclusions.txt` header (:12-19) makes adding one a plan amendment, and the spec does not say which numbering it means (see finding 5).
  An "environment" row is defined as "the oracle itself fails or hangs here". That is oracle-judged, and D1 says oracle differences never gate.
  The scout already shows the drift: the four `STREAM` and `Stream` oracle failures (:55-57) are to become environment rows if 9a finds no harness cause.
* **Fix:** give each owner a mechanical test.
  * **Deviation:** the row cites `phase-4-exclusions.txt` "N." under `DEVIATIONS -- permanent, chosen`, and the test checks that N exists there.
  * **Environment:** the row cites a committed oracle run record naming the test as failing or hanging there.
    Or, if Moritz prefers, environment rows require his ruling per row, recorded in the file.
  * **Phase 10, for rows that are renamed out and never run:** see finding 11.

## Major

### 5. The D59 exclusion example is wrong three ways

* **Spec:** :51-52, "`Class.TEST_SUBCLASSES_GC`, D59".
* **Evidence:**
  * D59 is a decision in `docs/superpowers/specs/2026-08-27-phase-5b-instances.md:1036`, not a numbered Deviation. The Deviations are 0-30, in `phase-4-exclusions.txt:715-1914`.
  * D59 was "CORRECTED, 2026-09-08 by Moritz" (:995) and "RESOLVED 2026-09-09 BY PHASE 5j" (:1021). :1033-1034 says "Nothing may cite it as licensing anything from 2026-09-08".
  * `scout-a-groups.tsv`'s `base/class Class` row has `tests_wrong` = `TEST_ACTIVATE TEST_METHODS`. `TEST_SUBCLASSES_GC` is not failing in the suite run.
  * The citation was copied from `phase-6-gate.md:1137`, which already breaks D59's own rule.
* **Fix:** delete the example.
  If a real Deviation example is wanted, take one from Deviations 9-30 that scout D section 5 names.

### 6. `StackFrame~executable` is missing from section 3

* **Spec:** :61-80.
* **Evidence:** scout A's refusal table has `StackFrame~executable`, Phase 9, 11 tests in 3 groups. The JSONL attributes Method 3, Routine 1 and StackFrame 7.
  `dispatch/context.rs` binds no `EXECUTABLE` for StackFrame, so the send falls to `dispatch.rs:2409` `Loud::native_method`, which names Phase 9.
  Criterion 3 would force it eventually, but no sub-phase owns it.
* **Fix:** add it to the `Loud::native_method` bullet at :64.

### 7. "249 tests in 36 groups" is wrong, and 52 of the 249 are owned twice

* **Spec:** :85-95, :107-108, :119, :124-125, :178.
* **Evidence:** summing the TSV's `tests_wrong` gives 249 tests in 49 groups.
  36 is the count of groups whose first cause is a wrong answer, and they hold 211 tests. The other 38 sit in 13 refusal-first groups.
  Scout A :229 carries the same error.
  Of the 249, 52 are witnesses that section 4 or 5 assigns to 9b or 9c:
  * `rexx_command`: 22
  * `rexxc`: 8
  * `METHOD.TEST_REXXC_…`: 1
  * Method and Routine compiled-file tests: 4
  * `SysFileTree`: 17

  Section 9 also gives "Section 3's silent divergences" to 9d whole.
* **Fix:** write "249 tests in 49 groups".
  Define 9d's set as those 249 minus the named 9b and 9c witnesses.
  Better, commit the 9d list as a file derived from the TSV.

### 8. The D3 audit lands in 9a while the labels it judges change in 9b and 9c, and it is not structural

* **Spec:** :22, :106, :175.
* **Evidence:** at 9a, the 37 `util(…, "Phase 10")` rows (`internal_routines.rs:99-164`) and `lib.rs:664` still name Phase 10.
  Gate groups reach them, so by D3 they are Phase 9's.
  The audit test therefore either fails at 9a's close or 9a relabels them, and the spec says neither.
  Separately, "a refusal … that a gate group reaches" is a runtime fact. "A structural test checks every `"Phase 10"` site against that list" has no input from which to decide it.
* **Fix:** state that 9a relabels the 30 routines and the package-settings write to "Phase 9", so that 9b and 9c remove those labels.
  Make each surviving `"Phase 10"` site carry a reason tag from a closed enum: `Rxapi`, `Extension`, `Row10NativeApi`, `Embedding`.
  The structural test then asserts every `"Phase 10"` literal carries one.
  The runtime half (no gate group reaches an untagged site) is then criterion 1's job, not this test's.

### 9. The "GUARD disposition" route in section 3 cannot be implemented as written

* **Spec:** :69-70.
* **Evidence:** both sites name Phase 9 through `Loud::native_method`:
  * `class_protocol.rs:577`
  * `object_protocol.rs:1220`

  `refusal-dispositions.tsv` fails "on a row for a constructor that names a phase (REHOME aside)".
  A GUARD row therefore requires a constructor with no owner, and `native_method` always passes `Some("Phase 9")` (`lib.rs:612`).
* **Fix:** "Each site is implemented as the oracle answers if a probe reaches it. Otherwise it moves to a refusal constructor that names no phase, with a GUARD row in `refusal-dispositions.tsv` naming the probes tried."

### 10. Removing `rexx-run` breaks the perf pin, the bench tools and the 9a harness, and is not "a string change"

* **Spec:** :23, :162-166, :177.
* **Evidence:** section 8 pins a binary built at the base commit, and that binary is `rexx-run`.
  After 9c, `callgrind.sh` (`NAME=BINARY` arguments) would compare `rexx` against pinned `rexx-run`. The resolver and option front end then land inside the +0.5% budget.
  Hard-coded `rexx-run` paths sit outside the 12 `CARGO_BIN_EXE` files:
  * `rexx-bench-suite.rs:181,186,577,581,582,635,638`
  * `report.rs:175,185`
  * `rexx-bench-band.rs:143`
  * `wallclock.sh:13,64,65`
  * `child.rs:50,58`
  * `PINNED.md`

  Scout C marks the `CARGO_BIN_EXE` claim "inferred … not run".
  Caller semantics change too. `rexx` resolves through `REXX_PATH` and `PATH` and exits 253 or 255, where `rexx-run` reads the literal path and exits 2. Every test asserting rc 2 or relying on the literal path changes.
  `group_runner.rs` uses `CARGO_BIN_EXE_rexx-run` both for sim-mode processes and for the `rexx` symlink (`interpreter_bin`). 9a builds the harness on that, and 9c changes it while 9b runs on it in parallel.
* **Fix:** either keep the `rexx-run` binary and add `rexx` and `rexxc` beside it, or make the switch its own 9c task with these elements:
  * the file list above
  * a re-pin of the perf baseline on `rexx` at the same commit, as two binaries with a recorded zero-change A/B
  * an audit of rc and path assertions

  Say which binary the suite harness invokes on the crate side, and whether that is in-process `Invocation` or a process. `group_runner` has both.

### 11. Seeding every `reaching_rxapi` test as a Phase 10 row takes `SysGetMessage`'s witnesses away from 9b

* **Spec:** :37, :49-51, :21.
* **Evidence:** `reaching_rxapi` flags `SysGetMessage.TEST_CATALOG_INVALID` and `TEST_GENCAT`, because the group's `catOpenSupported` calls `rxfuncquery("SysGetMessage")`.
  Scout A :86 calls them "inferred false positives".
  R2 makes `SysGetMessage` and `SysGetMessageX` Phase 9's. Those two tests are their catalog witnesses.
  D2's Phase 10 reason test ("refuses with a Phase 10 text") cannot run on a test that the harness renames out.
* **Fix:** state the reason test for renamed-out rows: the row's test is in `reaching_rxapi`'s output, and that output is asserted equal to the set of rows owned "Phase 10, rxapi".
  Ask Moritz whether `RXFUNCQUERY` on a built-in routine is Phase 10 work.
  If it is, record that `SysGetMessage`'s two catalog tests are not Phase 9 witnesses.

### 12. Section 7 omits roadmap edits that the spec's decisions require

* **Spec:** :143-158.
* **Evidence:**
  * R:47 and R:726, see finding 1.
  * R:780, the crate tree, still has `rexx-cli/  # Phase 9: rexx, rexxc, rxqueue, rxsubcom`, which D4 drops.
  * R:654, row 3, still says message text and substitutions are "deliberately not reproduced". R6 reverses that.
  * R:721, the L3-core paragraph, says the plan must "enumerate the excluded groups explicitly, by name". The spec makes the file per test. It also says "Phase 10 deletes that file", which conflicts with finding 13.
  * `phase-4-exclusions.txt:5284`, the second instance, moves to Phase 10 under R4 but is not re-owned.
  * `:5542` (`rexxc`) and `:6114` (INTERPRET) need closing text.
* **Fix:** add each edit to section 7.

### 13. Permanent exclusion rows contradict "Phase 10 deletes that file" and row 10's "no exclusions"

* **Spec:** :178, :184-185.
* **Evidence:** at 9d the file is "reduced to Phase 10, Deviation and environment rows".
  R:721 says Phase 10 deletes the file, and R:662 requires "L3-full with no exclusions".
  The environment rows (two socket samples that hang on the oracle, possibly the four `STREAM` tests) and the Deviation rows cannot be removed by Phase 10 work.
* **Fix:** split the file. `phase-9-exclusions.txt` holds only Phase 10 rows, which Phase 10 deletes.
  A permanent `suite-licences.txt` holds Deviation and environment rows.
  Amend R:662 to say "no exclusions but the licences file".

### 14. The known flaky and redirection failures have no owner category

* **Spec:** :20-21, :55-57.
* **Evidence:** `.github/known-test-failures/common.txt` lists `CHAROUT/TEST_STDOUT_START` and `LINEOUT/TEST_LINE_TRANSIENT` ("fail whenever output is redirected"). It also lists two `SYSSLEEP` timing tests ("flaky; an entry permits a failure, it does not require one").
  The harness redirects output and runs eight groups at once.
  "Environment" means "the oracle itself fails or hangs here", and these passed on the oracle in scout A's run.
  A flake turns criterion 1 red, and the only remedy the spec allows is an uncategorised row.
* **Fix:** state how these four are handled.
  One way is an environment row citing `common.txt` and its reason, allowed to pass or fail.
  Another is a bounded retry for the named timing tests only.

### 15. D5's restore surface has no exit criterion and leaves the crate's answer to an oracle `.rxo` open

* **Spec:** :24, :113-125, :186-189.
* **Evidence:** criterion 4 names only `rexx_command` and `rexxc`.
  The ooTest witnesses for restore are `routine~newFile` (`rexxc.testGroup:166,176`), `METHOD.testGroup:3090`, and the Method and Routine `test_new_array_from_file` and `test_new_file_compiled` tests.
  Nothing in ootest restores a compiled file through `::REQUIRES`, an external `CALL`, the `-e` encoded form via `.Routine~new(name, data)`, or `.Method` file loading.
  D5 lists all of those.
  "SOURCELINE, PARSE SOURCE, trace lines, measured, not assumed" has no witness.
  The spec also never says what this crate does when given an oracle-written `.rxo`. Scout C recommends `Error 3.903`, but a plan writer could equally try to read it or treat it as source.
* **Fix:** add a criterion with one corpus program per restore route and per observable named in D5.
  State that an oracle-written `.rxo` answers `Error 3.903 … cannot be run by this version`, rc 253.
  State what a `.rxo` from another crate version does, given R5 "not even between versions of this crate". Either it refuses with 3.903 or it re-checks the source.

### 16. D6's criterion covers only the TSV's rows, and through an oracle run

* **Spec:** :25, :188-189.
* **Evidence:** `parse-errors.tsv` has 1020 programs, 567 of them translation errors (scout C section 2).
  "Every parse-error message fills its inserts" is not bounded by that set.
  A message raised by no TSV row stays unfilled and the criterion passes.
  "Through `rexxc` on both sides" makes the oracle a gate input, against D1's own rule.
* **Fix:** the committed derivation script regenerates the TSV from the oracle at plan time.
  The gate compares this crate's `rexxc` output to the committed TSV.
  Add a structural test that every (major, sub) the raising API can emit has at least one TSV row, or a recorded reason why no program reaches it.

### 17. Sub-phase gates have no criteria

* **Spec:** :168-178.
* **Evidence:** `decisions.md` says "each with its own spec section and gate".
  Section 9 gives each sub-phase a "Delivers" cell and a gate-record file name, but the only criteria are phase-level (section 10).
  It is undefined whether 9a's gate requires the suite run to be green. It cannot be, but the spec does not say what it must show instead.
  It is equally undefined whether 9b closes with `SysGetMessage` unwitnessed (finding 11).
* **Fix:** a short criteria list per sub-phase, each naming the test or record that goes red.

### 18. Re-owning to "Phase 9.1" breaks criterion 3

* **Spec:** :152, :186-187.
* **Evidence:** `closed_phases.rs`'s `names()` matches the whole word with no trailing digit, so "Phase 6" already catches "Phase 6.1" (`CLOSED` at :32).
  Once "Phase 9" is added to `CLOSED`, any source string or open OWNER row naming "Phase 9.1" fails.
  The obvious new owner for `:6179`'s parity half (finding 1) is "Phase 9.1".
* **Fix:** say the parity row's owner is "Phase 9.1" and teach `closed_phases` that 9.1 is open.
  Or name the owner differently.

### 19. Phase 10 refusal sites outside `internal_routines.rs` are assigned to no sub-phase

* **Spec:** :99-109.
* **Evidence:** `"Phase 10"` has 61 sites, 24 of them outside `internal_routines.rs`. They include:
  * `lib.rs:511` `named_semaphore`, which is `SysOpenEventSem` and its siblings with a name. These are listed in section 4, but this refusal is a separate site.
  * `dispatch/native.rs:264` `handle_set` (Stream)
  * `dispatch/library.rs:838` classic-style libraries
  * `directives.rs:80` `REGISTERED`
  * `environment.rs:363` (STDQUE)

  D3 may make any of these Phase 9's, but no sub-phase owns that outcome.
* **Fix:** list them in section 4, or in a 9a task that classifies each against D3's list before 9b starts.

### 20. Existing exclusion-like lists coexist with the new file

* **Spec:** :47-53.
* **Evidence:** three lists already record tests that are known to differ or are left out of the gate:
  * `concurrency_tests.rs:2545` `DIFFERING`, 56 rows in 15 groups
  * `REST_LEFT_OUT`, which already carries `TEST_SUBCLASSES_GC`
  * `whole-groups/alone.txt`

  The spec does not say whether these are retired, kept, or must agree with `phase-9-exclusions.txt`.
* **Fix:** state the relationship. For example, `whole_groups` keeps its lists for sim mode, and a test asserts that every `DIFFERING` key is either fixed or present in the exclusion file.

## Minor

### 21. The header counts eight rulings where section 6 has seven

* **Spec:** :11, "rulings R1-R8".
* **Evidence:** section 6 has R1-R7.
* **Fix:** "R1-R7".

### 22. D3 attributes the "mislabels" to the wrong scout, and R2 was a ruling

* **Spec:** :22.
* **Evidence:** scout A never calls either item a mislabel.
  The RexxUtil scope finding is scout D section 2, and the package-settings site at `lib.rs:664` is cited in no scout report. It is verified in the tree, as `fn package_option_write`.
  RexxUtil's label was correct under D11 (R:115). R2 changed the ruling.
* **Fix:** cite scout D section 2 and the tree.
  Call RexxUtil "re-homed by R2" and the package write "mislabelled".

### 23. R6 widens what `decisions.md` reversed

* **Spec:** :140.
* **Evidence:** `decisions.md` says it reverses Phase 3's deferral "for secondary messages". The spec says "for parse errors".
* **Fix:** quote `decisions.md`'s wording, or ask Moritz whether the main message text was also deferred. Either way, add the R:654 edit (finding 12).

### 24. R3's new row adds terms the ruling does not contain

* **Spec:** :137, :152-156.
* **Evidence:** `decisions.md` says "targets every axis at 2x or better, with 1x as the stretch goal".
  The spec adds "on cycles", "entry 9" and "L3-full does not wait for it".
  "On cycles" picks an instrument Moritz did not name.
* **Fix:** mark these as proposals for Moritz, or drop them.

### 25. The `dispatch` regression figure drops scout B's caveat

* **Spec:** :155-156.
* **Evidence:** scout B section 3a says the 1.23x is from `86ca524e6`, "a different instrument (single-event runs, 3 reps), so the 0.34 jump is not a clean A/B".
  It also says the move partly lines up with Phase 6's accepted overruns (`phase-6-perf.md:1313-1400`).
  The spec states it as an unexplained regression "since 2026-09-10".
* **Fix:** carry the caveat.

### 26. One queued traceback item was already taken by Phase 6.1

* **Spec:** :93.
* **Evidence:** `2026-10-07-interpret-syntax-traceback-line` is listed among the items 6.1 takes (R:658).
  6.1 Task 5 (`59eb57f37`) landed the traceback line for INTERPRET and kept the inserts unfilled under 6.1 R3.
  What remains is D6's work, not 9d's.
* **Fix:** move it to D6, or mark its traceback half done.

### 27. "`rexxc` 11 of 11" does not count test methods

* **Spec:** :124, :188.
* **Evidence:** `rexxc.testGroup` has 8 `::method test*`. The 11 is the TSV's `oracle_ran`, the framework's count.
  `rexx_command`'s 30 is both counts.
* **Fix:** name the unit ("11 tests ran per the framework summary"), or pin both numbers.

### 28. D7 uses a roadmap decision number that collides with the spec's own

* **Spec:** :26, "D2 needs no image".
* **Evidence:** this is roadmap D2 (R:104, the saved image), and the spec has its own D2.
  A plan writer reading the spec reads the local one.
* **Fix:** write "roadmap D2".

### 29. The `rust/CLAUDE.md` correction is not a roadmap edit, and it states the problem inexactly

* **Spec:** :157-158.
* **Evidence:** `rust/CLAUDE.md:13` says "That separate `-O3` build is gone from this machine". :21 says `build/` is not `-O3`.
  Scout B found the Release build's source "3 days older", not only a different date.
  The section 8 budget compares the crate against itself, so this correction does not bear on any Phase 9 gate.
* **Fix:** move it out of section 7 into a 9a task, and quote both lines being changed.

### 30. The R1 claim is true but rests on an unchecked premise

* **Spec:** :135.
* **Evidence:** it is now checked.
  The only external `rxqueue` use outside `utilities/` is `RexxQueue.testGroup:600` (`TEST_EXTERNAL`), already renamed out as rxapi.
  `rxsubcom` appears only in comments (`OPTIONS.testGroup:199,222`).
  Scout C section 3 alone does not establish "they block none".
* **Fix:** cite that grep beside scout C.
