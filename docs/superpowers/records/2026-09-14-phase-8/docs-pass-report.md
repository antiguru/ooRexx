# Phase 8 L2 slice -- documentation pass report

Started 2026-09-15 at HEAD `362f50453` (Keep no routines or tables for a package whose translation
raised). Brief: `docs-pass-brief.md`. Written first; appended as work proceeded. Scratch:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/docs-pass/`.

## Status

Finished. Six commits, tree clean at `6be99d610`, no gate reading written anywhere.

## Commits (hashes read back from `git log`)

```
313807bc5 Record the measured API group partition and the two librxregexp builds   (2026-07-27-rust-rewrite.md)
6db4f085f Correct the Phase 8 specs after the final review and the fix rounds      (scoping, native-api)
6fe410516 Correct the Phase 8 plan, L2 walk and gate documents, and record the rounds (2026-09-14-phase-8.md, phase-8-l2.md, phase-8-gate.md)
3bb1d34d2 Record Phase 8's re-homed groups and post-review known gaps               (phase-4-exclusions.txt)
9de6b771f Say which librxregexp.so build phase-8.txt's programs load                (rust/corpus/phase-8.txt)
6be99d610 Name the test that pins the two 93.968 deliveries                         (native-api, one sentence)
```

The trailer names `Claude Fable 5.1`, the model that did the work, where the brief's trailer names
`Claude Opus 5` (the branch's convention from the earlier rounds). The session line is the brief's.

## Tests run

* `cargo test -j 4 -p rexx-exec --test licensed_divergences --test coverage --test builtin_status --test owners`
  from `rust/`, after the exclusions edit, statuses to
  `docs-pass/test-exec-status.txt`: exit 0; `test result: ok` for all four binaries, 26, 21, 22 and
  6 passed, 0 failed (`docs-pass/test-exec.txt`).
* `cargo test -j 4 -p rexx-inventory`: exit 0 (`docs-pass/test-inventory.txt`, five `ok` result
  lines, 0 failed).
* `cargo build --release -p rexx-exec` from `rust/`: `Finished ... in 0.03s`, nothing rebuilt, so
  the `rexx-run` used for every crate-side probe is the committed sources at `362f50453`.
* No gate command (G1-G5), no fmt, no clippy: docs only, per the brief.

## Commands re-run before writing (all 2026-09-15, worktree at `362f50453`)

* **The API group partition.** `/bin/grep -n -E "Package\.cls|Tester\.cls" ootest/ooRexx/API/*/*.testGroup`:
  every group loads its package file at its line 52 with `.context~package~loadPackage(...)`:
  `CLASSIC` -> `CLASSICPackage.cls`, `CONVERSION` -> `CONVERSIONPackage.cls`, `FUNCTION` ->
  `FUNCTIONPackage.cls`, `METHOD` -> `METHODPackage.cls`, and `INVOCATION`, `ProcessInvocation`,
  `ProcessRexxStart`, `RexxStart` -> `INVOCATIONTester.cls`. Each `.cls`'s `LIBRARY` names
  (`/bin/grep -oE 'LIBRARY [A-Za-z0-9]+' ootest/ooRexx/API/*/*.cls | sort | uniq -c`):
  `CLASSICPackage.cls` -> `orxclassic` only; `CONVERSIONPackage.cls` and `METHODPackage.cls` ->
  `orxmethod` only; `FUNCTIONPackage.cls` -> `orxfunction` only; `INVOCATIONTester.cls` ->
  `orxinvocation` only. `CLASSIC.testGroup:822,861,875` `call rxfuncadd ... 'orxclassic1'`.
  `readelf -d` on `/home/moritz/dev/repos/ooRexx/build/lib/liborx{method,function,invocation,exits,classic,classic1}.so`:
  `liborxmethod.so` and `liborxfunction.so` NEED `libc.so.6` only; `liborxinvocation.so` NEEDs
  `liborxexits.so` and `libc.so.6`; `liborxexits.so` NEEDs `librexx.so.4`, `librexxapi.so.4`,
  `libc.so.6`; `liborxclassic.so` NEEDs `librexx.so.4`, `librexxapi.so.4`, `libc.so.6`;
  `liborxclassic1.so` NEEDs `librexxapi.so.4`, `libc.so.6`. `nm -D --undefined-only ... | /bin/grep ' Rexx'`:
  `liborxmethod.so` and `liborxfunction.so` none; `liborxinvocation.so` none of its own (its
  `liborxexits.so` imports `RexxCreateInterpreter`, `RexxStart`, `RexxRegisterExitDll/Exe`,
  `RexxDeregisterExit`, `RexxQueryExit`, `RexxRegisterSubcomExe`, `RexxDeregisterSubcom`,
  `RexxVariablePool`, `RexxFreeMemory`); `liborxclassic.so` imports `RexxAddMacro`, `RexxAddQueue`,
  `RexxAllocateMemory`, `RexxClearMacroSpace`, `RexxClearQueue`, `RexxCreateQueue`,
  `RexxDeleteQueue`, `RexxDeregisterFunction`, `RexxDropMacro`, `RexxFreeMemory`,
  `RexxLoadMacroSpace`, `RexxOpenQueue`, `RexxPullFromQueue`, `RexxQueryFunction`,
  `RexxQueryMacro`, `RexxQueryQueue`, `RexxQueueExists`, `RexxRegisterFunctionExe`,
  `RexxReorderMacro`, `RexxSaveMacroSpace`, `RexxVariablePool`; `liborxclassic1.so` imports
  `RexxDeregisterFunction`, `RexxDeregisterSubcom`, `RexxQueryFunction`, `RexxQuerySubcom`,
  `RexxRegisterFunctionDll/Exe`, `RexxRegisterSubcomDll/Exe`. `testbinaries/orxinstance.cpp:909`
  calls `RexxCreateInterpreter`; `orxclassicexits.cpp:808` calls `RexxStart`. This agrees with the
  controller's partition in `progress.md` ("Triage"), with one addition the documents now carry:
  `RexxStart` and `ProcessRexxStart` also load `INVOCATIONTester.cls`, so all four embedding groups
  go through `orxinvocation`. No command contradicted the ruling.
* **`api/` and `testbinaries/` identity.** `diff -rq api/ /home/moritz/dev/repos/ooRexx/api/` and
  `diff -rq testbinaries/ /home/moritz/dev/repos/ooRexx/testbinaries/`: both empty. The oracle's
  `build/lib` holds `liborxclassic1.so`, `liborxclassic.so`, `liborxexits.so`, `liborxfunction.so`,
  `liborxinvocation.so`, `liborxmethod.so` (all dated Aug 5), and `build/bin` holds
  `provoke_locks` and `rexxinstance`: one product per `testbinaries/` target.
* **Two `librxregexp.so` builds.** `ls -laL`: this worktree's `build/lib/librxregexp.so` 32,392
  bytes, Jul 27; the oracle checkout's 126,408 bytes, Aug 5. `sha256sum`: `86b7ae42...` against
  `37cefa57...`. `readelf -n` Build IDs `afcaedc08ce5ee60bc63a9376062aaec60a4df21` against
  `1aa47dd99d89a2257640b99a55dba9cbddf9491f`. `diff -rq extensions/rxregexp /home/moritz/dev/repos/ooRexx/extensions/rxregexp`:
  empty. On both: NEEDED `libstdc++.so.6`, `libgcc_s.so.1`, `libc.so.6`; `nm -D --undefined-only | /bin/grep -ci rexx`
  = 0. Which instrument loads which: `rexx-api/tests/load.rs:30,109`, `invoke.rs:49`,
  `context.rs:43` and `rexx-exec/src/dispatch/library.rs:264-265` resolve `CARGO_MANIFEST_DIR`
  up to this worktree's `build/lib`; `tests/support/sidecar.rs:183` expands `{oraclelib}` to
  `oracle_root().join("lib")`, and `tests/support/oracle.rs:57` is the oracle checkout.
* **I3.** `sed -n 2555p` prints the `TRACE` value-lines bullet. The Phase 8 bullet ("**Phase 8**
  rebuilds `testbinaries/` unchanged ... open a decision block for it *before* writing the
  `extern "C"` entry points") was at `:2624` before my edits and is at `:2638` after them
  (re-printed at HEAD `6be99d610`: `sed -n 2638p` begins `- **Phase 8** rebuilds`). Both
  citations (`2026-07-27-rust-rewrite.md:254`, scoping `:167`) say `:2638`. The bullet is under
  `## 6. Generating the plans for Phases 2–10` (`:2548`).
* **I5.** `find . -name '*.c' -not -path './build/*' -not -path '*/target/*' -not -path './.git/*' | xargs /bin/grep -l oorexxapi.h`
  prints `./ootest/misc/dlOpenTest.c` alone; `/bin/grep -rn dlOpenTest --include=CMakeLists.txt --include='*.cmake' .`
  outside `build/` finds nothing.
* **M2, M3.** `ThreadContextStubs.cpp`: `RaiseException0` at `:1863`, `RaiseException1` `:1875`,
  `RaiseException2` `:1887` (closing brace `:1897`), `APIRaiseException` `:1899`. Header
  `oorexxapi.h:635-638` declares `RaiseException0`, `1`, `2` and `RaiseException` (array).
  `ffi.rs:168,172` fills `WholeNumberToObject` and `RaiseException0`; `layout.rs:514,592-595` are
  the thread-table slots. `rxregexp.cpp:73,130` raise `Rexx_Error_Incorrect_method` and `:83`
  `Rexx_Error_Invalid_template`, all through `RaiseException0`.
* **M9.** `git show --stat`: `b83ed8312` "Correct Phase 8's load-failure errors from a run"
  (removes four `98.979` and four `98.978` lines); `378d5613b` "Correct the argument-refusal
  numbers, from a run again"; `165373cf5` "Retire the last error number that was read rather than
  run" (removes four `98.978` lines).
* **M10.** `dispatch/native.rs:223` `deferred("handle_set", Family::Stream, "Phase 10")`, doc at
  `:99-105`; `git log -S handle_set` names `08d232ecc`, whose message says "`handle_set` moves to
  Phase 10".
* **M5 and M4, oracle re-run** (`docs-pass/m4`, `m5a`, `m5b`, fresh directories): `say
  .local~allIndexes` prints ten lines `SYSCARGS INPUT TRACEOUTPUT DEBUGINPUT STDOUT OUTPUT STDERR
  STDIN STDQUE ERROR`, rc 0; `say .Object~package~loadLibrary` is rc 168, `88.901 Missing
  argument; argument name is required.`; `say .Object~package~loadLibrary('nosuchlib_zz')` is rc
  158, `98.984 User additions are not allowed to the REXX package.`
* **M1.** `phase-7-gate.md:65-79` (section 4) names `method-bodies.txt`'s `answers` blindness and
  never the word `unanswered` (`/bin/grep -n unanswered phase-7-gate.md`: none). The label is
  `gate_tables/mod.rs:215` (`UNANSWERED`) and `:222` (`verdict_label`); the tally rule is `:307`
  ("A non-`agree` row above is a row still ...").
* **`unsafe_sites.rs:89-91`** grants `rexx-api/src/ffi.rs`, `rexx-api/src/load.rs`,
  `rexx-core/src/lib.rs`; blocks in `ffi.rs`, `load.rs`, `rexx-core/src/bytes.rs` (`:107-109`).
* **`RUNPATH`**: `readelf -d` on the oracle checkout's `bin/rexx` and `lib/librexx.so.4`, both
  `Library runpath: [/home/moritz/dev/repos/ooRexx/build/lib:]`.
* **Loader hooks.** `orxmethod.cpp:2729`, `orxfunction.cpp:969`, `rxregexp.cpp:224` set the hooks
  `NULL`; `orxinvocation.cpp:473-478,532-533` defines and installs `packageLoader` and
  `packageUnloader`. `rexx-api/src/layout.rs:313-314` carries `loader` and `unloader` as fields;
  `/bin/grep -n -i loader rust/crates/rexx-api/src/load.rs` finds only a doc line about the search.
* **KNOWN GAPS probes**, `docs-pass/probes/run.sh`, each side in its own fresh directory, oracle
  under the standard wrapper, crate `rust/target/release/rexx-run`, both with
  `LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib`, `</dev/null`, three descriptors to
  three files, the run directory rewritten to `<DIR>`:
  * `b12` (B12): `signal on syntax` around `r~doparse()` over the committed `re.cls`: oracle
    `position The NIL object` / `program <DIR>/re.cls` / `traceback items 2` (`Compiled method
    "DOPARSE" with scope "RE".`, `3 *-* say r~doparse()`); crate `position 3` / `program
    <DIR>/main.rex` / `traceback items 1`. rc 0 both, `code 88.901` both. Reproduced.
  * `r9` (R9): `'xaaby'~pos(.Req~new)` with a `REQUEST` method: oracle `request STRING` / `pos 2`
    rc 0; crate rc 168, `Error 88.909: Argument 1 must have a string value.` Reproduced.
  * `c1` (`LIBRARY REXX` `unresolved_external` in a required package): oracle `Error 90 running
    <DIR>/pk.cls line 3`, crate `<DIR>/main.rex line 3`; 90.998 and rc 166 both. Reproduced.
  * `c2` (`::class A public subclass NoSuchClassHere` in a required `k2.cls`): oracle `Error 98
    running <DIR>/k2.cls line 1`, crate `<DIR>/main.rex line 1`; 98.909, rc 158 both. Reproduced.
  * `f4` (residual re-review finding 4): `.Package~new('src', 'say .K', .context~package)` with
    `::class K public` in the caller: oracle `The K class`, crate `.K`; `answered a Package` and
    rc 0 both. Reproduced.
  * `mnew`: `say .Method~new('mm', 'return 43', .context~package)`: oracle `a Method` rc 0; crate
    rc 120 `method "NEW" of class "Method" is not implemented (Phase 5)`. Reproduced.
  * `tb`: untrapped `.Package~new('src', 'call nosuchroutine_zz')`: both rc 213 with `1 *-* call
    nosuchroutine_zz` / `Error 43 running src line 1` / 43.1; the oracle's stderr also carries
    `*-* Compiled method "NEW" with scope "Package".` and `1 *-* p = .Package~new(...)`, the
    crate's does not. Reproduced.
  * `cwd` (`RUNPATH`): the oracle's `librxregexp.so` copied into the run directory as
    `libzzregexp.so`, `LD_LIBRARY_PATH` the oracle's lib only: oracle `cwd copy 1`, crate `cwd
    copy 0`, rc 0 both. Reproduced.
  * `b7r1`, `b7r6` (B7, beyond the brief's list; run because the exclusions file's routine-half
    paragraph described the call as a loud refusal): `loadLibrary('rxmath')` then
    `RxCalcSqrt(16)`: oracle `load 1` / `sqrt 4` rc 0, crate `load 1` then `Error 43.1: Could not
    find routine "RXCALCSQRT"` rc 213; `::routine sq external "LIBRARY rxmath RxCalcSqrt"` then
    `RxCalcSqrt(16)`: oracle `main 4`, crate 43.1 rc 213. Silent on stdout, so "refuse loudly at
    the call" was true of the directive's own name and false of the library's.
  * Not run, by rule: `oracle-crashes.txt` entry 15 (Y1's segfault); the twenty L1 cases' driver.
* **Citations printed and landing:** `DirectiveParser.cpp:1682` (`createNativeMethod(internalname,
  library, getName)`) and `:1689` (the setter); `LibraryPackage.cpp:166-175` (`unload`, runs
  `package->unloader`), `:232-246` (version check, `loadRoutines`, then `package->loader`);
  `PackageManager.cpp:642-650` (`unload` over every package); `Interpreter.cpp:281`
  (`PackageManager::unload()`); `classes/PackageClass.cpp:1042-1049` (`parentPackage->findPublicClass`);
  `run.rs:3842` (`self.package_parents.get(&program)`, the one reader); `dispatch.rs:8266,8278`
  (`native_executable_new`, `args.len() > 2`); `dispatch.rs:3231,3242` (`string_conversion`,
  `blamed_string_conversion`); `error.rs:1794` (`FailureSite::Rendered`), `:1952,1976`
  (`self.delivery.lineless` in the report); `dispatch/library.rs:563`
  (`a_refusal_before_the_call_is_lineless_and_one_after_it_is_not`); `gate_tables/mod.rs:215,222,307`;
  `docs/superpowers/records/2026-09-12-phase-7/survey/E-external-resolution.md:305-312`
  (`parentPackage`, `findRoutine` / `findClass`); `corpus/oracle-crashes.txt:805` (entry 15) and
  `:568-573` (entry 10, `ENDLOCAL`); `556798fae` "Deliver QUALIFY, USERID, SETLOCAL, ENDLOCAL and
  VALUE's selectors"; `.env` of `library_load_retried`, `library_package_retried`
  (`LD_LIBRARY_PATH={oraclelib}:{run}`, copy into `{run}`) and `library_search_path_fixed` (copy
  into `{run}/widened`, no `LD_LIBRARY_PATH` line).

## Findings, one row each

| finding | corrected where | commit |
|---|---|---|
| I1 (the two-embedding partition) | scoping §3 (measured partition with commands); spec §1, §9 table row and "what the gate cannot see"; roadmap row 8; gate §5 line 14 and new §7; exclusions new entry | `6db4f085f`, `313807bc5`, `6fe410516`, `3bb1d34d2` |
| I3 (`:2555`) | D-U1 (`:254`) and scoping §7 (`:167`) both cite `:2638`, re-printed at HEAD | `313807bc5`, `6db4f085f` |
| I4 (re-homing not recorded) | roadmap rows 9 and 10; exclusions entry "FIVE OF THE EIGHT ..."; no claim that a test derives it | `313807bc5`, `3bb1d34d2` |
| I5 (`no .c file`) | spec §7 names `ootest/misc/dlOpenTest.c:49` and the find command | `6db4f085f` |
| I6 (crate tree, one module) | roadmap crate tree names `ffi.rs` and `load.rs`; scoping §4 notes the correction beside its quote | `313807bc5`, `6db4f085f` |
| I7 / B9 (two `librxregexp.so`) | D5 amendment; spec §3 and the §9 table row; gate §4; plan Global Constraints; l2 §1b; `phase-8.txt` header | all five group commits |
| M1 (Phase 7 §4 citation) | gate §6 paragraph cites `method-bodies.txt`'s blindness for the section and `gate_tables/mod.rs:215-222,307` for the rule | `6fe410516` |
| M2 (Task 7 tables and names) | plan Task 7 interfaces (five thread, two method-context, pointer names) and step 5 note; spec §7 family as `oorexxapi.h:635-638` declares it, `rxregexp`'s two numbers | `6fe410516`, `6db4f085f` |
| M3 (`:1863-1885`) | spec §7 `:1863-1897` with the three starts | `6db4f085f` |
| M4 (`allIndexes`) | l2 §3 cell and a sentence with the ten lines and the program shape | `6fe410516` |
| M5 (`loadLibrary` 98.984) | spec §3, plan Task 2 step 4: name argument; 88.901 without | `6db4f085f`, `6fe410516` |
| M6, M7 (`refusal_sites.rs` gaps) | not corrected: code, out of scope | -- |
| M8 (211 pointers not enumerated) | not corrected: plan Task 11's text is a record of what was asked, and the surface plan is out of scope; no in-scope document claims the enumeration exists | -- |
| M9 (correction commits) | gate §2 names `b83ed8312`, `378d5613b`, `165373cf5` with what each corrected | `6fe410516` |
| M10 (`handle_set`) | gate §7 "Refusals this phase re-homed"; scoping §5; exclusions entry | `6fe410516`, `6db4f085f`, `3bb1d34d2` |
| B6 ("all predating Phase 8") | exclusions paragraph rewritten with both re-run transcripts; l2 §5 heading, intro and third divergence; gate §5 | `3bb1d34d2`, `6fe410516` |
| B12 (condition object) | exclusions KNOWN GAPS entry with my `b12` transcript, no owner with reason | `3bb1d34d2` |
| B7 (routines 43.1 at other load sites; beyond the brief) | exclusions routine-half paragraph, with my `b7r1`/`b7r6` transcripts and "the surface half owes it" | `3bb1d34d2` |
| F1 protocol | spec §4 (93.968 measured through a forge, its deliveries), gate §7; no in-scope document described the NOSTRING mechanism (swept) | `6db4f085f`, `6fe410516`, `6be99d610` |
| F2 lineless / result-side 93.968 | l2 §1d sentence; spec §4; gate §7 | `6fe410516`, `6db4f085f` |
| F3 required package naming | l2 §5; exclusions; gate §1 row, §5, §7 | `6fe410516`, `3bb1d34d2` |
| F4 `~package`, X2/X3 records | gate §7 only: no in-scope document described the `REXX` answer (swept) | `6fe410516` |
| F5 `Libraries`, version refusal, 98.982 | spec §3 (hooks), §9 row (98.982 no corpus witness); gate §7; exclusions (98.982 instrument gap) | `6db4f085f`, `6fe410516`, `3bb1d34d2` |
| F6 search path once | gate §7 only: no in-scope document described the per-resolve read (swept) | `6fe410516` |
| F7 `value_of` unsafe, F8 wrapper, X1 slots | spec §2 and §8; gate §7 | `6db4f085f`, `6fe410516` |
| F11 sidecar, `.env` removed | l2 §1e; gate §7 | `6fe410516` |
| Z1 | gate §7; exclusions (unmeasured `~addRoutine`/`~addPackage`) | `6fe410516`, `3bb1d34d2` |
| R9 | exclusions KNOWN GAPS with my `r9` transcript | `3bb1d34d2` |
| R6 | gate §7 records both counts as ledger-recorded, commits left | `6fe410516` |
| `RUNPATH` empty element | exclusions, with `readelf` and my `cwd` transcript | `3bb1d34d2` |
| `LIBRARY REXX` `unresolved_external` | exclusions (inside the B6 paragraph), my `c1` transcript | `3bb1d34d2` |
| loader/unloader | spec §3 sentence; exclusions entry with the C++ citations and the hook table | `6db4f085f`, `3bb1d34d2` |
| Y1 segfault, entry 15, precedent | exclusions entry | `3bb1d34d2` |
| `Method~new`/`Routine~new` third argument | exclusions entry with my `mnew` transcript | `3bb1d34d2` |
| R10 blind spots | exclusions entry | `3bb1d34d2` |
| `::ATTRIBUTE` getter-first | exclusions entry (read, C++ cited) | `3bb1d34d2` |
| residual re-review finding 4 | exclusions entry with my `f4` transcript | `3bb1d34d2` |
| "loud, pre-existing, not counted" list | exclusions: the traceback line (my `tb` transcript), `Method~new`, the routine half; `~routines~items` rc 120 not given its own entry (a Phase 5 `StringTable` refusal, loud, the same debt class as `Method~new`'s) | `3bb1d34d2` |
| missing `Compiled method "NEW"` traceback line | exclusions entry | `3bb1d34d2` |
| Miri outside the gate; `collect_stress` L0 | exclusions entries | `3bb1d34d2` |
| gate document's round record | gate §7 with counts, commits, parked items and where | `6fe410516` |

## Not done

* **M6 and M7**: code findings in `tests/refusal_sites.rs`; out of scope for a docs pass.
* **M8**: the plan's Task 11 step 2 keeps "grouped by what they touch" as the record of what was
  asked; the surface plan is the controller's.
* **`~routines~items` rc 120** from the residual re-review's loud list has no entry of its own; it
  is a Phase 5 `StringTable` refusal of the same class as `Method~new`'s, which has one.
* **Not re-run**: the twenty L1 cases' driver (l2 §1a, §1f) and the L2 walk itself; the l2
  statements stand as the run they record. Y1's crash program was not run, by rule.
* **Trailer**: `Claude Fable 5.1` rather than the brief's `Claude Opus 5`, for accuracy; the
  session line is the brief's.
* **Beyond the brief, flagged**: the B7 extension of the routine-half paragraph (above). It does
  not contradict any finding; it adds a measured fact to a sentence that was false without it.
* **Untouched, per the brief**: `docs/superpowers/records/`, the surface plan, code, corpus
  programs; no gate reading anywhere.
