# Task 2 review, integration slice (everything outside `rust/crates/rexx-api/`)

Base `e7cb210d9`, head `c23c214f3`. Reviewer: integration slice. Written first and appended as the
review goes. Scratch: `scratchpad/t2-review-b/`, a fresh directory per probe.

## Passes over the diff

(appended as they happen)

* Pass 1 (read): `git diff e7cb210d9..c23c214f3` for `lib.rs`, `run.rs`, `ir.rs`, `dispatch.rs`,
  `dispatch/{library,executable,native}.rs`, `environment.rs`, `error.rs`, `internal_routines.rs`.
* Pass 2 (read): the C++ the new code cites, each line printed: `PackageClass.cpp` 693-772
  (`mergeRequired`, `mergeLibrary`), 822-911 (`findLocalRoutine`, `findPublicRoutine`,
  `findRoutine`), 1228-1250 (`processInstall`: libraries before requires), 1606-1640, 2007-2025;
  `HashContents::mergePut` (add-if-absent); `LibraryPackage.cpp` 256-299, 410-435;
  `PackageManager.cpp` 312-338, 520-526, 674-700; `ExternalFunctions.cpp` 104-135;
  `RexxActivation.cpp` 3062-3104; `Activity.cpp` 1093-1113; `RoutineClass.cpp` 205-211;
  `NumberStringClass.cpp` 706-750, 4073-4124; `Numerics.hpp` 86, 92; `Numerics.cpp` 272-302;
  `NativeActivation.cpp` 434, 454, 1922-1932, 2072-2081.
* Pass 3 (read): the new corpus witnesses and sidecars.
* Builds: `git archive` copies at `c23c214f3` (`t2-review-b/head`) and `e7cb210d9`
  (`t2-review-b/base`), release, own `CARGO_TARGET_DIR`s, both exit 0. Probes use
  `t2-review-b/ab.sh DIR` (fresh `mktemp -d` run dirs, three descriptors, paths normalised only
  for the comparison).

## Probes

### A: merged library routine against a required package's public routine (risk 1, concern 3)

Predictions, written before running, from `processInstall` merging every `::REQUIRES ... LIBRARY`
before any `::REQUIRES` and `mergePut` never replacing:
* `a1` (`::requires 'pk.cls'` then `::requires 'rxmath' LIBRARY`; pk.cls `::routine RxCalcSqrt
  public` returning `'pk' arg(1)`): oracle `a 4` / `b 5`; crate `a pk 16` / `b pk 25`.
* `a2` (the two directives in the other source order): the same as `a1` on both sides.
* `a3` (main requires `lib.cls`, which requires rxmath LIBRARY, then `pub.cls`): oracle `a 4` /
  `b 5`; crate `pk`.
* `a4` (`pub.cls` required before `lib.cls`): `pk` on both.

Ran (`ab.sh p/a1`..`p/a4`): **all four predictions confirmed.** `a1`, `a2`, `a3`: oracle rc 0
`a 4` / `b 5`, crate rc 0 `a pk 16` / `b pk 25`, stderr empty on both. `a4`: both `pk`. A silent
wrong answer, on `RxCalcSqrt(16)` and on `findRoutine`, whenever a program both requires a library
(directly or through a required package) and requires a package exporting a public routine of the
same name, unless the package is required before the one carrying the library.

Internal-against-library precedence: every routine table name under `interpreter/`
(`INTERNAL_ROUTINE`/`REXX_*_ROUTINE`, upcased) intersected with the routine table names of
`extensions/{rxmath,rxregexp,rxsock,platform/unix/rxunixsys,orxncurses}` is empty (command in
`t2-review-b/`, run 2026-09-15). So the documented `run.rs` divergence is unreachable with a shipped
extension; not probed further.

### B: load sites and the merged lookup's surface (risks 1 and 2)

Predictions, written before running:
* `b1` (main requires `loader.cls`, which requires `nested.cls`, and `third.cls`; the load is
  `loadLibrary` inside `loader.cls`'s routine, called from inside a loop whose call site ran before
  the load; then main, `third`, `nested`, an `INTERPRET`, a `loadPackage`ed `late.cls`, and
  `findRoutine` in main and in the loader): oracle `pre code 43.1`, `loop 1` missing then `load 1`,
  `loop 2 2`, `loop 3 3`, `main 4`, `third 5`, `nested 6`, `interp 3`, `late 7`, `find main The
  NIL object`, `find loader The NIL object`. Crate: the same.
* `b3` (main requires `mid.cls`, which requires rxmath LIBRARY): oracle `findRoutine` and
  `findPublicRoutine` answer a routine in both packages (`2 3 4 5`); `routines` and
  `publicRoutines` `0` with `The NIL object`; **`importedRoutines` is `mergedPublicRoutines`
  (`PackageClass.hpp:172`), so it holds the library's routines: `18 The Routine class` in both
  packages** (rxmath's table has 18 names, `t2-review-b/collide.sh`). Crate: everything but
  `importedRoutines`, which I predict answers `0` and fails on `~class` of `.nil`... i.e. differs.

Ran. **`b1` was a malformed probe** (a label inside a `DO` block: 47.2 on both, and nothing of the
subject ran); rewritten as `b1b` with the `SIGNAL ON SYNTAX` in a procedure, same predictions.
`b3` ran: oracle as predicted except `main importedRoutines` is `19` (the library's names plus
`MID`, merged from `mid.cls`), `mid routines`/`publicRoutines` `1` (its own `MID`). Crate: rc 120
at the first `~items`, `method "ITEMS" of class "StringTable" is not implemented (Phase 5)`, so the
counts are unobservable here; rewritten as `b3b` reading `['RXCALCSQRT']` directly. Prediction for
`b3b`: oracle `The Routine class`-valued entries print as the routine's string value (`RxCalcSqrt`
upcased? unknown, I do not predict the rendering), crate `The NIL object` for `importedRoutines`.

Ran `b1b`: **SAME**, every line as predicted, rc 0 both. `b3b`: **silent divergence**, rc 0 both:
`importedRoutines['RXCALCSQRT']` is `a Routine` on the oracle in main and in `mid.cls`, `The NIL
object` on the crate; every other line agrees (`findRoutine`, `findPublicRoutine` answer, `routines`
and `publicRoutines` `.nil`, `importedRoutines['MID']` `a Routine`). Base binary: `b3b` stops at
rc 159 on `findRoutine` (`.nil~call`), and `a1` already answered `pk 16` at base, so neither is a
regression; both sit on the table this task claims to build as `mergeLibrary` does.

Predictions for `b4` (the security manager on a program whose library merge comes two
`::REQUIRES` deep) and `b5` (the name-as-written traceback through the same two levels): oracle,
no `event CALL` line, `merged 4`, `pi 3.14`; `b5` `Error 88` naming `main.rex line 2` and
`Compiled routine "rxcalcsqrt"`. Crate: the same.

Ran `b4`, `b5`: **SAME**, as predicted (no `CALL` event, `merged 4`, `pi 3.14`; the `b5`
traceback names `"rxcalcsqrt"` at `main.rex line 2`, rc 168 both).

### C: the call-site table (risk 3, correctness half)

`c1`: `loadLibrary('rxmath')`, then a procedure's literal call `'rxcalcsqrt'('x')` twice from one
site, with `.context~package~addPackage(loadPackage('mid.cls'))` (mid requires rxmath LIBRARY)
between. Prediction: oracle `1 *-* Compiled routine "RXCALCSQRT".` (global table, upcased), then
`2 *-* Compiled routine "rxcalcsqrt".` (`addPackage` runs `mergeRequired`, which carries mid's
merged library routines into main, found first and run under the name as written), `find 1`.
Crate: if `addPackage` propagates the merge, `find 1`; the site's kept `LibraryRoutine` would still
print `"RXCALCSQRT"` on line 2.

Ran `c1`: the prediction was unobservable as written, and something else showed. The crate's
condition object `TRACEBACK` has no `*-* Compiled routine` line: oracle `1        *-* Compiled
routine "RXCALCSQRT".`, crate `1     10 *-*     return 'rxcalcsqrt'('x')` (both rc 0; `find 1`
agrees). Two follow-ups, predictions first:
* `c2`, the cache question asked through an uncaught error: `sq(4)` then `sq('x')` from one site,
  `addPackage` of mid.cls between. Oracle `2`, then the traceback `Compiled routine "rxcalcsqrt"`
  (merged, as written). Crate: `"RXCALCSQRT"` if the site kept `LibraryRoutine`.
* `c3`, whether the missing `TRACEBACK` line is this task's or older: `filespec('Z', 'a')` and
  `RxCalcSqrt('x')` each trapped. Prediction: both lines missing from the crate's list at head, and
  `filespec`'s missing at base too (pre-existing for internal routines).

Ran `c2`: **prediction confirmed, a divergence**: oracle rc 168 with `Compiled routine
"rxcalcsqrt"`, crate rc 168 with `"RXCALCSQRT"`, the other stderr lines and stdout `2` identical.
The site kept `Resolved::LibraryRoutine`, which the later `addPackage` merge should shadow.
`c3`: prediction confirmed; the condition object's missing `Compiled routine` line is the same at
base for `filespec` (and for the 43.1 RxCalcSqrt gave there), so pre-existing and not this task's.

Follow-ups, predictions first:
* `c4`: `loadLibrary`, then `RxCalcSqrt(16)` from one loop site twice with `addPackage` of a package
  exporting a public `RxCalcSqrt` Rexx routine between, then a fresh site. Oracle `1 4`, `2 pk 16`,
  `fresh site pk 16`. Crate head `1 4`, **`2 4`**, `fresh site pk 16`. Base: 43.1 on line 1 (the
  library routine was not callable), so this wrong answer is new.
* `c5`, the same shape over an internal routine (`filespec`), to see whether the kept-resolution
  hazard predates the task: oracle `1 b.c`, `2 pk N`, `fresh site pk N`; crate head and base
  `2 b.c`.

Ran `c4` and `c5`, head and base binaries: **every prediction confirmed.** `c4`: oracle `1 4` /
`2 pk 16` / `fresh site pk 16`; head `1 4` / **`2 4`** / `fresh site pk 16`, rc 0 both; base rc 213
43.1 on the first call. `c5`: oracle `2 pk N`; head and base both `2 b.c`. So a kept resolution
below the package lookup (`Internal`, and now `LibraryRoutine`) is not dropped when `addPackage`
changes what the package lookup answers; the hazard predates the task, and the task adds a
reachable silent instance of it (`c4`) plus the traceback-name one (`c2`).

### C, performance half (risk 3)

`t2-review-b/bench/run.sh`: callgrind totals, base then head per program, two rounds, fresh run
directory each: `ext200`/`ext400` (a loop calling an external `ext.rex` that returns its argument,
200 and 400 times, differenced for a per-call figure) and `cps` (`rust/bench-rexxcps/rexxcps.rex`).
Running in the background while the probes below go on.

### D: the lineless rule (risk 4)

Predictions, written before running, from the `Activity.cpp:1093-1113` frame walk (a native frame
reports its code's package, `.nil` until a directive binds it):
* `d1` `loadExternalRoutine` then `r~call('abc')` on main line 3: `Error 88 running main.rex line
  3`, 88.921, traceback `Compiled routine "RxCalcSqrt"` under `Compiled method "CALL"`. Crate same.
* `d1b` `r~callWith(.array~new)`: 88.901 at `main.rex line 3`. Crate same.
* `d2` `sq('abc')` from main, `sq` a `::ROUTINE ... EXTERNAL` in required `pk.cls`: `Error 88
  running pk.cls:` with no line, `Compiled routine "SQ"`. Crate same.
* `d2b` the same routine called as `sq(2, 0)` from a third package's routine: `pk.cls:` no line,
  88.905. Crate same.
* `d2c` `findRoutine('SQ')~call(1, 2, 3)` from the third package: `pk.cls:` no line, 88.922,
  traceback `Compiled routine "RxCalcSqrt"`. Crate same.
* `d3` a `loadExternalRoutine('r', 'LIBRARY rxmath rxcalcsqrt')` made before `loadPackage('pk.cls')`
  binds the entry, called after: `pk.cls:` no line (retroactive). Crate same.
* `d4` a `~define`d `loadExternalMethod` answer called with no argument: `first 88.901` with a
  `POSITION` of `5` and program `main.rex`; then after `loadPackage` of a package whose `::METHOD
  ... EXTERNAL` binds `RegExp_Pos`, uncaught, `Error 88 running pk.cls:` no line. I do not know
  `RegExp_Pos`'s signature; if no argument is not a boundary refusal the probe says nothing.

Ran all seven: **SAME on all three descriptors, every prediction confirmed** (`d1` line 3 88.921;
`d1b` line 3 88.901 under `CALLWITH`; `d2` `pk.cls:` no line, `"SQ"`; `d2b` `pk.cls:` 88.905
through the third package; `d2c` `pk.cls:` 88.922 with `"RxCalcSqrt"`; `d3` retroactive `pk.cls:`;
`d4` `first 88.901 5 main.rex` then `pk.cls:` no line after the later directive).

Predictions for the `LIBRARY REXX` path, which no witness drives from a required package:
* `d5` `fs('Z', 'a')` from main, `fs` a `LIBRARY REXX Filespec` directive in required `pk.cls`:
  the routine's package is the `REXX` package, which has no source line; I do not know how the
  oracle names that package in the `Error 40 running ...` line, so I predict only SAME.
* `d6` an unbound-looking `loadExternalRoutine('x', 'LIBRARY REXX filespec')~call()`: its package is
  the REXX package already (report `rx8`), so the same report as the witness `..._call_blame`.
  Predict SAME.

Ran `d5`, `d6`: **SAME** both (`d5` 40.904 at `main.rex line 2`, `"FS"`; `d6` `Error 88 running
REXX:` no line, `"Filespec"`). Follow-up, prediction first: `d7` `fs()` uncaught through the same
directive, and `d8` plain `filespec()`: 88.901 is the boundary's own, so by `d6` I predict `Error
88 running REXX:` on the oracle for both; crate SAME.

Ran `d7`, `d8`: **SAME**, `Error 88 running REXX:` no line on both.

### E: argument conversions the host side (`dispatch/library.rs`) adds

Reading `double_of` (`library.rs`): `number.format(u64::MAX)` never takes the exponential form
(`rexx-num/src/lib.rs:811-813` saturates `digits` to `i64::MAX`), so it writes every digit out.
Predictions, written before running:
* `e1`: oracle `a 1.234567890123457`; `b` unknown to me (whether a string argument's numberstring
  keeps its digits past `NUMERIC DIGITS 5`), crate `b 1.234567890123457`; `c 1.2346` both; `d
  +infinity 0 nan`; `e +infinity 0`; `f 2E-15`. Crate SAME on `a`, `c`-`f`.
* `e2`: oracle `g +infinity`, `h 0` at once. Crate: `1E+999999999` formats to a 1 GB string of
  zeros before `parse`, so slow and a gigabyte of memory (run here without the oracle's `ulimit`),
  and `1E-999999999` the same.
* `e3` (`positive_wholenumber_t` precision): I predict only SAME, having not read `int64Value`.

Ran `e1`, `e3`: **SAME** on all three descriptors. My `e1` renderings for `c` and `f` were wrong
(both sides print `c 1.234600000000000` and `f 0.000000000000002`), and `b` is `1.234567890123457`
on both. `e3`: `a 1.414213562 1.41 1.414213562373095`, `b 1.41 88.905 88.905 88.905 88.905`, `c
88.905 88.905 1.41` on both.

Ran `e2`: SAME under `ab.sh` (no cap on the crate side): `start`, `g +infinity`, `h 0`, rc 0.
**The cost prediction confirmed**: `/usr/bin/time -v` directly on the crate binary, 1.45 s wall
and `Maximum resident set size` 1971608 kB for the two conversions. Under the oracle's own
`ulimit -v 1048576`, the crate is **rc 134, `memory allocation of 999999999 bytes failed`, stdout
empty** (the `start` line lost), where the oracle answers at once inside that cap.

### F: the call-site table's second hit

Found while benchmarking (risk 3): **at base, an external routine called twice from one expression
site fails on the second call**: `f1` (`do i = 1 to 3; say i ext(i); end`, `ext.rex` beside) is
oracle `1 1` / `2 2` / `3 3` rc 0, base crate `1 1` then `Error 43.1: Could not find routine ""`
rc 213; `f2`, the same through `call ext i`, is SAME at base. Head: `f1` SAME. Cause, read:
`ir/drive.rs:246-270`, `Op::Call`'s arm leaves `spelling` as `b""` on a site-table hit ("The
spelling is recovered only when the site has nothing"), and `call_over_pushed_args` hands that
empty name on; base's cached `External` searched for `""`. Head no longer keeps `External`, which
fixes this without the report or the `ir.rs` comment saying so.

But every resolution this task added reads the name on that path: `call_over_values`
(`run.rs:4121-4126`) gives it to `call_checkpoint` and `run_package_routine`; `invoke_call_over`
gives it to `run_library_routine` for `MergedLibraryRoutine` and a library-bound `Routine`, and to
`run_internal_as(row, Some(name))` for a `LIBRARY REXX` one. Predictions, before running (oracle,
then crate head):
* `g1` merged, `say RxCalcSqrt(a)` over `4`, `'x'`: oracle `2`, then `Compiled routine
  "RXCALCSQRT"`; crate **`Compiled routine ""`**.
* `g2` the same through `loadLibrary` (the global table): the same pair.
* `g3` `sq(a)`, a directive in required pk.cls: oracle `"SQ"`; crate `""`.
* `g4` `fs(a, '/a/b.c')` over `'N'`, `'Z'`, `fs` a `LIBRARY REXX Filespec` directive: oracle `b.c`
  then 40.904 under `"FS"`; crate `""` in the traceback. The 40.904 message itself names
  `FILESPEC` (the row's), so I predict it unchanged.
* `g5` a security manager over `global.rex` looping `RxCalcSqrt(a)` twice and `filespec(a, ...)`
  twice: oracle `event CALL RXCALCSQRT` twice and `event CALL FILESPEC` twice; crate the second
  of each pair `event CALL ` with an empty name (the `filespec` half is base's already, if so).

Ran `g1`-`g5`: **every prediction confirmed.** rc and stdout agree on `g1`-`g4`; the stderr
traceback's first line is `*-* Compiled routine "".` on the crate against `"RXCALCSQRT"` (`g1`,
`g2`), `"SQ"` (`g3`) and `"FS"` (`g4`) on the oracle. `g5`, rc 0 both: the crate's second
`RxCalcSqrt` and second `filespec` checkpoints are `event CALL ` with no name. `g6` (the `filespec`
half alone) on the base binary: the same empty name, so the checkpoint half for internal routines
predates the task; for library routines, which did not run at base, all of it is new.

### H: load sites no witness drives

Predictions, before running:
* `h1` `::attribute a get external "LIBRARY rxmath RxCalcSqrt"` in a `loadPackage`d package (the
  `::ATTRIBUTE` load site, which `phase-8.txt`'s list omits): refused at translation with some
  code, then `main 4`. Predict SAME.
* `h2` rxmath loaded as `rxmath` and again as a relative path that names the same file (the run
  directory holds an empty `lib/`, so `lib` + `/../../...` + `.so` resolves from it on both
  sides): `a 1`, `b 1`, `c 4`, `d 5 6`. Predict SAME.

Ran `h1`, `h2`: **SAME** both (`loadPackage 90.998` / `main 4`; `a 1` `b 1` `c 4` `d 5 6`).

### C, performance half: results

`bench/results.txt` and `bench/results-callext.txt`, both `finished`, instruction totals:

| program | base r1 | base r2 | head r1 | head r2 |
|---|---|---|---|---|
| `cps` | 21,204,995,027 | 21,205,631,427 | 21,215,573,901 | 21,216,142,102 |
| `ext200` (function form) | 125,095,157 rc 213 | 125,122,405 rc 213 | 334,001,185 | 334,095,253 |
| `ext400` | 125,068,064 rc 213 | 125,075,155 rc 213 | 544,461,821 | 544,565,504 |
| `callext200` (`call ext i`) | 255,517,354 | 255,564,189 | 334,061,153 | 334,075,209 |
| `callext400` | 387,041,700 | 387,183,104 | 544,692,598 | 544,622,405 |

* `rexxcps`: head +0.05% instructions over base (+10.5M of 21.2G, both rounds). Nothing here.
* The function-form base runs are the `f1` bug (43.1 `""` on the second call), so the before side
  is `callext`: per call, (400 - 200) / 200, **base 657,860, head 1,052,950 instructions: +60% per
  external-file call**, with the fixed cost unchanged (intercepts 123.97M and 123.48M).
* `callgrind_annotate --inclusive=yes` on the `callext400` outputs: `Interp::external_program` is
  155,204,929 (40.10%) at base and 311,744,588 (57.23%) at head, i.e. one search per call became
  two, each about 388k instructions: the uncached resolution's search plus the consumer's own. The
  `Resolved::External` doc's "the second search is stats only, against a call that re-reads and
  re-parses the file anyway" is now paid every call and is 40% of such a call.
* Wall clock, 5000 `call ext i`, interleaved oracle/base/head x3: oracle 0.203 / 0.202 / 0.211 s,
  base 0.489 / 0.494 / 0.502 s, head 0.785 / 0.787 / 0.786 s.

`i1` (`call MathDropFuncs` under `::requires 'rxmath' LIBRARY`; `MathDropFuncs` is a no-op
answering `""`, read at `extensions/rxmath/rxmath.cpp:389-393`, and registers nothing), checking
whether the exclusions entry's "Two forms still refuse" is the whole of what refuses on the routine
path: prediction, oracle `start` / `result ` rc 0; crate rc 120 on the `CSTRING` result row naming
Phase 8 (report concern 2).

Ran `i1`: **confirmed**: oracle `start` / `result ` rc 0; crate `start` then rc 120 `rexx-exec:
Phase 8 owes the FromNative conversion for REXX_VALUE_CSTRING (15) is not implemented (Phase 8)`.

### J: other ways a `loadExternalMethod` answer becomes a method

`bind_loaded_method` is called from `define_method_object` and the `defineMethods` loop only.
Predictions, before running: `j1` `.K~defineClassMethod('POS', m)` then `.K~pos()`, `j2`
`.Directory~new~setMethod('POS', m)` then `d~pos()`, `j3` `self~setMethod('POS', m)` inside a
method then `k~pos()`: oracle `defined`/`set`/`installed` then 88.901 `Missing argument` at the
sending clause (line 4 or 5) rc 168 each. Crate: loud (rc 120 naming a phase) where the method is
installed without the binding; if any answers 88.901 identically, SAME.

Ran `j1`-`j3`. `j1` was a malformed probe (`defineClassMethod` is not understood by a user class,
97.1 on both, SAME). `j2` (`Directory~setMethod`) and `j3` (`self~setMethod`): oracle `set` /
`installed` then 88.901 at line 5 rc 168; crate rc 120 `a one-off method whose body this crate
does not hold is not implemented (Phase 5)` at the `setMethod`, stdout empty. Loud and owned by
Phase 5, outside Step 4 ("when a class defines it"); recorded, not an issue.

`k1`, the security manager's `CALL` name for a global-table routine called by a lower-case literal,
a symbol, and `call 'rxCalcSqrt'`, each a fresh site: prediction, SAME (I have not found where the
oracle's checkpoint takes its name, so I do not predict the spelling).

Ran `k1`: **SAME** (`rxcalcsqrt`, `RXCALCSQRT`, `rxCalcSqrt`: the checkpoint takes the name as the
call wrote it, on both).

`k2`, whether a kept resolution can cross packages through one `INTERPRET` text (which would carry
`MergedLibraryRoutine` into a package with no merge): two packages each `interpret "say helper()"`
with a private `helper` of their own, twice. Prediction: oracle `main`/`other` twice; crate SAME.

Ran `k2`: **SAME** (`main`/`other` twice on both).

### V: a version-refused library registers nothing (risk 1)

`scratchpad/rereview-int/ext/libforgever.so` (the fix re-review's forged library: `requiredVersion`
6.0.0, typed routine `ForgeverRoutine`; `readelf -d` lists no `NEEDED` at all), on
`LD_LIBRARY_PATH` for both sides (`t2-review-b/ab-ext.sh`). `v1`: trapped first `loadLibrary`,
second `loadLibrary`, `ForgeverRoutine()` by name, `loadExternalRoutine` over it, then a
`loadPackage`d package whose `::REQUIRES 'forgever' LIBRARY` would merge it and whose routine calls
it, and `findRoutine` through that package. Prediction, from `LibraryPackage::loadPackage`
refusing before `loadRoutines`: oracle `first 98.982`, `again 1`, `routine 43.1`, `loaded The NIL
object`, `merged code 43.1`, `find The NIL object`. Crate SAME.

Ran `v1`: **SAME**, every line as predicted.

### Instruments (risk 6), by reading the witnesses

* A witness that fails if registration happens only at `::REQUIRES`: **yes, several.**
  `library_routine_load_library.rex` (`loadLibrary` in `loader.cls`), `_directive.rex`
  (`::routine sq external` in pk.cls), `_load_external.rex`, `_load_external_method.rex` and
  `_method_directive_load.rex` carry no `::REQUIRES ... LIBRARY` in any file, and each prints a
  library routine's answer by name, which would be 43.1 under that mutant. So C3's red list is what
  the witnesses' text implies; its `library_routine_security_manager` miss is explained by
  `merged.rex`'s own `::requires 'rxmath' LIBRARY`, as the report says. I did not rebuild C3.
* C5's red set (`caller_blame`, `call_blame`, `library_method_loaded_defined`, `name_as_written`)
  and green set (`package_blame`, `loaded_defined_blame`) match which witnesses bind their code by a
  directive. C4's single red (`after_external_file`) is the only witness whose one site resolves to
  an external file first.
* **No witness calls a library routine twice from one expression site**, which is why `g1`-`g5`
  are green in the corpus, and none calls an external file twice from one function-form site
  (`f1`), which is why base's `""` went unnoticed. The kept-resolution table's instrument is
  `library_routine_after_external_file` alone.

### Not done

No test suite, gate, Miri or corpus run (the brief's rule); the report's `571 of 571`, Miri and
clippy/fmt claims are unverified here. No mutant built; controls checked by reading only.

---

## Spec Compliance (integration slice)

* ✅ **Step 3, registration at the one shared path**: `lib.rs:4715-4730` (`settle_library` calls
  `register_package_routines`, `:4737`, on `Ok(Some)` only). Probed past the witnesses: `b1b` (a
  load inside a required package's routine, then main, a third package, a package that package
  requires, `INTERPRET`, a later `loadPackage`, a loop site that ran before the load), `h1` (the
  `::ATTRIBUTE ... EXTERNAL` site), `h2` (one file under two spellings), `v1` (a version-refused
  library registers and merges nothing, three routes): all SAME x3.
* ✅ **Which packages see it** measured on the oracle (report `s1b`; `b1b` here).
* ✅ **Step 4**: `dispatch/executable.rs` `enter_routine` runs a `Loaded` or directive-bound library
  routine and a `LIBRARY REXX` row; `environment.rs` `bind_loaded_method` for `define`/
  `defineMethods`. `d1`, `d1b`, `d3`, `d4`, `d6` SAME. (`Directory~setMethod` and `self~setMethod`
  of such an answer stay loud Phase 5, `j2`/`j3`, outside the step's wording.)
* ✅ **Step 5**: the refusals are deleted (`lib.rs` `library_routine_call`,
  `required_library_routine`, `routine_without_a_body`, `dispatch.rs` loadExternalRoutine-on-REXX);
  `run/tests.rs:7083-7410` expectations updated; `refusal-sites.tsv` re-derived, and
  `c23c214f3`'s "only their line column changes" holds (38 changed lines, all `run.rs` line columns).
* ⚠️ **Step 6, "a typed routine answering through each load site"**: the sites `phase-8.txt:124-128` lists
  are witnessed; the `::ATTRIBUTE ... EXTERNAL` site is not (`h1` SAME). Classic style has only `rexx-api`'s unit test, justified in the report.
* ✅ **Ruling: `REGISTERED` re-homed to Phase 10 with the measurement as the reason**:
  `lib.rs:804-816`, `phase-4-exclusions.txt`; no witness or test here runs it against the oracle
  (`run/tests.rs:7092` is crate-only). `LIBRARY REXX` implemented: `d5`-`d8` SAME.
* ✅ **`"Phase 8"` on these paths**: `/bin/grep -rn '"Phase 8"' rust/crates/*/src` names
  `dispatch/native.rs:845` (the `OPEN` list) and `dispatch/library.rs:234`
  (`Unfilled`/`StaleHandle`/`Raised`, Tasks 3 and 5 by the report); the refusal table has no row
  naming Phase 8.
* ❌ **"`::REQUIRES ... LIBRARY` merges ... as `PackageClass::mergeLibrary` does"** (commit and
  report): the lookup order is not the oracle's (`a1`-`a3`), and `importedRoutines` does not see
  the merge (`b3b`). Issue I2.
* ❌ **"run under the name as written" / "the name as the call passed it"** (`Resolved` doc,
  `phase-8.txt`): false from the second call at one expression site (`g1`-`g4`). Issue I1.
* ✅ **Lineless rule**: `dispatch/library.rs:187-205` (`settle_native_call`) and `:239-241`; `d1`-`d8` SAME,
  including a directive-bound routine called from a third package and through `~call` there, a
  retroactive binding, and a `~define`d method before and after a later directive.
* ✅ **Oracle crash entry 15**: no new witness passes a `loadExternal*` answer as a context.

## Strengths

* The registration is where every load settles, not at a caller, and it survives the probes that
  sank the previous attempt: packages translated before the load, nested requires, a later
  `loadPackage`, `INTERPRET`, a loop site that missed before the load, two spellings of one file,
  a version-refused library.
* The blame model is right in every shape I could build (`d1`-`d8`): the caller's line for unbound
  code, the binding package with no line for bound code, retroactive binding, the `REXX` package's
  lineless `Error 88 running REXX:`, and the traceback's name per route.
* The `%g`/`newInstanceFromDouble` reproduction held on every edge I tried (`e1`, `e3`: numbers
  past `NUMERIC DIGITS`, `1E+400`, `1E-400`, tiny values, a 20-plus-digit precision, `-0`, `1e19`).
* `rexxcps` is flat: +0.05% instructions.
* The report records its own falsified control predictions (C3, C5) instead of smoothing them, and
  discloses the precedence gap and the rxapi side effect.

## Issues

### Critical

None.

### Important

**I1. Every library routine called a second time from one expression call site runs under the name
`""`.** `ir/drive.rs:250` leaves `spelling` as `b""` on a site-table hit, and this task's new
consumers read it: `run.rs:4121-4126` (`LibraryRoutine`: `call_checkpoint(name)` and
`run_package_routine(slot, name)`), `run.rs:4215-4231` (`MergedLibraryRoutine` and a library-bound
`Routine`: `run_library_routine(code, name)`; a `LIBRARY REXX` directive:
`run_internal_as(row, Some(name))`). Ran: `g1` (merged), `g2` (`loadLibrary`), `g3` (`::ROUTINE`
in a required package), `g4` (`LIBRARY REXX`): stdout and rc agree, the stderr traceback reads
`*-* Compiled routine "".` against the oracle's `"RXCALCSQRT"`/`"SQ"`/`"FS"`; `g5`: the security
manager's second `CALL` event carries an empty `NAME`. The checkpoint half predates the task for
internal routines (`g6` on the base binary); the traceback half is new, and it is the ordinary
shape (a library routine in a loop whose argument goes bad). No witness calls a site twice. **Fix**:
recover the spelling on a hit whenever the resolution reads it (every arm but `Builtin`), and add
witnesses that call each resolution twice from one expression site, erroring on the second, plus a
two-call security-manager witness.

**I2. A merged library routine is found after every `::ROUTINE`, including the required packages'
public ones; the oracle merges libraries first into the same table.** `run.rs:3767-3770` tries
`installed_routine` (own, then `merged_public_routines`, then parents) before
`merged_library_routine`; `environment.rs:1986-2004` does the same for `findRoutine`. The oracle's
`processInstall` (`PackageClass.cpp:1235-1247`) runs every `::REQUIRES ... LIBRARY` before any
`::REQUIRES`, `mergeLibrary` (`:756-772`) and `mergeRequired` (`:693-722`) both write
`mergedPublicRoutines` through `mergePut`, which never replaces. Ran: `a1`, `a2` (either source
order) and `a3` (library two levels down, required first): oracle `a 4` / `b 5`, crate `a pk 16` /
`b pk 25`, rc 0 both, a silent wrong answer on `RxCalcSqrt(16)` and on `findRoutine`; `a4` (the
public package required first) agrees. Same table: `importedRoutines` (`PackageClass.hpp:172`,
`mergedPublicRoutines`) holds the library's routines on the oracle and not on the crate (`b3b`:
`a Routine` against `The NIL object`, main and `mid.cls`). Both predate the task (`a1` answered
`pk 16` at base) but sit on the table the commit says it builds "as `PackageClass::mergeLibrary`
does", and the report's concern 3 left the order unmeasured. **Fix**: one ordered merged lookup
per package (libraries first, then each required package's public and merged entries in requires
order, add-if-absent), read by `resolve_call`, `findRoutine` and `importedRoutines`; witnesses
`a1`/`a3`/`b3b`.

**I3. Not keeping `Resolved::External` costs +60% per external-file call.** `ir.rs:509`. Ran,
callgrind, base then head, two rounds: `call ext i` 200 and 400 times, per call **657,860 base
against 1,052,950 head** instructions (fixed cost unchanged); `Interp::external_program` inclusive
155.2M (40.1%) at base, 311.7M (57.2%) at head, so one ~388k-instruction search per call became two.
Wall clock, 5000 calls interleaved x3: oracle 0.20 s, base 0.49-0.50 s, head 0.785-0.787 s.
`rexxcps` +0.05%, so only this axis moves. The correctness reason is real (`library_routine_after_
external_file`), but it needs only "a library registered since the site resolved": a registration
counter stamped into the kept `External` and compared on a hit, or a `package_routine(name)` probe
on a hit, keeps the answer and the cache. Either needs I1's spelling fix, since base's kept
`External` failed on its second function-form hit for exactly that reason (`f1`: base 43.1 `""`,
head SAME), a fix the task made without the report, the commit message or `ir.rs:500-507` saying
so. **Fix**: re-keep `External` with invalidation; add an `f1`-shaped witness (an external file
called twice from one function-form site), which nothing pins today.

**I4. `double_of` writes a Rexx number out digit by digit, so a huge exponent allocates
gigabytes.** `dispatch/library.rs:435-436`, `number.format(u64::MAX)` (`rexx-num/src/lib.rs:811-813`
saturates `digits`, so the exponential form is never taken). Ran `e2`
(`RxCalcSqrt('1E+999999999')`, `RxCalcSqrt('1E-999999999')`): oracle `+infinity` / `0` at once;
crate the same bytes but 1.45 s and 1,971,608 kB peak RSS (`/usr/bin/time -v` on the binary
itself), and **under the oracle's own `ulimit -v 1048576`, rc 134 `memory allocation of 999999999
bytes failed` with the earlier `start` line lost**. Both are valid numbers and legal arguments.
**Fix**: hand `parse::<f64>` the digits and exponent (`"{digits}e{exponent}"`), which it reads to
the same double; witness `e2`.

### Minor

**M1. A kept `LibraryRoutine` survives an `addPackage` that changes the answer.** `c4`: oracle
`2 pk 16`, crate `2 4`, rc 0 (at base the call was 43.1); `c2`: the merged routine's traceback name
stays upcased. The shape predates the task for `Internal` (`c5`, base and head `2 b.c` against
`2 pk N`), and `addPackage` of a same-named public routine mid-loop is rare. The comment at
`ir/drive.rs:1529-1531` ("Nothing invalidates one") is now false for two resolution kinds. Route
to whoever owns the call-site table; the I3 invalidation stamp could cover `addPackage` too.

**M2. `phase-4-exclusions.txt:4553-4564` marks the routine half CLOSED with "Two forms still
refuse".** The same path still refuses on conversion rows naming Phase 8 (`i1`: `call
MathDropFuncs` under `::requires 'rxmath' LIBRARY`, oracle `result ` rc 0, crate rc 120 on
`REXX_VALUE_CSTRING`) and aborts in `RxCalcSin(30, 3, 'X')` (report concern 1). An extent claim
that is short, and a set size in prose. **Fix**: name what still refuses by owner (Task 3's result
rows, Task 5's thread slots) without a count.

**M3. `phase-8.txt:124-128` follows "every load site" with a list that is not every load site.**
`::ATTRIBUTE ... EXTERNAL` is a load site the fix re-review measured, and no row names or witnesses
it (`h1` SAME). The block at `:137-139` says "the name as the call passed it", false past a first
call at one expression site (I1). **Fix**: drop the parenthetical list or add the `::ATTRIBUTE`
witness.

**M4. Two citations miss their subject.** `dispatch/library.rs:192` names
`Activity::createExceptionObject` for `Activity.cpp:1093-1113`, which is
`Activity::generateProgramInformation` (`:1080`; `createExceptionObject` calls it at `:1063`).
`double_of`'s `NumberStringClass.cpp:704` is the doc block's `@return`, two lines above
`doubleValue` (`:706`).

**M5. The `refusal-sites.tsv` row for `library_procedure_gone`** keeps the method-side witness
(`library_method_package_blame.rex`) while the constructor now has two routine-side sites in
`run_library_routine`; the "no route" answer holds for them too by reading (every
`library_codes` key names a held library), so this is wording only.

Noted, not this task's: the condition object's `TRACEBACK` lacks the `*-* Compiled routine` line for
internal and library routines alike (`c3`, same at base); a parse error inside a loop body renders
as `rexx-exec: 47.2` rc 120 against the oracle's `Error 47` rc 209 (seen in the malformed `b1`).

## Assessment

**Task quality (integration slice):** Needs fixes

The registration, the load-site coverage and the blame model hold under every probe past the
witnesses. The fixes are I1 (an empty routine name from the second call at any expression site, on
the task's own new paths, unwitnessed because every witness sends once), I2 (a silent wrong
answer on the merged table the task claims to build as the oracle does, measured now where the
report left it open), I3 (a +60% per-call cost on external-file calls that invalidation avoids),
and I4 (an abort on a legal argument under the oracle's own memory cap).
