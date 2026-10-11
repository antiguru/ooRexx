# Third review of the Phase 9 spec

Spec: `docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md`, the working copy after the controller's section 8 and 9 edit (HEAD `7bb9dec11`, no commit since `a98238be5` touches `rust/`, so every scout line citation still holds).
Read-only: grep and read only, nothing run.
Line numbers are the current spec's.
"R:" is `docs/superpowers/plans/2026-07-27-rust-rewrite.md`.

## Blockers

### 1. A loud refusal ends the group's run, and D1 has no rule for it, so a gate group holding one reached Phase 10 or Deviation row can never be green

* **Spec:** D1 :22, section 2 :64-69 ("its run here refuses with a `"Phase 10"` text"), :70-72 (Deviation rows), :73-76 (only hanging environment rows are renamed out), criterion 1 :308.
* **Evidence:** a `Loud` refusal is a process exit, rc 120, not a condition the framework records.
  Scout A section 3 iterated past each one by renaming the last started test and re-running, because "the crate's run is cut short (refusal, crash, timeout, OOM)".
  Scout D's probes all give "ours rc 120".
  Section 2 renames out only `Rxapi` rows and hanging environment rows.
  A Phase 10 row with any other reason is defined by its run refusing, so it runs, exits the process, and every later test in the group never starts.
  D1 then fails on "every test in its expected set ran".
  Gate groups with such a row today: `base/class/RexxQueue` (`TEST_BUG_1379`, `rexx_query_queue`), `base/runtime.objects/environmentEntries` (`.STDQUE`, `environment.rs:363`), the group reaching `::ROUTINE EXTERNAL` `REGISTERED` (`directives.rs:80`), and any test reaching `handle_set` (`dispatch/native.rs:264`) or a classic-style library (`dispatch/library.rs:838`).
  Deviation rows have the same shape where the deviation is a loud refusal: scout D section 5 lists Deviation 20 (`Message~result`, `Message~wait`, `EventSemaphore wait`) as `loud` rows in `method-bodies.txt`, and Deviation 28's wrong-type native rows are loud by 6.1 D3.
  The spec's own 9a gate ("the denominator check pass", :289) is unsatisfiable for the same reason while 9b's 37 RexxUtil refusals still abort 17 gate groups.
* **Fix:** either of these, stated in section 2.
  * The harness iterates as scout A did: a run cut short by a refusal renames the last started test out and re-runs, up to a stated cap, and the refusal text it captured is the mechanical check for that test's row (Phase 10 reason, Deviation number). The "ran" set is the union over iterations. This also makes the 9a and 9b gate tables computable.
  * Or every row whose test ends the run (a loud refusal, a hang) is renamed out on both sides before the run, as `Rxapi` rows are, and the owner check for such a row is a committed per-test probe record (the TSV's `tests_blocked` text or a one-test run), not "its run here refuses".
  Say which, and say that a Deviation row may be loud.

### 2. Criterion 4's corpus witnesses for D5 have no way to reach each side's own `rexxc`, so a 127-against-127 agreement passes, and the oracle-written `.rxo` witness needs the binary the harness rule forbids

* **Spec:** D5 :26, criterion 4 :320-321 ("One corpus program per restore route and per observable in D5 agrees with the oracle, including the 3.903 refusal of an oracle-written `.rxo`"), section 2 :44-47 (no oracle binary on the crate side's `PATH`).
* **Evidence:** `tests/corpus.rs` runs the oracle through `Oracle::wrapped` (`support/oracle.rs:323-336`), which sets `LD_LIBRARY_PATH` only and inherits the test process's `PATH`; the crate side runs in-process (`corpus.rs:96-115`), and an `ADDRESS` child gets `interp.env`, the same inherited `PATH` (`src/command.rs:557-559`).
  The sidecar `.env` overrides apply the same value to both sides (`corpus.rs:397`, `support/sidecar.rs:202-227`).
  So a corpus program that runs `rexxc` finds the same binary on both sides, or none: with none, both answer rc 127 and the differential is green.
  Scout C section 2 already recorded this shape for `METHOD`'s `rexxc` test ("Both sides fail it with rc 127 where `rexxc` is not on PATH").
  The 3.903 witness needs an `.rxo` the oracle wrote. The crate side cannot run the oracle's `rexxc` under section 2's `PATH` rule, and a corpus program has no per-side branch.
* **Fix:** name the instrument. Either a per-side `bin` directory for the corpus harness (the `group_runner.rs::interpreter_bin` shape, with the oracle's `bin` on the oracle side only), with the witness asserting the compile step's rc 0 and the 29-byte prefix on each side, or a dedicated test file in `rexx-exec/tests/` that spawns `CARGO_BIN_EXE_rexxc` and the oracle's `rexxc` explicitly.
  For the 3.903 witness, commit an oracle-written `.rxo` as a fixture beside the program (a `.d` directory), and state that the crate answers 3.903 on it, so the witness never needs the oracle binary at run time.
  Say which restore routes (`::REQUIRES`, external `CALL`, `.Method` file loading, `NewRoutine`, `LoadPackageFromData`) need a native test binary and where that binary comes from.

## Major

### 3. Section 8 contradicts itself after the edit: a "running total" measured once, and the 9c A/B difference both counted against the budget and added to the pin

* **Spec:** :267-278.
* **Evidence:** :270-273 says the A/B difference "is recorded and counted inside the running total" and then "The running total stays against the base-commit `rexx-run` pin, plus that recorded difference".
  The first sentence spends the difference from the +0.5%; the second excuses it by raising the baseline.
  :275-276 then says the budget "is measured once, at the Phase 9 close", so there is no running total.
  "This spec's base commit" (:268) names no commit. The scouts are at `a98238be5`, the spec commit is `bcb8d9909` or later, and `rust/` is unchanged between them, but a plan writer still has to pick one.
* **Fix:** "The budget is +0.5% Ir on every program, measured once at the Phase 9 close: this crate's `rexx` against `rexx-run` built at `a98238be5` (sha256 in 9a's gate record). The 9c A/B of `rexx` against `rexx-run` at one commit is recorded so that the front end's share of the close figure is known; it is inside the budget, not added to it."
  Or, if Moritz wants the front end excused, say so and give its own cap.

### 4. The sub-phase close check cannot fail on a corpus divergence

* **Spec:** :289-291 ("`cargo test --workspace --release -j 4` passes"), :301-302.
* **Evidence:** `corpus.rs`'s default mode is REPORT and exits 0; STRICT needs `REXX_CORPUS_GATE=1` (`oorexx-rexxcps-and-trace-gate-traps`, and `rust/CLAUDE.md:59`).
  `input_oracle.rs` and `parse_version_oracle.rs` skip under the same variable.
  6.1's G4 is exactly `REXX_CORPUS_GATE=1 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast` (`.superpowers/sdd/2026-10-07-phase-6-1/p61-gates/bggates.sh:22-34`).
  The spec's command has neither the variable nor `memcap`, so three sub-phases close with the differential unread, and the first gated run is the Phase 9 close.
* **Fix:** write the sub-phase close command as G4, or as "`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release -j 4 --no-fail-fast`".

### 5. What a "test" is, for the denominator, is undefined, and the two counts the spec pins for `rexxc` disagree

* **Spec:** :52-56, :57-58, criterion 4 :317-318 ("8 methods, 11 tests per the framework summary").
* **Evidence:** `rexxc.testGroup` has one class, no `inherit`, one `group~add`, and 8 `::method test*` lines; nothing in the file yields 11.
  Suite-wide, `::method test*` lines under `ootest/ooRexx` number 13,960, and the oracle's framework ran 23,170 tests in the judged groups (scout A), so the framework's count is not the method count anywhere.
  `whole_groups`' `mark_starts` writes `started fName`, the bare method name (`OOREXXUNIT.CLS:1485-1486`), uppercased by `started()` (`concurrency_tests.rs:2960-2966`).
  The spec's "tests run" column (:57) is the framework's figure, and D1's "every test that ran" is not tied to either.
  A plan writer who takes the framework's count as "ran" gets 11 against an expected set of 8 and a red `rexxc` group.
* **Fix:** in section 2: "A test is identified by its uppercased method name within its group. `ran` is the set of names in the `started` markers, as `whole_groups` reads them. The framework's own tests-run figure is reported in the table and is not the denominator."

### 6. The harness's crate side has no cap in `group_runner.rs`, `memcap` may be absent, and "runs a process" contradicts "group_runner's invocation"

* **Spec:** :34-37 ("using `support/group_runner.rs`'s copy and invocation"), :44 ("This side runs a process"), :49-50 ("Every crate run is under `memcap 2G`").
* **Evidence:** `group_runner.rs`'s crate side in normal mode is in-process (`watchdog::run_bounded_with`, :455); only sim mode spawns a process.
  No `memcap`, `ulimit` or `RLIMIT` appears in `group_runner.rs`.
  `rust/CLAUDE.md:92`: "`memcap` is not present in every session, and neither is the cgroup filesystem it needs", with the `ulimit -v 33554432` substitute and its size caveat.
  `tests/suite.rs` is a `cargo test` binary (`REXX_SUITE_GATE=1`); a test that execs `memcap` fails on a session without it, and the spec does not say whether that is a red gate, a skip, or a fallback.
* **Fix:** "The crate side is a child process of this crate's `rexx` under `memcap 2G`. When `command -v memcap` fails, the harness runs it under `ulimit -v` at a stated size instead and says so in the table; the gate record names which." Drop "invocation" from :36 and name the pieces reused: `fresh_copy`, `without_rxfuncquery`, `reaching_rxapi`, `skip`, `interpreter_bin`.

### 7. "Byte-preserving" contradicts the `group_runner.rs` code the spec reuses

* **Spec:** :43 ("Files are read and written byte-preserving"), :36.
* **Evidence:** `skip` reads with `from_utf8_lossy` and writes the result back (`group_runner.rs:668-669`, `:714`), so renaming a test out of a Latin-1 group file replaces every non-UTF-8 byte with U+FFFD. `without_rxfuncquery` uses `read_to_string` (`:295`) and panics on invalid UTF-8.
  Scout A section 3 ported both to Latin-1 for this reason.
  The groups renamed out include every environment row, and `incorrectCharacters` is the kind of group whose bytes are the test.
* **Fix:** "`skip`, `read_lossy` and `without_rxfuncquery` change to byte reads and writes; `whole_groups` runs on the same code."

### 8. The `rexx-run` switch's file list omits a test support file with a hard-coded path, and four prose sites

* **Spec:** :195-202.
* **Evidence:** scout C section 1 lists `rexx-exec/tests/support/arity.rs:374` (`.join("../../target/release/rexx-run")`, verified), `rexx-bench/src/bin/rexx-arms.rs:15`, `rust/CLAUDE.md:38`, `rust/corpus/README.md:295` and `rexx-exec/src/lib.rs:2718-2719`.
  Section 5 names none of them. `arity.rs` runs the binary, so it goes red when the file disappears.
* **Fix:** add `arity.rs:374` to the switch's list, and the four prose sites.

### 9. Section 4 takes `LIBRARY rexxutil` while `phase-4-exclusions.txt:6123-6135` still owns it to Phase 10, and section 7 does not re-own it

* **Spec:** :164, section 7 :250-252.
* **Evidence:** `phase-4-exclusions.txt:6123` "`LIBRARY rexxutil` NAMES THE ORACLE'S INTERNAL RexxUtil PACKAGE", `:6135` "OWNER: Phase 10, by that line, as the remainder of RexxUtil".
  Section 7 re-owns `:5284` and splits `:6179` only.
  `:5516` is another "OWNER: Phase 10" row the spec does not mention; whether R2 moves it is unchecked.
* **Fix:** add to section 7: "`:6135` is re-owned to Phase 9 (9b), and `:5516` is classified by 9a's D3 pass."

### 10. 9d's label edit names `closed_phases` only; two other tables carry the open-phase vocabulary

* **Spec:** :263, criterion 3 :313-314, section 7 :252 (parity re-owned to "Phase 9.1").
* **Evidence:** `refusal_dispositions.rs:56-57` has `OPEN_PHASES = ["Phase 9", "Phase 10", "Phase 11"]`, checked on every REHOME row.
  6.1 T6 pruned `"Phase 5"` from `owners.rs`'s `PHASES` and `SPLIT_TABLE_PHASES` vocabularies as well (6.1 spec section 7, T6).
  `closed_phases.rs:190-197` `names()` treats "Phase 9" as a match inside "Phase 9.1" (the next character is `.`, not a digit), and the spec knows this (:263), but a REHOME row cannot name "Phase 9.1" until `OPEN_PHASES` does.
* **Fix:** "9d removes `"Phase 9"` from `closed_phases`' open set, `refusal_dispositions.rs`'s `OPEN_PHASES` and `owners.rs`'s vocabularies, adds `"Phase 9.1"` where a row may name it, and teaches `names()` that `Phase 9` is not `Phase 9.1`."

### 11. `ReleaseLocalReference`'s witness has no instrument and no threshold

* **Spec:** :115-117, 9a gate :289 ("Every section 3 refusal's witness passes").
* **Evidence:** the figure comes from scout D's "forged routine from the queued file's text", a native library built outside the tree.
  `tests/library_routine_memory.rs` loads the oracle checkout's `librxmath.so`, asserts output only, and measures no `maxrss`.
  Nothing in `rexx-exec/tests/` builds a native library or reads peak memory, and "stop growing with the loop count" names no ratio.
* **Fix:** name the harness (a test that compiles the queued probe's C against the frozen headers, or a committed `.so` under `testbinaries/`'s build) and the rule: "peak RSS at 2,000,000 iterations is at most twice peak RSS at 200,000", or whatever Moritz accepts.

### 12. 9b's gate names no groups, and a routine with no suite witness has no exit

* **Spec:** :146-168, :290 ("The groups section 4 names pass").
* **Evidence:** section 4 lists routines, not groups; the groups are scout D section 2's 17 plus `SysFileTree` and `SysFileXXX`.
  `SysCls`, `SysGetKey`, `SysFork`, `SysCreatePipe`, `SysWait`, `SysSetPriority`, `SysQueryProcess` and `SysUtilVersion` have no group named for them anywhere in the scouts, and `SysGetKey` reads a terminal.
  Criterion 3 forbids a `"Phase 9"` label at the close and D3 forbids `"Phase 10"` without a reason, so a routine 9b cannot witness has nowhere to go.
* **Fix:** list 9b's gate groups by name (derive from `grep -rlw` over `ootest/ooRexx` as scout D did, committed as a file), and say what a routine with no suite witness gets: a corpus program where one is possible, else a GUARD row naming the probes, as section 3 allows for two sites.

### 13. The row 10 edit forgets `suite-not-run.txt`

* **Spec:** :235-236 ("no exclusions but `suite-licences.txt`"), D2 :23.
* **Evidence:** the 15 `base/rexxutil/platform/windows/*` groups and `bug2061` (`restrictOS("WINDOWS")`, `bug2061_newline_misalignment.testGroup:41`) never run on this host, in any phase, so `suite-not-run.txt` outlives Phase 10 as `suite-licences.txt` does.
* **Fix:** "no exclusions but `suite-licences.txt` and `suite-not-run.txt`", and say in D2 that `suite-not-run.txt` is permanent.

### 14. The 9a gate's "denominator check pass" is unsatisfiable at 9a

* **Spec:** :289.
* **Evidence:** finding 1's mechanism. At 9a's close the 37 RexxUtil routines still refuse and abort their groups mid-run, so the expected set is not run for 17 gate groups, and `Package_Options`' 54 tests behind the package-settings write are 9a's own.
  Even with finding 1 fixed by iteration, a test renamed out because it refuses as 9b's is in the expected set and did not pass.
* **Fix:** "9a's gate: the denominator check passes on every group with no test blocked by a refusal that 9b or 9c owns; the blocked tests are listed in the table with the refusal text."

## Minor

### 15. Row 9.1's Entry cell is "left for Moritz" inside an edit made "on agreement"

* **Spec:** :221, :240-243.
* **Fix:** get the ruling before the agreeing commit, or write the row with "Entry: 9" and mark the L3-full dependence as his open item in the row text.

### 16. The two `chapter7/Section4` and `bug2061` zero-test causes are readable now

* **Spec:** :55-56.
* **Evidence:** `Section4.testGroup:57-61`: one test, `ooTestType = DOC_EXAMPLE_NOISE_TEST`, a type the framework skips by default. `bug2061_newline_misalignment.testGroup:41`: `restrictOS("WINDOWS")`.
* **Fix:** record both reasons in the spec and let `suite-not-run.txt`'s reason vocabulary include the test type.

### 17. The denominator depends on an untracked checkout with no pinned revision

* **Spec:** :52-54, criterion 1 :308.
* **Evidence:** `ootest/` is git-ignored (`.gitignore:6`), `svn info` gives revision 13178.
* **Fix:** the gate record and the three exclusion files name the svn revision they were derived at, and the harness asserts it.

### 18. D6 widens `parse-errors.tsv`, whose only reader is `rexx-parse/tests/errors.rs`

* **Spec:** D6 :27, 9c Delivers :291.
* **Evidence:** the TSV has four columns (`class`, `expect`, `line`, `program`), and `errors.rs:47` is its only reader.
* **Fix:** 9c's Delivers names the new column and the `errors.rs` change.

### 19. `rexx` must keep the `rexx-sim:` line and the 8-bit exit truncation, which are not "set on the invocation"

* **Spec:** :183-184.
* **Evidence:** `rexx-run.rs` prints `rexx-sim: seed=... policy=...` to stderr unless `SIM_CHILD_ENV` is set, and `ExitCode::from(outcome.exit_code as u8)`; the 6.1 spec section 4 says the sim harness reads that line.
* **Fix:** add both to the list at :183-184.

### 20. D3's structural test faces six literal shapes, one of them a test file

* **Spec:** D3 :24.
* **Evidence:** the 61 `"Phase 10"` literals are `util(name, "Phase 10")` (37), `deferred(name, family, "Phase 10")` (11, `native.rs`), `owned_message(.., Some("Phase 10"))` (`lib.rs`, `library.rs`), `gap(.., "Phase 10")` (`directives.rs`), a tuple table (`rexx-inventory/src/lib.rs:27-36`), `redirect.rs` (2), and `dispatch/native/tests.rs` (1, a test).
* **Fix:** say the reason is a parameter of each constructor shape, that the inventory table gains a column, and that `tests/` literals are out of the test's scope.

### 21. The roadmap row's grep misses the two `ffi.rs` strings 9a relabels

* **Spec:** :255-258.
* **Evidence:** `grep -rn '"Phase 9"' rust/crates/*/src` gives six hits; `ffi.rs:4187` and `:4229` carry the phase inside a longer literal and are found only by `grep 'Phase 9'`. `closed_phases` reads whole literals, so it will find them at 9d.
* **Fix:** 9a's relabel task uses `closed_phases`' `mentions` over `rexx-api`, not the roadmap's grep.

### 22. R:655 and R:725 still cite the parity gate as ":39"

* **Spec:** section 7 :223-224.
* **Evidence:** rereview finding 35 asked whether the two back-references are corrected in the same commit; the spec does not answer.
* **Fix:** add both to the "on agreement" list, or say they stay.

## Checked and true

* Every scout section number the spec cites.
* `lib.rs:511`, `:612`, `:664`; `native.rs:264`; `library.rs:838`; `directives.rs:80`; `environment.rs:363`; `class_protocol.rs:577`; `object_protocol.rs:1220`; `layout.rs:379-383`; `ffi.rs:4187`, `:4229`.
* The 30 routine names against `internal_routines.rs`'s 37 `util(` rows minus the seven macrospace rows.
* `phase-4-exclusions.txt:715`, `:5284`, `:5542`, `:6114`, `:6179`.
* R:46, :104, :115, :654, :661, :721, :726, :780.
* `rust/CLAUDE.md:13`'s sentence, and scout B's Release build facts.
* `59eb57f37` ("Phase 6.1 Task 5: parse errors as SYNTAX, with the traceback line"); the queued `2026-10-07-interpret-syntax-traceback-line` has no RESOLVED line and names the unfilled inserts.
* G1-G9 and the separate TSan criterion in `phase-6-1-gate.md`; criterion 6's wording matches.
* `callgrind.sh` takes `NAME=BINARY` and names no `rexx-run`, and runs every program by default.
* `common.txt`'s four entries; `reaching_rxapi` emits `GROUP.TEST` uppercased; `rexx-inventory`'s `EXCLUDED` has `RXFUNCQUERY`; `Package DEFAULTOPTIONS` exists; `require.rs` is `pub(crate)`.
* `tests/suite.rs`, `phase-9-exclusions.txt`, `suite-licences.txt` and `suite-not-run.txt` do not exist yet.

Verdict: not ready. Two blockers, both in the instrument: a loud refusal ends a group's run and D1 has no rule for it, and criterion 4's corpus witnesses cannot reach a per-side `rexxc`.
