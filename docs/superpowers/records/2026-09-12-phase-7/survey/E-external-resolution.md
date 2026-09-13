# Survey E — external routine resolution and package loading from files

Surveyor E, Phase 7. Area: resolving a function-call or CALL name to an external Rexx program;
`::REQUIRES` file resolution; the file-backed package-loading APIs.

Probe root `E` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/survey-E`.
Four batches, scripts `E/b1.sh` … `E/b4.sh` (shared runner `E/lib.sh`), probe trees `E/p01` … `E/p04`,
every descriptor kept as `E/pNN/out/<case>.{o,c}.{out,err,rc}` (`o` oracle, `c` crate). Every probe
ran with cwd = `E/pNN/cwd`, the program in `E/pNN/main`, and PATH / REXX_PATH set only on the
command line (`env PATH=… REXX_PATH=…`); `E/pNN/rp` is the REXX_PATH directory and `E/pNN/pp` the
PATH one. Oracle: 5.3 trunk build at `/home/moritz/dev/repos/ooRexx/build`. Crate: snapshot binary
`h-5bcb28edb`. Below, `<E>` abbreviates the probe root inside quoted bytes; the files hold the full
path.

Claim marks: **Measured** (probe run, three descriptors), **Read** (file:line), **Inferred**.

## 1. Documented surface

Enumerated from `oodocs/rexxref/en-US/funct.xml` §"Search Order" and §"Locating External Rexx
Files", `instrc.xml` CALL and PARSE SOURCE, `dire.xml` ::REQUIRES and ::ROUTINE, and the
`fundclasses.xml` Package / Routine / Method method lists.

| # | Item | Citation | Contract (one line) |
|---|------|----------|---------------------|
| 1 | Function/subroutine search order | `funct.xml:205-214` | internal routines, then built-in functions, then external functions |
| 2 | Symbol vs literal target | `funct.xml:215-228`, `instrc.xml:688-693` | a symbol target is uppercased; a literal string is used as-is and bypasses the internal-label search |
| 3 | Case sensitivity by step | `funct.xml:229-247` | labels and built-ins are uppercase; "some steps of the external search may be case sensitive" |
| 4 | External function search order | `funct.xml:277-296` | own `::ROUTINE`s; public `::ROUTINE`s of `::REQUIRES` packages; macrospace pre-order; function/library packages; external file; macrospace post-order |
| 5 | Directories searched, in order | `funct.xml:297-320` | the calling program's directory (skipped for an initial program or a macrospace one); the current directory; an application extension path; REXX_PATH; PATH |
| 6 | Extension qualification | `funct.xml:321-330` | a name with at least one period is extension-qualified: searched unchanged, nothing added |
| 7 | Extension order | `funct.xml:331-352` | all directories per extension before the next extension; `.cls` first for ::REQUIRES / `Package~new(name)` / `loadPackage(name)`; then the caller's own extension; application extensions; `.REX` and `.rex` on Unix; then no extension |
| 8 | Case retry on Unix | `funct.xml:353-360` | each probe tried as given and lowercased, "not performed on the very last step" (no extension) — **contradicted by measurement, §2.1 t04** |
| 9 | REXX_PATH | `funct.xml:317` | directories searched after the current directory and before PATH |
| 10 | Errors during execution | `funct.xml:362-370` | an error in an external function becomes a syntax error in the caller, trappable with SIGNAL ON SYNTAX |
| 11 | Return values / RESULT | `funct.xml:371-395` | subroutine: value stored in RESULT, else RESULT dropped; function: must return a value |
| 12 | CALL: external routine | `instrc.xml:678-684`, `758-800` | neither built-in nor a label in the program; arguments via ARG / PARSE ARG / USE ARG / arg(); implicit PROCEDURE (caller's variables hidden); NUMERIC starts at defaults; EXIT may return from it |
| 13 | CALL `(expr)` target | `instrc.xml:694-700` | evaluated before the arguments, not uppercased |
| 14 | PARSE SOURCE | `instrc.xml:2305-2330` | `<os> <COMMAND|FUNCTION|SUBROUTINE|METHOD|REQUIRES> <path or package name or INSTORE>` |
| 15 | ::REQUIRES programname | `dire.xml:1040-1058` | literal or constant symbol, "any string valid as the target of a CALL"; "called as an external routine with no arguments"; searched with the file search; code before the first directive runs at ::REQUIRES time |
| 16 | ::REQUIRES LIBRARY | `dire.xml:1059-1068` | native library — **Phase 8** (brief) |
| 17 | ::REQUIRES NAMESPACE | `dire.xml:1069-1080` | qualifier for the loaded package's classes/routines |
| 18 | ::REQUIRES order and caching | `dire.xml:1105-1116` | directive order = search order and load order; a program loaded once is reused by later ::REQUIRES; its prolog runs on the first reference only |
| 19 | Local definitions win | `dire.xml:1119-1122` | own routines / classes beat same-named ones reached through ::REQUIRES |
| 20 | ::ROUTINE position | `dire.xml:1255-1259` | behaves like an external routine; after internal and built-in, before all other external routines |
| 21 | `Package~new(name[,source[,context]])` | `fundclasses.xml:3618-3665` | no source: `name` is a file, located with the file search; with source: in-memory package, optional Method/Routine/Package context for lookup scope |
| 22 | `Package~loadPackage(name[,source])` | `fundclasses.xml:4209-4235` | name only: located and loaded "as if named on a ::REQUIRES"; source array: in-memory; a previously loaded package is reused; the result is added to the receiver's dependent packages |
| 23 | `Package~findProgram(name)` | `fundclasses.xml:3985-4000` | resolves `name` in the receiving package's context, returns the full path or `.nil` |
| 24 | `Routine~newFile(filename[,context])` | `fundclasses.xml:4760-4790` | the file's text as a Routine; context Method/Routine/Package/"PROGRAMSCOPE" (default: the caller's scope) |
| 25 | `Method~newFile(filename[,context])` | `fundclasses.xml:2066-2100` | same for Method; "raises an error if the file cannot be read" |
| 26 | `Package~defaultOptions` (class) | `fundclasses.xml:3667-3690` | default settings of loaded / required packages — D12's area, not covered here |
| 27 | `.context~package` | used throughout `fundclasses.xml` examples (e.g. `:4005`) | the running package, the receiver for `loadPackage` / `findProgram` |

Not covered here: item 16 (Phase 8), item 26 (D12), macrospace (items 4/5 mention it; needs the RXAPI,
Phase 10), registered/library functions in item 4 (the `Sys*` surveyor).

## 2. Oracle behaviour

All **Measured** unless marked. Crate answers are quoted where they differ; for every call to an
external file the crate answers 43.1 with stdout empty, byte-identical in text to the oracle's own
43.1 (compare t05, t09, t11, t13 below, where both sides raise it) — the only difference across
batch 1 is whether a file is found.

### 2.1 Batch 1 — search order, extensions, quoting, case, paths (`E/p01`)

Tree: `zork.rex` in all of main/, cwd/, rp/, pp/ (each returns its directory's name); `zcwd.rex`
only in cwd/; `zrp.rex` only in rp/; `zpp.rex` only in pp/; `zboth.rex` in cwd/ and rp/; `zrp2.rex`
in rp/ and pp/. Env `PATH=<E>/p01/pp:/usr/bin:/bin REXX_PATH=<E>/p01/rp`.

| case | program (in main/) | oracle rc | oracle stdout | note |
|---|---|---|---|---|
| t01 | `call zork; say 'r=' result` | 0 | `zork from main`⏎`r= main` | the caller's directory beats cwd, REXX_PATH, PATH |
| t02 | `call zcwd…zrp…zpp…zboth…zrp2` | 0 | `zcwd ran 0`⏎`CWD`⏎`RP`⏎`PP`⏎`BOTH-cwd`⏎`RP2-rp` | cwd > REXX_PATH > PATH |
| t02n | t02 with REXX_PATH unset | 213 | `zcwd ran 0`⏎`CWD` | stderr `Error 43.1:  Could not find routine "ZRP".` — REXX_PATH is really read |
| t16 | `call zpp` with `PATH=<E>/p01/pp/::/usr/bin:/bin REXX_PATH=` | 0 | `PP` | a trailing `/` and empty entries are harmless |
| t03 | `call e2; call E4; call e5` with files `e2.REX`+`e2`, `E4`, `e5.REX`+`e5.rex` | 0 | `e2.REX`⏎`E4 noext upper`⏎`e5.rex` | `.REX` beats no extension; an uppercase no-extension file is found; the caller's own `.rex` beats the default `.REX` |
| t04 | `call e3` with only `e3` (lowercase, no extension) | 0 | `e3 noext lower` | **lowercase retry happens on the no-extension step too** — the doc line `funct.xml:355-357` says it does not; the code (§3.2) does |
| t05 | `call Mixed` with `Mixed.rex` | 213 | (empty) | `Could not find routine "MIXED".` — MIXED.rex / mixed.rex tried, never Mixed.rex |
| t06 | `call 'Mixed'` | 0 (then 40.3 on my `call LOWER`, a built-in — probe error) | `Mixed.rex` | quoted name keeps its case |
| t07 | `call r1` with `r1.cls`+`r1.rex` | 0 | `r1.rex` | `.cls` is never tried for a call |
| t08 | `main2.rxx`: `call c1` with `c1.rxx`+`c1.rex` | 0 | `c1.rxx` | the caller's extension is tried first |
| t09 | `call 'a.b'` with `a.b.rex` | 213 | (empty) | `Could not find routine "a.b".` — a period makes the name extension-qualified |
| t10 | `call 'sub/x.rex'`, `call './sub/x.rex'`, absolute; `sub/x.rex` in main/ and cwd/ | 0 | `main/sub/x.rex`⏎`cwd/sub/x.rex`⏎`main/sub/x.rex` | a relative path without `./` is joined to each search directory; `./` binds to cwd |
| t11 | `call zdir` with a **directory** `main/zdir.rex` and a file `cwd/zdir.rex` | 213 | (empty) | `Could not find routine "ZDIR".` — the directory match aborts that scan; the later file is never reached (§3.2) |
| t12 | `call 'zork.rex'`; `say 'zmain.rex'(1)` | 0 | `zork from main`⏎`main`⏎`zmain arg 1`⏎`ZM` | quoted name with extension, both call forms |
| t13 | `call 'zork.cls'` | 213 | (empty) | `Could not find routine "zork.cls".` |
| t14 | `say zcwd(); say 'ZCWD'(); say 'zcwd'(1,2)` | 0 | `zcwd ran 0`⏎`CWD`⏎`zcwd ran 0`⏎`CWD`⏎`zcwd ran 2`⏎`CWD` | function form; arg() counts |
| t15 | label `zcwd:` in the caller; `call zcwd`, `call 'zcwd'`, `length('abc')`, `'length'('abc')` with `length.rex` present | 0 | `internal label`⏎`zcwd ran 0`⏎`CWD`⏎`3`⏎`length.rex file` | label beats file; quoting bypasses the label; built-in beats file; a lowercase quoted name is not a built-in and reaches the file |

Crate on t15: prints `internal label`, then 43.1 on `call 'zcwd'` (rc 213).

### 2.2 Batch 2 — what the callee sees and answers (`E/p02`, env `PATH=/usr/bin:/bin`)

t01 — caller sets `.local~probe='yes'`, `caller_var=1`, `numeric digits 5; numeric fuzz 1; numeric
form engineering`, `address bash`, `trace o`, then `call callee 'A1','A2'`. Oracle rc 0, stdout:

```
src= LINUX SUBROUTINE <E>/p02/main/callee.rex
arg()= 2 a1= A1 a2= A2
digits= 9 fuzz= 0 form= SCIENTIFIC
address= BASH
trace= N
local= yes
symbols LIT LIT LIT LIT
r= ret
caller digits 5 address BASH trace O
```

So: PARSE SOURCE carries the resolved absolute path; arguments arrive; NUMERIC is at defaults;
**ADDRESS is inherited from the caller**; TRACE is not; `.local` is the same directory; `rc`,
`result`, `sigl` and the caller's variables are all unset (`symbol()` = `LIT`); the caller's settings
are untouched afterwards.

| case | shape | oracle | bytes |
|---|---|---|---|
| t02 | `call ex_ret` (`return`), `ex_retv` (`return 7`), `ex_exit` (`exit`), `ex_exitv` (`exit 8`); then `say ex_retv() ex_exitv()` | rc 0 | `LIT`⏎`7`⏎`LIT`⏎`8`⏎`7 8` — RETURN/EXIT without a value drop RESULT; with a value set it; EXIT with a value answers a function call |
| t03 | `say ex_ret()` | rc 212 | stderr `     1 *-* say ex_ret()`⏎`Error 44 running <E>/p02/main/t03.rex line 1:  Function or message did not return data.`⏎`Error 44.1:  No data returned from function "EX_RET".` |
| t04 | `say ex_exit()` | rc 212 | same, `"EX_EXIT"` |
| t05 (oracle only) | write `mod.rex` = `say 'v1'`, `call mod`, rewrite to `say 'v2'`, `call mod` | rc 0 | `v1`⏎`v2` — **a called file is not cached**; it is re-read on every call |
| t06 | `dir.rex`: prolog `say 'dir prolog'`, `say 'helper says' helper()`, `return 'DIRRET'`, `::routine helper` (private), `::routine foo public`, `::class pub public` + class method `hello`. Caller: `call dir; say result; say foo(); say .pub~hello; call dir; say result; say helper()` | rc 213 | stdout `dir prolog`⏎`helper says h`⏎`DIRRET`⏎`FOO`⏎`hello from pub`⏎`dir prolog`⏎`helper says h`⏎`DIRRET`; stderr 43.1 `"HELPER"` — **after the call the callee's public routines and classes are merged into the caller's package**, private ones are not; the body runs again on the second call |
| t07 | `perr.rex` = `say 'before'` / `say 'a' +`; caller `say 'pre'; call perr; say 'post'` | rc 221 | stdout `pre`; stderr `     2 *-* say 'a' +`⏎`     1 *-* call perr;`⏎`Error 35 running <E>/p02/main/perr.rex line 2:  Invalid expression.`⏎`Error 35.1:  Incorrect expression detected at "+".` — the callee is parsed whole before running (`before` never prints); the traceback lists the callee line above the caller's call line; the error names the callee file |
| t08 | t07 with `signal on syntax` in the caller | rc 0 | `pre`⏎`trapped 35  1`⏎`Incorrect expression detected at "+".` — trappable in the caller: rc 35, `condition('D')` empty, sigl 1 |
| t09 | `rerr.rex` = `say 'in rerr'; say 1/0`; `call rerr` | rc 214 | stdout `in rerr`; stderr `     1 *-* say 1/0`⏎`     1 *-* call rerr`⏎`Error 42 running <E>/p02/main/rerr.rex line 1:  Arithmetic overflow/underflow.`⏎`Error 42.3:  Arithmetic overflow; divisor must not be zero.` |
| t10 | t09 trapped | rc 159 (my `~traceback~toString` was a probe error: it is a List) | stdout `in rerr`⏎`trapped 42  1`⏎`Arithmetic overflow; divisor must not be zero.`⏎`<E>/p02/main/rerr.rex` — `condition('O')~program` names the callee |
| t11 | `ruser.rex` = `raise user oops description 'from callee'`; caller `call ruser; say 'after'` | rc 0 | `after` — an untrapped USER condition ends the callee and the caller continues |
| t11b | caller `signal on user oops` | rc 0 | `caught USER OOPS from callee` |
| t12 | `unread.rex` mode 000; `call unread` | rc 253 | stderr `     1 *-* call unread;`⏎`Error 3 running <E>/p02/main/t12.rex line 1:  Failure during initialization.`⏎`Error 3.1:  Failure during initialization: File "<E>/p02/main/unread.rex" is unreadable.` — resolved but unreadable is 3.1 with the absolute path, not 43.1 |
| t14 (oracle only) | `sh3.txt` = L1 L2 L3; caller `linein`, callee `linein`, caller `linein` | rc 0 | `caller linein L1`⏎`callee linein L2`⏎`caller linein L3` — **the stream table is shared** with the callee |
| t15 | caller `signal on novalue`; callee `say 'callee sees' undefinedvar` | rc 0 | `callee sees UNDEFINEDVAR`⏎`after` — condition traps are not inherited |
| t16 | `reqcallee.rex` has `::requires 'rq.cls'` (prolog `say 'rq prolog'`, `::routine rqf public`); caller `call reqcallee; say result; say rqf()` | rc 0 | `rq prolog`⏎`reqcallee body RQF`⏎`RC-ret`⏎`RQF` — the merge is transitive: what the callee required is callable from the caller afterwards |
| t17 | `say pscall(); call pscall; interpret "call pscall"` | rc 0 | `LINUX FUNCTION <E>/p02/main/pscall.rex`⏎`1`⏎`LINUX SUBROUTINE …`⏎`LINUX SUBROUTINE …` — FUNCTION vs SUBROUTINE; INTERPRET resolves through its parent |
| t18 | `call 'nest/outer.rex'`; `outer.rex` does `call inner` with `inner.rex` only in `main/nest/` | rc 0 | `outer:inner-in-nest` — "the calling program's directory" is the callee's own at every depth |

Crate on t08/t10: `trapped 43  1` then rc 120 `rexx-exec: CONDITION option "O" answers a
Directory, which is not implemented` — a separate gap outside this area.

### 2.3 Batch 3 — `::REQUIRES` and the package APIs (`E/p03`, env `PATH=/usr/bin:/bin REXX_PATH=<E>/p03/rp`)

Crate matches byte for byte on t01–t06, t09–t12, t14–t17, t18 (both descriptors and rc). Differences are
called out.

| case | shape | oracle | bytes |
|---|---|---|---|
| t01 | `::requires 'req1'` with `req1.cls`+`req1.rex`; `.cls` prolog prints PARSE SOURCE | rc 0 | `req1.cls prolog LINUX REQUIRES <E>/p03/main/req1.cls`⏎`CLS` |
| t02 | `::requires 'req1.rex'` | rc 0 | `req1.rex prolog`⏎`REX` |
| t03 | `::requires 'nope'` | rc 213 | stderr `     1 *-* ::requires 'nope'`⏎`Error 43 running <E>/p03/main/t03.rex line 1:  Routine not found.`⏎`Error 43.901:  Could not find file "nope" for ::REQUIRES.`; stdout empty |
| t04 | `c_a.cls` requires `c_b.cls` requires `c_a.cls` | rc 158 | stderr `     2 *-* ::requires 'c_a.cls'`⏎`     2 *-* ::requires 'c_b.cls'`⏎`     1 *-* ::requires 'c_a.cls'`⏎`Error 98 running <E>/p03/main/c_b.cls line 2:  Execution error.`⏎`Error 98.952:  Circular ::REQUIRES references detected with <E>/p03/main/c_a.cls.` |
| t05 | the same `::requires 'once.cls'` twice in one program | rc 0 | `once prolog`⏎`main`⏎`ONCE` — once |
| t06 | `u1.cls` and `u2.cls` both require `once.cls` | rc 0 | `once prolog`⏎`u1 prolog`⏎`u2 prolog`⏎`main`⏎`ONCE` |
| t07 | `.context~package~loadPackage('pk.rex')`, `~name`, `findRoutine('rfun')~call`, `findProgram('pk.rex')`, then `.Package~new('pk.rex')`, `==`, `rfun()` | rc 0 | `pk prolog`⏎`name= <E>/p03/main/pk.rex`⏎`R`⏎`findProgram= <E>/p03/main/pk.rex`⏎`same= 1 <E>/p03/main/pk.rex`⏎`rfun visible? R` — loadPackage resolves beside the program; `Package~new` afterwards hits the requires cache under the short name; loadPackage merges into the caller. **Crate**: identical through `findProgram=`, then rc 120 `the operator `==` applied to one of the interpreter's own objects is not implemented (Phase 5)` |
| t08 | `.Package~new('pk.rex')` first, cwd ≠ main/ | rc 213 | stderr `       *-* Compiled method "NEW" with scope "Package".`⏎`     1 *-* p = .Package~new('pk.rex');`⏎`Error 43 running <E>/p03/main/t08.rex line 1:  Routine not found.`⏎`Error 43.901:  Could not find file "pk.rex" for ::REQUIRES.` — **`Package~new(name)` does not search the caller's directory** (§3.4). **Crate diverges**: rc 0, `pk prolog`⏎`name= <E>/p03/main/pk.rex` |
| t09 | `.Package~new('nope.rex')` | rc 213 | same shape, `"nope.rex"` — crate identical including the `Compiled method "NEW"` line |
| t10 | `.Package~new('inmem', .array~of(...))` with `::routine af public`; then `say af()` | rc 213 | stdout `arr prolog`⏎`name= inmem`⏎`AF`; stderr 43.1 `"AF"` — an in-memory package is not merged into the caller |
| t11 | `.Routine~newFile(abs)~name` | rc 159 | 97.1 `Object "a Routine" does not understand message "NAME"` — probe error, redone as b4 t06 |
| t12 | `.Routine~newFile('pk.rex')` relative, cwd ≠ main/ | rc 253 | `       *-* Compiled method "NEWFILE" with scope "Routine".`⏎`     1 *-* r = .Routine~newFile('pk.rex');`⏎`Error 3 running <E>/p03/main/t12.rex line 1:  Failure during initialization.`⏎`Error 3.1:  Failure during initialization: File "pk.rex" is unreadable.` — **newFile does not search**, it opens the name as given (relative to the process cwd) |
| t13 | `.context~package~loadPackage('nm', .array~of("say 'lp arr'", '::routine lpf public', "return 'LPF'"))`; `say lpf()` | rc 0 | `lp arr`⏎`name= nm`⏎`LPF` — the source form runs the prolog and merges. **Crate**: rc 120 `rexx-exec: a loadPackage source array is not implemented (Phase 7)` |
| t14 | `::requires 'lib/rq.cls'` (subdirectory of main/) and `::requires 'rpq'` (`rpq.cls` in rp/) | rc 0 | `lib/rq prolog`⏎`rp rpq prolog`⏎`main` |
| t14n | t14 without REXX_PATH | rc 213 | stdout `lib/rq prolog`; 43.901 `"rpq"` |
| t15 | `::requires 'xq'` with `main/xq.rex` and `rp/xq.cls` | rc 0 | `rp xq.cls prolog`⏎`main` — every directory is tried for `.cls` before any for `.rex` |
| t15b | `call xq` | rc 0 | `main xq.rex prolog`⏎`XQ-rex` (crate 43.1) |
| t16 | `findProgram('xq')`, `('req1')`, `('nope')`, `('lib/rq.cls')` | rc 0 | `<E>/p03/main/xq.rex`⏎`<E>/p03/main/req1.rex`⏎`The NIL object`⏎`<E>/p03/main/lib/rq.cls` — findProgram uses the default resolve (no `.cls`) |
| t17 | `t17.rex` requires itself | rc 158 | `     1 *-* ::requires 't17.rex'`⏎`     1 *-* ::requires 't17.rex'`⏎`Error 98 running <E>/p03/main/t17.rex line 1:  Execution error.`⏎`Error 98.952:  Circular ::REQUIRES references detected with <E>/p03/main/t17.rex.` |
| t18 | `.context~package~name`; `loadPackage('pk')~name` | rc 0 | `main`⏎`<E>/p03/main/t18.rex`⏎`pk prolog`⏎`<E>/p03/main/pk.rex` — `.cls` tried, then `.rex` found |

### 2.4 Batch 4 — invocation forms, ADDRESS by route, contexts, edge names (`E/p04`)

| case | shape | oracle | bytes |
|---|---|---|---|
| t01r | oracle invoked as `rexx main/t01.rex` from `<E>/p04`; main and callee print PARSE SOURCE | rc 0 | `main LINUX COMMAND <E>/p04/main/t01.rex`⏎`callee LINUX SUBROUTINE <E>/p04/main/psrc.rex` — both absolute (crate: main line identical, then 43.1) |
| t01 | the same with a wrong cwd (`main/t01.rex` does not exist) | rc 253 | `Error 3:  Failure during initialization.`⏎`Error 3.901:  Failure during initialization: Program "main/t01.rex" was not found.` — **crate**: rc 2, `rexx-run: main/t01.rex: No such file or directory (os error 2)` (runner divergence, outside this area) |
| t02 | `address bash`, then `call addr` (file), `call raddr` (public `::routine` of a required package), `call laddr` (own `::routine`) | rc 0 | `file address BASH`⏎`required ::routine address sh`⏎`local ::routine address sh` — **only the file route inherits ADDRESS**; `::ROUTINE`s get the instance default |
| t03 | `PATH=pp2:/usr/bin:/bin` (relative entry), `cwd/pp2/zrel.rex` | rc 0 | `zrel via relative PATH entry` — relative PATH entries resolve against the process cwd |
| t04 | `call 'my prog.rex'`, `call 'my prog'` | rc 0 | `space name ok`⏎`space name ok` |
| t05 | `main3.REX`: `call c2` with `c2.REX`+`c2.rex` | rc 0 | `c2.REX` — the caller's extension, in its own case, first |
| t06 | `.Routine~newFile(abs pk2.rex)~call` where `pk2.rex` = `return helper2()` and the caller defines `::routine helper2` (`'from main'`); then with context `.Package~new('c2', .array~of('::routine helper2 public', "return 'from c2'"))`; `.Method~newFile(abs)~package~name`, with and without context | rc 0 | `from main`⏎`from c2`⏎`<E>/p04/main/pk2.rex`⏎`<E>/p04/main/pk2.rex` — **a newFile executable resolves routines through the caller's package by default**, or through the given context. **Crate**: rc 213 on the first line: `     1 *-* return helper2()`⏎`       *-* Compiled method "CALL" with scope "Routine".`⏎`     1 *-* say .Routine~newFile('<E>/p04/main/pk2.rex')~call`⏎`Error 43 running <E>/p04/main/pk2.rex line 1: …`⏎`Error 43.1:  Could not find routine "HELPER2".` |
| t14 | `.Routine~newFile(abs pk3.rex)~call`, `pk3.rex` = `return nohelper()` (nowhere defined) | rc 213 | exactly the crate's t06 shape with `"NOHELPER"` — so the traceback shape is right, only the scope is missing |
| t07 | `call pk2` (as an external file) with `::routine helper2` in the caller | rc 213 | `     1 *-* return helper2()`⏎`     1 *-* call pk2`⏎`Error 43 running <E>/p04/main/pk2.rex line 1:  Routine not found.`⏎`Error 43.1:  Could not find routine "HELPER2".` — **a called file does not see the caller's `::ROUTINE`s** (contrast t06: newFile does) |
| t08 | `::method go` does `call zsub`; `zsub.rex` beside the program only | rc 0 | `zsub found from a method` — a method frame searches its package's directory |
| t09 | `numeric digits 5; call numinh`, `numinh.rex` = `say digits()` + `::options digits 12 numeric inherit` | rc 0 | `callee digits 5`⏎`caller digits 5` — `NUMERIC INHERIT` takes the caller's settings over the callee's own `DIGITS 12` |
| t10 | `call '.hid'` with `.hid.rex` | rc 0 | `dot-hid.rex found` — a leading-dot name counts as extensionless (§3.2) |
| t11 | `call '../main/zork.rex'` from cwd/ | rc 0 | `main zork` — `../` binds to the process cwd |
| t12 | `.Package~new('pk2.rex')` with **cwd = main/** | rc 213 | found this time (through `.`), then its prolog `return helper2()` raises: `     1 *-* return helper2()`⏎`       *-* Compiled method "NEW" with scope "Package".`⏎`     1 *-* p = .Package~new('pk2.rex');`⏎`Error 43 running <E>/p04/main/pk2.rex line 1: …`⏎`Error 43.1:  Could not find routine "HELPER2".` — **crate**: same rc and last two lines, but the traceback is the first line only (the `Compiled method "NEW"` and caller lines are missing) |
| t13 | `.context~package~findProgram('pk2')`; `.Package~new('c3', .array~of('nop'))~findProgram('pk2.rex')` with cwd ≠ main/ | rc 0 | `<E>/p04/main/pk2.rex`⏎`The NIL object` — an in-memory package has no directory and no parent to fall back on. **Crate**: second line is `<E>/p04/main/pk2.rex` |

## 3. C++ mechanism

### 3.1 The resolution chain for a call (Read)

`RexxInstructionCall::execute` (`interpreter/instructions/CallInstruction.cpp:159-196`) and
`RexxExpressionFunction::evaluate` (`interpreter/expression/ExpressionFunction.cpp:180-214`) branch in
this order: a cached `externalTarget`; an internal label (`targetInstruction` / `target`); a built-in
(`builtinIndex`); else `RexxActivation::externalCall(resolvedTarget, name, …)` and the returned
`resolvedTarget` is stored in `externalTarget` for the next execution of that clause.

`RexxActivation::externalCall` (`interpreter/execution/RexxActivation.cpp:3062-3105`):

1. Step 2: `settings.parentCode->findRoutine(target)` — the package's own `::ROUTINE`s and the public
   ones merged from `::REQUIRES` (Inferred from `PackageClass::findRoutine`'s use of the merged
   tables; measured t05/t06 of §2.3). A hit is returned through `routine` and cached at the call
   site. **Only this step is cached**; the file route below leaves `routine` null.
2. Steps 2a/2b: the object-function and function exits (RXAPI exits — not reachable from Rexx).
3. Step 3: `SystemInterpreter::invokeExternalFunction`
   (`interpreter/platform/unix/ExternalFunctions.cpp:104-134`): macrospace pre-order
   (`callMacroSpaceFunction`, `RexxActivation.cpp:2989`, needs the RXAPI daemon — Phase 10); then
   `PackageManager::callNativeRoutine` (`interpreter/package/PackageManager.cpp:674-697`: uppercases
   the name, looks in `packageRoutines` — routines from loaded libraries, which is where RexxUtil's
   `Sys*` live — then registered routines); then `callExternalRexx`; then macrospace post-order.
4. Step 4: the scripting exit.
5. `reportException(Error_Routine_not_found_name, target)` → 43.1.

So a library routine such as `SysFileTree` precedes any file named for it; and the file search is
the last real step before 43.1.

`RexxActivation::callExternalRexx` (`RexxActivation.cpp:3121-3160`): an INTERPRET forwards to its
parent (why t17's interpreted call still resolves); `resolveProgramName(target, RESOLVE_DEFAULT)`;
if found, `LanguageParser::createProgramFromFile(filename)` — **a fresh parse on every call**, no
cache (t05) — then `routine->call(activity, target, arguments, argcount, calltype,
settings.currentAddress, EXTERNALCALL, resultObj)` and, after it returns,
`settings.parentCode->mergeRequired(routine->getPackageObject())` (t06, t16). The `::ROUTINE` route
at step 1 passes `OREF_NULL` for the environment; that one argument is why only the file route
inherits ADDRESS (t02 of §2.4): `RexxActivation`'s constructor sets `currentAddress` to the
instance default and then `setDefaultAddress(env)` only when `env != OREF_NULL`
(`RexxActivation.cpp:305-325`). NUMERIC is copied from the parent only when
`packageObject->isNumericInheritEnabled()` (`:293-297`, t09 of §2.4). `settings.calltype` is the
FUNCTION / SUBROUTINE string the call site passed and is what `sourceString()` prints
(`RexxActivation.cpp:4582-4605`: `<platform> <calltype> <code->getProgramName()>`).

`createProgramFromFile` (`interpreter/parser/LanguageParser.cpp:367-385`): `FileProgramSource::readProgram`
failing → `Error_Program_unreadable_name` (3.1, t12 of §2.2, message carries the resolved absolute
path); then `RoutineClass::restore` for a compiled image, else `createProgram(filename, buffer)`.
A parse error surfaces from here, at the call site, as the callee's error with the two-frame
traceback (t07/t08).

### 3.2 The file search (Read, confirmed by §2.1)

`RexxActivation::resolveProgramName` (`RexxActivation.cpp:3204`) → `code->resolveProgramName` →
`PackageClass::resolveProgramName` (`interpreter/classes/PackageClass.cpp:926-936`): calls
`activity->resolveProgramName(name, programDirectory, programExtension, type)` and, on a miss, the
`parentPackage`'s (the chain behind `newFile` / in-memory contexts). `programDirectory` and
`programExtension` come from the package's own name (`PackageClass::extractNameInformation`,
`:422-435`) — which is why t18 of §2.2 and t08 of §2.4 resolve beside the callee / the method's
package.

`InterpreterInstance::resolveProgramName` (`interpreter/runtime/InterpreterInstance.cpp:1167-1216`):

- Builds `SysSearchPath(parentDir, instance extension path)`
  (`interpreter/platform/unix/SysInterpreterInstance.cpp:123-144`): `parentDir`, then `"."`, then
  the instance extension path (unset for `rexx` from the shell), then `getenv("REXX_PATH")` (or the
  compile-time `ORX_REXXPATH`, not set in this build — `CMakeLists.txt:183-186`), then
  `getenv("PATH")`. Entries joined with `:`; empty entries are skipped later.
- `SysFileSystem::hasExtension(name)` (`interpreter/platform/unix/SysFileSystem.cpp:322-343`): scans
  backwards from the last byte while `name < endPtr`, stopping at `/` (false) or `.` (true) —
  **the first byte is never examined**, so `.hid` is extensionless (t10 of §2.4) while `a.b` is
  qualified (t09 of §2.1). Qualified: one `searchName(name, path, NULL)` and done.
- Otherwise, extensions in order: `.cls` when `type == RESOLVE_REQUIRES`; the parent's extension
  when present (in the parent's own case — t05 of §2.4); each of `instance->searchExtensions`
  (`.REX` then `.rex`, `SysInterpreterInstance::initialize`, `:70-72`); then `NULL`.
- `searchName` → `primitiveSearchName` (`SysFileSystem.cpp:377-436`): spellings = as given, then
  `strlower` (only if different); **for every extension including `NULL`** — the doc's "not on the
  last step" (`funct.xml:355-357`) is not in the code, and t04 of §2.1 shows the lowercase hit.
  Each spelling+extension goes to `checkCurrentFile` when `hasDirectory` (`~`, `/`, `./`, `../` —
  `:355-361`) else `searchPath`.
- `searchPath` (`:473-555`): for each non-empty entry, `entry/name`, `canonicalizeName` (`:628-660`:
  `~` expansion, a relative result is prefixed with **the process cwd**, then `normalizePathName`
  collapses `.`/`..`), `stat64`; a regular file returns true; **any other successful `stat` returns
  false immediately, abandoning the remaining entries for that spelling+extension** (t11 of §2.1).
  `checkCurrentFile` (`:438-468`) is the same test on one path.

`SysFileSystem::searchFileName` (`:91-170`, cwd then PATH only, no REXX_PATH) is a different routine
used by stream qualification, not by this search.

### 3.3 `::REQUIRES` loading (Read, confirmed by §2.3)

`RequiresDirective::install` (`interpreter/instructions/RequiresDirective.cpp:136`) →
`PackageClass::loadRequires(activity, name, RESOLVE_REQUIRES)` (`PackageClass.cpp:1303-1325`):
resolve (with `.cls` first), `instance->loadRequires(activity, target, fullName)`, null →
`Error_Routine_not_found_requires` (43.901); then `addPackage`.

`InterpreterInstance::loadRequires` (`InterpreterInstance.cpp:1021-1062`): instance cache by short
name, then by full name (each hit runs `activity->checkRequires(programName)` →
`Error_Execution_circular_requires` 98.952 when that name is still installing,
`interpreter/concurrency/Activity.cpp:3702-3709`); else `PackageManager::loadRequires`
(`PackageManager.cpp:716-796`: security-manager `checkRequiresAccess`, the process-wide weak
`loadedRequires` cache, macrospace before/after, `getRequiresFile` → `LanguageParser::createPackage(name)`
→ the same `createProgramFromFile`); then `addRequiresFile(shortName, fullName, package)` **before**
`package->runProlog(activity)` — the ordering the circularity check depends on. `runProlog`
(`PackageClass.cpp:2128-2140`) calls the main executable with calltype `REQUIRES` (t01 of §2.3) and
no arguments, or only `install()` when the prolog is disabled.

The array-source forms (`PackageManager.cpp:852-866`) are not cached. The in-store form
(`:869-892`) is.

### 3.4 The package APIs (Read, confirmed by §2.3/§2.4)

- `PackageClass::newRexx` (`PackageClass.cpp:158-226`): no source → `instance->resolveProgramName(name,
  OREF_NULL, OREF_NULL, RESOLVE_REQUIRES)` — **no parent directory, no parent extension** (t08 of
  §2.3, t12 of §2.4) — then `instance->loadRequires(activity, nameString, resolvedName)` (cache,
  43.901, prolog). With source → optional third-argument context (Method/Routine/Package, else
  `Error_Incorrect_method_argType` 93.953), `LanguageParser::createPackage(name, array, context)`,
  `runProlog`; not added to any package's requires (t10 of §2.3).
- `PackageClass::loadPackageRexx` (`:1842-1860`): `checkRexxPackage`, then `loadRequires(…,
  RESOLVE_REQUIRES)` in the receiver's context (parent dir + extension, t07/t18 of §2.3) or, with
  an array, `loadRequires(activity, name, array)` (`:1339-1358`: `instance->loadRequires` array
  form, then `addPackage`) — merged into the receiver either way (t13 of §2.3).
- `PackageClass::findProgramRexx` (`:946-968`): `RESOLVE_DEFAULT` in the receiver's context, then the
  parent package, else `.nil` (t16 of §2.3, t13 of §2.4).
- `RoutineClass::newFileRexx` (`interpreter/classes/RoutineClass.cpp:473-480`) and
  `MethodClass::newFileRexx` (`MethodClass.cpp:533`): `BaseExecutable::processNewFileExecutableArgs`
  (`interpreter/execution/BaseExecutable.cpp`, the block after its definition line): a missing
  context defaults to the **current Rexx frame's package**; a Method/Routine context yields its
  package; a Package is used as is. Then `LanguageParser::createRoutine(filename, sourceContext)`
  (`LanguageParser.cpp:231-249`): `readProgram` on the name as given → 3.1 (t12 of §2.3); the new
  package's `parentPackage` is the context, which is what `findRoutine` / `findClass` /
  `resolveProgramName` fall back to (t06 of §2.4).

### 3.5 Error numbers in this area (Measured, §2)

| condition | error | rc |
|---|---|---|
| no file (call) | 43.1 `Could not find routine "<name>"` | 213 |
| no file (::REQUIRES, `Package~new(name)`, `loadPackage(name)`) | 43.901 `Could not find file "<name>" for ::REQUIRES.` | 213 |
| found but unreadable (call) | 3.1 `Failure during initialization: File "<abs path>" is unreadable.` | 253 |
| `newFile` cannot open | 3.1 with the name as given | 253 |
| main program missing | 3.901 `Program "<name>" was not found.` | 253 |
| callee does not parse | the callee's own error (e.g. 35.1), two-frame traceback, callee file named | 256−N |
| callee raises | the callee's own error, two-frame traceback | 256−N |
| function call, no value | 44.1 `No data returned from function "<NAME>".` | 212 |
| circular ::REQUIRES | 98.952 `Circular ::REQUIRES references detected with <abs path>.` | 158 |

## 4. The crate today

- **The 43.1 site** — `rust/crates/rexx-exec/src/run.rs:3526-3563`: label → builtin →
  excluded-builtin loud gap → `installed_routine(name)` (`::ROUTINE`s, own and merged) →
  `rexx_lib::lookup` under `library_bootstrap` (the embedded `CoreClasses.orx` calls) → `None =>
  Err(Raised::routine_not_found(name))`. The file search goes in that last arm, after the library
  lookup, which is where the oracle's step 3 sits. The refusal's text and rc already match the
  oracle's byte for byte (§2.1).
- **The search that exists** — `Interp::resolve_search(program, name, requires)`
  (`rust/crates/rexx-exec/src/lib.rs:2966-2989`) over `require::search_entries` / `candidates` /
  `normalize` (`rust/crates/rexx-exec/src/require.rs:11-113`). Directory order, extension order,
  `.cls` gating, the as-given-then-lowercase retry on every step, `has_directory`, empty PATH
  entries, relative entries against cwd and `..` collapsing all match §3.2 — `::REQUIRES` is
  byte-identical on every case of §2.3 that does not go through `Package~new`. It reads the
  **process** environment and cwd: `std::env::var("REXX_PATH")`, `std::env::var("PATH")`,
  `std::env::current_dir()` (`lib.rs:2974-2977`). **Nothing named a shadow environment or shadow
  current directory exists in the tree today**: no `DIRECTORY` builtin (`bifDirectory`,
  `funct.xml:2461`, absent from `src/builtin/`), no `VALUE(…,'ENVIRONMENT')` handling
  (`src/builtin/` has no `ENVIRONMENT` string; `builtin/state.rs:27` only holds the default
  ADDRESS `sh`). Those are other surveyors' areas; this one consumes them.
- **Three places `require.rs` differs from §3.2**: (i) `has_extension` (`require.rs:87`) uses
  `rfind('.')` over the whole last component, so `.hid` counts as qualified — the oracle finds
  `.hid.rex` (t10 of §2.4); (ii) `resolve_search` keeps scanning after a candidate that exists but
  is not a regular file (`metadata().is_file()`, `lib.rs:2982`), where the oracle abandons that
  spelling+extension's path scan (t11 of §2.1); (iii) `~` is passed through unexpanded
  (`has_directory` recognises it, `normalize` does not expand it) — not measured on either side.
- **`::REQUIRES`** — `Interp::load_requires` (`lib.rs:2778-2810`): short-name cache, resolved-name
  cache, 43.901, read, parse, `ProgramId(self.programs.len())` pushed to `programs` with its path in
  `required_paths`, cached **before** `run_loaded(parsed, id, CallType::Requires)` (the comment cites
  `InterpreterInstance.cpp:1060`), `check_not_installing` → 98.952 (`lib.rs:2815-2823`). A callee that
  does not parse is `Loud::required_source` (`lib.rs:2794`) — a loud refusal where the oracle raises
  the callee's own error; the same conversion the external-call path needs (§5 Q2).
- **A second program** is a `ProgramId` index into `Interp.programs: Vec<Rc<Program>>`
  (`lib.rs:1522`); `package_path(id)` (`lib.rs:2708-2713`) answers `required_paths[id]` or the main
  `program_path`. `run_loaded` (`lib.rs:2267-2330`) installs directives, honours `::OPTIONS NOPROLOG`
  for `Requires` only, builds the main `Activation` with `call_type`, and `start_from_package`.
  `CallType` (`rust/crates/rexx-exec/src/activation.rs:380-405`) already renders `COMMAND`,
  `SUBROUTINE`, `FUNCTION`, `REQUIRES` for PARSE SOURCE; the crate prints the absolute path for the
  main program (t01r of §2.4) and for required files (t01 of §2.3).
- **`Package~new`** — `dispatch/package.rs:914-945`: file form calls
  `interp.load_package(running_program, name)` → `load_requires` → `resolve_requires(id, name)`,
  i.e. **with the caller's directory and extension** — the divergence of t08 §2.3 / t12–t13 §2.4.
  A context argument on the source form is the `Loud::executable_context()` refusal ("a newFile
  package context", `lib.rs:378-382`, raised at `package.rs:937-940`).
- **`loadPackage(name, source)`** — `package.rs:882-909`: the array form is
  `Loud::package_from_source()` ("a loadPackage source array", `lib.rs:409-413`), although
  `Interp::package_from_source` (`lib.rs:3000+`) already builds and runs such a package for
  `Package~new(name, array)` (t10 of §2.3 is byte-identical). What is missing is only the
  `add_imported_package` + `merge_required` that the file form does (`package.rs:904-907`).
- **`newFile`** — `dispatch.rs:8181-8200` → `Interp::new_file_executable(name, routine)`
  (`lib.rs:3765-3800`): reads the name as given (3.1 matches, t12 of §2.3), parses, and wraps the
  main body as a `::ROUTINE` / `::METHOD` directive of a new program. A context argument is the
  same `executable_context()` refusal (`dispatch.rs:8196`). **What "a newFile package context"
  means**: the C++ gives the new package a `parentPackage` — the caller's package by default, or
  the Method/Routine/Package passed — and routine, class and program resolution fall back to it.
  The crate's package has no parent, so even the default case diverges (t06 of §2.4: `helper2`
  from the caller is 43.1 here). The refusal names the argument; the defect is the missing parent
  chain.
- **Traceback of a prolog error under `Package~new`** — the crate prints only the callee frame
  (t12 of §2.4); the oracle adds `*-* Compiled method "NEW" with scope "Package".` and the caller's
  clause. The `~call` route already has the analogous line right (t14 of §2.4).
- **Output** is buffered in `Interp.out` / `Interp.trace` (`lib.rs:1692-1694`) as the controller
  states; a callee's `say` lands in the same buffer, and the oracle's t14 of §2.2 (shared stream
  table) has the same shape for streams.
- **Harness** — `rust/crates/rexx-exec/tests/support/oracle.rs:242-262`: `Oracle::run_in(path, cwd,
  env)` exists; the plain runner (`wrapped`, `:268-283`) sets the oracle's cwd to the program's own
  directory, so for a corpus program `.` and the program directory coincide and the cwd-vs-program
  precedence (t01 of §2.1) is invisible to it.
- The corpus rule `docs/superpowers/plans/phase-4-exclusions.txt:342-390` is the row this area
  deletes.

## 5. Design questions

**Q1. Where the external-file call plugs in, and what it reuses.**
Options: (a) a new `Resolved::External(path)` arm in `run.rs:3552`'s `None =>` position, loading the
file and running it through a new `run_external`; (b) resolve to a `ProgramId` and dispatch through
the existing `Resolved::Routine` machinery, treating the file as "a `::ROUTINE` whose package is the
file", which is what `RexxCode::call` is for both in the C++ (`interpreter/execution/RexxCode.cpp:177-192`).
Recommend (b): one activation-building path, one RESULT / 44.1 / EXIT-vs-RETURN implementation
(already right for `::ROUTINE`s), one error-propagation path. The per-route differences are three
flags on the activation: ADDRESS inherited (file) vs default (`::ROUTINE`), `call_type`
FUNCTION/SUBROUTINE, and `merge_required(caller, callee)` after return. Cost if wrong: a second
copy of the routine-call semantics that drifts (the shape the Phase 5 record warns about). Do
**not** cache the loaded program per call site or in `required_packages` — t05 §2.2 shows a re-read
each call; keep it out of the requires cache so a later `::REQUIRES` of the same file still runs
its prolog under `REQUIRES`.

**Q2. Error conversion for a callee that fails to parse.** The main program's parse error already
becomes the oracle's 35.1-style report (the `corpus/errors/parse-errors.tsv` gate); `load_requires`
does not reuse it (`Loud::required_source`). Options: (a) route the callee's `parse_program` error
through the same conversion as the main program, attributed to the callee's path and line, with the
caller's clause appended below in the traceback (t07/t08 §2.2) and trappable at the call site (rc
35, sigl = call line); (b) keep it loud. Recommend (a), and apply it to `::REQUIRES` in the same
change. Cost if wrong: every corpus witness with a malformed callee is a loud red instead of a
byte-identical error.

**Q3. Search fidelity fixes in `require.rs`.** (i) `has_extension` must scan backwards and never
examine byte 0 (t10 §2.4); (ii) a non-regular `stat` hit must end that spelling+extension's path
scan (t11 §2.1) — the `candidates` list is flat, so either the search loop learns the
spelling+extension boundaries or `candidates` returns groups; recommend groups
(`Vec<Vec<String>>`, one per spelling+extension); (iii) `~` expansion from the shadow environment's
HOME — unmeasured on the oracle, low priority; (iv) `Package~new(name)` needs a "global context"
search: `resolve_search(None, name, true)` (no parent dir, no parent extension, `.cls` first) rather
than the caller's context (t08 §2.3); `findProgram` on an in-memory package likewise (t13 §2.4).
Cost if wrong: (i)/(ii)/(iv) are each one measured divergence today.

**Q4. The shadow environment and shadow cwd (controller fact 1).** `resolve_search` must take its
PATH, REXX_PATH (and HOME for `~`) from the interpreter's environment map and its cwd from the
interpreter's directory field, and `normalize` relative candidates against that cwd; the file is
then opened by absolute path, so the process cwd is never consulted. Recommend a small
`SearchContext { parent_dir, parent_ext, cwd, rexx_path, sys_path }` built by the caller of
`resolve_search`, so the function is pure and unit-testable without touching the process. Whether
the shadow cwd is initialised from `std::env::current_dir()` at interpreter start is the
platform surveyor's call; this area only reads it. Cost if wrong: the harness's in-process threads
race on `set_current_dir`, and `ootest/testOORexx.rex` (which sets PATH via VALUE and cwd via
DIRECTORY before `'worker.rex'(…)`) cannot run.

**Q5. What the callee's activation gets.** From §2.2/§2.4: fresh variables; NUMERIC defaults unless
the callee's package has `NUMERIC INHERIT` (`start_from_package` already reads `::OPTIONS`;
the inherit path exists for `::ROUTINE`s — reuse it); ADDRESS = caller's current address (file route
only); TRACE from the callee's package settings (N); the same `.local`, stream table and output
buffers; no condition traps; `sigl` unset; PARSE SOURCE = `LINUX <FUNCTION|SUBROUTINE> <resolved
absolute path>`. Recommend making "environment to inherit" an `Option<Vec<u8>>` parameter of the
activation builder, `None` for `::ROUTINE`s — the C++'s exact shape. Cost if wrong: t02 §2.4.

**Q6. Merge after return.** `merge_required(caller, callee)` exists for `::REQUIRES`; after an
external call the same merge applies, and it must carry what the callee itself required (t16 §2.2).
Recommend calling it with the callee's `ProgramId` after every return (also after an error? not
measured — see §7). Cost if wrong: t06/t16 §2.2.

**Q7. `loadPackage(name, source)` and `newFile` contexts.** The array form is
`package_from_source` + `add_imported_package` + `merge_required`; recommend lifting the refusal in
the same slice as Q1 since it is two lines. The context (parent package) is a new field on the
package tables: `parent: Option<Package>` consulted by routine/class lookup and by
`resolve_search` on a miss. Recommend doing the default-parent case (caller's package) first —
it is the measured divergence (t06 §2.4) — and the explicit-context argument with it, since both
are the one field. Cost if wrong: `newFile` executables cannot call anything of the program that
built them.

**Q8. Corpus mechanics for environment-dependent witnesses.** The corpus runner gives the oracle
cwd = program directory and the process environment. Witnesses for cwd-vs-program-dir, REXX_PATH
and PATH need `Oracle::run_in` and a pinned environment; recommend a per-program sidecar
(`<name>.env`: `PATH=…`, `REXX_PATH=…`, `CWD=<relative dir>`) read by the harness, with fixture
files in a subdirectory beside the program (`<name>.d/`). The crate side sets its shadow
environment from the same sidecar. A pinned PATH also closes the "scratchpad on the search path"
hazard for the oracle. Cost if wrong: witnesses that depend on the developer's PATH.

**Q9. `unsafe` / dependencies / platform.** None of this needs `unsafe` or a new crate:
`std::fs::metadata` (regular-file test), `std::fs::read`, string splitting on `:`. Platform: `/`
and `:` are Unix; the Windows `.REX`-only and case-insensitive branch is out of scope. The
lowercase retry is `to_lowercase` on bytes — the C++ `strlower` is byte-wise ASCII; a non-ASCII
name would differ under Unicode lowercasing, so use ASCII lowercase.

## 6. Proposed task slices

Each slice names its corpus witnesses (programs whose oracle bytes it must match) and a paired
negative. Witness sources are the §2 probes; each becomes a corpus program plus a fixture directory.

1. **`require.rs` fidelity + `SearchContext`.** Pure changes: leading-dot names, non-regular
   abort, grouped candidates, context struct fed from the shadow env/cwd, ASCII lowercase. Unit
   tests as a `datadriven` table. Witnesses (through `::REQUIRES`, which already works): t14/t14n
   §2.3, t15 §2.3, t04-shaped `::requires 'e3'` with a lowercase no-extension file; negative: t03
   §2.3 (43.901 unchanged).
2. **External file call (Q1, Q2, Q5, Q6).** Witnesses: t01 §2.2 (callee view), t02 (RESULT), t03/t04
   (44.1), t06 (merge, re-run), t16 (transitive merge), t17 (call types + INTERPRET), t18 (nested
   directory), t15 §2.1 (label / builtin / quoted precedence), t08 §2.1 (caller extension), t12
   §2.1 (quoted with extension, function form), t10 §2.1 (`sub/`, `./`, absolute), t02/t07/t08/t09
   §2.4 (ADDRESS by route, no caller `::ROUTINE`s, method frame, NUMERIC INHERIT), t07/t08/t09/t10
   §2.2 (callee parse and runtime errors, trapped and not), t12 §2.2 (3.1 unreadable, fixture
   chmod 000 — note the harness must create it), t15 §2.2 (traps not inherited). Negatives: t05,
   t09, t11, t13 §2.1 (43.1 unchanged with a near-miss file present).
3. **Environment-driven search + harness sidecar (Q4, Q8).** Witnesses: t01 §2.1 (program dir
   beats cwd — needs cwd ≠ program dir), t02 (cwd > REXX_PATH > PATH), t02n (REXX_PATH unset),
   t16 (odd PATH entries), t03 §2.4 (relative PATH entry), t11 §2.4 (`../`), t01r §2.4 (relative
   invocation, absolute PARSE SOURCE). Negative: t02n.
4. **Package APIs (Q3-iv, Q7).** Witnesses: t08 §2.3 (`Package~new` global context → 43.901), t12/t13
   §2.4, t13 §2.3 (`loadPackage(name, array)` merges), t06 §2.4 (`newFile` default and explicit
   context), t14 §2.4 (its error traceback), t12 §2.4 (prolog-error traceback under `Package~new`),
   t07 §2.3 once Phase 5's `==` on packages lands (or rewritten with `~name` comparison), t18 §2.3.
   Negatives: t09 §2.3, t12 §2.3, t10 §2.3 (in-memory package not merged).
5. **Corpus rule lift.** Delete `phase-4-exclusions.txt:342-390`, drop the rule from
   `rust/corpus/phase-4c.txt`, and move the §2 probe trees under `corpus/` with their sidecars. This
   slice has no behaviour of its own; its witness is the whole set above going green under
   `REXX_CORPUS_GATE=1`.

## 7. Not done / not established

- `~` expansion in a call target (`resolveTilde`) — not measured; reads outside the probe directory.
- Whether `mergeRequired` runs when the callee ends in an error (the C++ merges only after a normal
  return at `RexxActivation.cpp:3149`; not measured — Inferred: no merge).
- Macrospace pre/post-order and registered/library routines in the chain (`Sys*`, Phase 8/10) —
  only their position is established (§3.1), not their behaviour.
- The security manager's `checkRequiresAccess` on the requires route (D12).
- Compiled images (`rexxc` output) called as external files (`RoutineClass::restore`).
- `Package~new(name, source, context)` semantics beyond the argument check; `Package~new` with a
  file **and** a context (the C++ ignores the context on the file branch — Read, `PackageClass.cpp:178-183`,
  not measured).
- `condition('O')~traceback` contents for a callee error (my probe used `~toString` on a List).
- `t06` §2.1 second half (`call LOWER` hit a built-in): the lowercase-retry-with-extension case is
  covered by t04 (no extension) and t03's `e5.rex` (parent extension) instead.
- The crate's `==` on package objects (t07 §2.3) and `condition('O')` (t08/t10 §2.2) are Phase 5
  gaps that blocked two crate-side checks; the oracle side of both is recorded.
- The runner's missing-program report (t01 §2.4: oracle 3.901 rc 253, crate rc 2 with an OS
  message) — recorded, outside this area.
- Windows behaviour of any of it.

<!-- SURVEY COMPLETE -->
