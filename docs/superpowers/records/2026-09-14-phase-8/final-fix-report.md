# Phase 8 L2 slice, final-review fix round: report

Brief: `final-fix-brief.md`. Start: HEAD `e64202ae7`. Scratch:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/final-fix/`.

Each section: reproduction (both sides), witness, control prediction (written before the run) and
result, commit, and anything not done.

## Pre-flight

Questions sent before implementing; rulings received (controller, 2026-09-14):
* Q1 (F5, a version-refused library): measure on a forged package in scratch, model what the
  oracle shows, witness at the Rust seam in unit tests.
* Q2 (F7): `pub unsafe fn value_of`, the one read moved into `load.rs`, `as_union` zeroing,
  read-back test into `ffi.rs`, `compile_fail` doctest whose failure reason is checked.
* Q3 (F8): a lifetime-carrying pointer wrapper built by `Contexts::method` from the whole `Owned`;
  no public constructor from a bare `&mut RexxMethodContext_`; no `unsafe` in `tests/`.
* Brief path correction: the sourceline companions are `rust/crates/rexx-parse/tests/sourceline_oracle/`.

Before-side binary for every reproduction below: `final-b/target-head/release/rexx-run` (built by
slice B from `e64202ae7`, which is still HEAD at the start). Runner: `final-fix/cmp.sh DIR FILE BIN
[EXTRA_LD_DIR]`, oracle under the standard wrapper, three descriptors to separate files.

## F1 -- native string arguments use the wrong protocol (A2 / B1)

**Reproduction (run)**, `final-fix/f1-repro-1/` (`main.rex` sends, `re.cls` declares `doparse`
= `RegExp_Parse`, CSTRING, and `does` = `RegExp_Match`, RexxStringObject), each case under
`signal on syntax` printing `condition('O')~code`, then one untrapped `r~does(.Raiser~new)`:

```
case                          oracle              before (head)
parse .object~new             88.909              parse object 0
parse STRING, no MAKESTRING   88.909              parse string only 0
parse MAKESTRING answers      makestring ran / 0  makestring ran / 0
parse .nil                    88.909              parse nil 0
parse MAKESTRING raises       40.1                88.909
match .object~new             88.909              match object 0
match STRING, no MAKESTRING   88.909              match string only 0
match MAKESTRING answers      makestring ran / 1  makestring ran / 0   (pattern was re-parsed from "The NIL object" one step earlier)
match .nil                    88.909              match nil 1
match MAKESTRING raises       40.1                88.909
parse .array~of('a*b')        0                   0
match .array~of('aab')        1                   1
untrapped raiser              rc 216, Error 40 running main.rex line 46 / 40.1   rc 168, Error 88 ... line 46 / 88.909
```
The traceback above the untrapped `Error` lines (`46 *-* raise syntax 40.1`, `REQUEST` with scope
`Object`, `DOES` with scope `RE`, `25 *-* say r~does(.Raiser~new)`) is identical on both sides.

## F2 -- 88.909 / 93.968 at the boundary are not lineless (B2)

**Reproduction (run).** 88.909, `final-fix/f2-p2e-1/` (a copy of B's `p2e`, NOSTRING trapped so
the before-side reaches the raise): oracle `Error 88 running .../re.cls:  Invalid argument.`,
before `Error 88 running .../main.rex line 3:`; traceback and `88.909` line identical, rc 168 both.

93.968 through slice A's `libforge.so`, two files (`f.cls` declares, `main.rex` sends), untrapped:
* argument side, `o~unknown(1)` (code 9 with an argument; `processArguments`, before
  `trapErrors`): oracle rc 163 `Error 93 running .../f.cls:` with **no line**; before rc 163
  `Error 93 running .../main.rex line 3:`. `final-fix/f2-sig-arg-1/`.
* result side, `o~retopt` (return word `OPTIONAL|int`; `valueToObject` inside the `try` that
  `trapErrors = true` opens at `NativeActivation.cpp:1301-1302`): oracle rc 163 `Error 93 running
  .../main.rex line 3:`, **with a line, against the sender**; before rc 0 prints `7` (A6, F9).
  `final-fix/f2-sig-ret-1/`.
* So the two sides' deliveries differ and the constructor is split.
* (for F9) missing argument to code 9, `o~unknown`: oracle rc 168 `Error 88 running .../f.cls:`
  no line, `88.901`; before rc 163, 93.968 against `main.rex line 3`. `final-fix/f2-sig-argmissing-1/`.

## F3 -- a load failure in a required package names the running program (B6)

## F4 -- `Method~package` of a library-backed method answers `REXX` (B5)

## F5 -- a failed library load is held as a miss (B3)

## F6 -- `LD_LIBRARY_PATH` read on every resolve (B4)

## F7 -- `ffi::value_of` can read uninitialised union bytes from safe code (A3)

## F8 -- `owner_of` reads outside the reference it derives from (A4)

## F9 -- two forged-signature divergences (A5, A6)

## F10 -- `a_library_named_twice_is_opened_once` cannot fail (B10)

## F11 -- instruments that run nothing (B8, B11)

(Sections below are appended in the order the work ran; the headings above hold the reproductions
written before the fixes.)

(F1, continued)

**Fix.** `Host::string_value` answers `Result<Option<ObjRef>, values::Raised>`; `values::Failure`
gains `Raised`. The interpreter's impl is `Interp::blamed_string_conversion` (the crate's existing
`requiredString()`, the one `required_string_argument` uses, with the `REQUEST` traceback line on a
raise), keeps a raised `Failure` on its `NativeFrame`, and `run_library_method` raises that.

**After (run)**, `final-fix/f1-after-1/` on `rust/target/release/rexx-run`: stdout, stderr, rc
SAME on all three (rc 216). B's `p4` (EXIT inside MAKESTRING), `p6`, `p2d`, `n1` re-run: SAME on
all three descriptors each.

**Witness.** `corpus/lang/library_string_argument.rex` + `.env` + `.d/re.cls` (methods declared in
the required package), row in `phase-8.txt`, companion in `rexx-parse/tests/sourceline_oracle/`
regenerated from a scratch copy (`final-fix/srcgen/gen.sh`, count 48, lines identical to the file).
Spawned: oracle vs after SAME x3; oracle vs before DIFF x3. Contract test in `rexx-api/tests/values.rs`:
`a_raise_inside_the_string_conversion_is_not_a_missing_string_value`.

**Control predictions (written before the runs).**
* M-D (`string_value` answers `Ok(Some(object))`, no conversion), spawned `rexx-run` on the witness:
  steps 1, 2, 4 still print `syntax 88.909` (the bytes read finds no string: agrees); step 3 loses
  `makestring ran` and prints `step 3 syntax 88.909` (DIFF); step 5 prints `syntax 88.909` where the
  oracle prints `40.1` (DIFF); steps 6-10 go through `RexxStringObject`, which registers the
  unconverted object, so `StringData` answers NULL and `StringLength` 0 into `RegExp_Match`: step 6
  either prints `match object <n>` or the process dies (not determinable by reading
  `automaton::match`); if it survives, step 8 loses `makestring ran`; the untrapped tail cannot
  print `Error 40`. Net: stdout DIFF, stderr DIFF, rc DIFF.
* The same witness on the before binary: DIFF x3 (already run above, so not a prediction).

**Control results (run).** M-D built with `--profile mutation` (worktree edit restored from a copy,
`cmp` clean; binary sha256 prefix `a37f4602cd49cfdf`), `final-fix/f1-MD-1/`: oracle rc 216, M-D rc
0; stdout DIFF, stderr DIFF (M-D prints nothing), rc DIFF. Steps 1, 2, 4 print `syntax 88.909` as
predicted; step 3 `syntax 88.909` with no `makestring ran`; step 5 `88.909`; step 6 survived and
printed `match object 0`; step 8 `match makestring 0` with no `makestring ran`; the untrapped tail
printed `0`. Confirmed. Not in the prediction: steps 11-12 (the array rows) also moved under M-D
(`step 11 syntax 88.909`, `match array 0`), since the unconverted array has no bytes either.

(F5, measurement for the Q1 ruling, run before any F5 code)

Forged package `final-fix/ext/libforgever.so`, built from `final-fix/ext/forgever.cpp` with
`g++ -shared -fPIC -std=gnu++11 -I api -I api/platform/unix` from the repository root:
`requiredVersion` 0x00060000, one method `Forgever_Seven` answering 7, one typed routine
`ForgeverRoutine` answering 8. Oracle and before-binary, `LD_LIBRARY_PATH=<oracle lib>:final-fix/ext`:

```
f5-ver-1 (loadLibrary trapped, then asks again)       oracle                 before
  first  .context~package~loadLibrary('forgever')     raised 98.982          first 0 (no raise)
  second .context~package~loadLibrary('forgever')     second 1               second 0
  .Method~loadExternalMethod(.. Forgever_Seven)       method 1               method 0
  .Routine~loadExternalRoutine(.. ForgeverRoutine)    routine 0              routine 0
f5-ver-first-* (the first ask is loadExternal*)
  loadExternalMethod first                            raised 98.982          first 0
  loadExternalRoutine first                           raised 98.982          first 0
  then loadLibrary                                    second 1               second 0
f5-ver-req-2 (after the trapped first ask, loadPackage('pk.cls'): ::requires 'forgever' LIBRARY, ::method seven external)
  loaded pk.cls / .K~new~seven                        loaded pk.cls / k 7    (never got there: first 0 fell through)
  .context~package~findRoutine('FORGEVERROUTINE')     The NIL object
  ForgeverRoutine()                                   43.1
  loadPackage('pk2.cls') (::routine fr external ...)  90.999
f5-ver-req-only-1: ::requires 'forgever' LIBRARY in the program, first ask   rc 158 98.982, SAME x3
```
So the oracle raises 98.982 on the first ask at every site, keeps the library (the raise is
between `packages->put` and `packages->remove`, `PackageManager.cpp:238-244`), answers it as
loaded on every later ask with its method table resolving, and never registers its routines
(`loadRoutines` follows the check, `LibraryPackage.cpp:232-237`).

(F3, reproduction) B's `a1`, `a2`, `a3`, `a5` transcripts re-read from `final-b/p/`: oracle names
`pk.cls line 3`, before names `main.rex line 3`, rc and the other lines identical (98.903 rc 158;
90.998, 90.999, 90.998 `GETNoSuchGet` rc 166). Re-run incidentally in `final-fix/f4-pkg-1/`
(`::attribute at external "LIBRARY rxregexp RegExp_Pos"` on `pk.cls` line 4): oracle `pk.cls line
4`, before `main.rex line 4`, both rc 166 and 90.998 `GETRegExp_Pos`.

(F4, measurement, run) What `~package` answers, oracle vs before, last path component printed:
```
f4-pkg-2 (pk.cls declares; main sends)                      oracle      before
  .Re~method('DOES') (::method does external LIBRARY rxregexp) pk.cls    REXX
  .Re~method('SEP')  (::method sep external LIBRARY REXX)       pk.cls    REXX
  main: loadExternalMethod('m','LIBRARY rxregexp RegExp_Match') pk.cls    REXX   (pk.cls bound RegExp_Match first)
  main: loadExternalMethod('m','LIBRARY REXX file_separator')  REXX      REXX
  main: loadExternalRoutine('r','LIBRARY rxmath RxCalcSqrt')   .nil (97.1 on ~name)  REXX
f4-pkg-3
  loadExternalMethod RegExp_Pos, nothing binds it              nil       REXX
  loadExternalMethod RegExp_Parse, before pk.cls binds it      nil       REXX
  the same object after loadPackage('pk.cls') binds it         pk.cls    REXX
  pk2.cls's own binding of RegExp_Parse                        pk2.cls   REXX
  loadExternalMethod RegExp_Parse after both                   pk.cls    REXX
  loadExternalRoutine RxCalcSqrt, nothing binds it             nil       REXX
f4-pkg-4
  loadExternalRoutine before / after pk.cls's ::routine sq external binds it   nil / pk.cls   REXX / REXX
  pk.cls's routine object                                      pk.cls    pk.cls
  loadExternalRoutine after binding                            pk.cls    REXX
  loadExternalMethod RegExp_Match, then pk3.cls binds REGEXP_MATCH: old / new spelling   nil / pk3.cls   REXX / REXX
```
Mechanism, read: `LibraryPackage::resolveMethod` (`LibraryPackage.cpp:374-400`) caches one
`NativeMethod` per procedure spelling; a directive's `setPackageObject` sets the package in place
while it is null and copies otherwise (`NativeCode.cpp:130-140`); `loadExternalMethod` answers
`new MethodClass(name, nmethod)` over the cached code (`MethodClass.cpp:588-595`). The brief's
premise (the sender's package) is not what the oracle answers; reported to the controller.

(F5, loader and unloader, read) `LibraryPackage::loadPackage` runs `loadRoutines` and then the
package `loader` after the version check (`LibraryPackage.cpp:232-246`), so a refused package's
loader never runs. `PackageManager::unload` (`PackageManager.cpp:642-650`, reached from
`Interpreter.cpp:281`) calls `unload()` on every package in the table, and `LibraryPackage::unload`
runs `package->unloader` when it is non-null (`LibraryPackage.cpp:166-175`), so the oracle does run
a version-refused package's unloader at termination. The crate calls neither `loader` nor
`unloader` for any library (`rexx-api/src/load.rs` reads the entry's name, version and two tables
only), so a held refused library cannot run its loader on a later ask here; the unloader half is a
divergence for any library with an unloader, refused or not, recorded here as read, not run.

(F4, record key, run) `final-fix/f4-key-1/`, `LD_LIBRARY_PATH=<oracle lib>:final-fix/libs`, where
`libs/libzzregexp.so` is a byte copy of the oracle's `librxregexp.so`: `loadExternalMethod` of
`RegExp_Parse` under `LIBRARY rxregexp`, `LIBRARY zzregexp` and `library rxregexp` all answer a nil
package; after `pk.cls` binds `LIBRARY rxregexp RegExp_Parse` they answer `pk.cls nil pk.cls`. So
two library names are two records and the keyword's case is not part of the key. `loadLibrary` of
`RXREGEXP`, `librxregexp` and `rxregexp.so` each answer 0, so no second spelling reaches the same
file under a different name. Key: the library name byte for byte (the `Libraries` key), the
procedure spelling byte for byte, and method or routine.

(F3, neighbour, run, not fixed) `final-fix/f3-rexxentry-1/`: `pk.cls` line 3 `::method x external
"LIBRARY REXX nosuchentry"` is oracle `Error 90 running .../pk.cls line 3` / 90.998 rc 166, before
`.../main.rex line 3`. That is the `unresolved_external` arm of `install_directives`, which predates
this range and is the KNOWN GAPS class "a directive-time error in a required package names the
program"; the brief scopes F3 to `resolve_directive_library`, so it is left as it is.

**F1 gates (run on the uncommitted tree that was then committed unchanged).** `cargo fmt --all
--check` exit 0; `cargo clippy -j 4 --workspace --all-targets -- -D warnings` exit 0;
`cargo test -j 4 -p rexx-api --no-fail-fast` exit 0 (values 37 passed, the rest unchanged);
`cargo test -j 4 --release -p rexx-exec --no-fail-fast -- --test-threads=8` exit 101, 1481 passed /
5 failed, the failing set being exactly G3's at `d1796f69c` (the three `ir::drive` counter tests,
`a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop`,
`the_l0_subset_passes_again_under_collect_on_every_allocation`);
`REXX_CORPUS_GATE=1 cargo test -j 4 --release -p rexx-exec --test corpus` exit 0, `532 of 532
matching`; `cargo test --release -p rexx-parse --test sourceline_oracle` exit 0.
Outputs: `final-fix/f1-{exec,corpus,srcline}.txt`, statuses `final-fix/f1-status.txt`.

**F1 commit: `04a286913`.**

(F2, continued)

**Design.** A lineless boundary constructor `native_argument_needs_a_string_value` beside the shared
`argument_needs_a_string_value`, which keeps its line: measured, oracle `say 'abc'~hasMethod(.nil)`
is `Error 88 running .../main.rex line 1` rc 168 (`final-fix/p-hasmethod-*/`). 93.968 is split by
the measurement above: `values::Failure::Signature` (parameters, `processArguments`) maps to a
lineless `incorrect_method_signature`, and a new `values::Failure::ResultSignature` (the declared
return type, `valueToObject`) maps to `incorrect_method_result_signature`, which keeps the line.

**Witness.** `corpus/lang/library_method_no_string_value.rex` + `.env` + `.d/re.cls`:
`r~does(.object~new)` untrapped, `does` declared in the required package. Unit test on the
delivery each refusal maps to, in `dispatch/library.rs`.

**Control predictions (written before the runs).**
* The witness on the F1 binary (`04a286913`): stdout SAME (empty), rc SAME (168), stderr DIFF on
  the `Error 88 running` line only. (Already seen on the draft, `final-fix/f2-witness-draft-1/`,
  so this one is not a prediction.)
* M-F2 (delete `lineless = true` from `native_argument_needs_a_string_value`), in-process corpus
  gate: `library_method_no_string_value` is the only phase-8 mismatch (stderr), because every other
  88.909 in the corpus is trapped (`library_string_argument` prints `condition('O')~code`, which
  delivery does not reach) or comes from the shared constructor; the delivery unit test for
  `NoStringValue` reddens.
* After the fix, through `libforge.so`: `f2-sig-arg` (`o~unknown(1)`) SAME x3; `f2-sig-ret`
  (`o~retopt`) still DIFF, printing 7, until F9 makes that return word a refusal.

**After (run)** on `rust/target/release/rexx-run` with the F2 change: the witness
(`final-fix/f2-after-witness/`) SAME x3, rc 168; `f2-after-sig-arg` (`o~unknown(1)` through
`libforge.so`) SAME x3, rc 163, `Error 93 running .../f.cls:`; `f2-after-sig-ret` (`o~retopt`)
still DIFF, rc 0 printing 7 against the oracle's rc 163 `.../main.rex line 3`. As predicted.

**Control results (run).** M-F2 (`lineless = false` in `native_argument_needs_a_string_value`,
restored from a copy, `cmp` clean): `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test
corpus` exit 101, `532 of 533 matching`, the one mismatch `lang/library_method_no_string_value.rex:
stderr differ` (the `running ... line` form); `cargo test -p rexx-exec --lib dispatch::library`
exit 101, `a_refusal_before_the_call_is_lineless_and_one_after_it_is_not` failed at
`library.rs:502`, the other 8 passed. Confirmed exactly. Outputs `final-fix/f2-MF2-{corpus,lib}.txt`.

**Refusal-site table.** `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites
-- --test-threads=1` re-derived columns 1-4 (line numbers moved; `argument_needs_a_string_value`'s
surface is now `body+send`, because the new constructor in `error.rs` calls it; its verdict carried).
Two new send rows measured by hand: `native_argument_needs_a_string_value` agrees/yes/88.909 on the
witness; `incorrect_method_signature` now agrees/yes/93.968 through `libforge.so`;
`incorrect_method_result_signature` not-run/no until F9 gives it a route. `SHARED_ANSWERS` gains the
new 88.909 row, and the table's header paragraph names the three-member group. Its "Measured" was
re-earned for the new pairs: transposing `answer` and `witness` of
`native_argument_needs_a_string_value` with `named_argument_needs_a_string_value`, and separately
with `argument_needs_a_string_value`, left all 5 `refusal_sites` tests green; the table was
restored from a copy each time (`cmp` clean).

(F3, continued)

**Witnesses** (drafted in `final-fix/f3-draft/`, each a two-file program whose `.d/pk.cls` line 3
is the directive): `library_required_method_library_missing` (`LIBRARY zorkolib z`, no `.env`: the
library is missing with or without the variable), `library_required_method_entry_missing`,
`library_required_routine_entry_missing`, `library_required_attribute_entry_missing` (each with the
`{oraclelib}` `.env`). On the F2 binary (`final-fix/bins/rexx-run-f2`), all four: stdout SAME
(empty), rc SAME (158, 166, 166, 166), stderr DIFF on the `Error N running` line only
(`final-fix/f3-before-*/`). Run, so not a prediction.

**Control predictions (written before the fix).**
* After the fix, the four witnesses SAME x3 on the spawned binary and in the in-process gate.
* The negative control is the F2 binary itself, whose `resolve_directive_library` is the mutant
  (both sites calling `blame_directive`): already DIFF on all four, above. The existing one-file
  witnesses (`library_{method,routine,attribute}_entry_missing`, `library_requires_missing`) stay
  green on both binaries, because a program's own directive names the program either way.

**F2 gates (run on the tree then committed unchanged).** fmt exit 0; clippy exit 0; `rexx-api`
tests exit 0; `cargo test --release -p rexx-exec --no-fail-fast` exit 101, 1482 passed / 5 failed,
the same five as G3; gated corpus exit 0, `533 of 533 matching`; sourceline exit 0.
Outputs `final-fix/f2-*.txt`.

**F2 commit: `03ceb04df`.**

(F3, results; resumed after an API session limit, state checked against `git status`/`git diff`
first: the code change, the four witnesses, their sidecars and companions were all in place and
match what is described here)

**Fix.** `resolve_directive_library` takes the `ProgramId` from `install_directives` and both of its
failure sites call `blame_directive_in(id, ...)`, the helper the `::REQUIRES ... LIBRARY` arm uses.

**After (run)**, spawned `rust/target/release/rexx-run` with the fix (`final-fix/f3-after-*/`): the
four new witnesses SAME x3 (rc 158, 166, 166, 166); the four one-file witnesses
(`library_{method,routine,attribute}_entry_missing`, `library_requires_missing`) SAME x3. Control:
the same four one-file witnesses on the F2 binary SAME x3 (`final-fix/f3-ctl-f2bin-*/`), and the
four new ones DIFF on it (above). Confirmed as predicted.

**F3 gates (run on the tree then committed unchanged).** `cargo fmt --all --check` exit 0; clippy
exit 0; `cargo test --release -p rexx-exec --no-fail-fast` exit 101, 1482 passed / 5 failed, the G3
five; gated corpus exit 0, `537 of 537 matching`; sourceline exit 0. `final-fix/f3-*.txt`.

**F3 commit: `40093e99b`.**

(F4, continued; the controller ruled (a) and (b) both in this round, the record keyed as measured)

**Witness** (drafted in `final-fix/f4-draft/`): `library_method_package.rex` + `.d/pk.cls`
(`::method doparse external "LIBRARY rxregexp RegExp_Parse"`, `::method sep external "LIBRARY REXX
file_separator"`, `::routine sq public external "LIBRARY rxmath RxCalcSqrt"`) + `.d/pk2.cls` (a
second binding of `RegExp_Parse`), loaded at run time with `loadPackage`, printing the last path
component of every `~package`, including the retroactive row (the same `loadExternalMethod` object
before and after `pk.cls` binds it). On the F3-era binary (`final-fix/bins/rexx-run-f2`, same
package reader): stdout DIFF on 12 of 13 lines (only `pk.cls routine` agrees), stderr SAME (empty),
rc SAME 0 (`final-fix/f4-before-witness/`). Run, not a prediction.

**Fix, as built.** `ExecutableSource::External { program }` (installed `EXTERNAL`, from
`external_packages`, so `LIBRARY REXX` directives too) and `ExecutableSource::Loaded { code }`
(`loadExternal*` over a non-REXX library), both reading as `Native` for source, flags and security
manager. `Interp::library_codes` / `library_code_rows`: one `Option<ProgramId>` per
`LibraryCodeKey { library, procedure, routine }`; `install_directives` collects what
`resolve_directive_library` bound over the package's first walk and, once every directive of the
package resolved, sets each unset row to that package. `Interp::source_package` is the one reader
(`~package`, `executable_package`, `make_method_private`); an unbound `Loaded` row is `.nil` for
`~package`.

**Control predictions (written before running the fixed binary).**
* The witness on the fixed binary: SAME x3.
* M-F4a (`installed_executable_source` never answers `External`): the `pk.cls method`,
  `pk.cls LIBRARY REXX method` and `pk2.cls method` lines go back to `REXX`; every other line
  unchanged; stderr and rc SAME.
* M-F4b (the bind loop after the first walk removed): `the same object after pk.cls binds it`,
  `the same routine after pk.cls binds it`, `loaded after both` and `routine loaded after binding`
  print `nil`; the three directive rows and the unbound rows stay right.
* B's `t7` (PACKAGE/PRIVATE access on library methods from a required package, and `setPrivate`)
  SAME x3 on the fixed binary, since `make_method_private` now records the declaring package where
  it recorded `REXX` and the access check for PRIVATE is by scope.

**Results (run).** Fixed binary (`final-fix/bins/rexx-run-f4`): the witness SAME x3; B's `t7`,
`t5b`, `t5c` SAME x3; the measurement probes `f4-pkg-2`, `f4-pkg-3`, `f4-pkg-4`, `f4-key-1` re-run
SAME x3 each (`final-fix/*-after/`). M-F4a (sha256 prefix `24d0547fb94ea880`): exactly the three
directive lines `REXX`, stderr and rc SAME. M-F4b (`6a59da1028dc7b77`): exactly the four
retroactive and after-binding lines `nil`. Both confirmed as predicted; worktree restored from a
copy after each (`cmp` clean).

(F5, more measurement, run) Untrapped first asks of the forged version-refused library
(`final-fix/f5-ver-untrapped-*/`), oracle rc 158 each, before (F4 binary) rc 0 each:
`loadLibrary('forgever')` is `Compiled method "LOADLIBRARY" with scope "Package".` over
`2 *-* say ...` and `Error 98 running .../main.rex line 2` / 98.982 (before prints `0`);
`loadExternalMethod` the same with `"LOADEXTERNALMETHOD" with scope "Method"` (before prints
`The NIL object`); `loadExternalRoutine` with `"LOADEXTERNALROUTINE" with scope "Routine"` (before
`The NIL object`).

Witness draft for the retry, `final-fix/f5-draft/library_load_retried.rex`: `loadLibrary('yyregexp')`,
then `address system 'cp'` of the oracle's `librxregexp.so` to `libyyregexp.so` in the run directory
(on `LD_LIBRARY_PATH`), then ask again. Each side in its own directory (`final-fix/cmp2.sh`),
`final-fix/f5-before-witness-2/`: oracle `first 0` / `second 1`, before `first 0` / `second 0`, rc 0
and stderr SAME. (A first attempt ran both sides in one directory, so the before-side's first ask
loaded the oracle's leftover copy and the `cp` then overwrote a mapped library: rc 139. That was the
runner, not a finding; the corpus harness empties the directory between sides.)

**F4 gates (run on the tree then committed unchanged).** fmt exit 0; clippy exit 0;
`cargo test --release -p rexx-exec --no-fail-fast` exit 101, 1482 passed / 5 failed, the G3 five;
gated corpus exit 0, `538 of 538 matching`; sourceline exit 0. `final-fix/f4-*.txt`.
Not reached: a `newFile`/`Package~new` context argument that is a `loadExternal*` object whose
package is still `.nil` answers here as a value that is none of the accepted classes (the
`executable_package` callers' existing error path); the oracle's answer for that shape was not
measured.

**F4 commit: `ca79611e5`.**

(F5, fix and results)

**Fix.** `rexx_api::load::open`/`open_path` answer `Err(Refused { failure, library })` for a
version refusal, the library carrying its method table and an unread (empty) routine table.
`Libraries` holds only `Rc<Library>` (`get`, non-replacing `hold`, private map, unchanged module).
`Interp::resolve_library` returns a held library as `Loaded` and otherwise hands `load::open`'s
answer to `Interp::settle_library`, which holds a loaded library, holds a refused one and answers
`Version`, and holds no miss. `Package~loadLibrary` and `loadExternalMethod`/`loadExternalRoutine`
raise 98.982 on `Version` where they answered 0 / `.nil`. The `Interp::libraries` comment now
states the oracle's rule with `PackageManager.cpp:229-248`.

**After (run)**, `final-fix/bins/rexx-run-f5`: every forged-package probe re-run SAME x3
(`f5-ver-1`, both `f5-ver-first-*`, `f5-ver-req-2` including `k 7`, `routine The NIL object`,
`call raised 43.1`, `pk2 raised 90.999`, `f5-ver-req-only-1`, the three `f5-ver-untrapped-*` at rc
158); the retry witness `library_load_retried.rex` SAME x3 through `cmp2.sh`
(`final-fix/f5-after-witness/`). The before binary on the same witness is DIFF (`second 0`), above.

**Witnesses.** Corpus `library_load_retried.rex` + `.env` (`LD_LIBRARY_PATH={oraclelib}:{run}`,
`RETRY_FROM`, `RETRY_TO`). Unit tests in `dispatch/library.rs`: `a_name_that_resolves_to_nothing_is_not_held`
(replaces `a_name_that_resolves_to_nothing_is_held_as_a_miss`), `a_version_refused_library_raises_once_and_is_held`
(the version answer faked at `settle_library` over a real `librxregexp.so`, then a
`resolve_library` of a name on no search path answering `Loaded`), and
`a_held_library_is_not_replaced_by_a_later_answer` rewritten over two opened libraries. The empty
routine table of a refused library is witnessed only by the scratch forged probes (no forged
library is committed or built by a test).

**Control predictions (written before the runs).**
* M-F5a (`settle_library` does not hold a refused library): `a_version_refused_library_raises_once_and_is_held`
  panics ("the refused library was not held"); on the forged probes `f5-ver-1` prints `second raised
  98.982` and stops (DIFF stdout), `f5-ver-req-2` raises 98.982 at `pk.cls`'s `::requires` (DIFF);
  the untrapped first-ask probes stay SAME.
* M-F5b (`Package~loadLibrary` maps `Version` to 0 again): `f5-ver-untrapped-*` for `loadLibrary`
  prints `0` and rc 0 (DIFF x3); the `loadExternal*` untrapped probes stay SAME; the unit tests stay
  green (they do not go through `loadLibrary`), which is a gap, recorded rather than closed, since
  a corpus witness needs a forged library.

**Control results (run).** M-F5a (sha256 prefix `3c2e620c1eaa56be`): `cargo test -p rexx-exec --lib
dispatch::library` exit 101, only `a_version_refused_library_raises_once_and_is_held` failed
(`library.rs:373`, the "not held" panic); `f5-ver-1` stdout DIFF, printing `second raised 98.982`;
`f5-ver-req-2` stdout/stderr/rc DIFF (rc 158, the untrapped `loadPackage` raising 98.982); the two
untrapped first-ask probes SAME. M-F5b (`f5cee0658edc7a56`): unit tests exit 0, 10 passed, as
predicted; `f5-ver-untrapped` for `loadLibrary` DIFF x3 (rc 0), the `loadExternalMethod` and
`loadExternalRoutine` ones SAME. Both confirmed; restored from copies, `cmp` clean.

Clippy then refused the unboxed `Refused` (`result_large_err`, 128 bytes), so `Refused::library` is
a `Box<Library>` (`Rc::from` of it where it is held). The release binary was rebuilt after that
change (`final-fix/bins/rexx-run-f5b`) and every forged probe plus the retry witness re-run: SAME x3
each (`final-fix/*-after2/`, `f5-after2-witness/`). `refusal-sites.tsv` re-derived:
`library_version` moved `body` -> `body+send` (the two dispatch sites now raise it) and was measured
by hand: agrees/yes/98.982 through the forged package (`f5-ver-untrapped-*`).

(F6, reproduction, run while the F5 gates ran)

Witness draft `final-fix/f6-draft/library_search_path_fixed.rex`: `mkdir` a subdirectory of the run
directory, `cp` the oracle's `librxregexp.so` into it as `libzzregexp.so`, `call value
'LD_LIBRARY_PATH', <that directory>, 'ENVIRONMENT'`, then `loadLibrary('zzregexp')` and
`loadExternalMethod(... 'LIBRARY zzregexp RegExp_Parse')`. `final-fix/cmp3.sh` (each side in its
own directory, `LD_LIBRARY_PATH=<oracle lib>` only), `final-fix/f6-before-witness-3/`: oracle
`loadLibrary 0` / `loadExternalMethod 0`, before (F5 binary) `1` / `1`, stderr and rc SAME.

**Found on the way, recorded and not fixed.** A first draft copied the library into the run
directory itself and the oracle answered `1` there. `readelf -d` gives both `build/bin/rexx` and
`build/lib/librexx.so.4` `RUNPATH [/home/moritz/dev/repos/ooRexx/build/lib:]`; the empty element
after the colon is the working directory, so the oracle's `dlopen` finds a library in the current
directory. Measured, `final-fix/f6-cwd-search-1/`: with `libzzregexp.so` in the working directory
and `LD_LIBRARY_PATH=<oracle lib>` only, oracle `cwd copy 1`, crate `cwd copy 0`. A property of the
oracle's build configuration rather than of `SysLibrary::load`; not modelled, and the witnesses
here keep every copied library out of the working directory except `library_load_retried`, whose
run directory is on `LD_LIBRARY_PATH` on both sides anyway.

**F5 gates (run on the tree then committed unchanged).** fmt exit 0; clippy exit 0 (after the
`Box`); `rexx-api` tests exit 0; `cargo test --release -p rexx-exec --no-fail-fast` exit 101, 1483
passed / 5 failed, the G3 five; gated corpus exit 0, `539 of 539 matching`; sourceline exit 0.
`final-fix/f5-*.txt`.

**F5 commit: `bab390715`.**

(F6, fix)

**Fix.** `Interp::library_search` is taken from the environment when the interpreter is built
(`Interp::new`, from the process as before) and again when an embedding hands one in
(`Interp::adopt_environment`, which `run_program` now calls instead of assigning `env`);
`resolve_library` reads that field and nothing reads `LD_LIBRARY_PATH` per resolve. No process
variable is read or written anywhere new.

**Witnesses.** Corpus `library_search_path_fixed.rex` + `.env` (`LD_LIBRARY_PATH={oraclelib}`,
`WIDEN_FROM`, `WIDEN_TO={run}/widened`); unit test
`a_search_directory_written_after_the_start_is_not_searched` (an interpreter started with no
`LD_LIBRARY_PATH` and then `env_set` to the oracle's directory answers `Missing`; the control started
with it answers `Loaded`). The two existing tests that set the variable after `Interp::new` now
hand it in through `adopt_environment`.

**Control predictions (written before the runs).**
* The witness on the fixed binary: SAME x3 (`loadLibrary 0`, `loadExternalMethod 0`).
* The F5 binary is the unfixed resolver: DIFF on stdout (`1` / `1`), already run above.
* M-F6 (`resolve_library` searches `library_search_of(&self.env)` again, so the live value):
  the unit test fails on its first assertion (`Loaded` where `Missing` is asserted); the gated
  corpus reports exactly `lang/library_search_path_fixed.rex` differing on stdout.

**Results (run).** Fixed binary (`final-fix/bins/rexx-run-f6`), `final-fix/f6-after-witness/`: SAME
x3. M-F6: `cargo test -p rexx-exec --lib dispatch::library` exit 101, only
`a_search_directory_written_after_the_start_is_not_searched` failed (`library.rs:355`, its first
assertion); `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` exit 101, `539 of
540 matching`, `lang/library_search_path_fixed.rex: stdout differ` the only mismatch. Confirmed;
restored from a copy, `cmp` clean. `final-fix/f6-MF6-*.txt`.

(F7, measurement before the fix, run) Does rustdoc check a `compile_fail` error code on this
machine's stable toolchain? `final-fix/doctest-probe/`, a scratch crate with two `pub unsafe fn`s
each called without `unsafe` in a doctest, one marked `compile_fail,E0133` (right) and one
`compile_fail,E0425` (wrong): `cargo +stable test --doc` (rustc 1.98.1) passes both, so stable does
not check the code; `cargo +nightly test --doc` (rustc 1.100.0-nightly 2026-09-08) fails the E0425 one
with `Some expected error codes were not found: ["E0425"]`. So the committed doctest's reason is
checked by running it once on nightly and by the stable counter-check (drop `unsafe` from
`value_of` and see the doctest fail to fail).

(F8, Miri) `rustup component add miri --toolchain nightly` fails as slice A found ("cleaning up
cached downloads: Read-only file system"). A toolchain install into a scratch `RUSTUP_HOME` was
started (`final-fix/miri-install.txt`); the result is recorded under F8.

(F8, Miri reproduction, run) The scratch install worked: `RUSTUP_HOME=final-fix/rustup-home rustup
toolchain install nightly --profile minimal --component miri,rust-src` gave `miri 0.1.0 (4b6d04e706
2026-09-13)` (the command's last line is an unrelated `rustup is not installed at <CARGO_HOME>`
error from the scratch `CARGO_HOME`; the toolchain and `cargo-miri` are in place). Run over a `git
archive` copy of HEAD `bab390715` in `final-fix/miri-tree/` (no worktree file involved),
`CARGO_TARGET_DIR=final-fix/miri-target`, `cargo miri test -p rexx-api --lib --offline`:
* the existing `ffi::tests::a_context_we_handed_out_recovers_its_owner`: `Undefined Behavior:
  attempting a read access using <150391> at alloc54815[0x18], but that tag does not exist in the
  borrow stack for this location` at `ffi.rs:44` (`(*owned).owner`), the tag "created by a
  SharedReadWrite retag at offsets [0x0..0x18]" at `&raw mut wrapper.context`
  (`final-fix/miri-before.txt`);
* a new test on the real path, `invoke::tests::a_stub_reaches_its_activation_through_the_context_it_was_handed`
  (a `#[cfg(test)]` stub in `ffi.rs` that calls `DropObjectVariable` through the context
  `Contexts::method` handed out): the same report at `ffi.rs:44`, the tag created at
  `load.rs:167` `&raw mut *context`, backtrace `owner_of` <- `activation_of` <-
  `drop_object_variable` <- the stub <- `NativeMethodEntry::call` <- `invoke::method`
  (`final-fix/miri-before-stub.txt`).
So finding A4 is confirmed under Stacked Borrows, and the new test is the witness.

(F7, built first in the scratch copy `final-fix/miri-tree/` while the F6 gates held the worktree)

**Fix.** `Value::as_union` starts from `ValueUnion { value_int64_t: 0 }` and writes the member into
it. `ffi::value_of` is `pub unsafe fn` with a `# Safety` section (the member's bytes are
initialised). Its one caller moved into `load.rs`: `NativeMethodEntry::call` takes the declared
result's `Repr`, writes element zero's word as a full zero before the stub runs, and answers the
value read under a local `SAFETY` note; `invoke::method` converts what `call` answered. The
read-back test moved from `tests/invoke.rs` into `ffi.rs`'s unit tests (with its `sample`), beside
a new `a_narrow_value_is_written_over_a_zeroed_word`. The compile-level witness is a
`compile_fail,E0133` doctest on `value_of` over A's union-literal probe.

In scratch: stable `cargo test -p rexx-api` all green (lib 13, doctests 3); `cargo miri test -p
rexx-api --lib -- every_repr_reads_back a_narrow_value` exit 0, both ok (`final-fix/miri-f7.txt`).

**Control predictions (written before the runs).**
* M-F7a (`as_union` back to the one-member literals): under Miri,
  `a_narrow_value_is_written_over_a_zeroed_word` reports Undefined Behavior (a read of
  uninitialised bytes in `value_of`); on stable it may pass or fail, since the upper bytes are
  whatever the stack held (not a reliable instrument there, recorded as such).
* M-F7b (`value_of` made safe again: `unsafe` dropped from the signature): the stable doctest run
  reports the `compile_fail` doctest as `Test compiled successfully, but it's marked compile_fail`.
* The doctest as committed, on nightly rustdoc (which checks the code): passes, so its compile
  error is E0133 and not an import or type error.

**Control results (run, scratch copy).** M-F7a (the `Uint8` arm back to `return ValueUnion {
value_uint8_t: v }`): Miri reports `Undefined Behavior: reading memory at alloc49107[0x0..0x8], but
memory is uninitialized at [0x1..0x8]` in `value_of` for `a_narrow_value_is_written_over_a_zeroed_word`
(`final-fix/miri-MF7a.txt`); the same test passes on stable in both debug and release, so on the
gate's toolchain that test cannot see the mutant: Miri is the only instrument that did. M-F7b
(`value_of` safe): stable `cargo test -p rexx-api --doc` fails `ffi::value_of (line 52)` with
`Test compiled successfully, but it's marked compile_fail`. The doctest as written on nightly
rustdoc, which checks the listed code: passes (`final-fix/f7-doc-nightly.txt`), so the failure is
E0133. All three as predicted; scratch files restored from copies (`cmp` clean).

(F8, built in the same scratch copy on top of F7)

**Fix.** `ffi::MethodContext<'a>`: a `*mut RexxMethodContext_` plus `PhantomData<&'a mut
RexxMethodContext_>`, with a `pub(crate) as_ptr` and a `#[cfg(test)] pub(crate) bare` constructor
for the in-crate tests whose tables never recover an owner; no public constructor from a bare
struct. `Contexts::method(&mut self) -> MethodContext<'_>` derives the pointer from `&raw mut
self.method`, the whole `Owned`. `invoke::method`, `invoke::signature`, `NativeMethodEntry::signature`
and `::call` take `&mut MethodContext<'_>`; `call` writes and clears `arguments` through the pointer
in `unsafe` blocks in `load.rs`. `owner_of`'s `# Safety` and `SAFETY:` now name provenance (derived
from the whole wrapper) rather than address. `ffi::tests::a_context_we_handed_out_recovers_its_owner`
now hands out `(&raw mut wrapper).cast()`. `tests/invoke.rs` runs every call through `Contexts`
(its C callback tables are gone; the recorders moved onto its `Host`), and `tests/context.rs`'s one
call site passes `&mut contexts.method()`. No `unsafe` in `tests/`.

In scratch: stable `cargo test -p rexx-api` all green; `cargo miri test -p rexx-api --lib` exit 0,
13 of 13 ok, including `a_context_we_handed_out_recovers_its_owner` and
`a_stub_reaches_its_activation_through_the_context_it_was_handed`, both UB before
(`final-fix/miri-f8-lib.txt`).

**Control prediction (written before the run).** M-F8 (`Contexts::method` hands out
`(&raw mut self.method.context)` again, the field rather than the wrapper): Miri reports the
Stacked Borrows read at `ffi.rs` `owner_of` for `a_stub_reaches_its_activation_through_the_context_it_was_handed`,
tag created at the field; every stable test stays green (Miri is the only instrument that sees it).

(F9, built in the same scratch copy on top of F8)

**Fix.** A5: `values::consumes_argument` answers `bool`, and a code the table does not know
consumes an argument (`processArguments`' `default:`, `NativeActivation.cpp:325-327`, `:672`);
`to_native` settles absence before looking the code up, so an unknown non-optional code with no
argument is `MissingArgument` (`:607-612`), with one `Signature`, optional and absent `Signature`.
`tests/values.rs`'s `a_code_the_table_does_not_know_is_a_signature_error` asserted the opposite
order as the rule and now asserts the oracle's three answers. A6: `from_native` reads the result word
unstripped (`:720`, `:855-858`), `invoke::method` looks the result up with a new
`values::result_repr` (unstripped) and stores the word as declared in element zero's `type`
(`:228`), so `OPTIONAL|int` runs the stub and then answers `ResultSignature`.

**Witnesses** (unit tests; no corpus witness, since the corpus loads only oracle-built extensions):
`invoke::tests::an_unknown_parameter_code_is_an_argument_position`,
`invoke::tests::a_result_word_carrying_the_optional_bit_is_refused_after_the_call`,
`tests/values.rs` `a_code_the_table_does_not_know_is_a_signature_error` (corrected) and
`a_result_word_carrying_the_optional_bit_is_a_signature_error`.

**Control predictions (written before the runs).**
* M-F9a (`to_native` looks the code up before the presence check again): `a_code_the_table_does_not_know_is_a_signature_error`
  fails on its first assertion and `an_unknown_parameter_code_is_an_argument_position` on its
  first; the other rexx-api tests stay green.
* M-F9b (`from_native` strips the optional bit again, and `result_repr` with it): both
  optional-bit tests fail; the rest stay green.
* The forged probes through a `rexx-run` built from the scratch copy: `a_table.rex` steps 1-5
  byte-identical to the oracle's (`step 1 syntax 88.901` ... `step 5 syntax 93.968`), and
  `f2-sig-ret` / `f2-sig-argmissing` SAME x3.

**F6 gates (run on the tree then committed unchanged).** fmt exit 0; clippy exit 0;
`cargo test --release -p rexx-exec --no-fail-fast` exit 101, 1484 passed / 5 failed, the G3 five;
gated corpus exit 0, `540 of 540 matching`; sourceline exit 0. `final-fix/f6-*.txt`.

**F6 commit: `49f3c5581`.**

(F9, results in the scratch copy) M-F9a: exactly `an_unknown_parameter_code_is_an_argument_position`
(`invoke.rs:574`) and `a_code_the_table_does_not_know_is_a_signature_error` (`tests/values.rs:418`)
failed, both on their first assertion. M-F9b: exactly the two optional-bit tests failed
(`invoke.rs:590`, `tests/values.rs:448`). A `rexx-run` built from the scratch copy (F7+F8+F9, with
`dispatch/library.rs` passing `&mut contexts.method()`): slice A's `a_table.rex` through `libforge.so`
SAME x3 on every line, including `step 1 syntax 88.901` and `step 5 syntax 93.968` (and F1's
`step 15 syntax 88.909`); `f2-sig-ret`, `f2-sig-argmissing`, `f2-sig-arg` SAME x3
(`final-fix/f9-*-2/`). All as predicted.

(F7, ported to the worktree) The F7 part was taken as a diff between two scratch snapshots and
applied to `rust/crates/rexx-api` with `patch -p1` (clean, one hunk offset), then `rustfmt`. On the
worktree: clippy exit 0; `cargo test -p rexx-api` exit 0 (lib 12, doctests 3); `unsafe_sites` 2
passed (the `unsafe` files are still `ffi.rs` and `load.rs`); Miri from the worktree sources
(`CARGO_TARGET_DIR` in scratch, `--locked`) runs the two F7 tests clean
(`final-fix/miri-f7-worktree.txt`; the pre-F8 owner test still reports its UB there, as expected
until F8); M-F7b repeated on the worktree file (restored from a copy, `cmp` clean): the doctest
fails with `Test compiled successfully, but it's marked compile_fail`.

(F11, drafted in a `git archive 49f3c5581` copy, `final-fix/f11-tree/`, while the F7 gates held the
worktree)

**Changes.** `corpus/lang/external_method_package_blame.env` removed. The sidecar reader moves out
of `tests/corpus.rs` into `tests/support/sidecar.rs` (`Sidecar`, a `Half` per removable part,
`sidecar_for`, `prepare_run_directory`, `resolved_environment`, and `invocation`, which is the
in-process environment layering `run_rust` did). `corpus.rs` uses it; its sidecar control now removes
one part at a time and requires each removal to move one of the two interpreters.
`collect_stress.rs`'s L0 loop and `ir_recorded.rs`'s corpus population run each program with its
sidecar (fixtures laid into the run directory, environment, standard input), so `phase-8.txt` stays
in their lists. `coverage.rs` only parses (`parse_program`), so it needs no sidecar and is unchanged.

**Control predictions (written before the runs).**
* The per-part control over the tree as changed: passes; every multi-part sidecar on disk
  (`external_search_order` fixtures/environment/CWD, `external_trace` environment/stdin, and the
  library witnesses' environment/fixtures) has every part live.
* The same control with `external_method_package_blame.env` put back: fails, naming
  `lang/external_method_package_blame.rex` and `Environment`.
* The old whole-sidecar control with that `.env` put back: passes (slice B's B8, re-measured).

**First result (run), prediction falsified.** The per-part control over the scratch tree (with
`external_method_package_blame.env` removed) failed, but not where expected: `lang/external_trace.rex:
neither interpreter answers differently without its Stdin` (`final-fix/f11-ctl-removed.txt`).
Measured directly on the oracle: `RXTRACE=ON rexx external_trace.rex` with the committed `.stdin`
(three empty lines and `trace off`), with `/dev/null`, and with stdin closed answer the same stdout
(`first ?R`, `second 2`, `after ?R`), rc 0, and stderr equal but for the run-directory path
(`final-fix/f11-external-trace-*/`): empty lines read like end of file, and `trace off` would be read
at the pause after the program's last clause. A second inert part, predating Phase 8, which the
whole-sidecar control could not see. Ruling asked of the controller (options: remove the `.stdin`,
make it load-bearing, exempt it, or drop the per-part control).

(F11, `external_trace.stdin`, ruling (b) with three conditions)

* **What the witness pins.** Added at `73aed8f25` (Close Phase 7): `RXTRACE=ON` was unimplemented
  and unrefused, and the witness shows the top-level program starting under `TRACE ?R` --
  `TRACE()` answering `?R` on the first clause, the banner, the clause echoes and the prompt,
  with the `.stdin` read by the pauses. The program comment adds "one line per traced clause".
* **`oracle-crashes.txt` entry 9** is `::OPTIONS TRACE ?<letter>` (a package option, an indefinite
  block with no console). The shape here is the `RXTRACE` environment variable with a program
  that carries no `::OPTIONS`, which the committed witness already runs to rc 0; every run below
  also had `timeout 10`.
* **Placement measured (run).** Candidate 1, a command at the pause after clause 10
  (`final-fix/f11-external-trace-candidate-{o,r}/`): both sides identical and every original line
  kept, but the pause then reads two lines, which would falsify "one line per traced clause".
  Candidate 2, the four lines `""`, `""`, `""`, `say 'typed at the last pause'`
  (`final-fix/f11-external-trace-candidate2-{o,r}/`): oracle and crate identical on stdout, stderr
  (modulo the run directory) and rc 0; stdout `first ?R` / `second 2` / `after ?R` /
  `typed at the last pause`, stderr the unchanged transcript, one line per traced clause (the pause
  after the command reads end of input). Chosen. The committed `trace off` line, which was read at
  the last pause and changed nothing observable, goes.

**F7 gates (run on the tree then committed unchanged).** fmt exit 0; clippy exit 0; `rexx-api`
tests exit 0; `cargo test --release -p rexx-exec --no-fail-fast` exit 101, 1484 passed / 5 failed,
the G3 five; gated corpus exit 0, `540 of 540 matching`; sourceline exit 0. `final-fix/f7-*.txt`.

(F8, ported to the worktree) The F8 files were copied from the scratch snapshot over the F7 commit
and formatted (`values.rs`, untouched by F8, came out byte-identical, so the formatting is the same
one). Clippy then refused `&mut MethodContext` parameters that are never used mutably
(`needless_pass_by_ref_mut`), so `invoke::method`, `invoke::signature`, `NativeMethodEntry::signature`
and `::call` take `&MethodContext<'_>`: the type holds the only borrow of the struct (a
`PhantomData<&'a mut>`), is neither `Clone` nor `Sync`, and `call`'s `SAFETY` note says so. On the
worktree: clippy exit 0; `rexx-api` tests exit 0 (lib 13, invoke 9, context 14, doctests 3);
`unsafe_sites` 2 passed; no `unsafe` block in `rexx-api/tests/` (the one hit is a comment). Miri from
the worktree sources: `cargo miri test -p rexx-api --lib` 13 of 13 ok under Stacked Borrows
(`final-fix/miri-f8-worktree.txt`) and 13 of 13 under `-Zmiri-tree-borrows`
(`final-fix/miri-f8-worktree-tb.txt`); M-F8 repeated on the worktree file (restored from a copy,
`cmp` clean): the same Stacked Borrows report for the stub test (`final-fix/miri-MF8-worktree.txt`),
stable tests green.

**Control predictions for F11 as changed (written before the runs, scratch tree with the new
`external_trace.stdin`).**
* Per-part sidecar control: passes.
* The same with `external_method_package_blame.env` put back: fails, naming
  `lang/external_method_package_blame.rex` and `Environment`.
* The same with the old `external_trace.stdin` put back: fails naming `external_trace` and `Stdin`
  (the first result above, repeated as the control).
* `external_trace.stdin` deleted: stdout differs from the run with it on both interpreters (the
  fourth line, `typed at the last pause`, is gone), stderr and rc the same; the gated differential
  itself stays green, since both sides lose the line together, which is why the per-part control
  is the instrument for it.
* Gated corpus differential over the scratch tree: every program agrees.

**Control results (run, scratch tree).** Per-part control with the new `external_trace.stdin` and
no `external_method_package_blame.env`: ok (`final-fix/f11-ctl-new.txt`). With that `.env` put back:
FAILED, `lang/external_method_package_blame.rex: neither interpreter answers differently without its
Environment` (`f11-ctl-envback.txt`). With the old `.stdin` put back: FAILED, `lang/external_trace.rex:
... without its Stdin` (`f11-ctl-oldstdin.txt`). `.stdin` deleted, spawned runs: the oracle's and the
crate's stdout each lose exactly the line `typed at the last pause` against their run with it;
the crate's stderr is the same with and without (modulo the run directory), rc 0 both
(`f11-external-trace-without{,-r}/` against `f11-external-trace-candidate2-{o,r}/`). All as
predicted. (The old whole-sidecar control over the unchanged tree with the `.env` present: ok,
`f11-oldctl-envpresent.txt`, which is B8 re-measured.)

**`collect_stress` over `phase-8.txt` alone (run, scratch only, not committed).** The L0 test in
`final-fix/f11-tree` with its subset narrowed to `["phase-8.txt"]` and two `eprintln!`s added,
`cargo test --release -p rexx-exec --test collect_stress -- the_l0_subset --nocapture`
(`final-fix/f11-stress-phase8{,-detail}.txt`): no plain-versus-stress mismatch (the test passed its
mismatch assertion and failed only on the zero-collection list, which the narrowing makes
incomparable: observed `[]`, and no `NO_ALLOCATION_PROGRAMS` member is in `phase-8.txt`); 924
collections; per program, only `library_requires_missing` and `library_required_method_library_missing`
report "Unable to load library" (both name `zorkolib` on purpose), `library_load_retried` prints its
17 bytes (`first 0` / `second 1`), and `library_method_external`, `library_method_package` and the
`uninit` programs print their output at rc 0. So under the sidecar the stress run reaches the
libraries rather than the load failure. The committed file was restored in scratch afterwards.

**`ir_recorded` with the sidecar (run, scratch tree).** `cargo test --release -p rexx-exec --test
ir_recorded` exit 0, 28 passed, `every_population_runs_without_a_refusal` ok
(`final-fix/f11-ir-recorded.txt`). Its passed count includes `support`'s own unit tests, which a
binary declaring `mod support;` compiles in, as every other such binary here does. Gated corpus
over the scratch tree: `540 of 540 matching` (`final-fix/f11-corpus-scratch.txt`).

**F8 gates (run on the tree then committed unchanged).** fmt exit 0; clippy exit 0; `rexx-api`
tests exit 0; `cargo test --release -p rexx-exec --no-fail-fast` exit 101, 1484 passed / 5 failed,
the G3 five; gated corpus exit 0, `540 of 540 matching`; sourceline exit 0. `final-fix/f8-*.txt`.

**F7 commit: `13268f0e1`. F8 commit: `8c839c2e9`.**

(F9, ported to the worktree) `patch -p1` of the scratch F9 diff (clean), `rustfmt`; clippy exit 0;
`rexx-api` tests exit 0 (lib 15, values 38). A release `rexx-run` from the worktree
(`final-fix/bins/rexx-run-f9`): `a_table.rex` SAME x3, `f9-wt-sig-{ret,argmissing,arg}` SAME x3 with
`o~retopt` now `Error 93 running .../main.rex line 3` and `o~unknown(1)` `Error 93 running
.../f.cls:`. `refusal-sites.tsv`'s `incorrect_method_result_signature` row now has its route and is
measured: agrees/yes/93.968 through the forged `o~retopt`.
Measuring that row made 93.968 an answer two send rows share, so `SHARED_ANSWERS` gains the pair
and the table header's transposition paragraph names it. The paragraph also lacked the 93.903 pair
`SHARED_ANSWERS` already held; it is added. Both transpositions re-measured (answer and witness
swapped, all 5 `refusal_sites` tests green, table restored from a copy, `cmp` clean).

(F10, drafted in a `git archive 8c839c2e9` copy, `final-fix/f10-tree/`, while the F9 gates held the
worktree)

**Change.** A `#[cfg(test)]` counter, `Interp::library_opens`, incremented in `resolve_library` just
before `load::open`. `a_library_named_twice_is_opened_once` asserts one open after two resolves of
`rxregexp` (the `Rc::ptr_eq` stays as a second assertion), with a control in the same test: two
resolves of `zorkolib`, which nothing holds, bring the count to 3. The version-refusal test also
asserts that the later `resolve_library` opened nothing.

**Control prediction (written before the run).** M-F (slice B's mutant: the held-answer early
return in `resolve_library` skipped with `.filter(|_| false)`): `a_library_named_twice_is_opened_once`
fails on `the same name was opened twice` (2 against 1), and
`a_version_refused_library_raises_once_and_is_held` fails with "the refused library was not held"
(the re-open answers `Missing` for a name on no search path); the other `dispatch::library` tests
pass.

**Control result (run, scratch).** M-F: `cargo test -p rexx-exec --lib dispatch::library` exit 101,
exactly `a_library_named_twice_is_opened_once` (`the same name was opened twice`, left 2, right 1,
`library.rs:309`) and `a_version_refused_library_raises_once_and_is_held` (`library.rs:399`, the
"not held" panic) failed, 9 passed (`final-fix/f10-MF.txt`). Confirmed; restored from a copy (`cmp`
clean), 11 passed again. Before this change the same mutant left all `dispatch::library` tests
green (slice B's B10).

**F9 gates (run on the tree then committed unchanged).** fmt exit 0; clippy exit 0; `rexx-api`
tests exit 0; `cargo test --release -p rexx-exec --no-fail-fast` exit 101, 1484 passed / 5 failed,
the G3 five; gated corpus exit 0, `540 of 540 matching`; sourceline exit 0. `final-fix/f9-*.txt`.

**F9 commit: `94158bd73`.**

(F10, ported) The two files copied from the scratch copy over the F9 commit (neither had changed
since the snapshot's `8c839c2e9`); fmt exit 0; clippy exit 0; `dispatch::library` 11 passed.
**F10 gates (run on the tree then committed unchanged).** `cargo test --release -p rexx-exec
--no-fail-fast` exit 101, 1484 passed / 5 failed, the G3 five; gated corpus exit 0, `540 of 540
matching`; sourceline exit 0. `final-fix/f10-*.txt`.

**F10 commit: `959029b6e`.**

(F11, ported) The five test files and the `.stdin` copied from the scratch copy (none had changed
since `49f3c5581`), `git rm` of the `.env`; fmt exit 0; clippy exit 0.
**F11 gates (run on the tree then committed unchanged).** `cargo test --release -p rexx-exec
--no-fail-fast` exit 101, 1524 passed / 5 failed, the G3 five (the passed count rises by the
`support` unit tests `collect_stress` and `ir_recorded` now compile in;
`every_population_runs_without_a_refusal` ok; `the_l0_subset_passes_again_under_collect_on_every_allocation`
still panics at `dispatch.rs:1506`, its pre-existing reason); gated corpus exit 0, `540 of 540
matching`, with `a_sidecar_changes_what_one_of_the_interpreters_answers` and
`every_sidecar_names_a_program_the_subset_runs` ok; sourceline exit 0. `final-fix/f11-*.txt`.

**F11 commit: `cf92ff4fb`.**

## Commits

```
04a286913 F1  Convert a native string argument by REQUEST('STRING') alone
03ceb04df F2  Report a native boundary refusal against the declaring package
40093e99b F3  Name a required package in its own library load failures
ca79611e5 F4  Answer the oracle's package for library-backed Method and Routine objects
bab390715 F5  Hold only a library that loaded, and keep a version-refused one
49f3c5581 F6  Take the library search path once, when the interpreter starts
13268f0e1 F7  Close the safe path to reading uninitialised union bytes
8c839c2e9 F8  Derive the method context pointer from the whole wrapper it heads
94158bd73 F9  Check argument presence before the code, and the result word unstripped
959029b6e F10 Count library opens so a second open of a held name is visible
cf92ff4fb F11 Give every harness the sidecar, and check each part of one on its own
```

## Not done

* **Out of scope by the brief, not started:** the thread context's lifetime (slice A finding 1),
  routine registration at every library load site (B7), a signature-level layout test (A finding
  7), the special codes as return types (A finding 8). B9 and B12 were not in the brief either.
* **Documentation** (`docs/`) untouched. Facts this round changed that the docs pass may meet:
  the native string conversion protocol and its raise channel (`Host::string_value`); the lineless
  boundary constructors and the two 93.968 deliveries; `Libraries` holding only loaded libraries
  and the version-refused rule; `loadLibrary`/`loadExternal*` raising 98.982; the library search
  taken at start; `Method~package` for `EXTERNAL` and `loadExternal*`; `value_of` unsafe and
  `MethodContext`; the sidecar reader's new home and the per-part control; `external_trace.stdin`.
* **Measured and left alone, recorded above:** the `LIBRARY REXX` `unresolved_external` arm still
  names the running program for a required package (F3 neighbour, the KNOWN GAPS class); the
  oracle build's `RUNPATH` empty element makes its `dlopen` search the working directory (F6); the
  oracle runs every held package's `unloader` at termination and the crate runs no loader or
  unloader for any library (F5, read, not run).
* **Not reached:** a `newFile`/`Package~new` context argument that is a `loadExternal*` object
  whose package is still `.nil` (F4); a committed witness for `loadLibrary`'s first-ask 98.982
  (F5's M-F5b is seen only by the scratch forged probe, since the corpus loads only oracle-built
  extensions); a refused library's empty routine table is witnessed only by the scratch forged
  probes.
* **Instruments outside the gate:** Miri ran from a scratch `RUSTUP_HOME` (F7, F8); the gate's
  stable toolchain does not check the `compile_fail` doctest's error code, and cannot see F7's
  zeroing mutant (Miri can). The collect-on-every-allocation L0 test stays red for its pre-existing
  panic, so `phase-8.txt` under stress was checked only by the scratch-narrowed run.
* **Gates not run by me:** the debug `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace` (G4)
  and the full workspace gates, per the brief.
