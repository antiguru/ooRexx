# Task 2 fix round 1 re-review, integration slice (everything outside `rust/crates/rexx-api/`)

Fix base `c23c214f3`, head `bd64f3197`. Findings under verification: integration I1-I4, M1-M5, the
unwitnessed `::ATTRIBUTE` load site. Written first and appended as the review goes. Scratch:
`scratchpad/t2-rereview-b/`, a fresh `mktemp -d` run directory per probe, three descriptors.

## Setup

* `git archive c23c214f3 rust api` into `t2-rereview-b/base`, `bd64f3197` into `t2-rereview-b/head`,
  release `rexx-run` each, own `CARGO_TARGET_DIR` (`target-base`, `target-head`).

## Passes over the diff

(appended as they happen)
* Pass 1 (read): `git diff c23c214f3..bd64f3197` for `ir.rs`, `ir/drive.rs`, `eval.rs`, `run.rs`,
  `environment.rs`, `dispatch/package.rs`, `lib.rs`.
* Pass 2 (read, oracle, each line printed): `RexxExpressionFunction::evaluate`
  (`expression/ExpressionFunction.cpp:176-215`) and `RexxInstructionCall::execute`
  (`instructions/CallInstruction.cpp:170-198`): **the oracle keeps a resolved routine at the
  instruction**, `setField(externalTarget, resolvedTarget)`, and `RexxActivation::externalCall`
  (`execution/RexxActivation.cpp:3062-3075`) sets `resolvedTarget` only on its Step 2,
  `settings.parentCode->findRoutine(target)`. So a call site that found a `::ROUTINE`, a merged
  public routine, or a merged library routine keeps it for good; a site answered by Step 2a (the
  security manager, `Activity.cpp:2656-2667`) or Step 3 (`SystemInterpreter::invokeExternalFunction`,
  `platform/unix/ExternalFunctions.cpp:104-135`: `PackageManager::callNativeRoutine`, then
  `callExternalRexx`) keeps nothing and re-resolves on every call. `findRoutine`
  (`classes/PackageClass.cpp:896-911`) is `findLocalRoutine` (own `routines`, then the **parent's
  local routines**, recursively, `:822-843`) and only then `findPublicRoutine` (own
  `publicRoutines`, own `mergedPublicRoutines`, then the parent's `findPublicRoutine`, `:852-883`).
  `addInstalledRoutine` (`:1432-1451`, behind `~addRoutine`/`~addPublicRoutine`) writes `routines`
  with `setEntry`, replacing. `mergeRequired` (`:693-722`) and `mergeLibrary` (`:756-772`) both go
  through `HashContents::merge` -> `mergeItem` -> `mergePut` (`support/HashContents.cpp:1171-1333`),
  which returns without writing when the index is present; the only other writers of
  `mergedPublicRoutines` are `callMacroSpaceFunction` (`RexxActivation.cpp:3014`), `callExternalRexx`
  (`:3149`), `loadLibrary` (`:3286`) and `addPackage` (`PackageClass.cpp:1388`), each through those two.
  `loadPackage` (`:1842-1858`) is `loadRequires` then `addPackage` (`:1312-1333`).

## Probes

### X: events that change what a kept call site should answer (risk 1)

The crate keeps every resolution but `Unresolved`, and drops a kept `Library`, `Internal`,
`LibraryRoutine`, `MergedLibraryRoutine` or `External` when `routine_generation` has moved. The
generation moves in `register_package_routines` (per routine a library registers) and in
`merge_routines` (when a merge inserts a name). It does **not** move in `install_routine`
(`dispatch/package.rs`, `~addRoutine`/`~addPublicRoutine`). Against the oracle's rule above, the
enumerated events:

| event | generation moves | oracle re-resolves a non-Step-2 site | probe |
|---|---|---|---|
| a library registers routines | yes, per routine | yes | `c4` family (implementer, witness) |
| `addPackage` / `loadPackage` inserting a name | yes | yes | witness `site_after_add_package` |
| a called external file's public routines merge | yes, when a name is new | yes | `x9` |
| `::REQUIRES` inside `INTERPRET` | n/a | n/a | `x12` |
| `~addRoutine` / `~addPublicRoutine` on the running package or a parent | **no** | yes | `x1`-`x4` |
| `.Routine~new` / `newFile` with a context | no (new program) | n/a to existing sites | `x5`, `x6`, `x13` |
| a security manager | no (not a resolution) | Step 2a runs each call | `s*` |
| an external file deleted after a site kept `External` | no | yes | `x8` |

Predictions, written before running (oracle / head / base `c23c214f3`):
* `x1` a kept `Internal` (`filespec`) then `.context~package~addRoutine('FILESPEC', r)`: oracle
  `2 added`, `fresh added`; head `2 b.c` (kept, generation unmoved), `fresh added`; base `2 b.c`.
* `x2` a kept `LibraryRoutine` (`loadLibrary('rxmath')`, `RxCalcSqrt(16)`) then the same addRoutine:
  oracle `2 added`; head `2 4`; base `2 4`.
* `x3` a kept `External` (`ext.rex` beside) then addRoutine('EXT'): oracle `2 added`; **head `2 ext
  2`** (kept, regression if so); base `2 added` (base kept no `External`).
* `x4` a kept `MergedLibraryRoutine` (`::requires 'rxmath' LIBRARY`), then addRoutine('RXCALCSQRT')
  and a `loadPackage` of `q.cls` exporting an unrelated public `QQ` (moves the generation): oracle
  `2 4` (Step 2 hit kept at the instruction), `fresh added`; **head `2 added`** (dropped and
  re-resolved into the own table); base `2 4`. `x4b`, the same without the `loadPackage`: all `2 4`.
* `x5` `.Routine~new('r', ['return rxcalcsqrt(16)', "::requires 'rxmath' LIBRARY"], .context~package)`
  with main declaring `::routine rxcalcsqrt` answering `'main' arg(1)`: oracle `main 16` (parent's
  local routine before own merged); **head `4`** (own merged first); base `main 16` (library table
  after the parent walk). `x5b`, the same with `::requires 'pub.cls'` exporting a public
  `rxcalcsqrt` answering `'pub' arg(1)`: oracle `main 16`; head and base `pub 16` (pre-existing).

Ran (`ab.sh`: oracle, head, base, each from its own fresh directory, three descriptors, `ulimit -v`
on all three):
* `x1`: **confirmed**, oracle `2 added` / `fresh added`; head and base `2 b.c` / `fresh added`, rc 0
  all. Pre-existing for `Internal`.
* `x2`: **confirmed**, oracle `2 added`; head and base `2 4`. Pre-existing at `c23c214f3`.
* `x3`: **confirmed, a regression**: oracle `1 ext 1` / `2 added` / `fresh added`; **head `2 ext 2`**;
  base `2 added` (SAME). rc 0 on all three, stderr empty. The kept `External` answers the old
  resolution where the oracle and the fix base answer the new one.
* `x4`: **confirmed, a regression in the other direction**: oracle `1 4` / `2 4` / `fresh added`;
  **head `2 added`**; base `2 4` (SAME). The oracle keeps a Step 2 hit at the instruction for good;
  head drops a kept `MergedLibraryRoutine` because an unrelated merge moved the generation, and the
  re-resolution finds the routine `addRoutine` put in front of it.
* `x4b` (no `loadPackage`): SAME on both, `2 4`.
* `x5`, `x5b`, `x5e`: not the probe I meant: `.Routine~new(name, source, context)` is rc 120
  `method "NEW" of class "Routine" is not implemented (Phase 5)` on head and base alike (loud). Re-run
  through `.Routine~newFile('r.rex', .context~package)`, same predictions:
* `x5c` (`r.rex`: `return rxcalcsqrt(16)` / `::requires 'rxmath' LIBRARY`): **confirmed, a
  regression**: oracle `main 16`; **head `4`**; base `main 16` (SAME). rc 0 all.
* `x5d` (`r.rex` requiring `pub.cls`): oracle `main 16`; head and base `pub 16`. Pre-existing: the
  crate's walk is own `routines`, own merged, then the parent's pair, where `findRoutine` is every
  `routines` up the parent chain first and every merged table after.

Predictions for the rest of the table, written before running:
* `x3c`, `x3` through `call ext i`: the same three answers as `x3` (head `2 ext 2`).
* `x8`, a kept `External` whose file `address system 'rm ext.rex'` removes between calls: oracle
  43.1 on the second call; head (the consumer searches again, `enter_external_program`) SAME; base
  SAME. If the crate refuses `address system`, loud and says nothing.
* `x9`, `ext.rex` answering `'file' arg(1)` and exporting `::routine ext public` answering `'pub'
  arg(1)`: oracle `1 file 1`, `2 pub 2` (`callExternalRexx` merged it), `fresh pub 3`; head, base SAME.
* `x12`, `interpret "::requires 'pub.cls'"`: an error on both, SAME.
* `x13`, `r.rex` (`return helper()`) through `.Routine~newFile('r.rex', .context~package)` from two
  packages each with a private `helper`, called A, B, A: oracle `A B A`; head SAME (no shared site).
* `x14`, a kept `Routine` whose name `addRoutine` replaces: oracle `2 old` (kept at the instruction),
  `fresh new`; head, base SAME.
* `m1`, identity: `findRoutine('RXCALCSQRT')` twice and `importedRoutines['RXCALCSQRT']` under
  `::requires 'rxmath' LIBRARY`, and the same for a public `::ROUTINE` of a required package: oracle
  `1 1` / `1 1`; head `0 0` for the library pair (a fresh `native_instance` per answer,
  `merged_routine_object`), `1 1` for the `::ROUTINE` pair.

Ran:
* `x3c`: **confirmed, the regression through `CALL`** (`Op::Call`'s table read): oracle and base `2
  added`, head `2 ext 2`, rc 0.
* `x8`: SAME on all three (43.1 `Could not find routine "EXT".` at line 2, rc 213).
* `x9`: SAME on all three (`1 file 1`, `2 pub 2`, `fresh pub 3`).
* `x12`: rc 157 and stdout SAME; stderr differs on head **and base** alike (the oracle's traceback
  carries `2 *-* ::requires 'pub.cls'` above the `interpret` line). Pre-existing, not this round's.
* `x13`: SAME (`A B A`). `x14`: SAME (`2 old`, `fresh new`).
* `m1`: `==` on a `Routine` is rc 120 (Phase 5) on head and base; re-run as `m1b` over
  `~identityHash`: oracle `library 1 1` / `routine 1 1`; head **and base** `library 0 0` / `routine
  1 1`. `findRoutine` hands out a fresh object per answer for a merged library routine (pre-existing),
  and `importedRoutines`, new at head, hands out yet another. Minor.

**What the table comes to.** The generation tracks the events that *add a Step 3 answer's
competitor through a merge or a library*, and nothing else. The oracle's rule is a different one:
it keeps a Step 2 hit for good and keeps nothing else. So the kept-resolution table is wrong in
both directions where the two rules part: a site kept below the package lookup survives
`~addRoutine` (`x1`, `x2` pre-existing; **`x3`/`x3c` new**, because `External` is kept again), and
a kept `MergedLibraryRoutine`, which the oracle never re-resolves, is re-resolved by any unrelated
generation move (**`x4`, new**).

Found on the way, not this round's (`x14s`, predictions not written first, so recorded as an
observation only): a call inside a method's argument list (`.array~of(foo())[1]`) re-resolves on
every pass, `2 new` on head and base where the oracle keeps `2 old`; `parse value foo()`, a `WHEN`
and a traced assignment keep it on all three. Some expression call paths have no site table.

### S: the name on a hit and the security manager (risk 2)

Predictions, written before running:
* `s1`: a security manager on two `newFile`d routines. `global.rex` calls, from one function site
  and one `CALL` site per loop over two values: a `loadLibrary` routine, `filespec`, an external
  file, its own `::ROUTINE`, and a `LIBRARY REXX filespec` directive; `merged.rex`
  (`::requires 'rxmath' LIBRARY`) calls `RxCalcPower` the same way. Oracle: an `event CALL` line
  before every call of the three Step 3 routes (`RXCALCSQRT`, `FILESPEC`, `EXT`, upcased symbols) and
  none for the three Step 2 routes. Head SAME; base, the second `CALL`-form event is fine (`Op::Call`
  reads its instruction) and the second function-form event of the external file is empty or
  missing.
* `t1`-`t7`, one uncaught error on the second pass of one `CALL` site: `t1` global `RxCalcSqrt`
  over `4`, `'x'`; `t2` merged; `t3` a `::ROUTINE ... EXTERNAL "LIBRARY rxmath RxCalcSqrt"` in a
  required package; `t4` a `LIBRARY REXX Filespec` directive over `'N'`, `'Z'`; `t5` `filespec`
  itself; `t6` an external file failing on its second call; `t7` a Rexx `::ROUTINE` failing on its
  second call. Oracle names the routine as the call wrote it (`"RXCALCSQRT"`, `"SQ"`, `"FS"`,
  `"FILESPEC"`; no `Compiled routine` line for `t6`/`t7`). Head SAME on all.

Ran:
* `s1`: **head SAME** on all three descriptors (an `event CALL` before each of the four calls per
  Step 3 route, `RXCALCSQRT`, `FILESPEC`, `EXT`; none for `myr`, `fs`, `RxCalcPower` merged). Base
  DIFF: the second function-form event of the global and internal routes is `event CALL ` with no
  name, as I1 said; the external file's is named at base (it was not kept there). I1's checkpoint
  half verified on every route.
* `t1`-`t5`, `t7`: **SAME** on head (and base: `Op::Call` reads its instruction) -- `"RXCALCSQRT"`
  global and merged, `"SQ"` naming `pk.cls` with no line, `"FS"`, `"FILESPEC"`, the Rexx routine's
  two lines.
* `t6`: rc and stdout SAME, stderr DIFF on head and base alike: the oracle's traceback carries the
  caller's `2 *-* call ext a` under the external file's line, the crate's does not. Not a hit
  question: `t6b` (`call ext 0` once) and `t6c` (`say ext(0)` once) are the same divergence on head
  and base. Pre-existing, out of scope.

### R: one ordered merged lookup (risk 3)

Read, oracle, printed: `processInstall` (`classes/PackageClass.cpp:1227-1260`) walks `libraries`
then `requires`; the namespace-qualified call (`instructions/CallInstruction.cpp:450`,
`expression/ExpressionQualifiedFunction.cpp:170`) is `findPublicRoutine`; `getRoutinesRexx`,
`getPublicRoutinesRexx`, `getImportedRoutinesRexx` (`:1606-1664`) each answer a **copy**. No
`StringTable` or `StringHashCollection` override of `mergeItem` exists (`grep -rn mergeItem
interpreter/` names only `HashCollection` and `HashContents`), so add-if-absent is every merge's rule.
The crate's `find_public_routine` (`dispatch/package.rs`) is `find_routine`.

Predictions, written before running:
* `r1`: main requires `a.cls`; `a.cls` exports a Rexx `RxCalcSqrt` (`'a' arg(1)`) and requires
  `b.cls`; `b.cls` exports a Rexx `RxCalcPower` (`'b' arg(1)`) and requires rxmath LIBRARY; main
  declares a private `RxCalcExp` (`'mainexp'`). Oracle: `sqrt a 16`, `power b 2`, `exp mainexp`,
  `log 0` (library through two merges); `findRoutine RXCALCSQRT a 4`; **`findPublicRoutine RXCALCEXP
  1`** (main's private routine is not public, the merged library one answers); `routines RXCALCEXP a
  Routine`, `publicRoutines RXCALCEXP The NIL object`, `imported RXCALCEXP a Routine`, `imported
  sqrt a 9`, `imported power b 2`; in `a.cls`, `a findRoutine power b 2`; then after
  `loadPackage('c.cls')` (public Rexx `RxCalcLog` and `newname`): `later log 0`, `later newname c`.
  Head: SAME except `findPublicRoutine RXCALCEXP mainexp` (the crate's `findPublicRoutine` is
  `findRoutine`); base the same as head there, and `imported RXCALCEXP The NIL object`.
* `r2`: `call ext` (whose `ext.rex` requires rxmath LIBRARY) then `say 'rxcalcsqrt'('x')` uncaught:
  oracle traceback `Compiled routine "rxcalcsqrt"` (the external file's merged library routines
  merged into the caller). Head SAME.
* `r3`: `loadLibrary('rxmath')`, then one site `'rxcalcsqrt'(a)` over `4`, `9`, `'x'`, with `call
  ext` (as `r2`) after the first pass: oracle `2`, `3`, then `Compiled routine "rxcalcsqrt"`. Head
  SAME (the merge moves the generation).
* `r4`: main requires `pub.cls` (Rexx `RxCalcSqrt`), then `loadPackage('lib.cls')` (requires rxmath
  LIBRARY): oracle `before pk 16`, `after pk 16`, `findRoutine pk 25`. Head SAME.

Ran:
* `r1`: **head SAME** on all three descriptors. **My `findPublicRoutine` prediction was falsified**:
  the oracle answers `mainexp`, because `PackageClass::findPublicRoutineRexx`
  (`classes/PackageClass.cpp:2021-2025`) calls `findRoutine`, not `findPublicRoutine`; the crate's
  delegation is the oracle's. Base DIFF on `imported RXCALCEXP The NIL object` and **`later log c`**
  (base's separate library table let `c.cls`'s later public `RxCalcLog` in front of the library
  routine merged at install); head fixes both.
* `r2`, `r4`: SAME on all three. `r3`: head SAME; base's traceback reads `Compiled routine ""` (I1).

Prediction for `r5`, before running: in `r.rex` run through `.Routine~newFile('r.rex',
.context~package)`, `.context~package~findRoutine('MAINR')` for main's private `::routine mainr`, and
`findRoutine('RXCALCSQRT')~call(16)` with `r.rex` requiring rxmath LIBRARY and main declaring
`rxcalcsqrt`: oracle `mainr a Routine`, `sqrt main 16`; head `mainr The NIL object` (no parent
walk in `package_find_routine`), `sqrt 4`; base `mainr The NIL object`, `sqrt 4` (base read the
merged library table in `package_find_routine` too).
* `r5` ran: **confirmed**. Oracle `mainr a Routine` / `sqrt main 16` / `call main 16`; head `mainr The
  NIL object` / `sqrt 4` / **`call 4`**; base `mainr The NIL object` / `sqrt 4` / `call main 16`. So
  `findRoutine` has never walked a parent (pre-existing, both halves), and the call half is `x5c`'s
  regression again.

### Performance (risk 1, the cost claim)

`t2-rereview-b/bench/run.sh`: callgrind `Ir` totals, base (`c23c214f3`) then head (`bd64f3197`) per
program, two rounds, a fresh run directory each, `bench/results.txt` ends `finished`, every rc 0.
Programs copied from the implementer's `fix1/bench/` (`cps/main.rex` is `cmp`-identical to
`rust/bench-rexxcps/rexxcps.rex`). Per call is (400 - 200) / 200.

| program | base r1 | head r1 | base r2 | head r2 |
|---|---|---|---|---|
| `call ext i`, per call | 1,064,045 | 663,157 | 1,062,501 | 663,248 |
| `x = ext(i)`, per call | 1,062,402 | 663,359 | 1,063,794 | 663,894 |
| `rexxcps` total | 21,215,458,980 | 21,258,067,047 | 21,216,307,522 | 21,258,695,402 |

* **The cost claim holds**: the implementer's 1,064,166 -> 664,907 (`6245fe1ca` -> `25bdb2dd5`)
  reproduces across the whole round, 1,064,045 -> 663,157 per `call ext i` (-37.7%), and the
  function form moves the same way.
* `rexxcps`: head **+0.20%** instructions over the fix base, both rounds (+42.6M and +42.4M). The
  implementer's +0.11%/+0.09% covered item 6 alone. Under the layout noise this tree has measured
  (several percent per axis from dead code alone), so not evidence of a cost, and not evidence of none.

### D: `double_of` (risk 4)

Read: `double_literal` is `Number::format(digit_count)`; `format` (`rexx-num/src/lib.rs:793-821`)
rounds to that many digits (a no-op at the number's own count) and takes the exponential form when
`adjusted >= digits` or `exponent <= -(2 * digits + 1)`, so the plain form is at most about three
times the digit count; a zero answers `"0"` before any of that.

Predictions, written before running, each value through `RxCalcSqrt(x, 16)` and `RxCalcPower(x, 1,
16)`, trapped: `d1` over `.5`, `-0`, `-0.0E+5`, `1e-999999999` (lower case), `' 4 '`, `'4'||'00'x`,
`'09'x||'4'`, `0`, `0.000`, `0E+999999999`, `'  -.5E-3  '`, `copies('9', 2000)`,
`copies('9', 2000)'E-2300'`, `'1'copies('0', 400)'E-400'`, `4.0000000000000000000000000000000000001`,
`2.5E+1`, `copies('1', 100000)'E-99990'`: SAME on all three descriptors, rc 0, the NUL byte and the
tab 88.921 (I do not predict whether `-0` prints `0` or `-0`, nor the tab). Head's peak RSS under
`/usr/bin/time -v` on the binary: under 100 MB.

Ran `d1`: **head SAME** on all three descriptors, rc 0 (`.5` `0.7071067811865476 0.5`; `-0`, `-0.0E+5`,
`1e-999999999`, `0`, `0.000`, `0E+999999999` all `0 0`; `' 4 '` `2 4`; NUL 88.921; `'  -.5E-3  '`
`nan -0.0005000000000000000`; 2000 nines `+infinity +infinity`; with `E-2300` `1.000000000000000E-150
1.000000000000000E-300`; the 406-character `1` `1 1`; the 100,007-character value `33333.33333333334
1111111111.111111`). The tab half of my prediction was void as written (it said both 88.921 and "not
predicted"): both sides answer `2 4`. Base: rc 134, `memory allocation of 999999998 bytes failed`,
stdout empty (I4 at the fix base). `/usr/bin/time -v` on the head binary itself, no cap: `d1` peak RSS
19,008 kB, the witness `library_routine_double_exponent.rex` 18,720 kB, 0.02 s each.

Citations printed: `NumberStringClass.cpp:704` is `bool NumberString::doubleValue(double &result)`
in this tree and the oracle's (the implementer is right, the review's M4 second half was wrong);
`Activity.cpp:1080` is `Activity::generateProgramInformation`, whose loop the cited `:1093-1113`
is; `NativeActivation.cpp:190-193` is `reportSignatureError`; `RexxErrorCodes.h:408` is
`Error_Incorrect_call_signature = 40918`.

### M: minors, witnesses, records (risk 5)

`refusal-sites.tsv`, by column, per commit (`python3` over `git diff <c>~1 <c>`): `cae7d5382` and
`2804a8986` change column 4 (the location) of 19 rows each and nothing else; `79f08bed6` changes
column 4 of every row below `error.rs:340`, replaces the two 93.968 rows with
`incorrect_native_signature` (`agrees` / `yes` / `40.918`), and changes `library_procedure_gone`'s
witness column alone. No data row at head has an empty verdict, reached, answer or witness column
(`awk -F'\t'` over the file), so the one row a refresh could have left empty is
`incorrect_native_signature`'s, measured here: the implementer's `m1a`-`m1c` copied, forged
`libforgesig.so` (`readelf -d` lists no `NEEDED`) on both sides' library path. **head SAME** on all
three (`40.918` rc 216: `m1a` `main.rex line 2` under `"FORGESELF"`, `m1b` `pk.cls:` no line under
`"SQ"`, `m1c` `main.rex line 2` under `"FORGEOPTIONAL"`); base 93.968 rc 163 on each.

Prediction for the method half the constructor now also builds, before running (forged
`libforge.so` from `final-a/ext`, no `NEEDED`): `m2a` (`o~unknown(1)`, a procedure declaring parameter
code 9, the class in `pk.cls`) 93.968 naming `pk.cls:` with no line, rc 163; `m2b` (`o~retopt`, a
result word with the optional bit) 93.968 naming `main.rex line 3`. Head and base SAME.

Ran `m2a`, `m2b`: **SAME** on head and base (93.968 rc 163; `m2a` `pk.cls:` no line under `Compiled
method "UNKNOWN" with scope "O"`, `m2b` `main.rex line 3`). The method half of the merged constructor
is unchanged.

The removed sidecar (`bd64f3197`): `library_routine_site_twice_rexx.rex` with its `.d/` from three fresh
directories, oracle under the wrapper, head crate with `LD_LIBRARY_PATH` unset (`env -u`) and with it
set: both crate runs `cmp`-identical to the oracle on all three (`b.c`, then 40.904 under `"FS"`, rc
216). Inert, as the commit says; its text ("The oracle's own build of librxmath.so") was copied from
a witness that loads rxmath. Nothing hidden.

Predictions for the exclusions entry's two examples, before running: `e1` `call MathDropFuncs` under
`::requires 'rxmath' LIBRARY`: oracle `start` / `result ` rc 0; head `start` then rc 120 naming
`REXX_VALUE_CSTRING`. `e2` `say RxCalcSin(30, 3, 'X')`: oracle 88.916 rc 168; head rc 120 naming
`RexxThreadInterface.NewStringFromAsciiz`, `before` kept.

Ran `e1`, `e2` (head only): **both as predicted** (`e1` oracle `result ` rc 0, head rc 120 `Phase 8
owes the FromNative conversion for REXX_VALUE_CSTRING (15)`; `e2` oracle 88.916 rc 168, head rc 120
`RexxThreadInterface.NewStringFromAsciiz`, `before` kept). The exclusions entry's examples hold.

### P: the implementer's open concern, a `Routine` found through a parent (risk 1)

Predictions, written before running. `r.rex` through `.Routine~newFile('r.rex', .context~package)`:
* `p1`: a loop site `foo()` finding main's local `foo` (`'main'`), then after the first pass
  `.context~package~loadPackage('pub.cls')` (into `r.rex`'s package, public `foo` answering `'pub'`),
  then a fresh site: oracle `1 main`, `2 main` (Step 2 hit kept at the instruction), `fresh main`
  (parent's local routines before own merged). Head `1 main`, `2 main` (kept `Routine`, as the
  oracle), **`fresh pub`** (own merged before the parent's local, pre-existing as `x5d`). Base the same
  as head.
* `p2`: a loop site `bar()` finding `bar` in the parent's merged table (main requires `mid.cls`
  exporting `bar` `'mid'`), then `loadPackage('pub2.cls')` into `r.rex`'s package exporting `bar`
  `'pub2'`, then a fresh site: oracle `2 mid` kept, `fresh pub2` (own merged before the parent's
  public). Head and base SAME.

Ran `p1`, `p2`: **every prediction confirmed** (`p1` oracle `fresh main`, head and base `fresh pub`;
`p2` SAME on all three). So the implementer's concern is not a defect: a kept `Routine` is what the
oracle keeps too, whichever package it was found in. The divergence under it is the fresh lookup's
order, and it predates the round.

`x4`'s shape with no `addRoutine`, predicted before running: `p3`, main requires rxmath LIBRARY,
`r.rex` (newFile'd with main as its context) loops `RxCalcSqrt(16)`, found in the parent's merged
table, and after the first pass `loadPackage('pub.cls')` merges a Rexx public `RxCalcSqrt` (`'pk'
arg(1)`) into `r.rex`'s own package, then a fresh site: oracle `1 4`, `2 4` (kept), `fresh pk 16`;
**head `2 pk 16`** (the merge moves the generation and the re-resolution reads `r.rex`'s merged table
first); base `2 4`, `fresh pk 16`.

Ran `p3`: **confirmed, a regression that needs no `addRoutine`**: oracle `1 4` / `2 4` / `fresh pk
16`; **head `2 pk 16`**; base SAME. Two ordinary operations reach it (`Routine~newFile` with a
context, `loadPackage`), rc 0, stderr empty.

Prediction for the "required package installed later" row, before running: `q1`, a kept `Internal`
(`filespec`) in main, then `loadPackage('b.cls')`, where `b.cls` requires `c.cls` exporting a public
`filespec` (`'pk' arg(1)`): oracle `2 pk N` (merged through `b.cls`'s merged table); head SAME (the
merge inserts the name and moves the generation); base `2 b.c`.

Ran `q1`: **confirmed**, oracle and head `2 pk N`, base `2 b.c`.

Prediction for `n1`, before running: the `Routine` objects `importedRoutines` now hands out for a
library entry, beside `findRoutine`'s: `~name`, `~package`, `~source~items` for
`importedRoutines['RXCALCSQRT']` and `findRoutine('RXCALCSQRT')` under `::requires 'rxmath' LIBRARY`,
in main and through a required `mid.cls` that requires it. I predict SAME only (no rendering).

Ran `n1` (malformed: `Routine` has no `NAME` on any side, 97.1 SAME) and `n1b` over `~package`,
`~source~items`, `~class`: **head SAME** (`The NIL object 0` for the library entry from
`importedRoutines` and from `findRoutine`, in main and in `mid.cls`; `mid.cls` for the imported
`MID`). Base 97.1 on `.nil`.

Read, not run: the two new `run/tests.rs` tests (`an_extension_reading_the_instance_reaches_its_table`,
`an_extension_reaching_an_unwritten_member_refuses_loudly`) load extensions from
`CARGO_MANIFEST_DIR/../../../build/lib` with `expect("the oracle's build directory is four above
this crate")`. That path is the worktree's own `build/` (`.gitignore`d; `bin/rexx` 16,496 bytes, Jul
27), not the oracle's `/home/moritz/dev/repos/ooRexx/build` (`bin/rexx` 62,600 bytes, Aug 5), and
the two `librxmath.so` differ (`cmp`: byte 41; 32,248 against 147,488 bytes). `readelf -d`: both
worktree libraries NEED only `libc`/`libm`, so no oracle interpreter is mapped. The pattern and the
message predate the round (`git blame`: `08d232ecc`, 2026-09-14); the round copied both into two new
tests whose doc comments cite oracle measurements.

Prediction for `o1`, before running (the argument-before-resolution order, to see whether the kept
`External` stamp can be taken before an argument moves the generation): `o1`, `RxCalcSqrt.rex` beside
main answering `'file' arg(1)`, one site `say RxCalcSqrt(loadit(a))` over `16`, `25`, where `loadit`
loads rxmath on its first call and answers its argument: oracle `4` then `5` (arguments are evaluated
before `externalCall` resolves); head `file 16` (resolved before the argument ran, pre-existing
order), then `5` (the stamp predates the load); base `file 16`, `5`.

Ran `o1`: **as predicted** on head and base (`file 16` / `5` against the oracle's `4` / `5`): the crate
resolves before the arguments run, where `RexxExpressionFunction::evaluate`
(`expression/ExpressionFunction.cpp:182-192`) evaluates them first. Pre-existing, out of scope.

Read: `merged_routine_object` hands out a fresh `native_instance` for a library entry on every ask
and `record_loaded_executable` inserts it into `executable_sources`, which `object_roots` lists as
"keyed by an object and holding none" and which nothing ever removes from (`grep` for `retain`/
`remove` on it: none). `importedRoutines` now does that once per library routine per send.
Prediction for `g1` (head, `/usr/bin/time -v` on the binary, no cap): a loop of `N` sends of
`importedRoutines` under `::requires 'rxmath' LIBRARY` grows peak RSS roughly linearly in `N`; the
same loop over `~routines` (no library entry) does not.

Ran `g1`: **confirmed**. `/usr/bin/time -v` on the head binary: 20,000 `importedRoutines` sends 89,608
kB, 80,000 sends 238,852 kB (about 2.5 kB per send); the `~routines` control 27,072 and 47,240 kB. `g2`,
400,000 sends: **oracle rc 0 under the wrapper (20,652 kB at 80,000); head under the same `ulimit -v`
rc 134 `memory allocation of 545259536 bytes failed`, stdout empty** (its `100000`.. lines lost);
head uncapped rc 0 at 838,208 kB, so it is growth and not the stack reservation. The same loop over
`findRoutine('RXCALCSQRT')` is rc 0 capped on head and base, 80,208 and 80,352 kB uncapped: the leak
predates the round, and `importedRoutines` multiplies it by the library's routine count.

Predictions, before running: `w1`, the `::ATTRIBUTE` witness through `ab.sh` from its own files: SAME
on head (`loadPackage 90.998`, `main 4`), base SAME. `u1` (out of scope, read in
`PackageClass::addInstalledRoutine`, which does not upcase): `.context~package~addRoutine('foo', r)`
then `say foo()` trapped, then `routines~hasIndex('foo')`: oracle 43.1 and `1`; crate (upcases the
name in `install_routine`) the routine's answer and `0`.

Ran `w1`: **SAME** on head and base (`loadPackage 90.998` / `main 4`). `u1`: **my prediction was
falsified**: the oracle answers `call added` and `hasIndex 0 1`, so it upcases the name too, and the
crate's `install_routine` agrees on the call (its `~hasIndex` is rc 120, Phase 5). Not an observation.

### Not done

No suite, gate, Miri or corpus run, and no mutant built (the brief's rule, and every defect below is
unwitnessed rather than a question of whether a witness can fail). The report's gate table is
unverified here. The `rexx-api` side of the refusal record is the sibling's.

---

## Finding Verdicts

* ✅ **I1** (empty name on a kept site's second call). `ir/drive.rs:246-259` reads the spelling off
  the node on every hit but `Resolved::Builtin`. Ran: `s1` head SAME on the three Step 3 routes in
  both forms, base `event CALL ` with no name; `t1`-`t5`, `t7` SAME; `r3` head SAME, base
  `Compiled routine ""`.
* ⚠️ **I2** (merge order). The reviewed shapes are fixed: `install_requires` (`lib.rs:3215-3261`)
  walks libraries first, `merge_routines` (`lib.rs:4797-4811`) is add-if-absent, which is
  `HashContents::mergePut` on every merge the oracle does (printed). `r1` head SAME where base says
  `imported RXCALCEXP The NIL object` and `later log c`; `r2`, `r4`, `n1b` SAME. **But the one table
  moved library routines into the step that runs before the parent walk**, a regression (new
  Important I-B below).
* ⚠️ **I3** (cost of not keeping `External`). The cost is fixed: per `call ext i` 1,064,045 ->
  663,157 instructions (-37.7%), the function form the same, both rounds. **The invalidation that
  bought it is incomplete in one direction and wrong in the other** (new Critical C1, new Important
  I-A).
* ✅ **I4** (`double_of`). `dispatch/library.rs:445-457`. `d1`, 17 edge values, head SAME; the
  witness 18,720 kB peak and 0.02 s; base rc 134 under the cap.
* ⚠️ **M1** (kept resolution past `addPackage`). The reviewed shape is fixed (`q1`, the witness
  `library_routine_site_after_add_package`). The false comment is replaced by another false comment
  (Minor m1).
* ✅ **M2**. `phase-4-exclusions.txt:4553-4573` names owners, no count; both examples measured
  (`e1`, `e2`).
* ✅ **M3** and **the `::ATTRIBUTE` load site**. `phase-8.txt:134`; `w1` SAME; "under the name the call
  passed" is true on every route (`s1`, `t1`-`t7`).
* ✅ **M4**. `dispatch/library.rs:192` corrected. The `:704` half of the finding was the review's own
  error: `sed -n 704p` prints `bool NumberString::doubleValue(double &result)` in both trees.
* ✅ **M5**. `refusal-sites.tsv:162` names `library_routine_package_blame.rex`, whose call matches.
  The re-derive changes only locations plus the replaced 93.968 pair; no row has an empty measured
  column; the new `incorrect_native_signature` row measured (`m1a`-`m1c` SAME on head, 93.968 on
  base; `m2a`, `m2b` 93.968 SAME on both).

## New Issues In The Fix

### Critical

**C1. A kept external file survives `~addRoutine`, answering the file where the oracle answers the
routine** (ran). The call-site table drops a kept `External` only when `routine_generation` moves,
and `install_routine` (`dispatch/package.rs`, behind `~addRoutine` and `~addPublicRoutine`) writes
`routines` without moving it. `x3` (`ext(i)`) and `x3c` (`call ext i`), after
`.context~package~addRoutine('EXT', r)` on the first pass: oracle and base `2 added`, **head `2 ext 2`**,
rc 0, stderr empty. A regression against `c23c214f3`, which kept no `External`, and the brief's
definition of Critical. The same mechanism answers stale for a kept `Internal` and `LibraryRoutine`
(`x1`, `x2`), which the fix base already did.

### Important

**I-A. A kept merged library routine is re-resolved where the oracle keeps it for good** (ran). The
oracle stores a routine at the instruction only on `externalCall`'s Step 2 (`findRoutine`), and a
merged library routine is a Step 2 hit, so it is never looked up again; `can_be_shadowed`
(`run.rs:175-184`) lists `MergedLibraryRoutine`, so any unrelated generation move re-resolves it.
`p3`: main requires rxmath LIBRARY, `r.rex` run through `.Routine~newFile('r.rex',
.context~package)` loops `RxCalcSqrt(16)`, and a `loadPackage('pub.cls')` exporting a Rexx
`RxCalcSqrt` merges into `r.rex`'s package after the first pass: oracle and base `2 4`, **head `2 pk
16`**. `x4` is the same through `addRoutine` plus an unrelated merge. Regression. **Fix for C1 and
I-A together**: the oracle's rule. Keep `Routine` and `MergedLibraryRoutine` unconditionally; drop
`Internal`, `LibraryRoutine`, `External` and `Library` on a generation move; move the generation in
`install_routine` too. Witnesses `x3`, `x3c`, `p3`, `x4`.

**I-B. A context-built package's merged library routine is found before its parent's own routine**
(ran). `package_routine_lookup` (`run.rs:3846-3873`) walks own `routines`, own merged table, then the
parent's pair; `findRoutine` is `findLocalRoutine` (`routines` up the whole parent chain) and only
then `findPublicRoutine` (`classes/PackageClass.cpp:822-911`). Merging library routines into the
merged table moved them ahead of the parent walk. `x5c`/`r5`: `r.rex` requires rxmath LIBRARY, is
built with main as its context, and main declares `::routine rxcalcsqrt`: oracle and base `main 16`,
**head `4`**, rc 0. The same order already put a required package's Rexx public routine ahead of the
parent's local one (`x5d`, `p1` `fresh pub`, base too). **Fix**: walk `routines` up the chain, then
the merged tables up the chain, in `package_routine_lookup`, and give `package_find_routine`
(`environment.rs:1978-2004`, no parent walk at all: `r5` `mainr The NIL object`) the same walk.
Witnesses `x5c`, `x5d`, `r5`.

### Minor

**m1. The invalidation's prose is false** (falsified by `x3`, `x4`, `p3`): `Resolved::can_be_shadowed`'s
doc (`run.rs:167-174`: "every step `resolve_call` takes after the running package's own routines",
though `MergedLibraryRoutine` is found in the package lookup and a `Routine` from a merged table or a
parent is not listed; the events named omit `~addRoutine`); `routine_generation`'s doc
(`lib.rs:1784-1787`, "Moved whenever a routine may become callable in front of resolutions a call site
kept"); `ir/drive.rs:1537-1541` and `:700-703` ("dropped by the table itself"); `phase-8.txt`'s
fix-round block ("drops a kept resolution that a routine made callable since would shadow"); commit
`25bdb2dd5`'s title.

**m2. `importedRoutines` leaks a `Routine` per library routine per send** (ran, `g1`/`g2`).
`merged_routine_object` (`environment.rs:2010-2019`) builds a fresh object and
`record_loaded_executable` inserts it into `executable_sources`, which nothing prunes. 400,000 sends:
oracle rc 0; head rc 134 under the wrapper's cap with stdout lost, 838,208 kB uncapped. The
`findRoutine` form predates the round (80 MB, head and base). One object per `library_codes` row would
also make `findRoutine` answer the same object twice, as the oracle does (`m1b` `library 1 1` against
`0 0`, head and base).

**m3. `double_of`'s doc** (`dispatch/library.rs:442-445`) says the exponential form is taken "wherever
the plain one would pad with zeros"; `Number::format` keeps the plain form while `exponent >
-(2 * digits + 1)` (`rexx-num/src/lib.rs:812-813`), so `0.01` is written `0.01`. Inferred, not run; the
"never much longer than the digits" half holds.

**m4. The two new `run/tests.rs` tests** load `CARGO_MANIFEST_DIR/../../../build/lib`, the worktree's
own `.gitignore`d `build/` (a different interpreter build: `bin/rexx` 16,496 against 62,600 bytes,
`librxmath.so` differing), under `expect("the oracle's build directory is four above this crate")`,
beside doc comments quoting oracle measurements. Copied from `08d232ecc`; both libraries NEED only
`libc`/`libm`.

## Out-of-Scope Observations

* A call inside a method's argument list (`.array~of(foo())[1]`) re-resolves on every pass on head
  and base (`x14s`: `2 new` against the oracle's `2 old`).
* The crate resolves a function before evaluating its arguments; the oracle after (`o1`).
* An error inside an external file loses the caller's traceback line, first call included (`t6b`,
  `t6c`).
* `interpret "::requires ..."`'s 99.914 traceback lacks the oracle's directive line (`x12`).
* `.Routine~new(name, source, context)` is rc 120 (Phase 5) (`x5`).

## Assessment

**Fix round (integration):** Needs another round

I1, I4 and the minors are fixed and measured past their witnesses, and the external-file cost is
back (-37.7% per call). But the call-site generation does not implement the oracle's rule, which is
"keep a `findRoutine` hit for good, keep nothing else". So it answers stale after `~addRoutine` for
the `External` kind the round began keeping again (C1). It also re-resolves a merged library routine
the oracle keeps (I-A). And the one merged table put library routines ahead of a parent's own
routines (I-B). All three are silent rc 0 wrong answers, all three are regressions against
`c23c214f3`, and none has a witness.
