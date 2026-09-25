# Phase 8 surface, Task 2 -- the routine half of the calling protocol, at every load site

BASE `e7cb210d9`. Written first and appended as the work goes.

Scratch: `scratchpad/surface-t2/`, a fresh directory per probe.

## Status

DONE_WITH_CONCERNS. Commits `4c7d65e97` and `c23c214f3`; details at the end.

Fix round 1: DONE_WITH_CONCERNS. Commits `a873b0678`, `fa7c51301`, `6245fe1ca`, `25bdb2dd5`,
`cae7d5382`, `2804a8986`, `79f08bed6`, `bd64f3197`; the "Fix round 1" section below.

Fix round 2: DONE_WITH_CONCERNS. Commits `eab19ac3b`, `c08f4badd`, `a15a8603a`, `5d08b1bce`,
`9ff11e465`; the "Fix round 2" section below.

Fix round 3: DONE_WITH_CONCERNS. Commits `c55caa62f`, `668c73827`, `a1ee6579c`, `273e8f608`,
`3f48151ce`, `cb13f0033`; the "Fix round 3" section below.

## Rulings taken before implementing (controller, 2026-09-15)

Asked because no `rxmath` routine answers on the BASE tree without rows Task 3 owns and a slot
Task 5 owns: `RxCalcSqrt` is `RexxRoutine2(RexxObjectPtr, RxCalcSqrt, double, x,
OPTIONAL_positive_wholenumber_t, precision)` (`extensions/rxmath/rxmath.cpp:409`), its body calls
`GetContextDigits` and `DoubleToObjectWithPrecision`, and `values.rs` holds `double` to_native and
`RexxObjectPtr` from_native as unfilled stubs.

* **Q1 (A):** this task fills `double` and `positive_wholenumber_t` to_native, `RexxObjectPtr`
  from_native and the thread table's `DoubleToObjectWithPrecision`, each with per-row tests, and runs
  the row-deletion control on one of the rows it adds. Witness several precisions. Anything beyond
  those four: stop and report the list first.
* **Q2 (b):** `CallContextInterface` becomes populated, a call context is wired through `Contexts`,
  and `GetContextDigits`, `GetContextFuzz`, `GetContextForm` are filled; the rest refuse naming
  Phase 8. Task 5 takes the call-context members `orxfunction.cpp` calls (the controller amends the
  plan; this task does not edit it).

## B7 reproduced at BASE

`scratchpad/surface-t2/ab.sh DIR` (copies DIR into two fresh directories, oracle and
`rust/target/release/rexx-run` built at `e7cb210d9`, three descriptors apart). `r1`, `r5`, `r6`,
`r7`: oracle rc 0 printing `4`, crate 43.1 rc 213. `r8` (`loadExternalRoutine(...)~call(16)`):
oracle `call 4`, crate rc 120 "a routine whose body this crate does not hold is not implemented
(Phase 5)". `r10` (`.K~define` of two `loadExternalMethod` objects): oracle `defined 1`, crate
rc 120 "method "INIT" of class "K" is not implemented (Phase 5)".

## Oracle measurements taken before implementing

### rxmath formatting, errors, calls (`p/m1`, `p/m2`, `p/s4`, `p/s5`; oracle only)

`m1` pins what `DoubleToObjectWithPrecision` has to reproduce: `RxCalcSqrt(16)` `4`,
`RxCalcSqrt(2)` `1.41421356`, `(2, 3)` `1.41`, `(2, 16)` and `(2, 20)` `1.414213562373095`,
`RxCalcPower(10, 10)` `1.00000000E+10` against `(10, 10, 3)` `1E+10`, `RxCalcSqrt(-1)` `nan`,
`RxCalcExp(1000)` `+infinity`, `RxCalcSqrt(1e-10)` `0.00001`, `RxCalcPower(10, 9)` `1.00000000E+9`
against `(10, 8)` `100000000`, `RxCalcPower(123456789, 1, 5)` `1.2346E+8`, `NUMERIC FORM
ENGINEERING` not changing `1E+10`/`1E+7`, and `RxCalcSqrt('nan')`, `'+infinity'`, `' 16 '`,
`'1e2'`, `16.0` each converting. `m2`, condition objects under `SIGNAL ON SYNTAX`: `'abc'` and
`.object~new` are 88.921 `Argument 1 must be a valid double value; found "abc".` / `found "an
Object".`; precision `0`, `-1`, `1.5`, `'x'` are 88.905 `Argument 2 must be a positive whole number;
found "0".`; no argument and `(,2)` are 88.901; three arguments 88.922 `2 expected`; precision
`1000000000000` and `3.0` accepted. `s4` (`::routine sq external "LIBRARY rxmath RxCalcSqrt"`): `sq(16)`
`4`, `RxCalcSqrt(9)` `3` from the same program, `findRoutine('SQ')~call(4)` `2`, `~callWith` `9`,
`.context~package~routines` holds `SQ` and not `RXCALCSQRT`, and `trace r` over `y = sq(100)`
traces only the clause and `>>>   "10"`. `s5`: a `loadExternalRoutine` answer's `~call(16)` `4`,
`~callWith(.array~of(2, 4))` `1.414`, `r[9]` `3`; `.K~define` of two `loadExternalMethod` answers
then `k~does('aab') k~does('abc')` `1 0`.

### Step 3: which packages see a library routine (`p/s1b`, `p/s2`, `p/s3`)

`s1b`: `main.rex` requires `pk1.cls` (a routine calling `RxCalcSqrt(16)` under `SIGNAL ON SYNTAX`)
and `pk2.cls` (a routine doing `.context~package~loadLibrary('rxmath')`). Oracle rc 0: before the
load pk1's routine answers `miss 43.1`; after it, pk1's routine (translated before the library
loaded) answers `4`, `main.rex` answers `8`, `call RxCalcSqrt 25` sets `RESULT` to `5`, the literal
name `'RxCalcSqrt'(36)` and `rxcalcsqrt(49)` answer `6 7`, and a `.Routine~newFile('late.rex')`
package built after the load answers `9`. `.context~package~findRoutine('RxCalcSqrt')` answers
`The NIL object`. **Every package sees it, loaded before or after, and no package's routine
directory holds it**, which is `LibraryPackage::loadRoutines` putting each routine into
`PackageManager::packageRoutines` (`interpreter/package/LibraryPackage.cpp`, `addPackageRoutine`),
consulted by `callNativeRoutine` after the package's own routines and before the external file
search (`interpreter/platform/unix/ExternalFunctions.cpp`, `invokeExternalFunction`). `s2`: a
`RxCalcSqrt.rex` and a `rxcalcpi.rex` beside the program lose to the loaded library (`4`,
`3.14159265`). `s3`: the program's own `::routine RxCalcSqrt` wins over it.

### The `REXX` and `REGISTERED` forms: predictions, written before running

* `rx1` `::routine fs external "LIBRARY REXX Filespec"`, `say fs('N', '/a/b.c')`: rc 0 `b.c`.
* `rx2` `::routine filespec external "LIBRARY REXX"` (entry defaults to the name): rc 0, found
  caselessly.
* `rx3` `"LIBRARY REXX nosuch"`: 90.999 rc 166 at install.
* `rx4` `::routine sv external "LIBRARY REXXUTIL SysVersion"`: rc 0 on the oracle (an internal
  package); the crate loads `librexxutil` and answers 98.903.
* `rx5` `.Routine~loadExternalRoutine('fs', 'LIBRARY REXX Filespec')~call('N', '/a/b.c')`: `b.c`.
* `rx6` `"LIBRARY REXX SysVersion"` (a REXXUTIL name through REXX): 90.999 rc 166.
* `reg`, three separate processes over a name nothing else uses: before, `rxfuncquery('ZZT2REGPROBE')`
  answers `1`; a program with `::routine zzt2regprobe external "REGISTERED rxmath ZzT2NoSuchProc"`
  answers 90.999 rc 166; a third process's `rxfuncquery` then answers `0`, because
  `PackageManager::resolveRoutine(function, package, procedure)` calls `RexxRegisterFunctionDll`
  unconditionally before resolving; `rxfuncdrop` answers `0` and a last `rxfuncquery` `1`.

### The `REXX` and `REGISTERED` forms: what ran (`p/rx1`-`rx10`, `p/reg0`-`reg4`)

* `rx1`, `rx2`, `rx5`: confirmed, `b.c` rc 0 each. `rx3`: confirmed, 90.999 `Unable to find
  external routine "nosuch".` rc 166, the directive line blamed. `rx6`: confirmed, 90.999 naming
  `SysVersion`.
* `rx4`: **falsified.** `"LIBRARY REXXUTIL SysVersion"` is `98.903 Unable to load library
  "REXXUTIL".` rc 158 on the oracle too; `REXXUTIL` is not a package `getLibrary` finds.
* `rx7`: a routine bound to `LIBRARY REXX Filespec` raises what `filespec` raises by name (40.904
  `FILESPEC argument 1 must be one of DELNP; found "Z".`, 88.901 with no argument);
  `findRoutine('FS')~package == .context~package` is `0`; `loadExternalRoutine('x', 'LIBRARY REXX
  filespec')` finds the entry caselessly, `~call('E', 'a.b')` answers `b`; `LIBRARY REXX nosuch`
  answers `.nil`. `rx8`: that object's `~package` is `The REXX Package` before and after another
  package binds the same entry with `::ROUTINE` (the internal package's routines already have a
  package, so no directive's binding is the first). `rx9`: uncaught, the traceback's first line is
  `*-* Compiled routine "FS".`, the name called. `rx10`: `trace r` shows only the clause and `>>>`,
  alike for `fs` and `filespec`.
* `reg0`-`reg4`, predictions against runs: `rxfuncquery` before `1` (confirmed); the `REGISTERED`
  directive 90.999 rc 166 (confirmed); a later process's `rxfuncquery` `0` (confirmed: the directive
  registered the name in the RXAPI daemon before failing to resolve it, and the registration outlived
  its process); `rxfuncdrop` **`1`, falsified**, and a last `rxfuncquery` still `0`.
  `RegistrationTable::dropCallback` (`rexxapi/common/RegistrationTable.cpp:424-469`) refuses a drop
  from a session other than the owner's, and `RexxRegisterFunctionDll` registers owner-only
  (`rexxapi/client/RegistrationAPI.cpp:440`). **Side effect left behind:** the machine's `rxapi`
  daemon (running before this task) holds a function registration `ZZT2REGPROBE -> rxmath
  ZzT2NoSuchProc` until it restarts. Only a program calling that name can observe it.

**Decision.** `REGISTERED` goes to Phase 10: the measurement shows the form writes the RXAPI
daemon's function registry, state shared between processes and outliving them, which is the
registry `RXFUNCADD`/`RXFUNCDROP`/`RXFUNCQUERY` (already Phase 10's) read and write, and which this
crate may not mutate. `LIBRARY REXX` is implemented here: it binds an entry of the `REXX` package's
own routine table (`interpreter/runtime/NativeFunctions.h` and
`interpreter/platform/unix/SysNativeFunctions.h`), whose three Unix entries this crate already runs
by name, and no other phase owns it.

### Error shapes a native routine's boundary raises (`p/e1`-`e11`)

The first frame with a package decides the reported program and line
(`Activity::createExceptionObject`'s frame walk, `interpreter/concurrency/Activity.cpp:1093-1113`),
and a native frame's package is its routine's (`NativeActivation::getPackageObject`, `:3153`). So:
a routine no directive has bound reports **the caller's program and line** (`e1`, `e2`, `e5`, `e6`,
`e8`: `Error 88 running main.rex line 2:` / `pk.cls line 4:`, and `e4`, `r~call(1,2,3)` from
`main.rex line 3`), and a routine a `::ROUTINE` directive bound reports **that package with no line**
(`e3`: `Error 88 running pk.cls:`). The traceback's first line is `*-* Compiled routine "<name>".`
with the name the call passed: upcased through `PackageManager::callNativeRoutine` (`e1`, and `e9`'s
literal `'rxcalcsqrt'(...)` after `loadLibrary`), the name as written when a `::REQUIRES ... LIBRARY`
merged the routine into the calling package (`e10`: `"rxcalcsqrt"`), the directive's name for a
`::ROUTINE` (`e3`: `"SQ"`), and the routine's own table spelling for `Routine~call` (`e4`:
`"RxCalcSqrt"`, under `Compiled method "CALL" with scope "Routine".`). `e11`:
`.context~package~findRoutine('RXCALCSQRT')` answers a `Routine` in a package with `::REQUIRES
'rxmath' LIBRARY` (`PackageClass::mergeLibrary`, `interpreter/classes/PackageClass.cpp:756`), though
its `routines` and `publicRoutines` both hold nothing.

## What was built (in progress, uncommitted)

**rexx-api.** `invoke::routine` beside `invoke::method`, both calling one private `run` that holds
the argument loop and the too-many check (ruling S2). `ROUTINE_CLASSIC_STYLE` rows are refused with
`Failure::ClassicStyle` before either stub call. `NativeRoutineEntry::{signature, call}` share two
generic `unsafe fn`s with the method row in `load.rs`. `CallContextInterface` is populated;
`ffi::CALL_CONTEXT` fills `GetContextDigits`/`Fuzz`/`Form`, `Contexts::call` links it to the thread
table, which gains `DoubleToObjectWithPrecision`. Rows filled: `double` and
`positive_wholenumber_t` to native (88.921 and 88.905, measured), `RexxObjectPtr` from native.
`Library::package_routines` is the upcased-name-to-spelling map `loadRoutines` builds.

**rexx-exec.** `Interp::settle_library` registers every loaded library's routines into
`package_routines` (the one path `resolve_library` settles through, so every load site). A slot keeps
its index when a later library replaces its row. `Resolved::LibraryRoutine` resolves after the
internal packages and before the external file search; the call-site cache no longer keeps
`Resolved::External`. A `::ROUTINE ... EXTERNAL "LIBRARY"` call, `Routine~call`/`callWith`/`[]` on a
directive-bound or `loadExternalRoutine` routine, and `~define`/`~defineMethods` of a
`loadExternalMethod` answer all run. A boundary refusal is lineless against the code's package only
where the shared code has one (a directive bound it), else reported at the caller's clause.
`double_text` reproduces `newInstanceFromDouble` (`%.*g` at `min(16, p) + 2`, round, format).

Probes at this point, all three descriptors identical: `r1`, `r5`, `r6`, `r7`, `r8`, `r10`, `m1`,
`m2`, `s1b`, `s2`, `s3`, `s4`, `s5`, `e1`-`e6`, `e8`, `e9`, `e12`, `e13` (`e13`: `RxCalcSqrt(1,2,3)`
called by name after a required package's `::routine sq external` bound the code reports `Error 88
running pk.cls:` with no line on both sides). Still divergent: `e7`, `e10`, `e11`, the
`::REQUIRES ... LIBRARY` package merge.

**Added after the list above.** `LIBRARY REXX` routines: a `::ROUTINE` binds the `REXX` package's row
(exact spelling, then caseless; 90.999 on the directive line for none), its calls run the row under
the called name, `Routine~call` under the row's spelling, `~package` answers the `REXX` package, and
`loadExternalRoutine('x', 'LIBRARY REXX Filespec')` answers a callable `Routine` (`.nil` for no
row). `REGISTERED` refuses at install naming Phase 10. `routine_external`'s default entry is the
name the parser kept (upcased symbol, string as written): `nm1`, `::routine 'zq' external "LIBRARY
rxmath"`, was `"ZQ"` here against the oracle's `"zq"`. `::REQUIRES ... LIBRARY` also merges the
library's routines into the requiring package's lookup (propagated to requirers), resolved as
`Resolved::MergedLibraryRoutine` where a `::ROUTINE` is, before the security manager, under the
name as written, and answered by `Package~findRoutine`.

Probes after these, all three descriptors identical: `rx1`-`rx6`, `rx9`-`rx11`, `rx13`-`rx15`,
`nm1`, `e7`, `e10`, `e11b` (`findRoutine('RXCALCSQRT')~call(49)` `7`, its package `.nil`), `sm1`
(an auditing manager sees `CALL RXCALCSQRT` for a call through `loadLibrary`'s global table and no
event for the same call merged by `::REQUIRES ... LIBRARY`; oracle and crate alike), `d1`, `d2` (a
`~define`d `loadExternalMethod` answer's 88.901 is reported at the caller's line until a directive
binds the same code, then against that directive's package with no line). Divergent only through
`==` on `Routine` objects, which refuses naming Phase 5 before and after this task: `rx7`, `rx8`,
`rx12`, `id1`. `reg1` refuses naming Phase 10 by decision.

## Negative controls: predictions, written before any was run

Each mutant is built with `--profile mutation` into a scratch `CARGO_TARGET_DIR`, and the corpus
ones are read through `ab.sh` over the new witnesses (named `W/<name>`) plus the pre-existing
`library_*` witnesses of `corpus/phase-8.txt`.

* **C1, the row-deletion control, on `double`** (`to_native: None` in its row). Red:
  `values.rs` `exactly_the_filled_rows_convert`, `the_double_row_converts_what_the_host_reads`,
  `a_double_the_host_cannot_read_is_88_921_naming_the_argument`,
  `a_raise_reading_a_double_is_the_hosts_condition`; `invoke.rs`
  `rxcalcsqrt_formats_at_the_callers_digits_when_no_precision_is_given`,
  `rxcalcsqrt_reads_a_precision_argument_that_exists`,
  `rxcalcsqrt_refuses_a_missing_and_an_extra_argument_before_running` (the three-argument call
  converts argument one before it counts). Green: `an_absent_double_is_88_901_or_a_zero` (absence
  is settled before the row is read) and every other `rexx-api` test.
* **C2, the classic-style check deleted from `invoke::routine`.** Red:
  `a_classic_routine_is_refused_before_its_stub_is_entered` alone.
* **C3, registration moved back to one caller** (the `register_package_routines` call deleted from
  `settle_library`, called from the `::REQUIRES ... LIBRARY` arm instead). Red:
  `W/library_routine_load_library`, `W/library_routine_directive` (`global`),
  `W/library_routine_load_external` (`main`), `W/library_routine_load_external_method`,
  `W/library_routine_method_directive_load`, `W/library_routine_after_external_file`,
  `W/library_routine_security_manager` (`global`), `W/library_routine_package_blame` (43.1 in place
  of 88.922). Green: the other new witnesses and every pre-existing one.
* **C4, the call-site table keeping `Resolved::External` again.** Red:
  `W/library_routine_after_external_file` alone (`2 file` where the oracle prints `2 4`).
* **C5, a boundary refusal lineless whatever the package** (the `if !packaged` reset deleted). Red:
  `W/library_routine_caller_blame`, `W/library_routine_call_blame`,
  `W/library_method_loaded_defined` (each loses ` line <n>`). Green:
  `W/library_routine_package_blame`, `W/library_method_loaded_defined_blame`, the pre-existing
  witnesses.
* **C6, the merged-library arm deleted from `resolve_call`.** Red:
  `W/library_routine_name_as_written` (`RXCALCSQRT`) and `W/library_routine_security_manager` (a
  `CALL` event for the merged call). Green: the rest, `library_routine_requires` included (every
  line it prints is the same through the global table).

## Miri

`RUSTUP_HOME=scratchpad/final-fix/rustup-home CARGO_TARGET_DIR=scratchpad/surface-t2/miri-target
rustup run nightly cargo miri test -p rexx-api --lib --locked` over the worktree sources, Stacked
Borrows (no borrow-model flag): 20 passed, 0 failed, 3 ignored (the child-process test and the two
that open the running image), exit 0 (`scratchpad/surface-t2/miri-sb.txt`). The new
`a_routine_reads_numeric_settings_and_builds_a_double_through_its_contexts` is among the passes: it
recovers the owner through the call context `Contexts::call` hands out and through the thread
context that context links.

## Negative controls: what ran

Restored from a copy after each and `cmp`-checked. The sweep (`scratchpad/surface-t2/ctl/sweep.sh`)
runs every `lang/library_*` and `lang/external_method*` row of `corpus/phase-8.txt`; against the
unmutated release binary it is green on every row (`ctl/baseline.txt`).

* **C1: confirmed.** Red exactly the four `values.rs` and three `invoke.rs` tests predicted;
  `an_absent_double_is_88_901_or_a_zero` and everything else green (`ctl/c1.txt`, exit 101).
* **C2: confirmed.** `a_classic_routine_is_refused_before_its_stub_is_entered` alone (`ctl/c2.txt`).
* **C3: partly falsified.** Red as predicted: `library_routine_load_library`, `_directive`,
  `_load_external`, `_load_external_method`, `_method_directive_load`, `_after_external_file`,
  `_package_blame`. **`library_routine_security_manager` stayed green**, against the prediction: its
  `merged.rex` carries the `::REQUIRES ... LIBRARY` that the mutant still registers from, so
  `global.rex` finds the routine in the global table either way. No pre-existing witness went red.
* **C4: confirmed.** `library_routine_after_external_file` alone.
* **C5: partly falsified.** Red as predicted: `library_routine_caller_blame`, `_call_blame`,
  `library_method_loaded_defined`. **Also red, not predicted: `library_routine_name_as_written`**,
  which is the same shape (a merged routine no directive bound, so its 88.921 carries the caller's
  line) and should have been on the list. `_package_blame`, `library_method_loaded_defined_blame`
  and the pre-existing witnesses green.
* **C6: confirmed.** `library_routine_name_as_written` and `library_routine_security_manager` alone.

Under C3 to C6 no pre-existing witness went red, so each mutant is seen only by this task's
witnesses.

## Step 5 and the refusal table

Deleted: `Loud::library_routine_call` ("a call to a ::ROUTINE EXTERNAL", Phase 8),
`Loud::required_library_routine` (Phase 8), `routine_without_a_body`'s library branch, the
`loadExternalRoutine on REXX` refusal, and `::ROUTINE EXTERNAL naming REXX or REGISTERED` (Phase 8),
now `::ROUTINE EXTERNAL naming REGISTERED` (Phase 10). `Loud::external_entry_point` (Phase 8)
became `Loud::library_procedure_gone`, an internal-inconsistency message naming no phase, whose one
use is a held library's row that stopped resolving. `run/tests.rs`: the install-refusal row is the
`REGISTERED` form; `a_library_backed_routine_installs_and_runs_when_it_is_called` and
`a_routine_a_required_library_exports_runs_rather_than_answering_43_1` assert `3.14159265` at rc 0,
the adjacent 43.1 pair kept. `refusal-sites.tsv` re-derived with `REXX_REFUSAL_SITES_REFRESH=1`; the
rows it left empty were measured: `native_argument_not_a_double` (agrees, 88.921,
`library_routine_argument_errors.rex`), `native_argument_not_positive` (agrees, 88.905,
`library_routine_caller_blame.rex`), `library_procedure_gone` (reached no, no route). Also corrected:
`corpus/lang/library_loads_once.rex`'s opening comment, which said `::REQUIRES LIBRARY` "registers
nothing a name can be looked up in" (companion regenerated), and `phase-4-exclusions.txt`'s
routine-half entry, marked closed with the two forms that still refuse.

Phase 8 refusals left on the routine path are the conversion rows and thread slots this task does
not own (`dispatch/library.rs`'s `Unfilled`/`StaleHandle`/`Raised` arm), Tasks 3 and 5.

## Step 6 witnesses

Under "Surface Task 2" in `corpus/phase-8.txt`, each with a `sourceline_oracle` companion regenerated
from a scratch copy (a directory holding only the copy, which still held only it afterwards):
`library_routine_requires`, `_load_library`, `_directive`, `_load_external`,
`_load_external_method`, `_method_directive_load`, `_after_external_file`, `_argument_errors`,
`_caller_blame`, `_package_blame`, `_name_as_written`, `_call_blame`, `_quoted_name`,
`_security_manager`, `_rexx_package`, `_rexx_package_missing`, `_rexx_package_call_blame`,
`library_method_loaded_defined`, `library_method_loaded_defined_blame`. The classic style has only
`rexx-api`'s unit test: `grep -rln REXX_CLASSIC_ROUTINE` over the C++ tree's sources names
`api/oorexxapi.h` and `extensions/platform/windows/rxwinsys/rxwinsys.cpp` alone, and the same over
the oracle checkout's `samples/` names nothing. In-crate:
`a_library_routine_answers_the_same_under_a_collection_at_every_allocation`, its stdout the oracle's.

## Gates

Commit `4c7d65e97`, run from `scratchpad/surface-t2/gates/run.sh` with the tree untouched until its
status file said `finished` (status: sha, then `git status --short` empty):
* `memcap 16G cargo test --release -j 4 -p rexx-api -p rexx-core -p rexx-exec -p rexx-parse
  --no-fail-fast`: **exit 101**, 79 `test result: ok` lines, 2 FAILED: `collect_stress`'s
  `the_l0_subset_passes_again_under_collect_on_every_allocation` (`a live value` at
  `dispatch.rs:1510`, the BASE failure Task 1 recorded), and `refusal_sites`'
  `the_table_holds_every_constructor_the_source_defines`, whose line column the merged-lookup guard,
  added after the refresh, had moved. Re-derived and committed as `c23c214f3`; `cargo test --release
  -p rexx-exec --test refusal_sites` then 5 passed.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test --release -j 4 -p rexx-exec --test corpus`: exit 0,
  `571 of 571 matching`.
* Miri, Stacked Borrows, `cargo miri test -p rexx-api --lib --locked`: exit 0, 20 passed, 3 ignored.
* Before the commit: `cargo fmt --all --check` exit 0; `cargo clippy -j 4 --workspace --all-targets
  -- -D warnings` exit 0.

## Concerns

1. **An `rxmath` routine that raises aborts the process.** `RxCalcSin(30, 3, 'X')` is 88.916 rc 168
   on the oracle; here `TrigFormatter` calls `context->String`, the thread table's refusing
   `NewStringFromAsciiz`, and the process aborts at rc 134 losing the `SAY` output before it
   (`p/sin1`). Loud, but reachable now the routines run. Task 5's slots.
2. **`MathLoadFuncs`/`MathDropFuncs` refuse** loudly: their `CSTRING` result is Task 3's row
   (`p/mlf`, rc 120).
3. **Name collisions are not reproduced**, none witnessed: a library routine named as an internal
   routine (the oracle's shared table lets the library replace it; here the internal one wins), a
   merged library routine against a merged public routine of the same name, and the oracle's order
   of all library merges before `::REQUIRES` merges.
4. **Routine identity**: the oracle answers one `Routine` per table entry, the same object from
   `loadExternalRoutine` twice and from the directive (`p/id1`); here each ask builds one. `==` on a
   `Routine` refuses naming Phase 5 before and after this task, so it is not a silent answer.
5. **Machine side effect**: the `rxapi` daemon holds `ZZT2REGPROBE -> rxmath ZzT2NoSuchProc` until it
   restarts.
6. Seen in passing, not this task's, not checked at BASE: `call .context~package~loadPackage(...)`
   (a parse error) prints `rexx-exec: 35.1: Invalid expression.` rc 120 where the oracle reports
   `Error 35 running ... line 2` rc 221; `.K~define` of a `Method` built from source text refuses
   naming Phase 5 (`p/def1`).

# Fix round 1

Base `c23c214f3`. The two reviews: `task-2-review-boundary.md` (probes `scratchpad/t2-review-a/`) and
`task-2-review-integration.md` (probes `scratchpad/t2-review-b/`). Scratch for this round:
`scratchpad/surface-t2/fix1/`, a fresh directory per probe. Order is the controller's. Nothing is
registered with the rxapi daemon in this round.

## Item 1 (integration I4): `double_of` and a huge exponent

Prediction for `fix1/i4a` (each value through `RxCalcSqrt(x, 16)` and `RxCalcPower(x, 1, 16)`, trapped),
written before running: at `c23c214f3` the crate aborts at rc 134 on the first value under the
wrapper's `ulimit -v`, `start` lost; the oracle answers `+infinity +infinity` for `1E+999999999`,
`0 0` for `1E-999999999`, `nan -infinity` for `-9.99E+999999999`, `+infinity +infinity` for
`1.8E+308`, `0 0` for `1E-400`, and something I do not predict for `1E+1000000000`, whose exponent
is past `Numerics::MAX_EXPONENT`. After the fix: identical on every line, that last one included or
recorded.

At `c23c214f3`, run: the prediction held (crate rc 134, `memory allocation of 999999999 bytes
failed`, stdout empty). Oracle: every predicted line, and `1E+1000000000` / `1E-1000000000` are
88.921. Fix: `double_of` reads `Number::format` at the number's own digit count, which keeps every
digit and takes the exponential form wherever the plain one would pad with zeros. After it, `i4a`
and the reviewer's `e1`/`e2` identical on three descriptors under the wrapper's cap. Witnesses:
`corpus/lang/library_routine_double_exponent.rex` (`i4a`), and the unit test
`a_double_argument_is_read_from_its_digits_and_exponent`, which also asserts the literal for
`1E+999999999` is `1E+999999999`.

Control I4-C, prediction before running: `double_literal` back to `format(u64::MAX)` reddens
`a_double_argument_is_read_from_its_digits_and_exponent` on the literal assertion (after allocating
the long strings, under `memcap 8G`), and nothing else in `rexx-exec --lib`.

Run: **confirmed**, that test alone red (824 passed, 1 failed), tree restored and `cmp`-checked.

Integration M4, folded in here because both citations are in `dispatch/library.rs`: the frame walk
at `Activity.cpp:1093-1113` is `Activity::generateProgramInformation` (`:1080`), corrected. **The
other half of M4 does not hold in this tree**: `sed -n 704p interpreter/classes/NumberStringClass.cpp`
prints `bool NumberString::doubleValue(double &result)`, the same in the oracle checkout, so `:704`
stays.

## Item 2 (boundary I2): the instance a thread context links

`liborxmethod.so` NEEDs `libc.so.6` alone (`readelf -d`). Prediction for `fix1/i2a` (`TestInterpreterVersion`
and `TestLanguageLevel`, each `RexxMethod0(size_t, ...)` reading `context->InterpreterVersion()` /
`LanguageLevel()` through the thread context's `instance`, `testbinaries/orxmethod.cpp:2219-2240`),
before running: oracle `before` / `version 328448` / `level 1542` / `as hex 50300 606` / `after`,
rc 0 (`REXX_CURRENT_INTERPRETER_VERSION` 0x50300 and `REXX_CURRENT_LANGUAGE_LEVEL` 0x606,
`api/oorexxapi.h:242`, `:249`); crate at `a873b0678` rc 139, both descriptors empty. After the
instance is handed out, the crate does not crash: **the result word is `size_t`, whose from-native
row is Task 3's**, so the crate answers `before` and then a loud rc 120 naming
`REXX_VALUE_size_t` rather than the oracle's value.

At `a873b0678`, run: as predicted, oracle `before` / `version 328448` / `level 1542` / `as hex 50300
606` / `after` rc 0, crate rc 139 with both descriptors empty. After the instance is handed out, as
predicted: `before`, then rc 120 `Phase 8 owes the FromNative conversion for REXX_VALUE_size_t (33)`.
Built: `RexxInstanceInterface` is populated; `ffi::INSTANCE` fills `InterpreterVersion` (the header's
0x50300) and `LanguageLevel` (`load::CURRENT_LANGUAGE_LEVEL`, 0x606, asserted against the header by
`tests/load.rs`); `Contexts` owns an `Owned<RexxInstance_, Activation>` and links it into the thread
context wherever it links the thread table; the refusing `instance_interface()` hand-out and its
test row are gone. Witnesses: `invoke::tests::a_stub_reaches_the_instance_through_the_thread_context`
(a stub reading both through the thread context, values 328448 and 1542), and
`run::tests::an_extension_reading_the_instance_reaches_its_table` (`orxmethod`, rc 120 with `before`
kept). No corpus row: the call still diverges on Task 3's `size_t` row.

## Item 3 (boundary I3): a refusing slot records and returns

Design, as ruled. `layout.rs` keeps one thread-local `Cell<Option<&'static str>>`, set by a refusing
stub (first one wins) and taken by `recording_refusals`, which `invoke::run` wraps around the stub's
call alone: the record in force before is saved and put back, so a nested native call keeps its own
and the cell holds nothing outside that call. A refusing stub returns the oracle's failure value (a
null handle, zero or `false`; `DisplayCondition` answers 49, `Error_Interpretation/1000`: 48 until fix round 2), and
`run` turns a record into `Failure::UnfilledSlot { entry }` before it reads the result, forgetting
any condition the extension raised; the host renders it as a loud rc 120 naming the member and
Phase 8.

**Members that still abort**, marked `aborts` in `layout.rs` and pinned by
`a_refusing_member_aborts_exactly_where_no_return_is_safe`, which derives the set from the header's
return types:
* the `Throw` members of the method and call tables (`ThrowException0/1/2`, `ThrowException`,
  `ThrowCondition`): the oracle leaves the extension by a C++ throw, so the extension does not expect
  control back;
* every member answering a data pointer the extension dereferences: `ObjectToCSelf`,
  `ObjectToCSelfScoped`, `PointerValue`, `BufferData`, `BufferStringData`, `MutableBufferData`,
  `SetMutableBufferCapacity` (`POINTER`s it reads or writes through), `ObjectToStringValue` and
  `StringData` (`CSTRING`; `StringData` is filled, so only a context addressing the refusing table
  reaches its stub), `GetInterpreterInstance` (`RexxInstance *`), the method
  table's `GetCSelf`, `AllocateObjectMemory`, `ReallocateObjectMemory` (`POINTER`) and
  `GetMessageName` (`CSTRING`), and the call table's `GetRoutineName` (`CSTRING`). A null answer
  there would turn a named abort into an unnamed SIGSEGV.
* `StringGet` records and returns 0: it copies into a buffer the extension supplies, so a zero
  answer writes nothing.

Prediction for `fix1/i3a` (`say 'before'` / `say RxCalcSin(30, 3, 'X')` / `say 'after'`), before
running: oracle `before`, then 88.916 `Argument 3 must be one of D, R, or G; found "X".` rc 168;
crate `before`, then rc 120 `rexx-exec: RexxThreadInterface.NewStringFromAsciiz is not implemented
(Phase 8)`, stdout `before` kept (the first member `TrigFormatter` reaches that nothing fills is a
`String` call, `extensions/rxmath/rxmath.cpp:177-181`, and both of its `String` calls are
`NewStringFromAsciiz`).

**Item 1 gates**, `a873b0678` in a `git archive` copy with the other top-level directories linked to
the worktree's (`fix1/gates/gate.sh`, statuses in `fix1/gates/status-a873b…txt`): gated corpus
exit 0, `572 of 572 matching`; release tests of the four crates exit 101: `collect_stress`'s L0
test (the BASE failure) and `collection_arity`/`introspection_arity`, whose failure was the gate
copy's (`this test compares the release binary; build it first`, a path under the copy's
`rust/target`). With that directory linked, the two binaries re-ran in the copy: 24 and 26 passed,
exit 0. (The script has since changed again; see "A gate that did not gate its tree" below.)

After the change, `fix1/i3a` ran as predicted: stdout `before` identical, crate rc 120 `rexx-exec:
RexxThreadInterface.NewStringFromAsciiz is not implemented (Phase 8)`, oracle 88.916 rc 168. Witness
`run::tests::an_extension_reaching_an_unwritten_member_refuses_loudly`; unit tests
`ffi::tests::a_refusing_entry_records_itself_and_returns` (bare null contexts, the three failure
values, the first member kept, the outer record restored), `ffi::tests::an_aborting_entry_refuses_loudly`
(`BufferData`, in a child), and
`invoke::tests::a_refused_member_is_the_calls_answer_and_a_nested_call_keeps_its_own`.

## Item 4 (boundary I1), and boundary M2, M3

`NativeRoutineEntry::stub` answers `None` for any style but `ROUTINE_TYPED_STYLE`, so its SAFETY note
rests on a check in `load.rs` itself; `invoke::routine` keeps its own check for the named refusal.
Negative test `invoke::tests::a_classic_row_publishes_no_signature_and_calls_nothing`. M2: the bound is
one private `invoke::bounded`, which `signature` and `routine` share. M3: the `rxmath` test no longer
asserts the stand-in's rendering of `4.0`; it asserts `doubles == [(4.0, 12)]` alone.

## Controls for items 2 to 4, predictions written before running

* **C-I2**, the instance handed out addressing `RexxInstanceInterface::REFUSING`: red
  `invoke::tests::a_stub_reaches_the_instance_through_the_thread_context` (the call answers
  `UnfilledSlot` naming `InterpreterVersion`) and
  `run::tests::an_extension_reading_the_instance_reaches_its_table` (stderr names
  `RexxInstanceInterface.InterpreterVersion` instead of the `size_t` row); nothing else in either
  crate's lib tests.
* **C-I3a**, `recording_refusals` putting back `None` rather than the outer record: red
  `a_refused_member_is_the_calls_answer_and_a_nested_call_keeps_its_own` alone (the outer call
  answers `Ok`); `a_refusing_entry_records_itself_and_returns` stays green, since its outer record is
  `None` anyway.
* **C-I3b**, `run` not forgetting the pending condition: the same nested test alone red, on
  `pending`.
* **C-I4**, `stub` without the style check: `a_classic_row_publishes_no_signature_and_calls_nothing`
  alone red; `a_classic_routine_is_refused_before_its_stub_is_entered` green.

Run (`fix1/mutate.sh`, each file restored from a copy and `cmp`-checked; outputs `ctl/c-i2.txt` etc.):
**all four confirmed.** C-I2: exactly the two tests, stderr `RexxInstanceInterface.InterpreterVersion
is not implemented (Phase 8)` (826 passed, 1 failed in `rexx-exec --lib`; 26/1 in `rexx-api --lib`).
C-I3a: the nested test alone, outer `Ok(Some(ObjRef(1)))`. C-I3b: the nested test alone, pending
`Some(40001)`. C-I4: the classic-row test alone, signature `Some([12, 15])`.

Before committing items 2 to 4: `cargo fmt --all --check` exit 0; `cargo clippy -j 4 --workspace
--all-targets -- -D warnings` exit 0; `cargo test -p rexx-api` all green; Miri, Stacked Borrows,
`cargo miri test -p rexx-api --lib --locked` from the worktree: 24 passed, 0 failed, 3 ignored, exit
0 (`fix1/miri-boundary.txt`), the new record, nesting, instance and classic-row tests among the passes.

**Items 2 to 4 committed** as `fa7c51301` (gates running on its archive copy while item 5 goes on).

## Item 5 (integration I1): the name a kept call site passes

`Op::CallArgs` (`ir/drive.rs`, `run_call_args`) leaves the spelling empty on a site-table hit, which
only a builtin's dispatch does not read. The other call ops read the name off their instruction or
node on every run. Predictions for the reviewer's `g1`-`g6` and `f1`, copied into `fix1/`, before
running at `fa7c51301`: `g1` (merged) and `g2` (`loadLibrary`) print `2` and then the traceback
`*-* Compiled routine "".` on the crate against `"RXCALCSQRT"`; `g3` (`::ROUTINE`) `""` against
`"SQ"`; `g4` (`LIBRARY REXX`) `""` against `"FS"`; `g5` and `g6` a second `event CALL ` with no name
for both `RxCalcSqrt` and `filespec`; `f1` identical (head keeps no `External`). After recovering the
spelling on a hit for every resolution but a builtin: all identical on three descriptors.

At `fa7c51301`, run: every prediction confirmed (`g1`-`g4` `""`, `g5`/`g6` a nameless second event,
`f1` identical). Fix: `run_call_args` reads the spelling off the node on a hit for every resolution
but `Resolved::Builtin`. After it: `g1`-`g6`, `f1` identical on three descriptors. The checkpoint half
predated the task for internal routines (`g6`, the reviewer's base run) and is the same code, so it is
fixed with the rest. Witnesses `library_routine_site_twice_{merged,global,directive,rexx,security}.rex`,
each calling its route twice from one expression site and failing on the second (the security one
auditing a library and an internal routine twice each), all identical.

Control C-I5, prediction before running: the hit branch restored to leaving the spelling empty
reddens exactly the five `site_twice` witnesses in the `phase-8.txt` library sweep.

Run: **confirmed**, exactly those five red of the sweep's 54 rows (`ctl/c-i5.txt`), `drive.rs`
restored and `cmp`-checked.

**Item 5 committed** as `6245fe1ca`.

## Item 6 (integration I3 and M1): keep an external file at a call site, invalidated

(Its kept-resolution rule is superseded by fix round 2's item 1: `MergedLibraryRoutine` is kept for
good there, and `~addRoutine` moves the generation.)

Built: a call site keeps every resolution but `Unresolved` again, `External` included, each stamped
with `Interp::routine_generation`; `Resolved::can_be_shadowed` names the kinds found after the
running package's own routines (`Library`, `Internal`, `LibraryRoutine`, `MergedLibraryRoutine`,
`External`), and a kept one of those is dropped once the generation has moved. As first built, the
generation moved in `register_package_routines` (every library load that settles) and in
`Package~addPackage` after its merge, the same mechanism for integration M1; where it moves in the
committed version is two paragraphs down. The `drive.rs` comments that said nothing invalidates a
kept answer now say the table drops a stale one.

Predictions, before running the new binary, for the reviewer's `c2`, `c4`, `c5`, `f1`, `f2` (copied
into `fix1/`) and `library_routine_after_external_file`: all identical on three descriptors (`c4`
`2 pk 16`, `c5` `2 pk N`, `c2` the traceback `Compiled routine "rxcalcsqrt"`, `f1` `3 3`).

Run, first build: `f1`, `f2`, `library_routine_after_external_file` identical; **`c2`,
`c4` and `c5` still diverged** (`2 4`, `2 b.c`, `"RXCALCSQRT"`). Their packages arrive through
`loadPackage`, which imports and merges before `addPackage` is sent, so `addPackage` merged nothing
and moved nothing. The generation now moves inside `merge_required` and `merge_library`, and only
when a merge inserts a name: a called external file's public routines merge into the caller after
every call, so moving it unconditionally would drop a kept `External` on every call. The move in
`Package~addPackage` itself is gone (its merge is `merge_required`), and `register_package_routines`
moves it once per routine it registers. After that:
`c2`, `c4`, `c5`, `f1`, `f2`, `library_routine_after_external_file` all identical.

Witnesses: `library_routine_external_site_twice.rex` (an external file from one expression site and
one `CALL` site, three passes each) and `library_routine_site_after_add_package.rex` (a kept
`LibraryRoutine` and a kept `Internal` shadowed by `addPackage` of a `loadPackage`d public routine),
both identical; the second diverges on the `fix1/bench/bin/rexx-run.before` binary (`6245fe1ca`).

Controls, predictions before running:
* **C-I6a**, `can_be_shadowed` answering `false`: red `library_routine_site_after_add_package` and
  `library_routine_after_external_file` in the sweep; `library_routine_external_site_twice` green.
* **C-I6b**, `merge_required` never moving the generation: red `library_routine_site_after_add_package`
  alone.

Run: **both confirmed** (sweep of 56 rows; C-I6a red exactly `library_routine_after_external_file` and
`library_routine_site_after_add_package`, C-I6b red exactly the latter); files restored and
`cmp`-checked.

**Item 6 committed** as `25bdb2dd5`, with `cae7d5382` re-deriving `refusal-sites.tsv` (line
columns only, checked by counting each changed row's name twice).

## Item 7 (integration I2): one ordered merged routine lookup

`PackageClass::processInstall` (`interpreter/classes/PackageClass.cpp:1227-1260`) installs every
`::REQUIRES ... LIBRARY` before any `::REQUIRES`, and `mergeLibrary` (`:756`) and `mergeRequired`
(`:693`) both add to `mergedPublicRoutines` without replacing; `findPublicRoutine` (`:852`) and the
namespace-qualified call (`instructions/CallInstruction.cpp:451`,
`expression/ExpressionQualifiedFunction.cpp:171`) read that one table, and `importedRoutines` answers
it. Predictions for the reviewer's `a1`-`a4` and `b3b` (copied into `fix1/`), and two new probes,
before running at `cae7d5382`:
* `a1`, `a2`, `a3`: oracle `a 4` / `b 5`, crate `a pk 16` / `b pk 25` now; identical after.
* `a4`: `pk` on both, before and after.
* `b3b`: crate `main importedRoutines The NIL object` (and in `mid.cls`) against `a Routine`; identical
  after.
* `i7n` (`::requires 'mid.cls' namespace m`, `mid.cls` requiring rxmath LIBRARY): oracle `qualified 4`,
  `call 5`, `mid imported a Routine`, `trapped 88.921`, then an uncaught 88.921 whose traceback names
  the routine as the call wrote it; the crate now 43.902 at the first line (the namespace lookup reads
  only `::ROUTINE`s); identical after.
* `i7e` (`::requires 'nosuchpkg.cls'` before `::requires 'zorkolib' LIBRARY`): oracle 98.903 naming
  `zorkolib`, the library installed first; the crate now 43.901 naming the package file; identical
  after.

At `cae7d5382`, run: every prediction confirmed (`a1`-`a3` `pk`, `a4` `pk` on both, `b3b` `.nil`,
`i7n` 43.902, `i7e` 43.901; the oracle's uncaught `i7n` traceback names `"RXCALCSQRT"`, the qualified
symbol upcased). Built: `merged_public_routines` holds `MergedRoutine::{Installed, Library}`, and the
separate merged-library table is gone; `install_requires` walks the `::REQUIRES ... LIBRARY`
directives, then the rest, each in source order; `merge_library` and `merge_required` add through one
`merge_routines`, which never replaces and moves the generation when it adds; the call
(`package_routine_lookup`), `findRoutine`, `importedRoutines` and `namespace_routine` read that one
table. After it: `a1`-`a4`, `b3b`, `i7n`, `i7e` identical; the 56-row library sweep and `c2`, `c4`,
`c5`, `f1` still identical.

Witnesses, each identical here and each diverging on the item 6 binary:
`library_routine_merge_order.rex` (`a1` in the program, `a3` and `a4` as two required packages),
`library_routine_imported.rex` (`b3b`), `library_routine_namespace.rex` (`i7n`),
`library_requires_install_order.rex` (`i7e`, the directives in a required package).

Controls, predictions before running:
* **C-I7a**, `install_requires` walking packages before libraries: red `library_routine_merge_order`
  (`main pk 16 pk 25`) and `library_requires_install_order` (43.901); `_imported` and `_namespace`
  green.
* **C-I7b**, `merge_routines` replacing an existing name: red `library_routine_merge_order` (both the
  program's line and `public first`); what else in the sweep I do not predict, and record.

Run: **C-I7a confirmed** (exactly `library_routine_merge_order` and `library_requires_install_order`
of 60 rows). **C-I7b partly falsified**: `library_routine_merge_order` alone red, as predicted, but
its program line stayed `main 4 5` (the last merge into the program is `pubfirst.cls`'s, whose own
replacing merge had ended on the library routine); the red lines were `library first pk 16` and
`public first 4`. Nothing else in the sweep went red.

### Item 6's callgrind comparison

`fix1/bench/run.sh`: callgrind `Ir` totals, `rexx-run.before` (`6245fe1ca`, sha256 `e0c7658a0ab03a5c…`)
then `rexx-run.after` (the tree committed as `25bdb2dd5`, sha256 `576f55c61dfc25f5…`) per program, two
rounds, a fresh run directory each (`fix1/bench/results.txt`). Per call is (400 - 200) / 200.

| program | before r1 | after r1 | before r2 | after r2 |
|---|---|---|---|---|
| `call ext i`, per call | 1,064,166 | 664,907 | 1,065,084 | 664,220 |
| `x = ext(i)`, per call | 1,065,472 | 663,636 | 1,064,327 | 664,450 |
| `filespec(...)`, per call | 1,510 | 1,611 | 1,652 | 1,551 |
| `RxCalcSqrt(i)`, per call | 9,612 | 9,551 | 9,587 | 9,602 |
| `rexxcps` total | 21,234,285,786 | 21,258,661,220 | 21,238,549,026 | 21,258,068,357 |

An external-file call costs 37.5% fewer instructions than at `6245fe1ca` (the reviewer's base figure
was 657,860 per call, so about 1% above it now). The internal and library routine rows move by less
than their round-to-round spread. `rexxcps` is +0.11% and +0.09% in instructions, under the layout
noise this tree measured (`dispatch` alone spans several percent with dead code); not evidence either
way.

**Item 7 committed** as `2804a8986`.

**Gates for `fa7c51301`** (items 2 to 4). A first run with the copy's `rust/target` linked to the
scratch target but `CARGO_TARGET_DIR` unset reported `559 of 572` and a `package_requires` failure:
every mismatch was a path under the link, which the oracle resolves and the harness does not
(`fix1/gates/corpus-fa7c51301-symlinked-target.txt`); `package_requires` passes 24 of 24 in the
worktree at `2804a8986`. The script now sets the variable as well as the link. Re-run: gated corpus
exit 0, `572 of 572 matching`; release tests of the four crates exit 101 with the BASE
`collect_stress` L0 failure alone.

## Boundary M1: the signature refusal a routine reports

A forged routine library, `fix1/ext/libforgesig.so`, built from `fix1/ext/forgesig.cpp` with
`g++ -shared -fPIC -std=gnu++11 -I api -I api/platform/unix` (`readelf -d` lists no `NEEDED`): two
typed routines declaring `CSELF` and `SCOPE`, which `processArguments` refuses outside a method
through `reportSignatureError` (`interpreter/execution/NativeActivation.cpp:190-193`,
`Error_Incorrect_call_signature` 40918 at `RexxErrorCodes.h:408`). Predictions before running, both
sides with that directory on `LD_LIBRARY_PATH` (`fix1/ab-ext.sh`): `m1a` (`ForgeSelf()` merged from a
required package's `::REQUIRES 'forgesig' LIBRARY`) on the oracle 40.918 `Incorrect call to routine.`
naming `main.rex line 2`, under `Compiled routine "FORGESELF"`; the crate today 93.968 naming the
same line. `m1b` (`sq()`, a `::ROUTINE ... EXTERNAL "LIBRARY forgesig ForgeScope"` in `pk.cls`):
oracle 40.918 naming `pk.cls` with no line; crate 93.968 the same way.

Run: `m1a` as predicted (oracle 40.918 rc 216, crate 93.968 rc 163, same line and traceback). `m1b`
**falsified on the crate side**: `SCOPE`'s row is not filled, so the crate refuses loudly on the
`Unfilled` row (rc 120) before any signature question; the oracle is 40.918 naming `pk.cls` with no
line as predicted. `m1b` now binds `ForgeSelf` instead, and a third routine, `ForgeOptional`
(`RexxRoutine0(OPTIONAL_int, ...)`, a result word carrying the optional bit), probes the after-call
half. Predictions: `m1b` oracle 40.918 `pk.cls` no line, crate 93.968 the same way; `m1c` oracle
40.918 naming `main.rex line 2` (the result refusal carries the sending line, as the method's does),
crate 93.968 the same way.

Run: `m1b` and `m1c` as predicted. Fix: the host's `refusal` takes the native frame's `method` flag
and builds both signature refusals through `Failure::error_number(method)`, one constructor
`Raised::incorrect_native_signature(number, before_call)` replacing the two method-only ones. After
it, `m1a`-`m1c` identical on three descriptors. The forged library cannot be a corpus witness (the
corpus loads the oracle's own extensions), so the witness is the unit test
`dispatch::library::tests::a_signature_refusal_is_numbered_for_a_method_or_a_routine`.

Control C-M1, prediction before running: `refusal` passed `true` for `method` at its one call reddens
that test alone in `rexx-exec --lib`.

Run: **falsified**, 828 passed, nothing red: the test called `refusal` directly and so never read the
frame's flag the mutant replaced. The test now goes through `settle_native_call` with a frame built
for each flag. Same prediction, re-run:

Re-run: **confirmed**, that test alone red (827 passed, 1 failed), the file restored and
`cmp`-checked. `refusal-sites.tsv`: the two method-only constructors gave way to
`incorrect_native_signature`, whose row is `agrees` / `yes` / `40.918` over the forged `ForgeSelf()`
probe; it builds both numbers with literal `syntax(M, N)` calls so the table's identifier check can
read them, and the `93.968` pair left `SHARED_ANSWERS` and the header paragraph together.

## Integration M2, M3, M5 and the Step 6 gap

* **M3 and the Step 6 gap**: `library_routine_attribute_directive_load.rex` (the reviewer's `h1`, a
  `loadPackage`d `::ATTRIBUTE ... GET EXTERNAL "LIBRARY rxmath RxCalcSqrt"` refused 90.998 and the
  load it made leaving `RxCalcSqrt` callable) is identical on three descriptors and joins the load-site
  block; that block no longer lists the sites in prose, and the refusal block says "under the name
  the call passed", true since item 5.
* **M2**: `phase-4-exclusions.txt`'s entry is "BUILT", and names what still refuses on the routine
  path by owner without a count: classic rows and `REGISTERED` (Phase 10), unfilled conversion rows
  (the surface plan's Task 3, `MathLoadFuncs`/`MathDropFuncs`), unwritten interface members (Task 5,
  `RxCalcSin` with a bad units argument, now a loud refusal), with the members that still abort.
* **M5**: `library_procedure_gone`'s witness column names the routine-side program too.

## Recorded, not fixed (as ruled)

* `Directory~setMethod` and `self~setMethod` of a `loadExternalMethod` answer refuse naming Phase 5
  (the reviewer's `j2`, `j3`).
* A condition object's `TRACEBACK` lacks the `*-* Compiled routine` line, for internal and library
  routines alike, the same at BASE (`c3`).
* A parse error inside a loop renders as `rexx-exec: 47.2` rc 120 where the oracle reports `Error 47`
  rc 209 (the reviewer's malformed `b1`).
* **The rxapi daemon**: nothing in this round registered anything with it; the earlier
  `ZZT2REGPROBE` stays recorded in the ledger.

**Minor group committed** as `79f08bed6`.

**A gate that did not gate its tree.** `6245fe1ca`'s and `2804a8986`'s gate runs compiled nothing
(`grep -c Compiling` 0 in both logs) and reported the corpus count of the tree before them (572 and
579 where their own lists are longer): `git archive` stamps each extracted file with its commit time,
which the previous revision's newer artifacts in the shared target directory shadowed, so each ran
the earlier revision's binaries against the earlier revision's `CARGO_MANIFEST_DIR`. Those logs are
renamed `*-stale-binary.txt` and are not gates. The runs that did compile (`a873b0678` 72 compile
lines, `fa7c51301` 4 with its new tests in the log, `cae7d5382` 9 with 579 corpus rows) stand. The
script now gives each revision its own target directory, and the invalid ones are re-run.

`cae7d5382`'s valid run found a real failure: `a_sidecar_changes_what_one_of_the_interpreters_answers`,
`library_routine_site_twice_rexx.rex`'s `LD_LIBRARY_PATH` sidecar is not load-bearing (that witness
loads no library). Removed in `bd64f3197`.

**Re-run gates**, each revision in its own target directory (`fix1/gates/status-<sha>.txt`), each log
with 72 `Compiling` lines, and each corpus count the one before it plus the witnesses the revision
adds (572 at `fa7c51301`, +5 site-twice, +2 item 6, +4 item 7, +1 attribute):

| revision | gated corpus | release tests of the four crates |
|---|---|---|
| `6245fe1ca` | `577 of 577 matching` | exit 101: the BASE L0 test; the sidecar test |
| `2804a8986` | `583 of 583 matching` | exit 101: the same two |
| `79f08bed6` | `584 of 584 matching` | exit 101: the same two |
| `bd64f3197` | `584 of 584 matching`, exit 0 | exit 101: the BASE L0 test alone (80 result lines ok, 1 failed) |

The sidecar failure at the three earlier revisions is the one `bd64f3197` removes
(`lang/library_routine_site_twice_rexx.rex: neither interpreter answers differently without its
Variable("LD_LIBRARY_PATH")`); at the first three the corpus binary's exit 101 is that test, with
every program matching. The L0 failure (`collect_stress`, `dispatch.rs:1510`) predates the task.

## Concerns (fix round 1)

* `RxCalcSin(30, 3, 'X')` is a loud rc 120 naming `NewStringFromAsciiz`, not the oracle's 88.916:
  Task 5's members.
* `orxmethod`'s version test reaches the instance and then refuses on Task 3's `size_t` row, so it has
  no corpus row.
* The call-site generation is conservative: any added routine or merge that inserts a name drops every
  kept shadowable resolution. It does not mark `Resolved::Routine` shadowable, which is right for the
  package's own `::ROUTINE`s; a routine found through a parent package is outside what I measured.
* `rexxcps` +0.11% / +0.09% in instructions after item 6, under this tree's measured layout noise.
* The reviewer's M4 citation `NumberStringClass.cpp:704` is correct in this tree; left as it was.
* Two gate runs (`6245fe1ca`, `2804a8986`) ran an earlier revision's binaries through a shared target
  directory; caught by their `Compiling` count and re-run per revision. The valid `cae7d5382` run
  found a real sidecar failure, fixed in `bd64f3197`.
* The 40.918 measurement rests on a forged library in scratch (`fix1/ext/libforgesig.so`), which the
  corpus cannot carry; the witness is a unit test.
* The earlier `ZZT2REGPROBE` rxapi registration (before this round) stays recorded in the ledger;
  this round registered nothing.

# Fix round 2

Base `bd64f3197`. The re-reviews: `task-2-rereview-integration.md` (probes `scratchpad/t2-rereview-b/`)
and `task-2-rereview-boundary.md` (probes `scratchpad/t2-rereview-a/`). Scratch for this round:
`scratchpad/surface-t2/fix2/`, a fresh directory per probe. Nothing is registered with the rxapi
daemon.

## Item 1 (integration C1 and I-A): the oracle's rule for a kept call site

Printed before building: `RexxExpressionFunction::evaluate` (`expression/ExpressionFunction.cpp:179-215`)
and `RexxInstructionCall::execute` (`instructions/CallInstruction.cpp:159-198`) evaluate the arguments,
then call through `externalTarget` when it is set, else a label (`target`, set at resolve time from
the labels table), else a builtin (`builtinIndex`), else `externalCall(resolvedTarget, ...)` followed by
`setField(externalTarget, resolvedTarget)`. `RexxActivation::externalCall`
(`execution/RexxActivation.cpp:3062-3105`) writes its `routine` reference once, at Step 2
(`routine = settings.parentCode->findRoutine(target)`); Steps 2a, 2b, 3 and 4 do not take it, so they
leave the null Step 2 wrote. The rule holds as ruled: a `findRoutine` hit is kept at the instruction
for good, a label and a builtin are fixed at resolve time, and a Step 3 answer is looked up again on
every call. In this crate's terms: `Label`, `Builtin`, `Routine` and `MergedLibraryRoutine` are kept
unconditionally; `Internal` (the REXX package's routine table), `LibraryRoutine` (a registered
library routine), `External` and `Library` are Step 3 answers, kept only while nothing that could put
a Step 2 answer in front of them has happened.

Predictions for the reviewer's probes (copied into `fix2/p/`, `fix2/ab.sh`: oracle, head, and the
`bd64f3197` binary as base, each in the same run path), written before running; head and base are
the same binary until the change:
* `x1` (kept `Internal`, then `addRoutine('FILESPEC')`): oracle `1 b.c` / `2 added` / `fresh added`;
  crate `2 b.c`.
* `x2` (kept `LibraryRoutine`, then `addRoutine`): oracle `2 added`; crate `2 4`.
* `x3`, `x3c` (kept `External`, function and `CALL`): oracle `1 ext 1` / `2 added` / `fresh added`;
  crate `2 ext 2`.
* `x4` (kept `MergedLibraryRoutine`, `addRoutine` and an unrelated `loadPackage`): oracle `1 4` / `2 4`
  / `fresh added`; crate `2 added`.
* `x14` (kept `Routine` replaced by `addRoutine`): SAME.
* `p3` (kept merged library routine of the parent, then `loadPackage` merging a Rexx `RxCalcSqrt`):
  oracle `1 4` / `2 4` / `fresh pk 16`; crate `2 pk 16`.
* `x5c`: oracle `main 16`, crate `4`. `x5d`: oracle `main 16`, crate `pub 16`. `r5`: oracle `mainr a
  Routine` / `sqrt main 16` / `call main 16`; crate `mainr The NIL object` / `sqrt 4` / `call 4`.
* `m1b`: oracle `library 1 1` / `routine 1 1`; crate `library 0 0` / `routine 1 1`.
After items 1 to 3: head SAME on all three descriptors for every one.

At `bd64f3197`, run (`fix2/before.txt`): **every prediction confirmed**, head and base alike.

Built: `Resolved::can_be_shadowed` becomes `kept_until_routines_change`, answering `true` for
`Library`, `Internal`, `LibraryRoutine` and `External` alone, so `Routine` and `MergedLibraryRoutine`
are kept for good; `install_routine` (`~addRoutine`, `~addPublicRoutine`) moves the generation through
a new `Interp::routines_changed`, which the two existing moves now call too. Prediction for item 1
alone, before running: `x1`, `x2`, `x3`, `x3c`, `x4`, `x14`, `p3` head SAME; `x5c`, `x5d`, `r5`, `m1b`
unchanged.

Run: **confirmed**, `x1`-`x4`, `x14`, `p3` head SAME, the other four unchanged (`fix2/item1.txt`).

Witnesses, predictions before running them: `library_routine_site_add_routine.rex` (`x3` through an
expression site and `x3c` through a `CALL` site, plus `x2` through `~addPublicRoutine` and `x1`, each
in a loop across the event, with a fresh site after the first) head SAME, base DIFF on stdout
(`function 2 ext 2`, `call 2 ext 2`, `library 2 4`, `internal 2 b.c`);
`library_routine_site_kept_merged.rex` (`x4` in the program, `p3` in a `newFile`d `r.rex` over
`RxCalcPower`) head SAME, base DIFF (`own 2 added 16`, `parent 2 pk 2 3`).

Run: **both confirmed** (base DIFF exactly on those lines). Sweep (`fix2/sweep.sh`, the oracle cached per
witness, the crate in the same run path, three descriptors) over the 63 `library_*`/`external_method*`
rows of `phase-8.txt`: head 0 red; the `bd64f3197` binary red exactly on the two new witnesses.

Controls for item 1, predictions written before running (`fix2/csweep.sh`, mutation profile, file
restored and `cmp`-checked, then the sweep):
* **C-R1a**, `MergedLibraryRoutine` back in `kept_until_routines_change`: red
  `library_routine_site_kept_merged` alone.
* **C-R1b**, `install_routine` not moving the generation: red `library_routine_site_add_routine` alone.
* **C-R1c**, `External` out of the set (kept for good): red `library_routine_site_add_routine` and
  `library_routine_after_external_file`; `library_routine_external_site_twice` green.

Run (`fix2/ctl/item1-controls.txt`): **all three confirmed**, exactly the predicted rows red.

The sweep of writes, before listing it: `.ROUTINES` answers `settings.parentCode->getRoutines()`
(`execution/RexxActivation.cpp:2877-2879`), the package's own `routines` table, which
`findLocalRoutine` reads. Prediction for `w1` (a kept `External` `ext`, then `.routines~put(r,
'EXT')`, then a fresh site), before running: oracle `1 ext 1` / `2 put 2` / `fresh put 3`; crate, head
and base, `2 ext 2` / `fresh ext 3` (the string table's entry is not the lookup's).

Run `w1`: **confirmed**, head and base `2 ext 2` / `fresh ext 3` against the oracle's `2 put 2` /
`fresh put 3`. The fresh site is wrong too, so this is not the kept site: a `.ROUTINES` entry put from
Rexx never reaches `Interp::routines`. Pre-existing; recorded under "Recorded, not fixed" below.

**Every write to a table the call's lookup reads**, from `grep -rn` over `rexx-exec/src` for
`.routines`, `package_public_routines`, `merged_public_routines`, `package_routines` and
`package_parents` (tests excluded), each writer read:

| table | writer | moves the generation | why |
|---|---|---|---|
| `routines`, `package_public_routines` | `install_directives` (`lib.rs`) | no | every caller (`run_loaded`, `new_file_executable`) passes a `ProgramId` pushed just before, so no code of that package has run and no call site reads its tables yet; a package that could read them through a merge gets them through `merge_routines` |
| `routines`, `package_public_routines` | `install_routine` (`~addRoutine`, `~addPublicRoutine`) | **yes, new** | an existing package's table |
| `merged_public_routines` | `merge_routines`, behind `merge_required` (`::REQUIRES`, `~addPackage`, `~loadPackage`, a called file's public routines) and `merge_library` (`::REQUIRES ... LIBRARY`, `install_requires` its one caller) | yes, when a name is added | add-if-absent, so a merge that adds nothing changes no answer |
| `package_routines`, `package_routine_codes` | `register_package_routines` | yes, per routine | a library load, from every load site (`~loadLibrary` included) |
| `package_parents` | `package_from_source`, `new_file_executable` | no | inserted for the `ProgramId` just pushed, before its code runs |
| the package's `.ROUTINES` string table | `install_routine`, and any Rexx `put` on it | not a lookup table here | `w1`: the lookup never reads it |

The sweep above once read `rows 63 red 63` just before the item 1 commit: I had passed the binary as
a relative path, and each run `cd`s into its witness directory (`timeout: failed to run command
'target/release/rexx-run'`). Re-run with the absolute path: 0 red. `sweep.sh` now refuses a relative
binary.

Before committing: `cargo fmt --all --check` exit 0; `cargo clippy -j 4 --workspace --all-targets --
-D warnings` exit 0; `cargo test -p rexx-exec --lib` 828 passed; `refusal_sites` red on moved
locations, re-derived (`REXX_REFUSAL_SITES_REFRESH=1`), column 4 of 19 rows changed and nothing else
(`python3` over `git show HEAD:` against the file), then 5 passed.

**Item 1 committed** as `eab19ac3b`; its gate and the callgrind comparison (`fix2/bench/`, `bd64f3197`
against `eab19ac3b`) running on their own copies.

## Item 2 (integration I-B): the parent chain's routines before any merged table

Predictions, written before building, for the copied `x5c`, `x5d`, `r5` (above) and the reviewer's
`p1`, `p2`, `r1`: at `eab19ac3b` `p1` head `fresh pub` against the oracle's `fresh main`, `p2` and `r1`
SAME; after the change, all six SAME.

Built: one `Interp::find_routine(program, upper)` (`environment.rs`) walks `routines` up the parent
chain, then `merged_public_routines` up the chain, answering the `MergedRoutine` found;
`package_routine_lookup` (the call) and `package_find_routine` (`~findRoutine`) both read it, so the
two can no longer disagree. The old walks (own `routines`, own merged, then the parent's pair; and
`findRoutine`'s own, with no parent at all) are gone. `package_public_routines` is not read: a public
routine is in `routines` in both implementations (`addInstalledRoutine` writes both, and so does the
directive install here).

Run: **confirmed**, all six head SAME (`fix2/item2.txt`). The "before" half ran on the `bd64f3197`
binary, whose lookup order item 1 does not touch: `x5c`, `x5d`, `r5`, `p1` DIFF as predicted, `p2`,
`r1` SAME.

Witness, prediction before running: `library_routine_parent_before_merged.rex` (`x5c`, `x5d` and `r5`
in one `newFile`d `r.rex`, both through a call and through `findRoutine`, plus a merged library
routine no parent shadows): oracle and head `library main 16` / `public main 16` / `findRoutine a
Routine` / `findRoutine library main 16` / `findRoutine public main 16` / `merged 8`; base DIFF
(`library 4`, `public pub 16`, `findRoutine The NIL object`, `findRoutine library 4`, `findRoutine
public pub 16`).

Run: **confirmed**, head SAME, base DIFF on exactly those five lines.

Controls for item 2, predictions written before running (read first: `library_context_package`'s
`.Package~new('fromsource3', 'call pkroutine', m)` and `rf2.cls` reach `pk.cls`'s own routines through
the parent; `library_package_discarded`'s context package declares nothing that survived):
* **C-R2a**, the `routines` walk reading the running package alone: red
  `library_routine_parent_before_merged` and `library_context_package`; nothing else in the sweep.
* **C-R2b**, the merged walk reading the running package alone: red `library_routine_site_kept_merged`
  alone (`r.rex`'s `RxCalcPower` falls to the registered library routine, which the `loadPackage`
  merge then shadows: `parent 2 pk 2 3`).

Run (`fix2/ctl/item2-controls.txt`, and `c-r2a-rerun.txt.red/` for C-R2a's outputs): **C-R2b confirmed**.
**C-R2a partly falsified**: the two predicted rows red (`library_context_package` rc 213, 43.1 `Could
not find routine "PKROUTINE"` in `fromsource3`), and a third, `library_package_discarded`, whose second
half I had not read: its `pk2.cls` does translate, and `.Package~new('fromsource', 'call pkr2', later)`
reaches `pkr2` through the parent (`Package~new context raised 43.1` against `pkr2 ran`). The sweep now
keeps each red row's outputs, since the second control's run had overwritten the first's.

Before committing: fmt check exit 0, clippy exit 0, `rexx-exec --lib` 828 passed, `refusal_sites`
re-derived (column 4 of 19 rows alone) then 5 passed, the 64-row sweep 0 red on the rebuilt binary.

**Item 2 committed** as `c08f4badd`.

### Item 1's cost, re-measured

`fix2/bench/run.sh`: callgrind `Ir`, `bd64f3197` (`rexx-run.before`, sha256 `b4b2dfb9…`) then
`eab19ac3b` (`rexx-run.after`, `714dceda…`) per program, two rounds, a fresh run directory each, every
rc 0 (`fix2/bench/results.txt` ends `finished`). Per call is (400 - 200) / 200.

| program | before r1 | after r1 | before r2 | after r2 |
|---|---|---|---|---|
| `call ext i`, per call | 664,466 | 663,738 | 663,876 | 663,564 |
| `x = ext(i)`, per call | 663,793 | 664,176 | 664,284 | 664,074 |
| `rexxcps` total | 21,258,593,775 | 21,262,224,072 | 21,258,188,351 | 21,262,044,532 |

An external-file call costs what it did at `bd64f3197` (within 0.1% both ways, both forms): the kept
`External` survives, since a called file's merge adds no name after its first call. `rexxcps` +0.017%
and +0.018%, under this tree's layout noise.

## Item 3 (integration m2): one `Routine` object per library routine an imported table answers

`merged_routine_object` builds a fresh `Routine` for a library entry on every ask and records it in
`executable_sources`, which nothing prunes. The oracle's `mergeLibrary` merges the library package's
own routine objects, one per routine the library exports, so every ask in every importing package
answers the same one.

Predictions, written before building, for `g2` (400,000 `importedRoutines` sends under `::requires
'rxmath' LIBRARY`, a line every 100,000), `g2f` (the same over `findRoutine('RXCALCSQRT')`) and `m1b`
(above), all under the oracle's cap: at `c08f4badd` `g2` oracle rc 0 with `100000`..`400000` / `done`,
crate rc 134 with stdout short of `done`; `g2f` SAME, rc 0; `m1b` crate `library 0 0`. After one object
per `library_codes` row: all three SAME, and the crate's peak RSS on `g2` and `g2f` (`/usr/bin/time -v`
on the binary, uncapped) under 40 MB each.

Built: `Interp::library_routine_objects`, one `Routine` per `library_codes` row, rooted under
`RootSet::add_global` as `program_routine_objects` is, which `merged_routine_object` answers from
the second ask on. After it: `g2`, `g2f`, `m1b` head SAME on three descriptors (`g2` rc 0 under the
cap). **The RSS half of the prediction is falsified for `g2`**: 124,108 kB peak (1.38 s), `g2f` 24,788
kB. Prediction for the question that decides whether that is growth or the collector's working set,
before running: the same loop at 100,000 and at 800,000 sends peaks within 10% of 400,000's.

Run: **confirmed**, 124,964 kB at 100,000 sends and 124,832 kB at 800,000 (both rc 0, `done`): the
collector's working set for a loop building a table per send, not growth. The `bd64f3197` binary peaks
at 238,972 kB already at 100,000 sends.

Witnesses, predictions before running them:
* `library_routine_object_identity.rex` (`m1b`, plus `findRoutine` in a required `mid.cls` that
  requires the same library): oracle `1` on all five lines; head SAME; base DIFF on the three library
  lines (`0`), SAME on the two `::ROUTINE` lines.
* `tests/library_routine_memory.rs`, `imported_routines_sent_in_a_loop_run_under_the_oracles_memory_cap`:
  the crate's binary under `ulimit -v 1048576` over `g2`, asserting rc 0, stdout `100000`..`400000`
  / `done`, empty stderr. The corpus cannot carry this: it caps the oracle's process and runs the crate
  in-process, uncapped. Green on the head tree; red under C-R3a below.

Run: the identity witness **confirmed** (base `0` on the three library lines); the capped test green on
the head tree (debug, 1 passed).

Control for item 3, prediction written before running: **C-R3a**, `merged_routine_object` without its
cache read (a fresh object every ask, as before): the sweep red on `library_routine_object_identity`
alone; `imported_routines_sent_in_a_loop_run_under_the_oracles_memory_cap` red on rc 134 with stdout
short of `done` (`fix2/ctest.sh`, the test binary built with the mutant in its own target directory).

Run (`fix2/ctl/item3-controls.txt`): **confirmed**, the sweep red on the identity witness alone (65
rows), and the capped test red: `memory allocation of 545259536 bytes failed`, stdout empty. The
status is a signal rather than 134 (`exec` hands the abort to the test's `Command` directly, so
`code()` is `None`); the prediction's number was the shell's rendering of the same abort.

**Item 3 committed** as `a15a8603a` (fmt, clippy, `rexx-exec --lib` 828, `refusal_sites` 5 with nothing
to re-derive, the 65-row sweep 0 red). Gate for `eab19ac3b`: 72 `Compiling` lines, gated corpus exit
0 `586 of 586 matching`; release tests exit 101 on the BASE `collect_stress` L0 test alone.

## Item 4 (boundary I-A): `DisplayCondition`'s failure value

Printed: `ThreadContextStubs.cpp:1948` `return Error_Interpretation/1000;`, `RexxErrorCodes.h:456`
`Error_Interpretation = 49000`. So 49; the round-1 line above ("`DisplayCondition` answers 48") was
wrong, and so were the slot and its test. Prediction, before running: with the test's assertion made
`49` and the slot still `failing 48`, `ffi::tests::a_refusing_entry_records_itself_and_returns` is red
on `(null, 0, 48)` against `(null, 0, 49)` and nothing else in `rexx-api --lib` is; with the slot
`failing 49`, green.

Run: **confirmed**, the one test red on `(0x0, 0, 48)` against `(0x0, 0, 49)` (26 passed, 1 failed),
then with the slot `failing 49` every `rexx-api` binary green. Miri, Stacked Borrows, `MIRIFLAGS`
unset, `cargo miri test -p rexx-api --lib --locked` from the worktree: 24 passed, 0 failed, 3 ignored,
exit 0, the changed test among the passes (`fix2/miri-item4.txt`). fmt and clippy exit 0.

**Item 4 committed** as `5d08b1bce`.

## Item 5: prose

* Integration m1: `Resolved::can_be_shadowed`'s doc, `routine_generation`'s doc, both `drive.rs`
  comments and `phase-8.txt`'s block were rewritten with item 1 (commit `eab19ac3b`), since each
  described the rule item 1 replaced; this report's round-1 item 6 section carries a note that its
  rule is superseded. Commit `25bdb2dd5`'s title stays as it is.
* m3: `double_of`'s doc states `Number::format`'s rule: exponential where the plain form would pad
  the integer part or put more zeros after the point than there are digits.
* m4: the four `run/tests.rs` tests loading `CARGO_MANIFEST_DIR/../../../build/lib` (the two from
  round 1 and two older ones with the same text) and `dispatch/library.rs`'s helper, which had the
  same sentence, now say they load this worktree's own `build/lib`, three directories above the crate,
  as the D5 amendment in `2026-09-14-phase-8-native-api.md` records; the helper is
  `worktree_library_directory`, and the two round-1 tests' docs say the measurements used the oracle
  checkout's build.
* Boundary M-A: `ffi.rs` cites `InterpreterInstanceStubs.cpp:106`. M-B: `load.rs` names the instance
  table's `LanguageLevel` stub at `:84`. M-C: the `clear_pending` comment covers a condition raised at
  any point in the call. M-D: `NativeRoutineEntry::stub`'s doc says a style that is neither classic
  nor typed is refused as an unreadable signature where the oracle calls it as typed.

Checks: fmt, clippy exit 0; `rexx-api` every binary green; `rexx-exec --lib` 828; `refusal_sites` 5
with nothing moved; Miri as above, 24 passed, 3 ignored, exit 0 (`fix2/miri-item5.txt`).

**Item 4 and item 5 committed** as `5d08b1bce` and `9ff11e465`.

## A final pass over the integration re-review's probes

Every `t2-rereview-b/p/` probe with a `main.rex` but `sig` (the forged-library ones need that
directory), copied into `fix2/rr/`, through `fix2/ab.sh` on the final binary. Prediction, before
running: head SAME on all but `x5`, `x5b`, `x5e` (`.Routine~new` with a context, Phase 5), `x12`
(`interpret "::requires"`'s traceback), `t6`, `t6b`, `t6c` (an external file's error without the
caller's line), `o1` (resolution before arguments), `x14s` (a call inside a method's arguments),
`m1` (`==` on a `Routine`, Phase 5), `e1` and `e2` (Task 3's `CSTRING` row and Task 5's member), each
pre-existing or owned elsewhere.

Run (`fix2/rr-final.txt`; my first count read no lines at all, because `d1`'s NUL byte made the file
binary to the `grep` wrapper, re-counted with `/bin/grep -a`): 46 probes, head SAME on 33. **Partly
falsified**: the twelve predicted DIFF, and a thirteenth I left out, `u1`, which the reviewer had
already recorded as a loud `StringTable~hasIndex` refusal naming Phase 5 (rc 120, `call added` kept),
the same on the `bd64f3197` binary. `e2` is SAME on stdout and DIFF on stderr and rc, as predicted.

## Recorded, not fixed (fix round 2, as ruled)

* A call inside a method's argument list (`.array~of(foo())[1]`) re-resolves on every pass (`x14s`).
* A function is resolved before its arguments are evaluated, where the oracle evaluates them first
  (`o1`).
* An error inside an external file loses the caller's traceback line (`t6`, `t6b`, `t6c`).
* `interpret "::requires ..."`'s 99.914 traceback lacks the directive's line (`x12`).
* `.Routine~new(name, source, context)` refuses naming Phase 5 (`x5`, `x5b`, `x5e`).
* `Refused::Unfilled` renders its owner twice (`... owes the FromNative conversion for
  REXX_VALUE_size_t (33) is not implemented (Phase 8)`); Task 3's rows retire the case.
* Found this round (`w1`, pre-existing, the same on `bd64f3197`): a `.ROUTINES` entry put from Rexx
  (`.routines~put(r, 'EXT')`) is never found by a call, where the oracle's `.ROUTINES` is the
  package's own `routines` table (`RexxActivation.cpp:2877-2879`) and the call finds it. Not a cache
  question: a fresh site misses it too.

## Gates (fix round 2)

`fix2/gates/gate.sh`, one `git archive` copy and one target directory per revision, as in round 1.
`eab19ac3b`'s run is above. **The scratch filesystem filled** while `c08f4badd`'s and `5d08b1bce`'s runs
went in two lanes beside the final callgrind comparison: `/tmp` is a 63 GB tmpfs, most of it other
sessions' scratch. `c08f4badd`'s corpus stage died on `rustc-LLVM ERROR: IO failure on output stream:
No space left on device`, `5d08b1bce`'s test stage on `os error 28`, and `a15a8603a`'s and
`9ff11e465`'s runs wrote nothing at all. None of those is a gate: the logs are renamed
`*-disk-full.txt`. I deleted round 1's finished gate targets and copies, `eab19ac3b`'s, and two
finished mutation target directories of mine (explicit paths), which left 18 GB free; the script now
deletes each run's own copy and target once its status is written, and the four re-run in one lane.

### The round's cost, end to end

`fix2/bench2/run.sh`, the same programs and interleaving, `bd64f3197` (`b4b2dfb9…`) against the final
tree `9ff11e465` (`d5d10646…`), every rc 0, `results.txt` ends `finished` (its rows were checked
complete after the disk filled, since callgrind wrote into the same filesystem):

| program | before r1 | after r1 | before r2 | after r2 |
|---|---|---|---|---|
| `call ext i`, per call | 664,349 | 664,444 | 664,106 | 663,570 |
| `x = ext(i)`, per call | 664,219 | 664,238 | 663,533 | 664,447 |
| `rexxcps` total | 21,257,673,305 | 21,263,261,638 | 21,259,230,056 | 21,262,937,196 |

External-file calls unchanged within 0.15% both ways; `rexxcps` +0.026% and +0.017%.

Re-run, one lane, each status ending `finished` with the free space it left:

| revision | `Compiling` lines | gated corpus | release tests of the four crates |
|---|---|---|---|
| `eab19ac3b` (item 1) | 72 | exit 0, `586 of 586 matching` | exit 101, the BASE L0 test alone |
| `c08f4badd` (item 2) | 72 | exit 0, `587 of 587 matching` | exit 101, the BASE L0 test alone |
| `a15a8603a` (item 3) | 72 | exit 0, `588 of 588 matching` | exit 101, the BASE L0 test alone; `imported_routines_sent_in_a_loop_run_under_the_oracles_memory_cap ... ok` |
| `5d08b1bce` (item 4) | 72 | exit 0, `588 of 588 matching` | exit 101, the BASE L0 test alone |
| `9ff11e465` (item 5) | 72 | exit 0, `588 of 588 matching` | exit 101, the BASE L0 test alone |

Each corpus count is the one before plus the witnesses the revision adds (584 at `bd64f3197`, +2, +1,
+1, +0, +0).

## Concerns (fix round 2)

* A `.ROUTINES` entry put from Rexx is never found by a call (`w1`), found while sweeping the writes
  item 1 asked for; pre-existing, recorded, not fixed.
* The kept-resolution rule now matches `externalCall`'s Step 2, but a Step 3 answer is still kept
  between routine-table writes rather than looked up on every call. The sweep of writers is the
  argument that nothing else changes a Step 3 answer; a file appearing on the search path does not
  move the generation, and a kept `External` searches for its path again when entered (`x8` SAME).
* C-R2a was partly falsified (a third witness red for a reason I had not read) and the final probe pass
  missed `u1` in its DIFF list; both recorded above.
* The disk-full episode voided four gate runs; all four re-ran clean. The scratch tmpfs is shared and
  mostly other sessions' data, so a later task gating several revisions at once can hit it again.
* `DisplayCondition`'s 48 came from my own arithmetic in round 1; the reviewer's derivation over every
  failure-path literal is what found it.

# Fix round 3

Base `9ff11e465`. The re-review: `task-2-rereview2.md` (probes `scratchpad/t2-rereview2/`). Scratch:
`scratchpad/surface-t2/fix3/`, a fresh directory per probe. Nothing is registered with the rxapi daemon.

A correction to fix round 2's concern 3: the `/tmp` space was this session's own scratch, not other
sessions' (the coordinator measured 39 GB of it); my "mostly other sessions' data" was not measured.

## Item 1 (C-1): a call's routine is resolved after its arguments

Printed before building. `RexxExpressionFunction::evaluate` (`expression/ExpressionFunction.cpp:179-215`):
`:185` `RexxInstruction::evaluateArguments(...)` ("evaluate the arguments first"), then `:190`
`externalTarget` if set, `:196` the label `target`, `:201` `builtinIndex`, else `:210`
`externalCall(resolvedTarget, ...)` and `:214` `setField(externalTarget, resolvedTarget)`.
`RexxInstructionCall::execute` (`instructions/CallInstruction.cpp:159-198`): `:166`
`evaluateArguments`, then the same four arms at `:171`, `:177`, `:182`, `:188-197`. The label is
fixed before the clause first runs (`RexxExpressionFunction::resolve`, `:154-167`, from the labels
table) and the builtin index when it is parsed (`:92`, `InstructionParser.cpp:1049`). **So the oracle
does resolve after the arguments**: everything `externalCall` searches (`findRoutine`, the exits,
Step 3) happens once the arguments have run, and only a label or a builtin is decided before.

The crate, read: the compiled-argument ops (`Op::CallArgs`, `Op::CallNamed`) run their argument ops
before the call op reads its site, so they already resolve after the arguments. The two ops whose
arguments do not compile (`Op::CallExpr`, `Op::Call`) and the three uncached callers (`eval_call`,
`exec_call`, the qualified forms) resolve first and evaluate the arguments inside `invoke_call`.

Design, following the oracle rather than a generation check: `resolve_call` splits into the part
the oracle decides before the arguments (a label, a builtin, the Phase 4 excluded builtins' refusal)
and the rest (the routine lookup, the internal packages, the registered library routines, the file
search). The tree-argument paths decide the first part before the arguments and the rest after them,
and a call site remembers the answer taken after them. A kept Step 3 answer (`External` and the other
kinds `kept_until_routines_change` names) is used again after the arguments only where they moved no
generation, and is looked up again where they did. A kept `Routine`/`MergedLibraryRoutine` stays kept
whatever the arguments do, as `externalTarget` does. A namespace-qualified call is left as it is.

Predictions (`fix3/ab.sh`: oracle, head worktree build, base `9ff11e465`), written before building,
for the reviewer's probes copied into `fix3/p/`:
* At `9ff11e465`: `o3`, `o3c`, `o3a` oracle `helper 16` on all three passes, crate `4` on all three;
  `o2` oracle `pk 16` ×3, crate `4` ×3; `o2r` oracle `added 16` ×3, crate `main 16` ×3; `o1` oracle
  `4` / `5`, crate `file 16` / `5`.
* After the change: all six head SAME on three descriptors, `o1` included.

At `9ff11e465`, run (`fix3/before.txt`): **every prediction confirmed**. After the change
(`fix3/item1.txt`): **all six head SAME** on three descriptors, `o1` included (`4` / `5`), so the
recorded `o1` divergence closes.

Built: `resolve_call` is `resolve_fixed_call` (label, builtin, excluded builtin) then
`resolve_routine_call` (everything else). `CallResolution` is `Settled(Resolved)` or
`AfterArguments { kept, site }`; `invoke_call` evaluates the arguments once for every kind, then
`settle_after_arguments` answers a settled resolution as it is, a kept Step 3 answer where the
generation it was read under is still current, and otherwise `resolve_routine_call`, remembered at
the site. `Op::CallExpr` and `Op::Call` start from `site_resolution_before_arguments`; `eval_call`,
`exec_call` and `resolve_and_run_call` from `resolve_fixed_call` with no site; the qualified forms are
`Settled`.

Predictions for three shapes the reviewer's probes do not cover, written before running them:
* `o4`, a kept external file `ext` whose argument's call does `addRoutine('EXT')` on pass 2: oracle `1
  ext 1` / `2 added 2` / `3 added 3`; base `2 ext 2` / `3 added 3`; head SAME.
* `o5`, `foo(step(i))` where nothing is named `foo` until `step` adds it on pass 1: oracle `1 added 1`
  / `2 added 2`; base 43.1 `Could not find routine "FOO"` on pass 1, rc 213; head SAME.
* `o6`, `zzmade(mk(1))` where `mk` writes `zzmade.rex` beside the program: oracle `made file 1`; base
  43.1 rc 213; head SAME (the file search now runs after the arguments too).

Run: **all three confirmed** (base `2 ext 2`, 43.1 twice; head SAME).

Witnesses, predictions before running them (oracle and head SAME, base as listed):
* `library_routine_resolved_after_arguments.rex` (`o3`, `o3c`, `o3a`, one routine name each, three
  helper files): base `function 1 4` ×3, `call 1 1` ×3, `assignment 1 1` ×3 against `helper 16`,
  `helper 0`, `helper 0`.
* `library_routine_resolved_after_arguments_merge.rex` (`o2r` in the program, `o2` in a `newFile`d
  `r.rex`): base `addRoutine N main 16` and `loadPackage N 4` on every pass.
* `library_routine_resolved_after_arguments_search.rex` (`o1`, `o4`, `o5`): base `library file 16`,
  `file 2 ext 2`, then 43.1 rc 213 at `unresolved` pass 1. `o6` is not a corpus witness: the
  harness gives the oracle and the crate one working directory, so the file the oracle's run wrote
  would already be there for the crate's.

Run: **all three confirmed**, head SAME, base DIFF on exactly the lines listed.

Controls for item 1, predictions written before running (`fix3/csweep.sh`: mutation profile in its own
target directory, the file restored and `cmp`-checked, then the 68-row `library_*`/`external_method*`
sweep):
* **C-R3a**, `site_resolution_before_arguments` resolving the whole name before the arguments on a
  miss (the old order at the two tree-argument ops): red all three new witnesses (`function N 4`,
  `addRoutine N main 16`, the search witness's 43.1), and nothing else.
* **C-R3b**, `settle_after_arguments` using a kept Step 3 answer whatever the arguments moved: red
  `library_routine_resolved_after_arguments_search` alone (`file 2 ext 2`).

Run (`fix3/ctl/item1-controls.txt`): **both confirmed**, exactly the predicted rows red.

A gated corpus run in the worktree on the change before the witnesses were listed: `588 of 588
matching`; its two red tests were the new witnesses' sidecars with no `phase-8.txt` row yet, which I
had written during the run (`fix3/corpus-item1-local.txt`). Not a gate; the gate below is.

Before committing: fmt check exit 0, clippy exit 0, `rexx-exec --lib` 828 passed, `refusal_sites`
re-derived (column 4 of 19 rows alone) then 5 passed, the 68-row sweep 0 red on the rebuilt binary.
The helper sat between a test-only `thread_local!` and its counter at first; moved above that block
before the commit.

**Item 1 committed** as `c55caa62f`; gate and callgrind comparison running (the comparison adds a
`nested` pair, `x = ext(f(i))`, whose argument is a call, as `o3`'s shape is; which op each
program's site compiles to is not measured).

## Item 2 (m-1): an identity test that cannot call two objects the same

Predictions, written before running: `m1` (`==` on the `Routine` objects themselves) oracle `library 1
1` / `routine 1 1`; crate at `c55caa62f` rc 120, a Phase 5 refusal of `==` on a `Routine`, as the
round-1 re-review recorded. `strict` (`==` on the `identityHash` strings, a strict comparison with no
numeric reading): oracle and crate `distinct objects 0` / `same object 1` / `findRoutine twice 1`.

Run: **both confirmed**. `==` on a `Routine` is still the Phase 5 refusal, so the witness compares the
`identityHash` strings with `==`, which on `strict` answers `0` for two distinct objects on both sides.

## Item 3 (m-4): `loadExternalRoutine` and `EXTERNAL "LIBRARY ..."`

The correction first: fix round 2's claim that `a15a8603a` "closes the routine objects not shared
concern" was wider than what it built. It made the object an imported-routine table answers one per
library routine. The oracle's `loadExternalRoutine` (`RoutineClass.cpp:501-535`,
`PackageManager::loadRoutine`, `LibraryPackage::resolveRoutine`) and a `::ROUTINE ... EXTERNAL
"LIBRARY lib entry"` directive (`DirectiveParser.cpp:2691-2693`) answer that same object too, and
the crate answered a fresh one for each.

Read: `loadExternalRoutine` (`dispatch.rs`, `load_external`) builds an object and records it
`Loaded { code }` under the row `library_code` answers for the table's own spelling, which is the row
`merge_library` keys too. That is the record `merged_routine_object` gives its shared object, so
answering the shared object there adds no mechanism. A directive's object is built with the package's
`.ROUTINES` table and recorded `Directive { program, directive }`, which `~source`, `~package`, the
flag readers and `~addRoutine` read (`dispatch/executable.rs`, `install_routine`); making it the shared
object changes what each of those answers, which is new mechanism. So: build the first, record the
second.

Predictions, written before building, for `id2` (the reviewer's `id1` with `==` on the hash strings):
oracle `1` on every comparison and `calls 4 5 6`; at `c55caa62f` `loadExternalRoutine 0 0` and
`directive 0`, the rest `1`; after, `loadExternalRoutine 1 1`, `directive 0` still.

Built: `Interp::library_routine_object(code)`, the body `merged_routine_object` had, and
`load_external` answers it for a routine outside the `REXX` package. After it: `id2` head
`loadExternalRoutine 1 1`, `directive 0`, the rest `1`, **as predicted**.

Witness, prediction before running: `library_routine_object_identity.rex` rewritten with `==` on the
hash strings, plus `loadExternalRoutine` under both spellings and a `distinct objects` control line:
oracle and head `1` on every identity line, `loadExternalRoutine 1 1 4`, `distinct objects 0`; base
(`9ff11e465`) `loadExternalRoutine 0 0 4`, the rest as the oracle.

Run: **confirmed** (base `loadExternalRoutine 0 0 4`, head SAME). The sweep's cached oracle output for
this witness was the old program's, so it was deleted by path before the sweep re-ran.

Control, prediction written before running: **C-R3c**, `load_external` building a fresh object for a
routine again: red `library_routine_object_identity` alone in the 68-row sweep.

Run: **confirmed**, the identity witness alone red.

Before committing: fmt check and clippy exit 0, `rexx-exec --lib` 828 passed, `refusal_sites` 5 with
nothing to re-derive, the 68-row sweep 0 red on the rebuilt binary.

**Items 2 and 3 committed** as `668c73827`. The directive route is recorded for the
exclusions list (item 6).

## Item 4 (m-5): the memory test without the oracle checkout

The convention, found: the test files that run the oracle call `support::oracle::locate`
(`/bin/grep -ln "locate()" tests/*.rs`; some behind `REXX_CORPUS_GATE`, and ungated ones such as
`licensed_divergences.rs` and `datetime_zone.rs`), which asserts the binary exists and fails naming it
("the oracle binary is missing at ... a machine reporting \"0 of 0 matching\" here would look
identical to one where every program actually passed"). The existence checks in those files that I
read (`collection_scopes.rs`, `method_bodies.rs`) guard their own inputs, not the oracle. (My first
list came from a `grep | head` and was short.) Built: the test calls `locate()` and asserts `librxmath.so` is in its `lib_dir()` (a new
accessor on `Oracle`) with a message naming the path, then passes that directory, as before.

Control, prediction written before running: **C-R4**, the library name made `librxmath_absent.so`: the
test red on the new assertion, its message naming `.../lib/librxmath_absent.so`, and no run of the
binary (so no `memory allocation` line anywhere in the output).

Run: **confirmed**, red at the new assertion naming `/home/moritz/dev/repos/ooRexx/build/lib/librxmath_absent.so`,
`memory allocation` nowhere in the output; the file restored and `cmp`-checked. Green again on the
restored tree (1 passed). fmt and clippy exit 0.

**Item 4 committed** as `a1ee6579c`.

### Item 1's cost

`fix3/bench/run.sh`: callgrind `Ir`, `9ff11e465` (`rexx-run.before`, `d5d10646…`) then `c55caa62f`
(`rexx-run.after`, `bdd45d35…`) per program, two rounds, a fresh run directory each, every rc 0,
`results.txt` ends `finished`. Per call is (400 - 200) / 200.

| program | before r1 | after r1 | before r2 | after r2 |
|---|---|---|---|---|
| `call ext i`, per call | 664,359 | 663,705 | 664,102 | 664,278 |
| `x = ext(i)`, per call | 664,333 | 664,135 | 663,744 | 663,972 |
| `x = ext(f(i))`, per call | 666,077 | 666,681 | 665,650 | 665,416 |
| `rexxcps` total | 21,262,611,727 | 21,141,644,883 | 21,261,641,135 | 21,142,196,280 |

External-file calls unchanged within 0.1% in all three shapes. `rexxcps` is **-0.57%** and -0.56% in
instructions. I do not attribute that: one resolution half now runs before the arguments and the other
after, and this tree has measured layout alone moving an axis by several percent, so it is not
evidence of a gain.

## Item 5 (m-3): prose

* `kept_until_routines_change`'s doc: the generation is one counter for the interpreter, so any
  routine-table write in any package or any library registering routines drops every kept kind it
  names; and "for good" is now "for as long as the site's compiled body lives".
* The `drive.rs` sentence the review named was replaced with item 1; the helper doc that took its
  place said "the oracle would search for again", and now says the oracle searches for those kinds
  on every call.
* `phase-8.txt`'s block: the site keeps the routine `findRoutine` found, is per compiled body, and a
  body compiled under a second trace setting has its own site (`a9`).
* `package_routine_lookup`'s doc: `Routine~newFile`/`Method~newFile` record the context's package
  or the caller's, and `Package~new` records a context's (`package_from_source`).

Checks: fmt, clippy exit 0; `rexx-exec --lib` 828; `refusal_sites` re-derived (column 4 of 19 rows)
then 5 passed. **Committed** as `273e8f608`.

## Item 6: the recorded divergences reach `phase-4-exclusions.txt`

Every probe re-run for the entries, on the oracle and on this crate's release build of `273e8f608`
(`fix3/ab.sh`, one run path per side, three descriptors; `fix3/ex/`). The entries quote those runs.
`.Routine~new` with a context is already an entry ("Method~new AND Routine~new REFUSE ANY THIRD
ARGUMENT"), and `collect_stress`'s L0 stop is too; the entries for `x5` and m-2 point at those rather
than repeat them.

Predictions, before running (oracle / crate): `rw3` `second put 2` / `foo putfoo 3` against `second ext
2` / `raised SYNTAX 43`; `j2`, `j3` `set`/`installed` then a `pos` answer against rc 120 naming
Phase 5; `c3` a `*-* Compiled routine` line in each `TRACEBACK` against none; `b47` (a label inside a
`DO`) `Error 47` rc 209 against `rexx-exec: 47.2` rc 120; `x14s` `2 old` on the method-argument line
against `2 new`; `t6`, `t6b`, `t6c` the caller's `call`/`say` line in the traceback against none;
`x12` a `::requires` line in the traceback against none; `x5` a `Routine` answer against rc 120 Phase 5;
`a9`, `a9m` `2 main` / `2 4` against `2 added` twice; `ns1` `ns parent public mainpub` / `ns parent
merged frompub` against `raised SYNTAX 43` twice; `nf2` `new mainr` / `find a Routine` against 43.1 rc
213; `ar1` `added 4` against `raised SYNTAX 88 88.914`; `i2a` the oracle's version lines against the
doubled `... (33) is not implemented (Phase 8)`; `id3` `directive 1 4` against `directive 0 4`.

Run (`fix3/ex-runs.txt`): the divergences predicted, with two predictions **falsified in detail**:
`j2`/`j3` on the oracle raise 88.901 (the probe sends `pos` with no argument) rather than answering,
and on the crate the refusal comes at `setMethod`, before `set`/`installed` prints. `b47`'s stdout is
empty on both. The entries quote the runs as they came out.

Written: one entry each in `phase-4-exclusions.txt`'s Phase 8 section under "RECORDED BY SURFACE TASK
2 AND ITS FIX ROUNDS, NOT FIXED", with the transcripts and an owner or the reason for none: the
`.ROUTINES` put; `setMethod` of a loaded method; the missing `Compiled routine` line; a label in a
`DO`; `x14s`; `t6`/`t6b`/`t6c`; `x12`; `Routine~new` with a context (pointing at the existing
"Method~new AND Routine~new REFUSE ANY THIRD ARGUMENT" entry); `a9`/`a9m`; `ns1`; `nf2`; `ar1`; the
directive route of item 3; `Refused::Unfilled`'s doubled suffix (owner: Task 3); and m-2 (pointing at
the existing "collect_stress's L0 TEST STOPS BEFORE ANY phase-8.txt PROGRAM" entry). `o1` closed
with item 1 and has no entry. The entries avoid the `LICENSED DIVERGENCE WITNESS:` and `KNOWN GAP:`
markers the tests parse. After writing them I checked each claim against a run or a line read and
tightened four sentences that said more than that (the `.ROUTINES` mechanism, `install_routine`'s
acceptance rule, where the root lives now, and what the gated corpus does).

The tests that read the file, after the edit: `licensed_divergences` 22, `coverage` 21,
`builtin_status` 26, `owners` 6 passed; `rexx-inventory` every binary green.

**Item 6 committed** as `3f48151ce`. Gate for `c55caa62f`: 72 `Compiling` lines, gated corpus exit 0,
`591 of 591 matching`; release tests exit 101 on the BASE L0 test alone. The other four revisions
are gating in one lane.

After the item 6 commit I read the block's header again: it said every entry answered the same on an
earlier revision, which was measured for the entries that say so (`.ROUTINES`, the `TRACEBACK` line,
`x14s`, `a9`) and not for `j2`/`j3`, `b47` or the doubled suffix. Corrected in `cb13f0033`; the
readers re-run green (22, 21, 26, 6, `rexx-inventory` green). Queuing its gate, I first typed a
commit id by hand and killed that waiter before it ran; the queued script names the id `git cat-file`
confirmed.

A final pass over the second re-review's probes (every `t2-rereview2/p/` directory with a
`main.rex`, copied into `fix3/rr2/`) on the final binary. Prediction, before running, against that
review's recorded head verdicts at `9ff11e465`: `o2`, `o2r`, `o3`, `o3a`, `o3c` move to SAME; `id1`
stays DIFF (its `directive` line, and `=` on hashes), with `loadExternalRoutine` now `1 1`; every other
verdict unchanged.

Run (`fix3/rr2-final.txt`, compared line by line in `python3` against each `p/*.ab.txt`'s head
verdict; a first `join` over unsorted input printed nothing usable): **as predicted**. `o2`, `o2r`,
`o3`, `o3a`, `o3c` moved from DIFF to SAME; the other 42 recorded verdicts are unchanged, `id1`
included (`loadExternalRoutine 1 1` now, `directive 0`). Of the probes that review recorded no verdict
for, `ar1` is the recorded divergence and `gc3`, `gc4`, `ih` differ only in the raw `identityHash`
values they print (addresses on the oracle, handle bits here), which no witness compares.

## Gates (fix round 3)

`fix3/gates/gate.sh`: one `git archive` copy and one target directory per revision, one lane, each
copy and target deleted once its status was written; `/tmp` stayed at 31-35% used.

| revision | `Compiling` lines | gated corpus | release tests of the four crates |
|---|---|---|---|
| `c55caa62f` (item 1) | 72 | exit 0, `591 of 591 matching` | exit 101, the BASE L0 test alone |
| `668c73827` (items 2, 3) | 72 | exit 0, `591 of 591 matching` | exit 101, the BASE L0 test alone |
| `a1ee6579c` (item 4) | 72 | exit 0, `591 of 591 matching` | exit 101, the BASE L0 test alone; `imported_routines_sent_in_a_loop_run_under_the_oracles_memory_cap ... ok` |
| `273e8f608` (item 5) | 72 | exit 0, `591 of 591 matching` | exit 101, the BASE L0 test alone |
| `3f48151ce` (item 6) | 72 | exit 0, `591 of 591 matching` | exit 101, the BASE L0 test alone |
| `cb13f0033` (item 6 header) | 72 | exit 0, `591 of 591 matching` | exit 101, the BASE L0 test alone |

588 at `9ff11e465` plus item 1's three witnesses; item 2 rewrote an existing one.

## Concerns (fix round 3)

* A kept Step 3 answer is still used again while the generation stands, not searched for on every
  call as the oracle does. The writes table is the argument that nothing else changes those answers: a
  registered or internal routine comes before any file in the search, and a kept `External` searches
  for its file again when entered. `o6` (a file an argument writes) agrees because a miss now searches
  after the arguments; it is not a corpus witness, since the harness shares one directory between the
  two interpreters.
* `rexxcps` moved -0.57% in instructions at item 1, not attributed.
* The directive route of the shared library `Routine` is recorded, not built.
* Two exclusions-entry predictions were falsified in detail (`j2`/`j3`), my first list of `locate()`
  users came from a truncated `grep`, and the item 6 header over-claimed; each corrected and recorded
  above.
