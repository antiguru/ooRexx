# Re-review of the Phase 9 spec, after the fix round

Spec: `docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md` (untracked), at HEAD `a98238be5`.
Scope: the 30 findings of `spec-review.md`, new false statements, and new contradictions.
Read-only: grep and read only, nothing run.
Line numbers are the revised spec's.
"R:" is `docs/superpowers/plans/2026-07-27-rust-rewrite.md`.

## Part 1: the 30 findings

1. Addressed. R8 records Moritz's ruling, section 7 amends the parity gate and R:726, splits `:6179`, and gives parity to 9.1. The line cited for the gate is wrong, see 35.
2. Addressed in intent. The denominator exists, but it breaks green groups, see 31 and 32.
3. Addressed. It now contradicts the "Until 9c" sentence, see 33.
4. Addressed. Each owner has a check.
5. Addressed. The D59 example is gone.
6. Addressed (:92).
7. Partly. "249 tests in 49 groups" and the 52 are right (TSV re-summed: 22+8+1+4+17). Two 9b and 9c items are still left in the 9d set, see 38.
8. Addressed. 9a relabels, and the closed reason set and structural test are in D3.
9. Addressed, with the fix's wording (:99-102).
10. Partly. The switch is its own task with the file list and an rc/path audit, and the harness side is a process. The re-pin is not executable as written (see 34), and 9b and 9c are still "independent" while the switch rewrites 9b's harness (see 43).
11. Addressed. The controller ruling and the `Rxapi` owner check (:61-63) cover it.
12. Addressed. Every edit is listed, but one line number is wrong (35) and the timings conflict (41).
13. Addressed. The split into two files and the row 10 wording are in place.
14. Addressed. Flaky is a category of its own (:73-75). It collides with D1, see 31.
15. Addressed. D5 states the 3.903 answer for oracle and other-version files, and criterion 4 has a corpus program per route.
16. Addressed (D6, criterion 4).
17. Addressed. Each sub-phase has a gate cell.
18. Addressed. Section 7 has 9.1, with `closed_phases` taught it.
19. Addressed. Section 4 names `lib.rs:511` and two others, plus "whichever other" from 9a's classification.
20. Addressed (:81-83).
21. Addressed. R1-R8 match section 6.
22. Addressed (D3 "Why").
23. Addressed. R6 quotes `decisions.md`.
24. Addressed. Entry, instrument and L3-full dependence are left for Moritz.
25. Addressed. The caveat is carried (:236-239).
26. Addressed (:136-137).
27. Addressed. Criterion 4 pins both counts.
28. Addressed ("roadmap D2").
29. Partly. It became "9a also corrects" (:246), but it still sits in section 7, whose preamble puts every edit in the agreeing commit. 9a's Delivers cell does not list it.
30. Addressed. The grep evidence is cited in R1.

## Part 2: new defects

### 31. Blocker: D1's set equality turns red every group holding an exclusion row that runs

* **Spec:** D1 :22 ("the tests this crate ran equal the group's expected set"), :51-52 (expected set is the methods "minus the group's rows in the two files"), :65-66, :67-75.
* **Evidence:** only `Rxapi` rows are renamed out (:61-62), so only they are absent from "ran".
  Every other row kind runs.
  * A "Phase 10, any other reason" row runs and refuses (:65-66, "its run here refuses").
  * A Deviation row runs and fails.
  * A flaky row "is allowed to pass or fail" (:74), so it runs.
  * An environment row runs too.

  Each of these is in "ran" and not in the expected set, so ran is not equal to expected, and the group is red.
  The environment candidates (`samples/scclient`, `samples/sfclient`) hang on the oracle until 300 s (scout A section 4 bins). Unless the harness renames them out, D1's "the run ends normally" fails as well.
* **Fix:** in D1, write: "A group is green when every test in its expected set ran and passed, every test that ran is in the expected set or has a row in one of the two files, and the run ends normally."
  State that environment rows for tests that hang are renamed out on both sides, as `reaching_rxapi` tests are.

### 32. Major: `suite-not-run.txt` is never subtracted, so every `restrictOS` group is red

* **Spec:** D2 :23 ("Two files hold every test the gate does not require to pass"), :51-55.
* **Evidence:** the expected set is the methods "minus the group's rows in the two files".
  `suite-not-run.txt` is a third file, and no sentence subtracts it.
  `extract` (`rexx-extract/src/lib.rs:65`) is line-based with no comment handling. It enumerates the methods of the 15 Windows `restrictOS` groups that run zero tests here (scout A section 4), so their expected set is non-empty against zero ran.
  D2's "two files hold every test" is false the moment a third file exists.
  Scout A also lists two zero-test groups whose cause was "not read" (`doc/rexxref/chapter7/Section4`, `regressions/bug2061_newline_misalignment`). The latter uses `restrictOS` (`ooTest.frm:1297` callers). For the former, no scout supports "tests inside block comments".
* **Fix:** "minus the group's rows in `phase-9-exclusions.txt`, `suite-licences.txt` and `suite-not-run.txt`".
  Reword D2 to name three files, or say that `suite-not-run.txt` holds tests the framework does not run, not tests it runs and lets fail.
  9a determines why `chapter7/Section4` runs zero tests before giving it a row.

### 33. Major: the harness asserts that `rexxc` resolves inside `bin`, and `rexxc` does not exist until 9c

* **Spec:** :45-47.
* **Evidence:** "the harness asserts on each run that `rexx` and `rexxc` resolve inside that `bin`" is immediately followed by "Until 9c, … `rexxc` is absent".
  An absent `rexxc` cannot resolve inside `bin`, so every 9a and 9b suite run fails the assertion, or the implementer drops it.
* **Fix:** "Until 9c, the harness asserts that `rexxc` resolves nowhere on the crate side's `PATH`. From 9c on, it asserts that `rexxc` resolves inside `bin`."

### 34. Major: the 9c re-pin cannot be built "at the same commit", and its pass condition contradicts itself

* **Spec:** :255-257, :274.
* **Evidence:**
  * The pin is `rexx-run` "built at this spec's base commit" (:253).
    `rexx` does not exist at that commit, and the switch task removes `rexx-run` (D4), so no commit holds both binaries.
    "Re-pins on `rexx` at the same commit" therefore has no referent.
  * "The difference must sit within the noise band, so the front end's own cost is visible instead of being absorbed" fails both ways.
    If the difference is within noise, nothing is visible.
    If the front end costs anything, the A/B fails, and the spec gives no remedy.
  * It does not say which pin the running +0.5% total uses after the re-pin. If the re-pin resets the baseline, 9a and 9b's drift leaves the total (`per-step-threshold-hides-cumulative-drift`).
* **Fix:** "The switch task first adds `rexx` beside `rexx-run` in one commit, and A/Bs the two binaries built from that commit. Their difference is recorded and counted inside the running total. The next commit removes `rexx-run`. The running total stays against the base-commit `rexx-run` pin, plus the recorded A/B difference."
  Or ask Moritz which of the two pins the total is measured against.

### 35. Major: the parity gate is R:46, not R:47

* **Spec:** :215 "Global Constraints, the parity gate (:47)". `decisions.md:28` repeats ":47".
* **Evidence:** R:46 is "**Shipping gate (parity).** No phase from 2 onward closes…". R:47 is "**Phase 1 viability threshold (1.5×).**".
  An editor applying section 7 by line would amend the Phase 1 bullet.
  The ":47" came from `spec-review.md` finding 1, which was wrong.
  R:655 and R:725 cite the same gate as ":39", which is stale too.
* **Fix:** write ":46".
  Correct `decisions.md:28` the same way.
  Decide whether the R:655 and R:725 back-references are corrected in the same commit.

### 36. Major: `extensions/rexxutil/` does not exist in the oracle

* **Spec:** :160-161, "The oracle source is `extensions/rexxutil/`."
* **Evidence:** no such directory under `/home/moritz/dev/repos/ooRexx`. RexxUtil is `interpreter/platform/unix/SysRexxUtil.cpp` and `interpreter/runtime/RexxUtilCommon.cpp`.
  No scout report names `extensions/rexxutil/`.
* **Fix:** "The oracle source is `interpreter/platform/unix/SysRexxUtil.cpp` and `interpreter/runtime/RexxUtilCommon.cpp`."

### 37. Major: the "Phase 9" source strings R4 moves to Phase 10 have no owner, so criterion 3 cannot pass

* **Spec:** R4 :205, criterion 3 :284-285, section 7 :240-243, 9a Delivers :272.
* **Evidence:** criterion 3 requires that no source string names Phase 9 at the close.
  `closed_phases.rs` reads every string literal in `rexx-api` whole (`mentions`, :202-220, crate list :225-231).
  These remain:
  * `layout.rs:379-383`: the five `REFUSING_MEMBERS` rows, each `"Phase 9"`.
  * `ffi.rs:4187`: `"RexxInstanceInterface.AttachThread is not implemented (Phase 9)"`.
  * `ffi.rs:4229`: `"… AddCommandEnvironment from another thread is not implemented (Phase 9)"`.

  R4 makes all of them Phase 10's.
  D3's classification covers only `"Phase 10"` sites.
  Section 7 re-owns only `phase-4-exclusions.txt:5284`.
  No sub-phase relabels these strings to `"Phase 10"` with the `Embedding` reason.
* **Fix:** add to 9a's Delivers: "relabel `REFUSING_MEMBERS` and the two `ffi.rs` assertion texts to Phase 10 with the `Embedding` reason (R4)".

### 38. Major: the 9d set still holds a 9b test and the D6 parse-message tests

* **Spec:** :117-121, :157, D6 :27.
* **Evidence:** `scout-a-groups.tsv` lists `SysFileXXX.TEST_FILEEXISTS_NIL_INVALID` in `tests_wrong`. Section 4 gives it to 9b, but it is not among the 52 subtracted.
  D6 makes the `incorrectCharacters` (and per D6, `Literals`) failures D6's, which 9c delivers. Their `tests_wrong` entries (scout A section 4: `Literals` 12 tests) are also not subtracted.
  Finding 7's double ownership therefore persists for these.
* **Fix:** subtract them by name, giving the new count.
  Or state that 9d owns whichever of them 9b and 9c leave failing.

### 39. Minor: D6 misreads scout A on `Literals`

* **Spec:** D6 :27, "the `incorrectCharacters` and `Literals` groups fail on them (scout A section 4)".
* **Evidence:** scout A section 4 has "`expressions/Literals` (12: 15.2/15.3 whitespace in binary/hex literals), `incorrectCharacters` (the message substitutes `&1`/`&2` unfilled)". Only `incorrectCharacters` is an insert failure.
* **Fix:** "the `incorrectCharacters` group fails on them (scout A section 4)". Leave `Literals` in the 9d set.

### 40. Minor: "They stop every `samples` group at load" is wrong for `~libraryPath` and for "every"

* **Spec:** :91.
* **Evidence:** scout A section 4: only `RexxInfo~executable` refuses at load, in `samples/samples`, `scserver`, `sfserver` and three Windows sample groups.
  `~libraryPath` is one in-test refusal (refusal table).
  `scclient` and `sfclient` hang on the oracle, and `fileNameDialog_demo` has no tests.
* **Fix:** "`RexxInfo~executable` stops `samples/samples`, `scserver`, `sfserver` and three Windows sample groups at load."

### 41. Minor: three different times for the section 7 edits

* **Spec:** :213 ("made in the commit that marks this spec agreed"), :243 (`:5542` and `:6114` "closed when 9c and 9d deliver them"), :246 ("9a also corrects"), :272 (9a Delivers "The roadmap edits (section 7)").
* **Fix:** split section 7 into "on agreement" and "at delivery".
  Put the `:5542`/`:6114` closures in the 9c and 9d Delivers cells, and the `rust/CLAUDE.md` correction in 9a's.
  Drop "The roadmap edits" from 9a, or say which ones.

### 42. Minor: roadmap row 9 still says the subclass work starts from the patch

* **Spec:** :104-105 ("serves as a reference, not a base"), :221-223 (the row 9 edit).
* **Evidence:** R:661: "starting from `.superpowers/sdd/2026-10-07-phase-6-1/scout-d-prototype.patch`". The row 9 edit does not change it.
* **Fix:** add "'starting from' the prototype patch becomes 'with the prototype patch as a reference'" to the row 9 edit.

### 43. Minor: 9b and 9c are "independent", but 9c's switch rewrites 9b's harness

* **Spec:** :267, :192.
* **Evidence:** 9b's gate runs "under the suite harness" (:273). The switch changes `group_runner.rs`'s `interpreter_bin`, which that harness uses (:47).
* **Fix:** "9b's gate runs before the switch task lands, or after it, never across it." Or order the switch after 9b closes.

### 44. Minor: the three new files have no path

* **Spec:** D2 :23, :53.
* **Evidence:** R:721 names `docs/superpowers/plans/phase-9-exclusions.txt`. The spec gives no directory for `suite-licences.txt` or `suite-not-run.txt`, and R:721's path is not restated.
* **Fix:** give all three paths in section 2.

### 45. Major: a `DIFFERING` key is not a test, so "present in one of the two files" cannot be checked

* **Spec:** :82-83.
* **Evidence:** `concurrency_tests.rs:2545` `DIFFERING` rows are (group file, part, mode, key, reason).
  `fn key` (:2996) makes the key a whole-run summary: `"{outcome}, rc {status}, last started {last}, failing [{...}]"`.
  The two files are per test.
  This wording came from `spec-review.md` finding 20's suggested fix, which assumed per-test keys.
* **Fix:** "A test asserts that every test named in a `DIFFERING` key's failing list, and its `last started` test, is fixed or has a row in one of the two files."

### 46. Major: the list of what `rexx` keeps from `rexx-run` drops the transient-descriptor flags

* **Spec:** :176, "It keeps what `rexx-run` owns: signal handlers, sinks, `REXX_SWITCH_MODE` and the wide stack."
* **Evidence:** `rexx-run.rs:33-43` sets `with_standard_transient` from the real descriptors. Its comment says that `QUERY STREAMTYPE` and 93.958 turn on it, and that this process is "the one place it is legitimate".
  `rexx-run.rs:38` also sets `ProgramInput::Stdin`.
  Scout C section 1 lists "real-descriptor transient flags" among what `rexx-run` owns. The spec dropped it.
  The wide stack is not in `rexx-run.rs`: no `stack_size`, only `INTERPRETER_STACK_BYTES` in a message (:139). It comes with `run_program`.
* **Fix:** "It keeps everything `rexx-run.rs`'s `main` sets on the invocation: signal handlers, standard input, the transient flags of the real descriptors, sinks and `REXX_SWITCH_MODE`."

### 47. Minor: D7's `RXFUNCQUERY` citation does not say what it is cited for

* **Spec:** D7 :28, "`RXFUNCQUERY` reads the RXAPI registry (`rexx-inventory`'s `EXCLUDED` table)".
* **Evidence:** `rexx-inventory/src/lib.rs:31` is `("RXFUNCQUERY", "Phase 10")`. The table records the phase only. RXAPI is named only in the comment on `QUEUED` (:32-35).
* **Fix:** cite the table for "Phase 10's" and drop it as the source of "reads the RXAPI registry". Or cite the oracle's `RXFUNCQUERY` implementation for that clause.

## Checked and true

These new or changed facts were traced and hold:
* The 30 routine names against `internal_routines.rs:99-164`: 37 rows, minus the 7 macrospace rows.
* `lib.rs:664` is `package_option_write`'s Phase 10 text (the `fn` is at :662). Its doc names `Package~options` and `~defaultOptions` with a value, and the oracle `rexx -o` sends `DEFAULTOPTIONS` with a value (`rexx.cpp:210,213`).
* `lib.rs:511`, the `ffi.rs` lines, `REFUSING_MEMBERS`, and `closed_phases`' "Phase 9.1" matching.
* The `common.txt` entries, and the absence of the four `STREAM`/`Stream` tests there.
* `DEVIATIONS -- permanent, chosen` at `phase-4-exclusions.txt:715`, and the rows at `:5284`, `:5542`, `:6114`, `:6179` (430 MB).
* R:654, :661, :721, :726, :780, :104, :115.
* `rust/CLAUDE.md:13`'s quoted sentence, and scout B's Release build facts.
* Every queued-item slug.
* `59eb57f37`, and the 6.1 stopping rule.
* Scout B's ratios and caveat.
* Scout C's `rexx`/`rexxc` options, rc 253/254/255 and the 29 bytes.
* R1's `RexxQueue.testGroup:600` and `OPTIONS.testGroup:199,222`.
* The test method counts (30, 8), and the `METHOD`, `Method` and `Routine` test names.
* `rexximage/` is empty.
* The `SysGetMessage` test names and `catOpenSupported`.
* Scout A's harness bounds.
