# Final review, slice C: Task 11 and claim documents

Range: `659312de0..e64202ae7`. Task 11 range: `d1796f69c~2..e64202ae7` (commits `4122da9ac`, `d1796f69c`, `e64202ae7`).
Reviewer: read-only. Scratch: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/final-c/`.

Each finding: severity, file:line, what is wrong, the command that shows it (with output), and whether it was run or inferred.

## Part 1: Task 11 review (no task review existed)

### 1.1 `refresh()` and `delegated()` in `rust/crates/rexx-exec/tests/refusal_sites.rs`

Baseline (RUN): scratch copy of `rust/` at HEAD under `final-c/head/rust` with the repo's `interpreter/`, `api/`, `build/` etc. symlinked beside it (the `rexx-lib` build script reads `../../../interpreter/RexxClasses/CoreClasses.orx`, so a bare copy of `rust/` does not build). `CARGO_TARGET_DIR=final-c/t-head cargo test -j 4 -p rexx-exec --test refusal_sites`: 5 passed, exit 0 (`final-c/baseline-head.txt`).

**Defeat 1, prediction written before the run.** Mutation: `delegated()` returns `body.to_string()` unchanged.
* `a_reached_row_carries_its_own_site_identifier_and_an_unreached_one_does_not` FAILS, naming `missing_internal_argument`: its row is `reached=yes`, `answer=88.901`; its own body (`error.rs:357-362`) calls `Raised::missing_named_argument(argument)` and contains no `syntax(`; the `/// 88.901 for a routine...` doc line is stripped by `code_lines` (it cuts at the first `//`), so `88.901` appears in neither the body nor the construction-site code.
* The other four tests pass unchanged.
* Unobservable by this mutation: whether the one-level limit is enough (no constructor in the tree delegates two levels, as far as I can see from `error.rs:339-362`; not asserted).

**Defeat 1, result (RUN, `final-c/defeat1.txt`):** exactly as predicted. `a_reached_row_carries_its_own_site_identifier_and_an_unreached_one_does_not` FAILED with `missing_internal_argument: reached=yes, but "88.901" is none of this constructor's own identifiers (its syntax(M, N) numbers are []) ...`; the other four passed; exit 101. So `delegated()` is load-bearing for exactly one row, and the test can fail. Mutation reverted from a saved copy.

**`delegated()` pattern (read, not run):** it matches `"{prefix}{name}("` with `prefix` in `Raised::`, `Loud::`, so `Raised::foo(` cannot match inside `Raised::foo_bar(` (the `(` must follow the name) and `Raised::xfoo(` cannot match `foo` (the `::` must precede it). `syntax` itself (`error.rs:119`, `-> Raised`) is a constructor by the scanner's rule and has its own row (checked: `awk '$2=="syntax"'` finds 1), so every constructor that calls `Raised::syntax(` "delegates" to it; harmless, since `syntax`'s body builds a struct and names no `syntax(` of its own. One level only: no constructor in `error.rs:315-362` delegates two deep (read, not asserted).

**`refresh()` (read against the diff at `4122da9ac`, RUN via a table comparison script):** old table 216 rows, new 240; dropped: none; added: 24, of which 21 are off the send surface (written `off-send-surface` with no measurement, correct) and 3 are new send-surface rows written empty and then measured (`incorrect_method_signature`, `missing_internal_argument`, `missing_native_argument`); 1 row moved `body` -> `body+send` (`argument_not_in_list`) and was NOT carried (its old `off-send-surface` was replaced by a measurement). No row changed kind, and no row that stayed on the send surface changed surface. So in this refresh nothing was carried across a meaning change. Structurally: a rename drops the old row and writes the new one empty (red until measured); a move on or off `send` is refused by the `on_send ==` filter; a kind change with the same name IS carried (no such case here; the identifier check would redden it only if the number changed). A behaviour change with the same number and name is invisible to every test in this file, and the header says so ("The test does not re-run a probe"). Not new.

**Confirmed (RUN):** `the_table_holds_every_constructor_the_source_defines` FAILED at `4122da9ac~1` (`final-c/pre4122-refusal.txt`, exit 101) and passes at HEAD, so the gate document's attribution of that row's departure to `4122da9ac` holds.

**Independent enumeration (RUN, python over `crates/rexx-exec/src/**/*.rs` with `#[cfg(test)] mod` blocks dropped and `//` stripped, matching `fn NAME(...) -> Loud|Raised` across lines):** 240 constructor names; table 240 rows; sets equal; every definition line equal except `from`. Two instrument gaps, both pre-existing and both Minor (findings M6, M7 below).


### 1.2 The four measured rows in `rust/corpus/refusal-sites.tsv`

All RUN, from fresh directories under `final-c/probes-oracle/NN-*` (oracle, standard wrapper) and `final-c/probes-crate/NN-*` (this crate, `final-c/t-head/release/rexx-run` built from the scratch copy of HEAD, `LD_LIBRARY_PATH` = the oracle's `build/lib`), stdout/stderr/rc as three files, compared with `cmp` after substituting each side's own probe directory with `<DIR>` in stderr.

| row | witness as run | oracle | crate | verdict |
|---|---|---|---|---|
| `argument_not_in_list` (40.904, `agrees`, `yes`) | `say .Method~newFile('nofile.rex', 5)` | rc 216, `Error 40.904:  NEWFILE argument 2 must be one of "PROGRAMSCOPE", Method, Routine, or Package object; found "5".` | rc 216, byte-identical on all three | confirmed |
| `missing_internal_argument` (88.901, `agrees`, `yes`) | `say .Stream~new('/dev/null')~position` | rc 168, `Error 88 running REXX:  Invalid argument.` / `Error 88.901:  Missing argument; argument 1 is required.` | identical | confirmed (`running REXX`, no line: the internal-package delivery the constructor sets) |
| `missing_native_argument` (88.901, `agrees`, `yes`) | `r = .RegularExpression~new("a*b"); say r~parse()` over `::requires "<repo>/extensions/rxregexp/rxregexp.cls"` | rc 168, `Error 88 running <repo>/extensions/rxregexp/rxregexp.cls:  Invalid argument.` / `Error 88.901: ...` | identical | confirmed |
| `incorrect_method_signature` (93.968, `not-run`, `no`) | none; the row says no route without a compiled extension | not run | not run | the reason holds: `rxregexp.cpp:55-153` declares only `int`, `CSTRING`, `OPTIONAL_CSTRING`, `CSELF`, `RexxStringObject` (printed), all in the crate's table; `NativeActivation.cpp:190` `reportSignatureError` is the site |

Neighbour also run: `method_argument_not_in_list` (`.Package~defaultOptions('FORM')`) rc 163, `Error 93.914`, identical on both sides.


### 1.3 `docs/superpowers/plans/phase-8-gate.md` against the raw gate output

Raw files: `scratchpad/gate8/{rev,status,g1..g5}.txt`. `rev.txt` = `d1796f69c`; `status.txt` = G1 0, G2 0, G3 101, G4 101, G5 0, ALLDONE. All RUN.

Figures checked (command: `/bin/grep -a "^test result:" gN.txt | sed | awk` summing passed/failed; `/bin/grep -a -n "... FAILED"`; `/bin/grep -a -n -E "[0-9]+ of [0-9]+"`):

| claim (gate.md line) | raw | verdict |
|---|---|---|
| G3 exit 101, 2471 passed / 5 failed (:122) | status G3 exit=101; sum passed=2471 failed=5 | confirmed |
| G4 exit 101, 2471 / 6 (:123) | status G4 exit=101; sum passed=2471 failed=6 | confirmed |
| G5 exit 0, 531 of 531 (:124) | g5.txt `531 of 531 matching`, `29 passed; 0 failed` | confirmed |
| corpus 531 of 531 in G4 (:126) | g4.txt:1681 `531 of 531 matching` | confirmed |
| `base/keyword` 892 of 896 (:127) | g3:2461, g4:2471 `892 of 896 bodies passing` | confirmed |
| `base/bif` 4992 of 4999 value, 186 of 186 raise (:128) | g3:1362, g4:1362 | confirmed |
| assertion table 4247 of 4259 (:128-129) | g3:1321, g4:1321 | confirmed |
| G3 failing set = the five named (:131-136) | g3 FAILED lines 785, 789, 865, 1552, 1562: exactly those five | confirmed |
| G4 failing set = those + `concept_and_class_gate_table` (:136-137) | g4 FAILED lines 785, 789, 865, 1557, 1562, 2159 | confirmed |
| `concept_and_class_gate_table` green in G3 (:148) | g3:2161 `... ok` | confirmed |
| 82 gated rows, File 50 / Stream 24 / StreamSupplier 8, all unanswered (:151-152) | g4:2095 `File instance ... 50 row(s): unanswered=50`; `Stream instance 24 unanswered=24`; `StreamSupplier instance 8 unanswered=8`; `7: 94 rows, 82 not yet agree`; `gated by this run: 82 row(s)` | confirmed |
| `gate_tables/mod.rs:209` (:149) | line 209 is `pub fn verdict_is_gated(phase: &str) -> bool {`, 210 `corpus_gate() && ...` | lands |
| 516 of 516 at Phase 7's close (:126) | `phase-7-gate.md:153` `**516 of 516**` | confirmed (quoted, not re-run) |
| fifteen new programs (:127) | `phase-8.txt` non-comment lines: 4+1+1+3+3+3 = 15; 531-516 = 15 | confirmed |
| G3 set "every member older than this phase" (:131) | `phase-7-gate.md:159-162` G3 set = the five + `the_table_holds_every_constructor...` | confirmed by quotation |
| `directive_option_gate_table` red in G4 at Phase 7's close (:143) | `phase-7-gate.md:166` names it among G4's gate-only failures | confirmed by quotation |

Attribution RUN: `directive_option_gate_table` under `REXX_CORPUS_GATE=1` FAILED at `08d232ecc~1` (`final-c/pre08d-gate-d.txt`, exit 101) and passed at `08d232ecc` (`final-c/08d-gate-d.txt`, exit 0), each in its own `CARGO_TARGET_DIR`. The gate document's ":143 passes at `08d232ecc`" holds. Table D's owners in g4 (:2311-2314): `7: 2 rows, 0 not yet agree`, `deferred-parse-error-rendering: 2 rows, 2 not yet agree`, `gated by this run: 0 row(s)`; rows 12 and 18 of that table are `::REQUIRES LIBRARY agree` and `::ROUTINE EXTERNAL agree` -- matches :144-146.

`every_closed_phase_this_table_owns_rows_for_is_gated` is `ok` in g3 at :1832 (table C) and :2173 (table D) -- matches :92-94. No owner `8` appears in either table's "by owning phase" block (g4:2149-2157, :2305-2314).

Citation NOT landing (Minor, M1 below): :151-152 "the hazard Phase 7's own close named in its section 4".


## Part 2: claim documents across `659312de0..e64202ae7`

### 2.1 Cross-document drift

Facts stated in more than one document, each place printed (all RUN as greps over the seven documents plus the four roadmap blocks and the exclusions section, `/bin/grep -a`):

| fact | where stated | agree? |
|---|---|---|
| error numbers 98.903 / 90.998 / 90.999 / 98.982 / 98.984 / 88.901 / 88.909 | spec §3, §4, §9; plan Tasks 2, 5, 8; gate §2; l2 §1 | agree; no document carries 98.979 (pre-`b83ed8312`) or 98.978 except as "reachable from no surface" (spec :83, plan :172, gate :58); 93.968 / 40.918 appear only as `reportSignatureError` (spec :128, plan :279-280). Pre-correction set at `cf55c605d` printed: plan {40.918, 93.968, 98.978, 98.979, 98.982}, spec {98.978, 98.979, 98.982}; so gate §2's "five ... all five were wrong" is the distinct set {40.918, 93.968, 98.978, 98.979, 98.982}, which holds. But see M9: the gate attributes the corrections to two commits and the first three were `b83ed8312`'s |
| function-pointer total 218 = 7+142+26+21+11+11, from 220 header declarations minus the two typedefs at :259-260; 211 unfilled | scoping §1-2, spec §1-2, plan :25, plan Task 11 :456, roadmap row 8 (no number) | agree; re-run: `grep -cE '\(RexxEntry \* ?[A-Za-z_][A-Za-z0-9_]*\)'` = 220; per struct 7/142/26/21/11/11 (struct ranges 482-493, 503-674, 682-712, 718-743, 749-763, 769-784); the two non-members are :259 and :260 |
| the seven context functions and their tables | scoping §2 table; plan Task 7 :331-332; spec §1 | scoping and tree agree (`layout.rs:511,538,539,572,589` thread; `:636,638` method); plan Task 7 disagrees (M2) |
| which groups are embedding: `RexxStart`, `ProcessRexxStart`; six not | scoping §3, spec §1 and §9, surface Task 6, roadmap row 8, gate §5 :14 | agree with each other, disagree with the tree (I1) |
| what L2 stops at: `ooTest.frm:49`, `.local~hasEntry` | l2 §3, gate §5, roadmap row 8, D-L2, exclusions :4540 | agree; `ootest/ooTest.frm:49` printed: `if \ .local~hasEntry('OOTEST_FRAMEWORK_VERSION') then do`; reproduced on the scratch HEAD binary (probe 05: rc 120, `method "HASENTRY" of class "Directory" is not implemented (Phase 5)`) |
| the nine refusing `Directory` methods | l2 §3 table, exclusions :4541-4543, D-L2, row 8 ("Nine") | agree; measured on both `.local` (probes 03-12) and `.environment` (probes 13-14): all nine refuse rc 120 naming Phase 5, `at` and `put` agree with the oracle byte for byte |
| Phase 5 closed 2026-09-09 | exclusions :4543, D-L2 | `phase-5-gate.md:5` `CLOSED 2026-09-09` |
| owners: embedding = Phase 9; queue/RXAPI/`handle_set` = Phase 10; `Directory` debt = Phase 5 | spec §1, plan :76-78, surface Task 1, roadmap rows 9-10, `native.rs:223` | agree; `handle_set` moved to Phase 10 at `08d232ecc` (`git log -S`), recorded in the ledger Ruling 23 only (M10) |
| `unsafe` in two modules | D-U1, spec §8, plan :33, surface :48, gate §2 | agree with each other and with `unsafe_sites.rs:88-92` (ffi.rs, load.rs, plus rexx-core/src/lib.rs's older grant); the roadmap's own crate tree at :636-638 still says one module (I6) |
| "the oracle's own `build/lib/librxregexp.so`", "that exact file", never rebuilt | spec §3, D5 amendment, gate §4, plan :43, phase-8.txt :7-9, l2 §1b, surface :54 | the tree loads TWO files under that name (I7) |
| the roadmap `:2555` requirement | D-U1 :242, scoping §7 :141 | both cite a line that no longer holds the Phase 8 bullet (I3) |

### 2.2 Citations

Every `file:line` in the seven documents and the four roadmap blocks was printed (`sed -n`), including the bare `:NNN` forms the regex extraction missed. Landing: `SysLibrary.cpp:71,88,90,94-95,115`; `LibraryPackage.cpp:199-205,211,211-216,232-235,237-247`; `PackageManager.cpp:214,947,968`; `NativeActivation.cpp:190-193,219,294,327,1264-1310,1282-1286,1296-1306,1787-1807`; `NativeActivation.hpp:177-179`; `ActivationApiContexts.hpp:58-95,71-75`; `Activity.hpp:503`; `ThreadContextStubs.cpp:1863` (2265 lines: matches "2,265-line"); `oorexxapi.h:135-174,190-208,253-255,259-260,482,493,503,548,576,577,615,635,674,682,693,695,718,749,769,1041,4333`; `rexx.h` 37 `RexxReturnCode REXXENTRY`; `gate_tables/mod.rs:209`; `ooTest.frm:49`; `roadmap:2624`. At the commit they were written against: `lib.rs:785,801,820` and `native.rs:217` (51b8ca8ae), `dispatch.rs:8311-8344`, `package.rs:402-412` (cf55c605d) all land; at HEAD the code has moved, which the survey's "on this date" covers.

Not landing at HEAD: `roadmap:2555` (I3, cited twice); `phase-7-gate.md` "section 4" for the `unanswered` rule (M1); `ThreadContextStubs.cpp:1863-1885` under-covers its subject (M3).

### 2.3 Extent claims

Re-run (RUN unless marked):

* scoping §1: 220 declarations, two non-members, 7/142/26/21/11/11, 37 in `rexx.h`: hold.
* scoping §2: seven distinct `context->` names with uses 4,3,3,3,2,1,1: hold (`grep -oE 'context->[A-Za-z_0-9]+' | sort | uniq -c`). Five value types across the five methods: hold (`rxregexp.cpp:55,93,105-109,138-141,149-152` printed). `RaiseException0` at :73,:83,:130, with `Rexx_Error_Incorrect_method` at :73,:130 and `Rexx_Error_Invalid_template` at :83, so spec §7's "rxregexp uses `Rexx_Error_Incorrect_method`" names one of two.
* scoping §3: eight `.testGroup` under `ootest/ooRexx/API`, 409 in the suite: hold. The nine `testbinaries/` sources and three `.def`: hold (`ls`).
* scoping §4: ten crates at 51b8ca8ae: hold (`git ls-tree`). §5: the `Phase 8` refusal list at 51b8ca8ae: holds (`git grep -n "Phase 8" 51b8ca8ae -- rust/crates`). §6: 20 of 12059: hold.
* spec §3 / D5 amendment: `NEEDED` = libstdc++, libgcc_s, libc and zero `rexx` imports: hold for BOTH extension builds (I7); `liborxmethod.so` zero: holds; `build/bin/rexx` NEEDED librexx.so.4 + librexxapi.so.4: holds for the oracle checkout's binary; 386 unmangled exports of `librexx.so.4`: holds (`nm -D --defined-only | awk '$2 ~ /^[TW]$/' | grep -vc '^_Z'` = 386; the looser count including data symbols is 402, so the sentence's count depends on an unquoted filter).
* spec §4 "about thirty `REXX_VALUE_*` cases": 28 `case REXX_VALUE_` lines in :219-470. Holds as "about".
* spec §7 "no `.c` file includes `oorexxapi.h`": FALSE (I5). "`orxclassic1.c` ... includes `rexx.h`, which does not include it": holds (`rexx.h` includes rexxapitypes.h, rexxapidefs.h, rexxplatformdefs.h, rexxplatformapis.h).
* exclusions :4552-4554 "`rxregexp_package_entry+0x30` is zero with no relocation while `+0x38` relocates to `rxregexp_methods`": holds on both builds (`RexxPackageEntry` :262-273 puts `routines` at 0x30 and `methods` at 0x38; `readelf -rW` shows `R_X86_64_64 rxregexp_methods` at entry+0x38 and nothing at +0x30).
* l2 §1a "every row came back rc=0 out=0 err=0": NOT re-run (the loop is quoted; see Not reached). §1b "every file in the set sends to `self`": holds (the quoted loop printed nothing). §5 "every caller of `Interp::trace_invocation` in `run.rs` passes `self.program_path` unconditionally": holds in substance at HEAD and at c9616b3d0 -- the three callers at `run.rs:7806,7820,7835` pass `&package` where each is `let package = self.program_path.clone().into_bytes()` five lines above; the quoted grep alone does not show that.
* l2 §3 table: every row re-measured on both sides (probes 01-14); every value holds except the `allIndexes` abbreviation (M4).
* l2 §5: `::class A public subclass NoSuchClassHere` in a required `k2.cls`: oracle `Error 98 running <DIR>/k2.cls line 1` + `98.909`, crate `<DIR>/t.rex` (probe 25): the divergence is reproduced as written. §1d "`RegExp_Init` raising 38.0 ... `Error 38 running <dir>/t.rex line 1`": holds on both sides with the two-file `re.cls` shape (probe 27, identical); with `rxregexp.cls`'s `RegularExpression` the blame is `rxregexp.cls line 42` on both sides (probe 24), so the sentence is right for the shape it names.
* gate §1 "where it is asserted": `rexx-api/tests/{load,layout,handles,invoke,values,context}.rs` exist; `corpus/lang/library_*_missing.rex` are four; `native_entries.rs`, `unsafe_sites.rs` exist. §4 no owner `8` in tables C or D: holds (g4). §6 fifteen programs: holds.
* surface :36-41: `entry_point` private with `has_entry_point()` the reader (`load.rs:90,96`): holds. `Libraries` exposes `new`, `get`, `hold` (`libraries.rs:36-51`): holds. `values::repr`: not checked.

### 2.4 The surface plan

See C1, I1, I2 (prerequisites and partition), M8. Also checked: Task 1's `api/oorexxapi.h:190-208` holds (`ROUTINE_TYPED_STYLE`/`CLASSIC` at :200-201); Task 2's `usedArglist` exists (`NativeActivation.cpp:224,311,680`); Task 3 step 2's "the handle is a function of the object" matches `handles.rs:27`; Task 4's "`ExitContextInterface`, whose consumer is `testbinaries/orxexits`" holds (`orxinstance.cpp`, one of orxexits' two sources, names `RexxExitContext` 20 times); Task 7 step 2 matches the tree (`closed_phases.rs:36` CLOSED = ["Phase 7"], `native.rs:836` OPEN has "Phase 8"). Task 1's premise "install and then refuse loudly at the call" matches `lib.rs:565` and `run/tests.rs:7384`.


## Findings index

Task 11 step compliance (plan :454-465): Step 1 gate readings: done, five gates not four, both failing sets enumerated and attributed, every figure and both attributions confirmed against the raw output or by re-running (1.3). Step 2 surface plan: written, but the two embedding groups are not re-homed (I4) and the 211 pointers are not enumerated (M8). Step 3 KNOWN GAPS with transcripts: done (`phase-4-exclusions.txt:4535-4579`, transcripts by reference to `phase-8-l2.md`). Step 4 roadmap row 8 and Rung: done (`L2, blocked`). Step 5 commits: three, hashes quoted from `git log`. The refusal-sites refresh the ledger scoped into Task 11 (:488-510): done and load-bearing (1.1).

### Critical

* **C1. Surface plan Task 6 cannot run against the tree, and the plan does not say so.** `docs/superpowers/plans/2026-09-14-phase-8-surface.md:145-155` runs six ooTest API groups "with the single-group form". Every one of the eight groups is `::requires 'ooTest.frm'` (`/bin/grep -n "::requires 'ooTest.frm'" ootest/ooRexx/API/*/*.testGroup`: 8 hits, at :70 or :72 of each), and the framework stops at `ooTest.frm:49` on `.local~hasEntry` -- reproduced on the scratch-built HEAD binary, probe 05: rc 120, `rexx-exec: method "HASENTRY" of class "Directory" is not implemented (Phase 5)`. D-L2 (`2026-07-27-rust-rewrite.md:537-550`) records that no phase owns the unblocker, and the surface plan carries no task for it; Task 7 step 3 mentions D-L2 only as a reason not to close. Task 6 has no prerequisite line and its deliverable is unreachable from the tree as it stands. RUN.

### Important

* **I1. The "six non-embedding, two embedding" partition is false against `testbinaries/` and the groups.** Stated in scoping §3 (:100-105), spec §1 (:19-22) and §9 (:239, :252-253), surface Task 6 (:149-152), roadmap row 8 (:530) and gate §5 (:14). `INVOCATION.testGroup:52` and `ProcessInvocation.testGroup:52` load `INVOCATIONTester.cls`, whose `callInstanceProgram` (`testbinaries/orxinvocation.cpp:382`) calls `invokeProgram` (`orxinstance.cpp:913`), which calls `RexxCreateInterpreter` (`orxinstance.cpp:909`) -- the embedding API the spec puts in Phase 9. `CLASSIC.testGroup:822,861,875` `call rxfuncadd ... 'orxclassic1'` (`RexxRegisterFunctionDll`, `orxclassic1.c`), and `orxclassic.cpp` calls `RexxCreateQueue`, `RexxAddMacro`, `RexxLoadMacroSpace`, `RexxVariablePool` -- the queue and macro-space registries scoping §1 assigns to Phase 10. Only `METHOD` (`LIBRARY orxmethod`, and `orxmethod.cpp` imports no flat or embedding symbol) is shown to be extension-only; `CONVERSION`/`FUNCTION` load `CONVERSIONPackage.cls`/`FUNCTIONPackage.cls`, not examined. RUN (greps and `sed` over the named lines).
* **I2. Surface plan Task 5 needs a decision it does not name.** `:132-141` "Build `testbinaries/` ... with no source edit". `testbinaries/CMakeLists.txt:61,83,93,102,115` link every target against `rexx rexxapi`; `rexxinstance` and `provoke_locks` are executables calling `RexxCreateInterpreter` (`rexxinstance.cpp`, `provoke_locks.cpp`); `orxclassicexits.cpp` calls `RexxStart`, `RexxRegisterExitExe` (12), `RexxRegisterSubcomExe`. Linked against the oracle's `build/lib`, the build says nothing about this crate; linked against this crate, there is no `librexx.so`/`librexxapi.so` (no cdylib crate exists; the D5 amendment at `:231-237` puts the SONAME question in Phase 9). The roadmap's own row says "compile unchanged", which the headers alone can satisfy; the task should say which of the two it means and what `rexx`/`rexxapi` resolve to. RUN (CMake and source greps); the consequence is inferred.
* **I3. `roadmap:2555` no longer holds the Phase 8 bullet.** D-U1 `2026-07-27-rust-rewrite.md:242` "which is what `:2555` requires of this phase" and scoping §7 `:141` "which the roadmap's `:2555` requires". `sed -n 2555p` prints the `TRACE` value-lines bullet; the bullet meant is at `:2624`. It was `:2555` at `51b8ca8ae` (`git show 51b8ca8ae:... | sed -n 2555p`); this phase's own insertions into the same file (the D5 amendment, D-U1, D-L2) moved it. Cited twice, once from the file that moved it. RUN.
* **I4. Task 11 Step 2's re-homing was not done.** Plan `:458-459`: "The two that are embedding are re-homed to Phase 9 with the spec's section 9 as the reason." `/bin/grep -n "RexxStart\|ProcessRexxStart" 2026-07-27-rust-rewrite.md phase-4-exclusions.txt`: no hits; row 9 (`:531`) is unchanged; spec §9 `:252-255` requires the re-homing be recorded in `phase-4-exclusions.txt`. The surface plan's Task 6 step 1 (`:150-152`) instead instructs its own executor to "re-home them there explicitly". RUN.
* **I5. Spec §7 `:206`: "no `.c` file includes `oorexxapi.h`" is false.** `find . -name '*.c' -not -path './build/*' -not -path '*/target/*' | xargs grep -l oorexxapi.h` prints `./ootest/misc/dlOpenTest.c` (`:49` `#include <oorexxapi.h>`). Nothing in the tree builds that file (no CMake reference outside `build/`), so the paragraph's conclusion may stand, but the enumeration given as its reason does not. RUN.
* **I6. The roadmap's crate tree still grants one module.** `2026-07-27-rust-rewrite.md:636-638`: "`src/ffi.rs` # the only module carrying `#[allow(unsafe_code)]`". D-U1 in the same file (`:239-` ) grants `ffi.rs` and `load.rs`; `unsafe_sites.rs:88-92` lists both plus `rexx-core/src/lib.rs`. Scoping §4 `:109-111` quotes the annotation as the plan's. One file, two answers, and the phase's first decision is the fact that changed. RUN.
* **I7. "The oracle's own `build/lib/librxregexp.so`" names two different files.** Spec §3 `:90-96` ("that exact file"), D5 amendment `:219-230`, gate §4 `:86-89`, plan `:42-44`, `phase-8.txt:7-9`, l2 §1b `:72-74`, surface `:54-55`. `rexx-api/tests/{load,invoke,context}.rs` and `dispatch/library.rs:238-243` load `<repo>/build/lib/librxregexp.so` (Build ID `afcaedc0...`, from a 27 Jul 2026 build whose `bin/rexx` is 16496 bytes and answers `... 6.06 27 Jul 2026`); the corpus differential's `{oraclelib}` (`tests/corpus.rs:237` = `oracle_root()/lib` = `/home/moritz/dev/repos/ooRexx/build/lib`) gives both sides the 30 Jul build (Build ID `1aa47dd9...`, `bin/rexx` 62600 bytes, `... 30 Jul 2026`). `sha256sum` differs. The amendment's measurements (NEEDED set, zero `rexx` imports, the `+0x38` relocation) hold on both, so no instrument is wrong; the sentence "loading that exact file removes the question of whether the two sides compiled against the same header" is true only of the differential, and the unit tests load a binary the oracle never runs. RUN.

### Minor

* **M1.** gate `:151-152` "the hazard Phase 7's own close named in its section 4": `phase-7-gate.md:65-79` names `method-bodies.txt`'s `answers` blindness; `unanswered` and the "counted as open" rule appear nowhere in that file (`grep -n unanswered`: none). Lands beside its subject. RUN.
* **M2.** plan Task 7 `:331-332` puts all seven pointers "behind the method-context table" and names `WholeNumber`/`RaiseException`; the tree (`layout.rs:511,538,539,572,589`) has five in the thread table under `WholeNumberToObject`/`RaiseException0`, which scoping §2 `:49-52` warned is the difference between the inline names and the pointers; `:350` "`RaiseException`, `RaiseException0`, `RaiseException1`" where `ffi.rs:119` fills only `RaiseException0`. Spec §7 `:182` vs `:190` enumerate the family two different ways. RUN.
* **M3.** spec §7 `:192` `ThreadContextStubs.cpp:1863-1885` for "Each of `RaiseException0`, `1` and `2`": `:1863` is `RaiseException0`, `:1875` `RaiseException1`, `:1887` `RaiseException2`; the range ends inside the second. RUN.
* **M4.** l2 §3 `:273` `allIndexes` -> oracle `SYSCARGS`: `say .local~allIndexes` prints ten lines (`SYSCARGS INPUT TRACEOUTPUT DEBUGINPUT STDOUT OUTPUT STDERR STDIN STDQUE ERROR`); the table shows the first without saying so, and the eleven one-line programs are described, not quoted. RUN.
* **M5.** spec `:85`, plan `:174`: "`.Object~package~loadLibrary` is 98.984": with no argument it is 88.901 rc 168 (probe 18); 98.984 rc 158 needs a name argument (probe 19, `'nosuchlib_zz'`). The sentence omits the argument. RUN.
* **M6.** `refusal_sites.rs:206` `found.insert(name, ...)` keys constructors by bare name: three `fn from(...) -> Raised` (`error.rs:1640,1652,1668`, the `From<ArithError>`, `From<FormatError>`, `From<&ParseError>` impls) collapse to one row `from` at `:1668`. Off the send surface, so no verdict is touched, but "every constructor" is two short and the refresh inherits it. RUN (independent enumeration).
* **M7.** `surface()` (`refusal_sites.rs:302,313`) cannot see a call spelled `crate::name(` to a constructor defined in `lib.rs`: `routine_without_a_body` (`lib.rs:861`) is constructed only at `run.rs:4281` as `crate::routine_without_a_body(` and its row's surface column is empty (`awk -F'\t' '$3==""'`). `run.rs` is `body`, so the verdict is right by accident; the same spelling in `dispatch.rs` or `dispatch/` would hide a send-surface site. (The other `crate::` hits, `accessor_variable`/`delegate_variable` at `dispatch.rs:2331,2351`, call the same-named `Option`-returning free functions at `lib.rs:1239,1249`, not the `Loud` constructors, whose rows are correctly `send`.) RUN.
* **M8.** plan Task 11 `:456-457` "the 211 function pointers ... grouped by what they touch": the surface plan's Task 3 `:108-110` names six group labels and no members; the 211 are enumerated nowhere. Read.
* **M9.** gate `:57-59` "Corrected at `378d5613b` and `165373cf5`": the first three of the five (98.982 for `::REQUIRES`, 98.978/98.979 for the entry-missing paths) were corrected at `b83ed8312` (`git show b83ed8312 --stat`; its diff removes four `98.979`); the two named commits corrected two and retired one. RUN.
* **M10.** `handle_set` was re-homed from Phase 8 to Phase 10 at `08d232ecc` (`native.rs:223`, `git log -S`), recorded only in the ledger (Ruling 23, `progress.md:644`); the gate document has no re-homed-refusals section (Phase 7's gate has one at §6) and the exclusions entry does not mention it. RUN.

Confirmed and not findings: every figure in gate §6 (1.3); both failing-set attributions (RUN at the commits named); the four measured rows (1.2); the refresh's behaviour on this diff and the defeat (1.1); all the counts in 2.3 marked "hold"; every citation in 2.2 marked landing.

## Not reached

* l2 §1a's claim that the twenty extracted cases exit 0 with empty descriptors on both sides, and §1f's "AGREE for every row": the driver is quoted but I did not re-run the twenty.
* `CONVERSIONPackage.cls`, `FUNCTIONPackage.cls`, `CLASSICPackage.cls`, `INVOCATIONTester.cls`: which binaries they load and by what directive; I1 rests on the test groups' own lines and the C++ of `orxinvocation`/`orxinstance`/`orxclassic`.
* The ledger's own transcripts (Rulings 21-23, the E0616 re-review, the `Libraries` control) were read as claims, not re-run.
* gate §3's four negative controls and §2's "three tests that could not fail": not re-run; slice A/B territory.
* `values::repr`, `RequestGlobalReference`'s table, and whether the crate's stub for an unfilled pointer (`layout.rs:381` panics with "not implemented (Phase 8)") is the loud refusal the plan's Task 3 step 3 asked for, or an abort across `extern "C"` (spec §7 says a panic there aborts the binary).
* `rust/target/release/rexx-run` in the working tree: not used; its mtime (13:03:53) precedes the last `src` commit (`c9616b3d0`, 13:05:21), so a scratch binary was built instead.
* The spec's section 2 layouts (`RexxInstance_` etc.) against `oorexxapi.h`; `rexx-api/tests/layout.rs` is the instrument and slice A's subject.
* `phase-8-l2.md` §6's `svn status` statement about `ootest/`.
* Whether `ooTest.frm` would get past `:49` if the nine `Directory` methods answered, i.e. whether the next blocker is also Phase 5's: the documents do not claim it and I did not probe past the refusal.
