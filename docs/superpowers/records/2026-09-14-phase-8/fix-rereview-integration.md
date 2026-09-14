# Fix re-review, rexx-exec slice: F1-F6, F10, F11

Range `e64202ae7..cf92ff4fb`. Reviewer read-only. Scratch:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/rereview-int/`.
Build: `git archive cf92ff4fb rust` in that scratch, `CARGO_TARGET_DIR` in scratch, `-j 4`.

(Written first; sections appended as the work runs.)

## Counts

**Critical 0, Important 1 (R1), Minor 9 (R2-R10).** R2, R4 (mechanism) and R9 predate the round
and are recorded because the round's claims or new paths meet them. Every finding was run unless its
heading says read.

* R1 Important: F4's per-spelling record is wrong for routines; the oracle shares one routine object
  per library entry found caselessly, so `loadExternalRoutine('r', 'LIBRARY rxmath RxCalcSqrt')~package`
  stays `.nil` after a `::routine ... external "LIBRARY rxmath RXCALCSQRT"` binds it (oracle
  `pk.cls`), and the F4 commit message says routines behave like methods.

## 1. Original findings, by their original probes

Binary: `rereview-int/target/release/rexx-run`, built from `git archive cf92ff4fb rust` (sha256
prefix `2f46608f84d4c18d`). Runner `rereview-int/cmp.sh NAME SRCDIR`: copies the probe's `.rex`/`.cls`
from `final-b/p/<name>/` into a fresh `rereview-int/p/NAME/`, oracle under the standard wrapper and
the build in that same directory, `</dev/null`, three descriptors to separate files, `cmp`.

| B probe | finding | oracle | cf92ff4fb | verdict |
|---|---|---|---|---|
| `p2c` (`doparse(.object~new)`) | B1 | rc 168, `Error 88 running .../main.rex:` / 88.909 | SAME x3 | closed |
| `p3` (MAKESTRING raises 40.1) | B1 | rc 216, `Error 40 ... line 11` / 40.1, `REQUEST` line in the traceback | SAME x3 | closed |
| `p5` (STRING, no MAKESTRING) | B1 | rc 168, 88.909 | SAME x3 | closed |
| `p2f`, `p2d`, `p4`, `p6`, `n1` (B1 neighbours) | B1 | rc 168 / 2 / 0 / 0 / 0 | SAME x3 each | no regression |
| `p2e` (required `re.cls`, NOSTRING trapped) | B2 | rc 168, `Error 88 running .../re.cls:` no line | SAME x3 | closed |
| `a1` `a2` `a3` `a5` | B6 | `pk.cls line 3`, rc 158/166/166/166 | SAME x3 each | closed |
| `a4` (`::requires 'zorkolib' LIBRARY`, the control row) | B6 | `pk.cls line 3`, rc 158 | SAME x3 | unchanged |
| `t5b` (`~package` of EXTERNAL and `loadExternalMethod`) | B5 | `main.rex` for all three | SAME x3 | closed |
| `t5c` (`LIBRARY REXX` form) | B5 | `main.rex` | SAME x3 | closed |
| `t7` (PACKAGE/PRIVATE access, `setPrivate`) | B5 | `inside 1 1 1` ... `trapped 97.3`, `trapped2 97.2` | SAME x3 | no regression |
| `miss` rewritten with paths in my scratch, each side in its own directory (`p/orig-miss-{o,r}`) | B3 | `first 0` / `second 1`, rc 0, stderr empty | `first 0` / `second 1`, rc 0, stderr empty | closed |
| `env2` (reads `final-b/libs`, read only) | B4 | `after 0` / `method 0` | SAME x3 | closed |

Per original finding: **B1 closed** (and its M-D instrument gap, section 3); **B2 closed**
(88.909 and both 93.968 deliveries); **B3 closed**; **B4 closed**; **B5 closed** for every shape
slice B and the implementer probed, with R1-R3 left in the new model for routines and binding
timing; **B6 closed**; **B8 closed** for the named `.env`, with R5 one level finer; **B10 closed**
(M-F reddens two tests); **B11 partly**: fixed in the code (I2), with no witness that can fail (R10).

Also re-run for F2's interpreter half, with a copy of slice A's `final-a/ext/libforge.so` in my
scratch and the implementer's probe programs: `final-fix/f2-sig-arg-1` (`o~unknown(1)`, parameter
code) rc 163 `Error 93 running .../f.cls:` no line, `f2-sig-ret-1` (`o~retopt`) rc 163 `Error 93
running .../main.rex line 3:`, `f2-sig-argmissing-1` rc 168 `Error 88 running .../f.cls:` 88.901:
SAME x3 each. Nested boundary raises (`src/nest1`: an argument's `MAKESTRING` sends a second
library method an `.object~new`, untrapped, rc 168 naming `re.cls`; `src/nest2`: the inner argument's
`MAKESTRING` raises 40.1, trapped, `trapped 40.1 16`): SAME x3.

F3 neighbours: a nested `::requires` (`src/f3-nested`, `main.rex` -> `mid.cls` -> `pk.cls` line 3)
SAME x3. Through `loadPackage` (`src/f3-lp-lib`, `f3-lp-entry`, `f3-lp-rout`) the `running` line
now names `pk.cls line 3` as the oracle does (base named `main.rex line 3`), and stdout and rc agree;
stderr still differs, because the crate prints only `3 *-* <directive>` where the oracle adds
`*-* Compiled method "LOADPACKAGE" with scope "Package".` and `2 *-* p = ...loadPackage('pk.cls')`.
That traceback half predates the round and is not F3's: base lacks the same two lines, and so does
the `::requires 'zorkolib' LIBRARY` arm, which used `blame_directive_in` before the round
(`src/f3-lp-req`, base and `cf92ff4fb` alike). Not counted.

## 2. F4 and F5 beyond the committed witnesses

"base" below is `final-b/target-head/release/rexx-run` (slice B's build of `e64202ae7`, the
round's start), run in the same probe directory, so "predates the round" is run, not read.
Every probe source is under `rereview-int/src/<name>/`, every run under `rereview-int/p/<name>/`.

### R1. Important -- F4's record keys a routine by its spelling byte for byte; the oracle shares one routine object per library entry, found caselessly (run)

`rust/crates/rexx-exec/src/lib.rs:2076-2087` (`LibraryCodeKey`, documented as "The code object a library shares between every binding of one procedure (`LibraryPackage::resolveMethod` ...)" with "The procedure name byte for byte")
and `dispatch.rs:8419-8423` (`native_load_external` building the key from the spelling asked).
For methods the per-spelling key is right: `LibraryPackage::resolveMethod` caches a `NativeMethod`
under the spelling asked (`package/LibraryPackage.cpp:383`, `:391-392`). Routines are not built
that way. `LibraryPackage::loadRoutines` makes one `RoutineClass` per table entry at load time
(`:275-293`), and `resolveRoutine` answers that object for any spelling, caselessly
(`:420-429`, "try to locate name caseless ... return the routine"). `RoutineClass::loadExternalRoutine`
answers it as is (`classes/RoutineClass.cpp:531-532`, `resultOrNil(routine)`), and the
`::ROUTINE ... EXTERNAL "LIBRARY"` directive calls `routine->setPackageObject(package)` on it
(`parser/DirectiveParser.cpp:2684-2693`). So every spelling of a routine is one object. Ran,
`src/f4b` (`pk.cls`: `::routine sq public external "LIBRARY rxmath RXCALCSQRT"`):

```
                                        oracle     cf92ff4fb   base
early = loadExternalRoutine(RxCalcSqrt) nil        nil         REXX
  after loadPackage('pk.cls')           pk.cls     nil         REXX
pk.cls's routine                        pk.cls     pk.cls      pk.cls
loadExternalRoutine RxCalcSqrt after    pk.cls     nil         REXX
loadExternalRoutine RXCALCSQRT after    pk.cls     pk.cls      REXX
loadExternalRoutine rxcalcsqrt after    pk.cls     nil         REXX
```

rc 0 and stderr empty on all three. Silent. Not a regression from right to wrong (base answered
`REXX` on every loaded line), but it is the round's own new model answering wrong on an ordinary
case variation, and the F4 commit message states the opposite: "a different procedure spelling
... is a different code object; routines behave the same way through their own table". The
committed witness `library_method_package.rex` checks "another spelling" for a method only. The
controller's ruling fixed the key as "(library, spelling, kind)" "measured for two spellings of the
library name"; the procedure-spelling half was measured for methods and applied to routines.

### R2. Minor -- a second `::ROUTINE ... EXTERNAL` binder reports the first binder's package on the oracle (run; predates the round; the F4 commit message says otherwise)

Same mechanism as R1: `DirectiveParser.cpp:2691` discards `setPackageObject`'s answer, so a second
binder's routine directory holds the shared object whose package the first binder set. Ran,
`src/f4a3` (`pk.cls` and `pk2.cls` each `::routine sq public external "LIBRARY rxmath RxCalcSqrt"`,
both loaded with `loadPackage`):

```
                         oracle    cf92ff4fb   base
pk.cls  findRoutine('SQ') pk.cls   pk.cls      pk.cls
pk2.cls findRoutine('SQ') pk.cls   pk2.cls     pk2.cls
loadExternalRoutine       pk.cls   pk.cls      REXX
```

The `pk2` line predates the round. It is recorded because the F4 commit message
("a second package's binding of the same procedure reports its own package ... routines behave the
same way") is false for routines, and (inferred, not built) the per-row model F4 built could carry the fix (the routine
directive would read its row rather than its declaring package). `src/f4a` additionally showed
`r1 == r2` is a loud rc 120 ("the operator `==` applied to one of the interpreter's own objects is
not implemented (Phase 5)"), pre-existing and loud.

### R3. Minor -- F4 binds only once every directive of the package resolved; the oracle binds as each directive is translated (run)

`lib.rs:2967-2972` ("A package whose directives all resolved binds each procedure's shared code to
itself"). The oracle's `createNativeMethod` (`DirectiveParser.cpp:1381-1388`) sets the package while
translating that directive, so a later directive's failure does not undo it. Ran, `src/f4e`
(`pk.cls`: `::method a external "LIBRARY rxregexp RegExp_Parse"` then `::method b external "LIBRARY
rxregexp NoSuchEntry"`; `early = loadExternalMethod(RegExp_Parse)`; `loadPackage('pk.cls')` trapped):

```
                              oracle   cf92ff4fb  base
trapped                       90.998   90.998     90.998
early after failed binder     pk.cls   nil        REXX
loaded after failed binder    pk.cls   nil        REXX
```

Silent; needs a trapped package load failure. The comment states the crate's rule as if it were the
model rather than a narrowing.

### R4. Minor -- a retried `loadPackage` of a package whose library failed answers it without its classes (run; predates the round; newly reachable through F5's held version-refused library)

Observed: after a trapped library failure inside `loadPackage`, the oracle's second `loadPackage`
of the same file installs its classes, and the crate's answers without them. Mechanism, read and not
run on the crate side: the oracle resolves the library inside translation
(`DirectiveParser.cpp:1381`), before any package object is kept; where the crate keeps the package
was not traced. Ran with an oracle-built library, `src/f5-t3` via `cmpsep.sh` (each side in its own directory; `pk.cls` binds
`LIBRARY yyregexp`; the library is copied into a search directory between the two asks):

```
                     oracle           cf92ff4fb        base
first loadPackage    raised 98.903    raised 98.903    raised 98.903
second loadPackage   second loaded    second loaded    second loaded
.K                   The K class      .K               .K
.K~new('a*b')~doparse('x')  0         raised 97.1      raised 97.1
```

Control that it is the library timing and not every install failure: `src/pk-retry`
(`::class K public subclass NoSuchBase`, then the base put into `.environment`, then
`loadPackage` again) is SAME x3 (`first raised 98.909` / `second loaded` / `k class .K`): the oracle
caches that package too. Through F5, `src/f5-t2` (first ask of the forged `forgever` is a
`::method ... external` in `pk.cls`, trapped; `pk.cls` loaded again; `pk4.cls` binds later):

```
                            oracle          cf92ff4fb
pk loaded again             pk loaded again pk loaded again
.K~new~seven                k 7             step 3 syntax 97.1
loadExternalMethod ~package pk.cls          nil
after pk4 binds: k4 / m     pk4.cls pk.cls  pk4.cls pk4.cls
```

Before the round the retry could not succeed at all (the version answer was held and raised again),
so this is the pre-existing caching met by F5's new success path, and F4's retroactive bind then
names the wrong package for `m`. No committed test reaches it.

### F5 four-site transcript, forged extension, re-run (all SAME)

`rereview-int/ext/libforgever.so`, built in my scratch from a copy of `final-fix/ext/forgever.cpp`
(`g++ -shared -fPIC -std=gnu++11 -I <worktree>/api -I <worktree>/api/platform/unix`;
`requiredVersion` 0x00060000, `Forgever_Seven` method, `ForgeverRoutine` typed routine), search path
`<oracle lib>:rereview-int/ext` on both sides.

First ask, untrapped, one site per program: `loadLibrary`, `loadExternalMethod`,
`loadExternalRoutine`, `::requires ... LIBRARY` in the program, `::method ... external`,
`::routine ... external`, `::attribute ... get external`, `::requires ... LIBRARY` in a required
`pk.cls`, `::method ... external` in a required `pk.cls` (`src/f5-u-*`): all nine SAME x3, rc 158,
98.982, including the `Compiled method "LOADLIBRARY" with scope "Package"` traceback lines and
`pk.cls line 2/3` for the required package. The last five sites were not in the implementer's
measured four.

Later asks after a trapped first `loadLibrary` (`src/f5-t1`): `loadLibrary` again `1`;
`loadExternalMethod` answers a Method whose `~package` is `nil`; `loadExternalRoutine` `.nil`;
`loadPackage('pk.cls')` (`::requires 'forgever' LIBRARY` + `::method seven external`) loads;
`.K~new~seven` 7; the earlier Method now `pk.cls`; `ForgeverRoutine()` 43.1; a `::routine ...
external` in `pk2.cls` 90.999; a `::attribute seven get external` in `pk3.cls` runs, 7;
`findRoutine('FORGEVERROUTINE')` `.nil`; `loadLibrary('rxmath')` 1. All SAME. Step 14,
`.object~subclass('Z')~~define('S', m)~new~s`, is oracle `define 7` against a loud rc 120 "method
"S" of class "Z" is not implemented (Phase 5)": B7's related half, loud, out of this round's brief.

### F4 neighbourhood that agrees (run, SAME x3)

* `src/f4d`: `define` of a `loadExternalMethod` answer onto a class: `.K~method('P')` is `nil`,
  then `pk.cls` after `pk.cls` binds the procedure; a directive method `define`d onto another class
  keeps `pk.cls`.
* `src/f4f`: a `::requires` binder (before the program's first clause): `loadExternalMethod` of the
  bound procedure answers `re.cls`, under `library rxregexp` too; `loadExternalRoutine` of a routine
  `re.cls` binds, `re.cls`; a `::attribute pos get external "LIBRARY rxregexp RegExp_Pos"` accessor
  and `loadExternalMethod(RegExp_Pos)`, `re.cls` both.
* `src/f4g`: the program and a required `re.cls` both bind `RegExp_Match` and `RxCalcSqrt`:
  `loadExternal*` answers `main.rex` for both (the program is translated first).
* `src/f4a3`: a third and fourth package binding a method procedure later each report their own.

### A library that loaded normally: routine answers unchanged by the round (run)

Slice B's `r1`..`r10` (B7's table) on the oracle, `cf92ff4fb` and base: `r4` and `r9` SAME x3 as
before; `r1` `r2` `r3` `r5` `r6` `r7` `r8` `r10` still differ from the oracle as B7 recorded, and on
all ten the `cf92ff4fb` stdout, stderr and rc are byte-identical to base's (`cmp`). B7 was outside
the brief; nothing in the round moved it.

## 3. F11 harness changes

Pristine runs in my copy (`rereview-int/tree`, `CARGO_TARGET_DIR=rereview-int/target`, shell
`LD_LIBRARY_PATH` empty): `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`
exit 0, `540 of 540 matching`, `a_sidecar_changes_what_one_of_the_interpreters_answers ... ok`;
`--lib dispatch::library` 11 passed; `--test ir_recorded` 28 passed, exit 0;
`--test collect_stress -- the_l0_subset` exit 101, `thread 'rexx-interp' panicked at
crates/rexx-exec/src/dispatch.rs:1506:9: a live value` (the pre-existing red).

### Control predictions, written before the runs (corpus files edited in my copy only, restored by `git archive` + `cmp`)

* **C1**, delete `lang/external_trace.env` (the live part of a two-part sidecar): with no `RXTRACE`
  there is no pause, so the `.stdin` becomes inert. The control fails with
  `lang/external_trace.rex: neither interpreter answers differently without its Stdin`.
* **C2**, `.env` restored, add `lang/library_method_external.stdin` holding one line the program
  never reads: the control fails naming `lang/library_method_external.rex` and `Stdin`, after the
  stems before it pass.
* **C3**, restored, remove only the `LD_LIBRARY_PATH={oraclelib}` line of
  `lang/library_search_path_fixed.env`: the program asks only for `zzregexp`, which is not in the
  oracle's directory, so that variable is inert beside the live `WIDEN_*` pair. The control stays
  ok (it removes the environment as one part) and the gated differential stays 540 of 540.

### Control results (run)

* C1: `a_sidecar_changes_what_one_of_the_interpreters_answers ... FAILED`, `corpus.rs:733:13`,
  `lang/external_trace.rex: neither interpreter answers differently without its Stdin`
  (`every_sidecar_names_a_program_the_subset_runs ... ok`). Confirmed.
* C2: FAILED, `lang/library_method_external.rex: neither interpreter answers differently without
  its Stdin`. Confirmed. So the per-part control can fail, on a part made inert by deleting its
  neighbour and on an inert part added beside a live one.
* C3: control ok, `540 of 540 matching`, exit 0. Confirmed; see R5.
* Restoration: `git archive cf92ff4fb rust/corpus` extracted beside it, `diff -r` against my
  copy's `rust/corpus` empty.

### R5. Minor -- the per-part control treats a `.env` as one part, so an inert variable beside a live one escapes, and `library_search_path_fixed.env` carries one (run)

`rust/crates/rexx-exec/tests/support/sidecar.rs:75` (`Half::Environment => rest.environment.clear()`).
C3 above: removing `LD_LIBRARY_PATH={oraclelib}` from `library_search_path_fixed.env` leaves the
control and the differential green, because the program loads only the copy it makes. The file's
comment says "The oracle's own build of librxregexp.so is the library on the search path the
interpreter starts with", which is true of the setup and does nothing. This is B8's shape (text
carried between witnesses, inert) one level finer than F11's fix reaches. The control's doc says
"Asking per part is what catches an inert part beside a live one", which holds at the granularity of
fixtures / environment / cwd / stdin and not below it; nothing in the doc claims per variable, so
the defect is the inert line, not the doc.

### `external_trace.stdin` against its originating commit

`git log --follow` on `external_trace.rex` and `.stdin`: added at `73aed8f25` ("Close Phase 7"),
whose message says "`RXTRACE=ON` was unimplemented and unrefused ... It is delivered for the
top-level program, byte identical, with `corpus/lang/external_trace.rex` and an `.env` sidecar as
the witness"; the `.stdin` changed only at `cf92ff4fb`. Spawned, `RXTRACE=ON`, same directory,
`p/xtrace-with` (committed `.stdin`) and `p/xtrace-without` (`/dev/null`): oracle and build SAME x3
in both; stdout `first ?R` / `second 2` / `after ?R` plus `typed at the last pause` only with the
`.stdin`; stderr identical between the two variants except the run directory (the `+++ "LINUX
COMMAND ..."` line, the four clause echoes with `>>>` values, and the `+++ Interactive trace.`
banner once). So the witness still pins `RXTRACE=ON` starting the program under `TRACE ?R` (the
banner, echoes and `?R` answers are unchanged), and the typed command now makes the pause cadence
observable: a build that read a line at a different pause would print it elsewhere or not at all
(inferred from the transcript, not mutated).

### Mutants and instrumentation, predictions written before building

Built in a second copy, `rereview-int/mut` (`git archive cf92ff4fb rust`, same read-only symlinks),
`CARGO_TARGET_DIR=rereview-int/target-mut`, one change at a time, restored from a pristine copy and
checked with `cmp` before the next; binary sha256 prefixes recorded.

* **M-D** (`dispatch/library.rs` `string_value` answers `Ok(Some(object))`, no conversion): the gated
  corpus reports exactly `lang/library_string_argument.rex` and `lang/library_method_no_string_value.rex`
  differing, each on stdout, stderr and exit (`538 of 540`); `dispatch::library` 11 pass. Slice B's
  prediction on the old tree was "every phase-8 witness stays SAME"; these two are the F1 and F2
  witnesses, the only phase-8 programs passing a non-string to a string parameter.
* **M-F** (`resolve_library`'s `self.libraries.get(name)` early return disabled with
  `.filter(|_| false)`): `a_library_named_twice_is_opened_once` fails on `the same name was opened
  twice` (left 2, right 1) and `a_version_refused_library_raises_once_and_is_held` panics "the
  refused library was not held"; the other 9 `dispatch::library` tests pass; the gated corpus stays
  `540 of 540` (a second open of a loaded library is dropped by the non-replacing hold).
* **M-F6** (`resolve_library` searches `library_search_of(&self.env)`, the live value), with R5's
  line also removed from `library_search_path_fixed.env` in the mutant copy:
  `a_search_directory_written_after_the_start_is_not_searched` fails; the gated corpus is `539 of
  540` with `lang/library_search_path_fixed.rex` differing on stdout. That is, the inert variable is
  not what lets the witness see M-F6.
* **I1** (`collect_stress` L0 loop prints each `rel_path` before running it; no other change):
  the pre-existing panic happens on a program listed before `phase-8.txt`'s, so the committed L0
  test never reaches a phase-8 program and F11's sidecar change there has no gate witness.
* **I2** (`ir_recorded::compare` prints exit code and the first stderr line for `lang/library_*`
  cases; no other change): every library witness exits with the status the corpus records for it,
  none with 158 and `Unable to load library "rxregexp"`; `every_population_runs_without_a_refusal`
  stays ok. And, with the sidecar plumbing removed from `compare` (I2b), the same test stays ok while
  the printed statuses fall to the load failure: `ir_recorded`'s assertion cannot see whether a
  program received its sidecar.

### Mutant results (run)

* **M-D** (test binaries `corpus` `892f80fcdf3d9ab5`, `rexx_exec` lib `b90090a7509ce3d3`): gated
  corpus exit 101, `538 of 540 matching`, `[UNCLASSIFIED] lang/library_string_argument.rex: stdout,
  stderr, exit code differ` and `[UNCLASSIFIED] lang/library_method_no_string_value.rex: stdout,
  stderr, exit code differ`, nothing else; `dispatch::library` 11 passed. Confirmed exactly. B's
  M-D gap (no witness sees the host's conversion) is closed, by two committed witnesses.
* **M-F** (`corpus` `2a204e83f80251f7`, lib `fe72a061bb7c9397`): `dispatch::library` exit 101, 9
  passed, 2 failed: `a_library_named_twice_is_opened_once` at `library.rs:309:9`, `the same name was
  opened twice`, left 2, right 1; `a_version_refused_library_raises_once_and_is_held` at
  `library.rs:399:13`, `the refused library was not held`. Gated corpus `540 of 540`, exit 0.
  Confirmed exactly. B10 closed.
* **M-F6** with R5's line removed (`corpus` `0cb83701091f661f`, lib `d45d8baa0256bfb0`):
  `a_search_directory_written_after_the_start_is_not_searched` failed at `library.rs:356:9`
  (`matches!(late.resolve_library(b"rxregexp"), LibraryLoad::Missing)`), 10 passed; gated corpus
  `539 of 540`, `lang/library_search_path_fixed.rex: stdout differ`, the per-part control ok.
  Confirmed exactly: the witness sees M-F6 without the `LD_LIBRARY_PATH` line, so the line is inert
  for the mutant as well as for the fix.
* **I2** (`--test ir_recorded -- every_population_runs_without_a_refusal --nocapture`): ok, and
  every `lang/library_*` case printed its status: the ordinary witnesses exit 0
  (`library_method_external`, `library_loads_once`, the three `uninit`, `library_method_package`,
  `library_load_retried`, `library_search_path_fixed`), the error witnesses exit with their own
  numbers (88.901, 88.922, 38 at rc 218, 40.1 at rc 216, 88.909, 90.998, 90.999), and only the two
  `zorkolib` programs print 98.903. Confirmed: under the sidecar the sweep reaches the libraries.
* **I1** (`--test collect_stress -- the_l0_subset --nocapture`): 505 `STRESS` lines, the last
  `lang/condition_object_syntax.rex`, then `thread 'rexx-interp' panicked at
  crates/rexx-exec/src/dispatch.rs:1506:9`; that program is in `phase-7.txt` (`grep -l`), and no
  `phase-8.txt` program is printed. Confirmed.
* **I2b** (I2's print kept, `compare`'s sidecar replaced by `Sidecar::default()`): the test is still
  ok, while the library cases print 98.903 `Unable to load library "rxregexp"` (every `.env`-only witness, `grep -c` 11),
  43.901 `Could not find file "re.cls"` / `"pk.cls" for ::REQUIRES` for the fixture ones, 97.1 for
  `library_method_package`, and `cp` usage errors for the two copying programs. Confirmed; see R10.
* Restoration after each: `cp` from `rereview-int/pristine-src` (a third `git archive cf92ff4fb
  rust`) and `cmp`; after M-F6, `diff -r pristine-src/rust mut/rust` empty.

### R10. Minor -- F11's sidecar delivery in `collect_stress` and `ir_recorded` has no witness that can fail (run)

`rust/crates/rexx-exec/tests/collect_stress.rs` (the L0 loop) and `tests/ir_recorded.rs` `compare`.
I1: the committed L0 test stops at `lang/condition_object_syntax.rex` in `phase-7.txt`, so no
phase-8 program runs under the gate there; the implementer's report and the progress log say this,
and `dispatch/library.rs`'s test doc says it too ("aborts on a pre-existing panic before it gets
there"). I2b adds the half nobody recorded: `ir_recorded`'s only assertions are "finished" and "no
body refused", both of which a 98.903 before the first clause satisfies, so removing the sidecar
from `compare` leaves `every_population_runs_without_a_refusal` green. So B11's defect (the harness
reaches only the load failure) is fixed in the code, as I2 shows, and would come back silently: the
per-part sidecar control lives in `corpus.rs` and covers that harness alone. Nothing claims
otherwise in the tree; the F11 commit message's "both harnesses now run each corpus program with
its fixtures, environment and input" is true today.

## 4. Comments, doc comments, commit messages

Read: every `+` comment and doc-comment line of the eight commits in `rust/crates/rexx-exec`,
`rust/corpus` headers and `.env` comments, `rexx-api/src/values.rs`'s F1 lines, and the eight commit
messages. Every C++ citation the eight commits add, found by `git show <c> -- rust/crates/rexx-exec
rust/corpus rust/crates/rexx-api/src/values.rs | grep '^+' | grep -o '[A-Za-z/]*\.[ch]pp:[0-9-]*'`
and the same over the messages, printed from the worktree's `interpreter/` (byte-identical to the
oracle's tree for every file involved, `cmp`): `PackageManager.cpp:229-248` (`loadLibrary`, lands),
`:238-244` (put / load / remove, lands), `:240-244` (lands), `LibraryPackage.cpp:374-400`
(`resolveMethod`, lands; see R1 for what it does not cover), `NativeCode.cpp:130-140`
(`setPackageObject`, lands on the function and its branch, the copy itself is `:141-143`),
`NativeActivation.cpp:190-193` (`reportSignatureError`, lands), `:1301-1310` (`trapErrors = true`
through `valueToObject`, lands). `values.rs`'s `ObjectClass.cpp:1341` lands on
`RexxInternalObject::requiredString()`.

### R6. Minor -- two counts in commit messages and the report are wrong (run)

* F4 `ca79611e5`: "Before, 12 of its 13 lines differ" (report: "stdout DIFF on 12 of 13 lines
  (only `pk.cls routine` agrees)"). The implementer's own saved run `final-fix/f4-before-witness/`
  has 11 of 13 differing: `pk.cls routine pk.cls` and `LIBRARY REXX REXX` agree. Same count on
  their `bins/rexx-run-f2` re-run here (`p/w-pkg-f2`) and on base (`p/w-pkg-base`), via `paste` of
  `o.out`/`r.out` and a line compare.
* F1 `04a286913`: "Before this change the same program answered rc 168 with five silent
  conversions." Base on `library_string_argument.rex` (`p/w-strarg-base`): six steps where the oracle
  raises 88.909 print a value (`parse object 0`, `parse string only 0`, `parse nil 0`, `match object
  0`, `match string only 0`, `match nil 1`), besides `match makestring 0` against `1` and the two
  40.1 steps answered 88.909.

### R7. Minor -- `Interp::executable_package`'s doc became false in F4 (read, and run on the crate side)

`rust/crates/rexx-exec/src/environment.rs:2076-2077`: "or `None` for an object this crate did not
build". Since `ca79611e5` it also answers `None` for a `loadExternal*` object this crate built whose
row is unbound. Its callers then report the value as none of the accepted classes: crate only,
`p/ctx-crate-only2`, `.Package~new('x', 'say 1', .Method~loadExternalMethod('m', 'LIBRARY rxregexp
RegExp_Pos'))` is rc 163 `93.953: Method argument 3 could not be converted to type Method, Routine,
or Package object` for a Method. The oracle was not run on this shape: `BaseExecutable::
processNewExecutableArgs` takes `getPackage()` (`BaseExecutable.cpp:276`), which is
`resultOrNil(package)` (`:123-124`), so it hands `.nil` on as a `PackageClass *`; reading, that is
not a defined answer to match.

### R8. Minor -- `library_opens` counts asks, not opens (read)

`rust/crates/rexx-exec/src/lib.rs` (the `#[cfg(test)] library_opens` field): "How many times
[`Interp::resolve_library`] has opened a library". It is incremented before `load::open` for every
unheld name, so `zorkolib`, which opens nothing, counts; the F10 test's control depends on exactly
that ("a name nothing is held for opens every time it is asked"). The number is right for the
test's purpose; the word is not.

### R9. Minor, predates the round -- a user `REQUEST` method is never sent by any string conversion (run)

`src/f1-more2` (argument kinds through `re.cls`'s `does`/`doparse`): every kind agrees (`MAKESTRING`
answering a number, `.nil`, an array; `.MutableBuffer`; a stem with and without a default; a class
object; `12`; `1e30`; a Method; a Directory; a `MAKESTRING` raising 93.900) except a class that
defines `REQUEST`: oracle prints `request STRING` and matches (`1`), the crate raises 88.909.
`src/req-builtin` shows the same on built-in conversions (`'xaaby'~pos(.Req~new)`: oracle `request
STRING` / `pos 2`, crate rc 168 88.909), so it is the shared `string_conversion`, which F1 was told
to reuse, not F1. Base answered `request override 0` there. Not in the KNOWN GAPS block (`grep -i
request` finds only `requestArray` entries). `f1-more` step 1 (`.MyStr~new`, a String subclass) is a
loud rc 120 Phase 5 refusal, pre-existing.

### Checked and true

* F2 message, "'abc'~hasMethod(.nil) is "Error 88 running main.rex line 1"": `p/hasm`, SAME x3,
  rc 168, `line 1`.
* F11 message, the old `.stdin` inert: `git show cf92ff4fb~1:rust/corpus/lang/external_trace.stdin`
  against `/dev/null`, oracle, `RXTRACE=ON`: stdout and stderr byte-identical, rc 0 both.
* F6 message, `RUNPATH`: `readelf -d` on the oracle's `bin/rexx` and `lib/librexx.so.4`, both
  `Library runpath: [/home/moritz/dev/repos/ooRexx/build/lib:]`.
* F11 message, "coverage.rs only parses": its entry point is `parse_program` (lines 24, 104, 132,
  141, 166 by `grep -n run_program\|parse_program`, no `run_program`).
* F5 message and comments: first ask 98.982 at the named sites and more, later asks loaded,
  methods bind and run, routines never register (section 2); `a_version_refused_library_raises_once_and_is_held`'s
  doc ("`loadLibrary` is 98.982 and then `1`, and a method of the library binds and runs") matches
  `f5-t1`.
* F3 message and doc, F10 message ("three dlopens against two under gdb, eight of eight tests
  green" is B10's measurement), `library_string_argument.rex`'s header (`RegExp_Parse` takes
  `CSTRING`, `RegExp_Match` `RexxStringObject`: `extensions/rxregexp/rxregexp.cpp:108`, `:141`),
  the two F5 and F6 witness headers.
* The F4 witness header ("shares the library's code object for that procedure spelling") and
  `LibraryCodeKey`'s doc are false for routines; that is R1, not counted twice.

## 5. F6 environment paths

* A program that writes `LD_LIBRARY_PATH` through `VALUE(..., 'ENVIRONMENT')` and then loads:
  `env2` (section 1, `loadLibrary` and `loadExternalMethod`) SAME x3; `src/env3` (the write, the
  value read back, then `loadPackage` of a `::requires 'zzregexp' LIBRARY` package and of a
  `::method ... external "LIBRARY zzregexp ..."` package, then `loadExternalRoutine`) SAME x3:
  `reads back 1`, `pk raised 98.903`, `pk2 raised 98.903`, `routine The NIL object`.
* An embedder handing the variable in through `Invocation::with_environment`: the in-process
  corpus harness is that embedder (`tests/corpus.rs` `run_rust` -> `sidecar::invocation` ->
  `with_environment`, which `execute` hands to `Interp::adopt_environment`, `lib.rs:5690`). With the
  shell's `LD_LIBRARY_PATH` empty, the pristine gated corpus is `540 of 540`, and the per-part
  control passed, which for each `LD_LIBRARY_PATH`-only `.env` means the crate's in-process answer
  moved when the environment part was removed (the oracle's cannot, per the control's own doc:
  `Oracle::wrapped` adds the directory; read, not traced). So the handed-in value reaches the search. The unit-test side:
  `interp_that_can_see_rxregexp` goes through `adopt_environment`, and M-F6 above reddens the test
  that separates a start value from a later write.
* M-F6 re-run above: confirmed.

## Not reached

Silence below is not coverage.

* **Debug build and the full gates**: every run here is `--release` (binary and tests), so no
  `debug_assert` was exercised; no G1-G5 gate command, no clippy, no fmt, no `rexx-api` tests (F2's
  and F5's `rexx-api` halves are the boundary slice's).
* **`.Method~new` / `.Routine~new` / `.Package~new` / `newFile` with an unbound `loadExternal*`
  object as context** on the oracle: not run, because reading `BaseExecutable.cpp:276` and `:123-124`
  it passes `.nil` on as a `PackageClass *`; R7 records the crate side only.
* **F1's `push_temp` of a converted string** was not put under collect-on-every-allocation beyond
  what the implementer's narrowed run and the two in-crate stress tests cover; `library_string_argument`
  was not run under the stress mode here.
* **F5 with a loader or unloader**: the forged package has neither; the crate runs no loader or
  unloader for any library (the implementer's read, not re-read here).
* **F5 through `::REQUIRES ... LIBRARY` in a package required twice, and a refused library whose
  methods are `define`d**: the second is B7's loud refusal (f5-t1 step 14); the first not probed.
* **F4 with a `::attribute ... external` having no `get`/`set`** (the `GET`/`SET` prefixed
  procedure spellings) as a binder, and **`setPrivate`/`setProtected` on a `Loaded` object**: not
  probed.
* **Two library-name spellings reaching one file**: not constructed. The implementer's `loadLibrary`
  of `RXREGEXP`, `librxregexp`, `rxregexp.so` all answer 0 and B's `sp` probe covered spaces, so no
  second spelling that loads was found; a path-bearing name (`LIBRARY /abs/dir/rxregexp`) was not
  tried on either side.
* **R4's crate-side mechanism** (where the crate keeps a package whose install failed) was not
  traced.
* **R1's fix shape**: whether routine directives should read a shared row, and the second-binder
  answer it implies (R2), is inferred from the C++ and not built.
* **`tests/corpus.rs`'s other new code** beyond the control (the moved sidecar reader's callers,
  `every_sidecar_names_a_program_the_subset_runs`): read, not mutated.
* **The implementer's F7-F9 and the sourceline companions** of the new witnesses: out of this slice,
  not compared.
* **Concurrency** (`~start`, `GUARD`, a second activity) on any of the round's paths: not probed.
* **Scratch state**: `rereview-int/tree` (pristine `cf92ff4fb`, corpus restored and `diff -r` clean),
  `rereview-int/mut` (restored, `diff -r` against `pristine-src` clean), `ext/libforgever.so` and
  `ext-forge/libforge.so` (forged, scratch only). Nothing in the worktree was written but this file;
  `git status --short` in the worktree was empty at the end (this file sits under the ignored
  `.superpowers/`, `git check-ignore -v`) and HEAD still `cf92ff4fb`.
