# Re-check of the Phase 9 spec after the fable review

Spec: `docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md` at `394876a6d`.
Read-only: grep and read only, nothing run. `rust/` is unchanged between `a98238be5` and `394876a6d` (`git log a98238be5..394876a6d -- rust/` is empty), so every scout line citation still holds.
Line numbers are the spec's at `394876a6d`.

## 1. The 22 findings

| # | State | Why |
|---|---|---|
| 1 | addressed | :56-60 iterate past a run-ending crate run (rename the last started test, cap 60, refusal text recorded per test); :79-80 and :89-90 say a row's test may end the run and a Deviation row may be loud. |
| 2 | addressed | :355-363 name `tests/compiled.rs`, per-side `rexxc` spawned explicitly, rc 0 and the 29-byte prefix asserted, a committed oracle-written `.rxo` fixture, and the native routes left to `METHOD`'s test under the harness. |
| 3 | addressed | :304-313: one measurement at the close against `rexx-run` at `a98238be5`, the 9c A/B inside the budget. Matches the decisions.md ruling. |
| 4 | addressed | :336-338 add `REXX_CORPUS_GATE=1 memcap 8G ... --no-fail-fast`. The command is labelled "6.1's G4" and is not G4 verbatim (finding 5 below). |
| 5 | addressed | :61-63 define a test as its uppercased method name and `ran` as the `started` set. :353-354 now say 30 and 8; `rexx_command.testGroup` has 30 `::method test*` lines, `rexxc.testGroup` 8 (counted). |
| 6 | addressed | :47-51 child process, :52-55 `memcap 2G` with the `command -v memcap` fallback to `ulimit -v 4194304`, :37-38 name the five reused pieces. |
| 7 | addressed | :44-46; `read_lossy` exists (`group_runner.rs:134`), `skip` :667, `without_rxfuncquery` :294. |
| 8 | addressed | :225-228 add `arity.rs:374`, `rexx-arms.rs`, `rust/CLAUDE.md:38`, `rust/corpus/README.md:295`, `lib.rs:2718-2719`. All five verified. `report.rs` is `rexx-bench-suite/report.rs:175,185`. |
| 9 | addressed | :184-185 and :284-285; `phase-4-exclusions.txt:6123-6135` is the `LIBRARY rexxutil` row with `OWNER: Phase 10` at :6135, and :5516 is `OWNER: Phase 10` (verified). |
| 10 | partly | :297-300 name `OPEN_PHASES` correctly. Two of the other names do not exist and the vocabularies that do carry the open set are not named (finding 1 below). |
| 11 | addressed | :133-137 give the 2x rule, the committed probe, the build and rusage. The probe is C++ and the sentence says C (finding 6 below). |
| 12 | addressed | :187-192: gate groups derived by `grep -rlwi` and committed, corpus program or unit test for a routine with no suite witness. |
| 13 | addressed | D2 :24 and :266-267. |
| 14 | partly | :324 scopes the 9a denominator check to groups with no 9b/9c-owned refusal, but "denominator check" is still undefined and collides with 9d's failing tests (finding 4 below). |
| 15 | addressed | :273-274 "Entry: 9", the L3-full question marked as Moritz's open item in the row text. |
| 16 | addressed | :66-68; `bug2061_newline_misalignment.testGroup:41` is `group~restrictOS("WINDOWS")`, `Section4.testGroup:59` sets `DOC_EXAMPLE_NOISE_TEST` (verified). |
| 17 | addressed | :70-71; `svn info ootest` answers 13178 today. |
| 18 | addressed | :326 names the new message column and `rexx-parse/tests/errors.rs`. |
| 19 | addressed | :209-211; `rexx-run.rs:69,132` print `rexx-sim:`, `:148` is `ExitCode::from(outcome.exit_code as u8)`. |
| 20 | partly | D3 :25 lists the constructor shapes and scopes `tests/` out, but the list is not the set of `"Phase 10"` literals under `src/` (finding 2 below). |
| 21 | addressed | :291-292; `closed_phases.rs:202` is `fn mentions(crate_dir)`, `:190` is `fn names`. |
| 22 | addressed | :254; R:655 and R:725 both read "the Global Constraints `:39` parity gate" and the gate is at R:46 (verified). |

## 2. Findings

### 1. major. The 9d vocabulary sentence names two things that do not exist and misses the vocabularies 9a's relabel reddens

* **Spec:** :297-300 ("9d removes `"Phase 9"` from the open-phase vocabularies (`closed_phases`' open set, `refusal_dispositions.rs`'s `OPEN_PHASES`, `owners.rs`'s `PHASES` and `SPLIT_TABLE_PHASES`)"), and D3 :25 / section 4 :168-169 (9a relabels the 30 `util` rows and `handle_set` from Phase 10 to Phase 9).
* **Evidence:** `closed_phases.rs` has no open set; its vocabulary is `const CLOSED: &[&str] = &["Phase 5", "Phase 6", "Phase 7", "Phase 8"]` (`:32`), so 9d's edit is an addition to `CLOSED`, which is what the sentence this commit replaced said. `owners.rs` has no `PHASES`; it has `SPLIT_TABLE_PHASES = &["4b", "4c"]` (`:315`) and `EXCLUSION_PHASES = &["Phase 10"]` (`:321`), neither of which contains `"Phase 9"`, so there is nothing to remove. The vocabularies that do hold the open set are elsewhere: `tests/internal_routines.rs:107` `const PHASES: &[&str] = &["Phase 10"]` in `every_unimplemented_row_names_an_open_phase_that_owes_it`, `src/dispatch/native/tests.rs:150` `const OPEN: &[&str] = &["Phase 10"]` in `every_deferred_entry_point_names_an_open_phase`, and `tests/bif_assertions.rs:780` `PHASES = &["4b", "4c", "Phase 10"]`. The first two go red the moment 9a relabels a `util` row or `handle_set` to `"Phase 9"`, and the spec names no edit to them at 9a or 9d. (My finding 10 named `owners.rs`'s `PHASES`; the spec copied it. The error is mine.)
* **Fix:** replace :297-300 with: "9a adds `"Phase 9"` to `tests/internal_routines.rs`'s `PHASES` and `src/dispatch/native/tests.rs`'s `OPEN` for the rows it relabels. 9d adds `"Phase 9"` to `closed_phases`' `CLOSED`, removes it from those two and from `refusal_dispositions.rs`'s `OPEN_PHASES`, adds `"Phase 9.1"` where a row may name it, and teaches `closed_phases`' `names()` that `Phase 9` is not `Phase 9.1`."

### 2. major. D3's shape list is presented as the set of `"Phase 10"` literals under `src/`, and two literals under `src/` fit no shape

* **Spec:** D3 :25 ("The reason is a parameter of each constructor shape that names Phase 10 (`util`, `deferred`, `owned_message`, `gap`, `redirection`, `rexx-inventory`'s `EXCLUDED` table, which gains a column), and a structural test asserts every `"Phase 10"` literal under `src/` carries one; literals in `tests/` are out of its scope").
* **Evidence:** `grep -rn '"Phase 10"' rust/crates/*/src` gives 60 hits: 37 `util(` (`internal_routines.rs`), 11 `deferred(` (`dispatch/native.rs`), 5 `EXCLUDED` rows, 2 `Loud::redirection(` (`redirect.rs:426,521`, a `Loud` associated fn at `lib.rs:561`, not under `dispatch/`), 3 `owned_message(` (`lib.rs:511,664`, `dispatch/library.rs:838`), 1 `gap(` (`directives.rs:80`, a local closure over `owned_message`), and two that fit none: `environment.rs:363` `let mut owed = [(ObjRef::NIL, None), (ObjRef::NIL, Some("Phase 10"))];` (the `.STDQUE` owed entry, rendered at `:364-377` with no constructor), and `src/dispatch/native/tests.rs:150` `const OPEN: &[&str] = &["Phase 10"];`, a vocabulary in a `#[cfg(test)]` module under `src/`, which "literals in `tests/`" does not name. As written, the structural test is red on `:150` or silently excludes it.
* **Fix:** in D3: "... (`util`, `deferred`, `owned_message` and the `gap` closure over it, `Loud::redirection`, `rexx-inventory`'s `EXCLUDED` table, which gains a column, and the `.STDQUE` owed entry at `environment.rs:363`, which becomes a constructor call), and a structural test asserts every `"Phase 10"` literal under `src/` outside `#[cfg(test)]` modules carries one; literals in `tests/` and in `tests.rs` modules are vocabularies and are out of its scope."

### 3. major. The 9a relabel to `Embedding` lands on shapes D3 does not cover, so either D3's test excludes `rexx-api` or 9a has an unstated edit

* **Spec:** :289-290 ("9a relabels the `"Phase 9"` strings R4 moves to Phase 10 with the `Embedding` reason: the five `REFUSING_MEMBERS` rows (`layout.rs:379-383`) and the `ffi.rs:4187` and `:4229` assertion texts") against D3 :25.
* **Evidence:** `layout.rs:379-383` is a tuple table `("RexxInstanceInterface.Terminate", "Phase 9")` with no reason column, and `ffi.rs:4187` and `:4229` are bare string literals inside assert messages. None of D3's shapes is in `rexx-api`. After 9a these become `"Phase 10"` literals under `rexx-api/src/` with no reason, and D3's test "every `"Phase 10"` literal under `src/` carries one" is then red, or it scans only `rexx-exec` and D3's "every" is false. The spec says which crates the test scans nowhere.
* **Fix:** add to :289-290: "`REFUSING_MEMBERS` gains a reason column like `EXCLUDED`, and the two `ffi.rs` texts are built by one constructor that takes the reason. The D3 test scans `rexx-exec` and `rexx-api`, the crates `closed_phases` scans."

### 4. major. "The denominator check" is undefined, and under D1's reading the 9a gate cannot pass

* **Spec:** 9a gate :324 ("The denominator check passes on every group with no test blocked by a refusal 9b or 9c owns"), D1 :23 (green is "every test in its expected set ran and passed"), section 3 :142-147 (9d owns 249 tests in 49 groups that fail here, fixed at 9d).
* **Evidence:** the phrase "denominator check" appears at :324 and nowhere in section 2, which defines expected set, ran, and passed separately. If it means D1's green rule, the 49 groups with a 9d-owned failure (`scout-a-groups.tsv`, 49 rows with non-empty `tests_wrong`, 36 in the wrong-answer bin and 13 in the loud-refusal bin) cannot pass it at 9a, and 9a cannot close. If it means "started equals expected", the gate is satisfiable but the spec does not say so.
* **Fix:** in section 2 after "What a test is": "The denominator check is: the set of names started equals the expected set. Pass or fail is criterion 1's." Then :324 reads as intended.

### 5. minor. The sub-phase close command is labelled "6.1's G4" and is not G4

* **Spec:** :336-337 ("6.1's G4: `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release -j 4 --no-fail-fast`").
* **Evidence:** G4 is `bggates.sh:27`: `REXX_CORPUS_GATE=1 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast`. The spec's line drops `timeout 2400`. The fix I offered in finding 4 had the same omission.
* **Fix:** either quote G4 verbatim, or write "the G4 command without its `timeout`".

### 6. minor. The `ReleaseLocalReference` probe is C++, not C

* **Spec:** :135-136 ("The probe's C source is committed, and the test builds it with the system `cc` against `api/`'s frozen headers"); decisions.md carries the same wording.
* **Evidence:** the queued file's probe (`2026-09-29-release-local-reference-memory.md:22-43`) is `relloop.cpp`, uses `context->String(...)` and `context->ReleaseLocalReference(s)`, the member-call wrappers `oorexxapi.h` declares under `__cplusplus`, and was built with `g++ -shared -fPIC -O1 -static-libstdc++ -I<oracle>/api -I<oracle>/api/platform/unix`. `cc` on a `.c` file does not compile it.
* **Fix:** "The probe's C++ source is committed, and the test builds it with the system `c++` against `api/`'s frozen headers". Correct decisions.md's ruling line to match, or have Moritz say he wants a C rewrite of the probe.

### 7. minor. "9a adds `"Phase 9.1"` to `OPEN_PHASES` already, for its split of `:6179`" has a false reason

* **Spec:** :300.
* **Evidence:** `OPEN_PHASES` is consulted only for `REHOME` rows of `corpus/refusal-dispositions.tsv` (`refusal_dispositions.rs:643-649`). That test reads `phase-4-exclusions.txt` only to check citations (`:679`, `:692`, `:737`). The `:6179` split is an `OWNER:` line in `phase-4-exclusions.txt`, policed by `closed_phases.rs:370` (`no_open_exclusions_row_names_a_closed_phase`) against `CLOSED`, which does not contain `"Phase 9"` until 9d. So the 9a addition does nothing for the split.
* **Fix:** delete the sentence, or keep it with its real reason: "so that a 9a `REHOME` row may name Phase 9.1", if 9a writes one.

## 3. Checked and true in the new text

* `read_lossy`, `skip`, `without_rxfuncquery`, `fresh_copy`, `reaching_rxapi`, `interpreter_bin` all exist in `group_runner.rs`.
* `closed_phases.rs` `mentions` (`:202`) and `names` (`:190`).
* `lib.rs:662-664` is `fn package_option_write` with `owned_message("a package settings write", Some("Phase 10"))`; `lib.rs:511` is `named_semaphore`.
* `lib.rs:2718-2719` both name `rexx-run`; `rust/CLAUDE.md:38`; `rust/corpus/README.md:295`; `arity.rs:374`; `rexx-arms.rs:15`; `child.rs:50,58`; `rexx-bench-suite/report.rs:175,185`; `wallclock.sh:13,64,65`; `callgrind.sh` names no `rexx-run`.
* `api/` exists at the repo root with `oorexxapi.h`; "frozen header" is the term `rexx-api/src/layout.rs:12,590` uses.
* `Loud::redirection(what, owner)` at `lib.rs:561`; `util(name, owner)` `internal_routines.rs:85`; `deferred(name, family, owner)` `dispatch/native.rs:122`; `owned_message(name, Option<owner>)` `lib.rs:946`.
* 249 tests in 49 groups: re-summed from `scout-a-groups.tsv` column `tests_wrong` (49 rows, 249 space-separated names). 22 + 8 + 1 + 4 + 17 = 52.
* 30 `::method test*` in `rexx_command.testGroup`, 8 in `rexxc.testGroup`, one class and no `inherit` in each.
* R:46 is the parity bullet; R:655 and R:725 cite `:39`; R:654, :721, :726, :780, :115 as cited; `phase-9-exclusions.txt` named at R:721.
* `phase-4-exclusions.txt:5284` (Phase 9), `:5516` (Phase 10), `:5542` (Phase 9), `:6114` (Phase 9), `:6123-6135` (`LIBRARY rexxutil`, Phase 10 at `:6135`), `:6179` (Phase 9, the file's last line).
* "three rounds, then the figures go to Moritz" is the 6.1 spec `:204`.
* `429752 KB` and `20296 KB` at 2,000,000 passes in the queued file match "430 MB against the oracle's 20 MB".
* `samples/scclient`, `samples/sfclient` exist; `common.txt`'s four entries; `OPTIONS.testGroup:199,222` (under `base/directives/`).
* `tests/compiled.rs`, `tests/suite.rs`, the three exclusion files: absent, as the spec expects.

Verdict: not ready. No blocker. Four majors, all in the label machinery and the 9a gate: the 9d vocabulary sentence is half fictional and misses the two tests 9a's relabel reddens, D3's shape list and scope do not cover two literals under `src/`, the `Embedding` relabel lands on shapes D3 has no reason parameter for, and "denominator check" is undefined where 9a's gate leans on it.
