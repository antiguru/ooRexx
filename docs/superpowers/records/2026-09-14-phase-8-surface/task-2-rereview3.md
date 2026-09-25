# Task 2 fix round 3 re-review

Fix base `9ff11e465`, head `cb13f0033`. Findings under verification: C-1, m-1 to m-5
(`task-2-rereview2.md`), against `progress.md` "Task 2 fix round 3: sent" and the report's "Fix
round 3". Written first and appended as the review goes. Scratch:
`scratchpad/t2-rereview3/`, a fresh directory per run, three descriptors. `corpus/oracle-crashes.txt`
read first; no probe below is one of its entries (no `loadExternal*` object is used as a context).
Nothing registered with rxapi. Read-only on the worktree.

## Log

(appended as it happens)
- Setup. Worktree clean at `cb13f0033`; `cargo build --release --bin rexx-run` "Finished" with no
  compile line (`ce749ec5...`). `git archive 9ff11e465 rust api` into `t2-rereview3/base` and
  `cb13f0033` into `head`, `interpreter/` linked beside each; each built release in its own
  `target-base`/`target-head` (both exit 0). `ab.sh` runs oracle (`o`), head worktree binary (`h`),
  base archive binary (`b`), each from a fresh copy renamed to the same run path, `ulimit -v
  1048576`, three descriptors.
- Printed, oracle (`/home/moritz/dev/repos/ooRexx/interpreter`, `diff -q` identical to the
  worktree's copy for both files): `ExpressionFunction.cpp:179-215` and `CallInstruction.cpp:159-198`
  as the report quotes them (arguments at `:185`/`:166`, then `externalTarget`, label, builtin,
  `externalCall` + `setField`). `ExpressionFunction.cpp:154-167` and `CallInstruction.cpp:136-149`
  (`resolve`: the label from the labels table, at install). `CallInstruction.cpp:286-335`
  (`RexxInstructionDynamicCall::execute`): the name is evaluated and traced, **then the arguments,
  then** the label (case-sensitive `labels->get`) and `resolveBuiltin`, never cached. `:429-460`
  (qualified): arguments first, then `findNamespace`/`findPublicRoutine` every execution.
  `:549-560`, `:570-630` (`CALL ON`): `resolve` fixes the label; the `CALL ON`/`OFF` instruction
  only sets the trap; at delivery the label, else the builtin index, else `externalCall` with the
  condition object as the one argument, not cached. `RexxActivation.cpp:3062-3105`: Step 2
  `routine = findRoutine(target)` is the only write to the out-parameter; Step 3
  (`ExternalFunctions.cpp:104-135`) is macro space, `callNativeRoutine`, `callExternalRexx`. So the
  report's reading holds for every form: what can differ before and after the arguments is only
  what `externalCall` searches; a label and a builtin are static in every form, dynamic included.
  **One thing the report does not say**: `setField(externalTarget, resolvedTarget)` runs after
  `externalCall` returns, so a Step 2 routine whose call raises is not kept by the oracle, and a
  recursive entry of the same instruction before the outer call returns sees no kept target.
- Crate, read: the round changed `Op::CallExpr` (`drive.rs:698-717`), `Op::Call` (`:1529-1539`),
  `eval_call` (`eval.rs:530`), `exec_call` (`run.rs:4689`), `resolve_and_run_call` (`run.rs:4553`,
  the `CALL ON` delivery at `:3300`). `Op::CallArgs`/`Op::CallNamed` (`drive.rs:255`, `:304`) are
  unchanged and already resolve after their argument ops. `native_shape` (`compile.rs:1186-1208`)
  is asked with no path for an argument, so any argument that is a call, a message or anything but
  literal/variable/stem/compound/arithmetic/prefix makes the call `Op::CallExpr`/`Op::Call`.
  `resolve_routine_call` and `settle_after_arguments` are `&self`: no allocation, no collection
  between the argument values and the call. `resolve_routine_call`'s only `Err` is
  `Loud::internal_routine`. `Resolved::kept_until_routines_change` (`run.rs:199-207`) names
  `Library`, `Internal`, `LibraryRoutine`, `External`; **`Unresolved` is not in it**, so a site
  that remembered `Unresolved` keeps it for good (`ir.rs:460-466`), before this round and after.
  `routine_generation` only increments (`lib.rs:4821-4824`, the one writer).

### Matrix, predictions written before running

Oracle rule predicted throughout: arguments first; Step 2 kept at the instruction; Step 3
searched on every call. Head predicted SAME unless named; base named per probe.
- `e1` (error precedence, count, traps, one program, `SIGNAL ON` chained through labels): a missing
  routine with a raising argument through `x = nosuch(1/0)` (CallArgs), `x = nosuch(boom())`
  (CallExpr), `call nosuch 1/0` (CallNamed), `call nosuch boom()` (Call), a method argument
  (`eval_call`), `INTERPRET`, `call (nm) boom()`: `42.3` on each. `nosuch(inc())` and `call nosuch
  inc()` with `inc` counting: count `1`/`2` and `43.1`. `SIGNAL ON NOVALUE` with `nosuch(undefvar)`
  and `nosuch(inc(), undefvar2)`: `NOVALUE UNDEFVAR`, then count `3` and `NOVALUE UNDEFVAR2`.
  Without the trap, `nosuch(undefvar3)`: `43.1`. Base SAME (base raised 43.1 after the arguments
  too).
- `e2`: `CALL ON USER` naming a routine nothing answers, untrapped: the oracle's 43.1 report; head
  and base SAME or both DIFF the same way.
- `t1` (`trace i`) and `t2` (`trace r`): a label, a builtin, an internal routine (`filespec`), an
  external file, a merged library routine, a `::routine`, each as a function with a literal
  argument, a function with a call argument, `CALL` with each argument shape: SAME on three
  descriptors, base SAME. `t3`: the same for a registered library routine (`loadLibrary`).
- `l1`: a label after the call site, a label `length:` shadowing the builtin, the quoted forms
  skipping the label, `call (nm)` with `'LENGTH'` and `'length'`, each with a literal and a call
  argument. SAME, base SAME (not predicting the oracle's `call (nm)` answer for `'length'`).
- `b1`: `length(mk())`, `'LENGTH'(mk())`, `call length mk()`, `filespec('N', mk())` where `mk`
  writes `length.rex`/`filespec.rex`: the builtin and the internal routine answer. SAME, base SAME.
  `zzmade(mkz())` where `mkz` writes `zzmade.rex`: oracle `file`; base 43.1 (trapped, `raised`).
- `i1`: `interpret 'say i foo(step())'` in a loop, `step` adding `FOO` on pass 2: oracle `1 main` /
  `2 added` / `3 added`; base `2 main`. `i2`: the loop inside one `INTERPRET`: oracle `main` /
  `added` / `added`; base `main` / `main` / `main`.
- `k1` kept external file, `call ext step(i)`, `step` doing `loadPackage` of a package exporting
  `EXT` on pass 2: oracle `ext` / `pub` / `pub`; base `ext` / `ext` / `pub`.
- `k2` kept internal `filespec` in a `say`, `addRoutine('FILESPEC')` in the argument on pass 2:
  oracle `b.c` / `added` / `added`; base `b.c` / `b.c` / `added`.
- `k3` kept registered library routine (`loadLibrary` first), `loadPackage` of a package exporting
  `RxCalcSqrt` in the argument on pass 2: oracle `4` / `pk 16` / `pk 16`; base `4` / `4` / `pk 16`.
- `k5` nested: `ext(foo(step(i)))`, `ext` a kept file, `step` adding `EXT` on pass 2 two calls deep:
  oracle `ext` / `added` / `added`; base second pass `ext`.
- `k6` recursion: `rec(n)` returns `ext(rec(n - 1) || addmaybe(n))`, `addmaybe` adding `EXT` at
  n = 2, `rec(0)` answering `end`; called `rec(3)` then `rec(3)` again: oracle first call `ext`
  inside, `added` from the level where the add ran; the second call `added` at every level (the
  outer instruction kept `added`). Head SAME. Base: not predicted with confidence.
- `k8` a kept file whose argument moves the generation on every pass without touching `EXT`: SAME,
  base SAME.
- `k9` kept `::routine foo`, `addRoutine('FOO')` in the argument on pass 2: oracle `main` × 3
  (`externalTarget`); SAME, base SAME. `k10` the same for a merged library routine: `4` × 3.
- `k11` a site in `sub:` whose argument deletes `ext.rex` on pass 1 (43.1, trapped) and rewrites it
  on pass 2: oracle `pass1 raised 43.1` / `sub 2 ext 2`. **Head: `pass2 raised 43.1`** (pass 1
  remembers `Unresolved` after the arguments, kept for good). Base: pass 1 kept `External` before
  the arguments; not predicted what its entry does with the file gone; pass 2 `sub 2 ext 2`.
- `k12` a `::routine foo` that raises when called (trapped), then `addRoutine('FOO')`, then the
  same site again (in `sub:`): oracle `added` (a raised call is not kept, `setField` never ran);
  head and base `raised` again or `main`-shaped: predicted DIFF on both (pre-existing).

Ran (`p/*.ab.txt`):
- `e1`, `e2`, `l1`: **SAME on all three sides**, as predicted. `e1`: `42.3` from every shape
  (CallArgs, CallExpr, CallNamed, Call, method argument, `INTERPRET`, `call (nm)`), counts `1`,
  `2`, `3`, `4` (one evaluation per call), `NOVALUE UNDEFVAR`/`UNDEFVAR2`, `43.1` untrapped. `l1`:
  `lbl 1`, `lbl 2`, `lbllen abc`, `lbllen abcd`, `3`, `5`, `lbllen x`, `lbllen xx`, `3`, `4`,
  `lbllen q`, `lbllen qq`, then `call (nm)` with `'length'` is 43.1 rc 213 on all three (the
  dynamic label search is case-sensitive and so is `resolveBuiltin`). `e2`: all three print `start`
  and stop at rc 0.
- `t1`, `t2`, `t3`: **head `cmp`-equal to base on stderr and stdout** in all three. `t3` (registered
  library routine, `trace i` and `trace r`, four shapes each) SAME. `t1`/`t2` DIFF on stderr on
  both head and base, the same two lines: the oracle's `>>>   "ext 1"` after `>F>   EXT => "ext 1"`
  for `x = ext(1)` and `x = ext(id(2))` is absent here (the `CALL` forms and every other kind
  agree). Pre-existing at `9ff11e465` (out of scope below).
- `b1`: head SAME (`6` `6` `6` `b.c` `b.c` `file z`); base DIFF only on the `zzmade` line (43.1), as
  predicted.
- `i1`: **as predicted**, head SAME (`1 main` / `2 added` / `3 added`), base `2 main`.
- `i2`: **my prediction was wrong** about the oracle: `1 main` / `2 main` / `3 main` on all three
  sides. Pass 1 already found `main` at Step 2 and kept it, so the add on pass 2 is never seen.
  Agrees with the rule; the prediction misapplied it.
- `k1`, `k2`, `k3`, `k5`: **as predicted**, head SAME, base DIFF on pass 2 only (`2 ext 2`, `2
  b.c`, `2 4`, `2 ext foo 2`).
- `k6`: head SAME (`first added( added( ext end ) )` / `second added( added( added( end ) ) )`);
  base `first ext ext ext end`. So a site entered recursively inside its own arguments, with the
  generation moved by the inner level, settles each level after its own arguments.
- `k8`, `k9`, `k10`: SAME on all three, as predicted.
- `k11`: **my prediction was wrong**: head SAME (`pass1 raised 43.1` / `sub 2 ext 2`), base SAME.
  The "`Unresolved` kept for good" reading in the Crate bullet above is **false**:
  `Calls::remember` (`ir.rs:517-529`) returns without recording `Unresolved`, so a miss is never
  kept. Retracted; the table's `get` reading was right and incomplete.
- `k12`: head and base DIFF, the same: oracle `pass1 raised 42.3` / `sub 2 added 2`; head and base
  `pass2 raised 42.3`. A `::routine` whose call raised stays kept here; the oracle never ran
  `setField`. Pre-existing (out of scope below).

Predictions for two more shapes, written before running:
- `k13`: the `Op::Exec` paths. `call (nm) step(i)` with `nm = 'EXT'` a file, `step` adding `EXT`
  on pass 2 (`exec_call`), and `.array~of(ext2(step(i)))[1]` the same for `EXT2` (`eval_call`):
  oracle `dynamic 1 ext 1` / `dynamic 2 added 2` / `dynamic 3 added 3`, the method argument the
  same. Head SAME. Base `2 ext 2` and `2 ext2 2`.
- `q1`: a namespace-qualified call whose argument `addPublicRoutine`s the name into the namespace
  package (the kind the round leaves resolving first): oracle `fn late` / `call late` (arguments
  first, `CallInstruction.cpp:436` then `:450`); **head and base `fn raised SYNTAX 43` / `call
  raised SYNTAX 43`**.
- `k14`, the report's first concern made concrete (a kept Step 3 answer the file system changes
  without moving the generation): `sub` calls `ext(id(n))`, then `ext.rex` is deleted, then `sub`
  again. Oracle `sub 1 ext 1` / `raised 43.1 ...`. Head and base: not predicted with confidence
  (whatever `enter_external_program` does when its second search misses); head the same as base,
  since neither the kept kind nor its generation check changed for this shape.

### The shared `loadExternalRoutine` object (risk 4), predictions written before running

- `L1`: `loadExternalRoutine` before anything binds the row, then `loadPackage` of a package that
  `::requires 'rxmath' LIBRARY` and answers its `findRoutine('RXCALCSQRT')`, then `loadPackage` of
  a package with `::routine sq public external "LIBRARY rxmath RxCalcSqrt"`, then a second
  `loadExternalRoutine` in lower case; `~package` (as the witnesses' `show`) and `==` on hash
  strings at each step. Oracle: `before nil`, `same 1`, `late 1`, `call 4 5 6 7`; the `after
  requires` and `after directive` packages not predicted. Head SAME. Base `same 0` and `late 0`,
  the packages the same as head.
- `L2`: the merged object first (main `::requires 'rxmath' LIBRARY`), then `loadExternalRoutine`,
  then the directive package: oracle `loaded 1`, `imported 1`; packages not predicted. Head SAME;
  base `loaded 0`.
- `L3`: `loadExternalRoutine('myname', ...)~call('x')`, trapped then untrapped: the traceback's
  `Compiled routine` line is the recorded missing line (absent here on both); the untrapped
  report's routine name not predicted. Head the same as base or closer to the oracle.

Ran: `L1`, `L2` **as predicted**: oracle and head `before nil` / `after requires nil` / `same 1 nil` /
`after directive pk.cls pk.cls pk.cls` / `late 1 pk.cls` / `call 4 5 6 7`, and `merged nil` /
`loaded 1 nil` / `after directive pk.cls pk.cls pk.cls` / `imported 1`; base `same 0`, `late 0`,
`loaded 0`, `imported 0`, packages identical. So a `::REQUIRES ... LIBRARY` binds no package on
either side, the first `EXTERNAL "LIBRARY ..."` directive names it retroactively for the shared
object whichever route made it, and `importedRoutines` answers the same object. `L3`: head and base
identical (stdout DIFF on both): the trapped `TRACEBACK` lacks both `*-* Compiled routine
"RxCalcSqrt".` and `*-* Compiled method "CALL" with scope "Routine".` (the recorded `c3` family, a
second missing line here); the untrapped report is SAME on all three.

### The memory test (risk 5), prediction written before running

Read: `locate()` (`tests/support/oracle.rs:130-147`) asserts `bin/rexx` is a file, with no gate
and no skip; `licensed_divergences.rs:172` and `datetime_zone.rs:55` call it, as the test's doc says.
Control, in the `head/` archive (file backed up, `sed` renames the library in the new assertion):
red at that assertion naming `/home/moritz/dev/repos/ooRexx/build/lib/librxmath_absent.so`, no
`memory allocation` text; restored (`cmp`-equal to the worktree's), `1 passed`.

Two more kept-answer shapes, predictions written before running:
- `k15`: a kept external file `RxCalcSqrt` whose argument loads rxmath on pass 2 (`loadLibrary`):
  oracle `1 file 16` / `2 4` / `3 4`. Head SAME if a library registering routines moves the
  generation (the writes table says it does). Base `2 file 16`.
- `k16`: `Op::CallArgs` (unchanged by the round) whose native argument `o + 1` runs a Rexx `+`
  method that adds `EXT` on pass 2: oracle `1 ext 7` / `2 added 7` / `3 added 7`; head and base SAME.

Deep recursion, because `CallResolution` is wider than `Resolved` and sits in the frames of the
recursive call paths (a Rust stack overflow before the depth guard would be a crash where the
guard raises). Unbounded recursion through `Op::CallArgs` (`deep1`), `Op::CallExpr` (`deep2`) and
`Op::Call` (`deep3`), `SIGNAL ON SYNTAX` in main, the depth reached kept in `.local`. Oracle: some
error at a depth of its own (not predicted; the crate is not expected to match it). **Head: the
same condition and depth as base on all three**, and no signal death; a difference in depth or a
rc 134/139 at head only would be the regression.
- `ar2`, checking the `ar1` entry's mechanism sentence ("install_routine accepts only a Routine
  whose record names a Rexx body") against `install_routine` (`dispatch/package.rs:837-844`, which
  accepts any record with a `routine` directive): `~addRoutine` of a `::ROUTINE sq EXTERNAL
  "LIBRARY rxmath RxCalcSqrt"` directive's object. Oracle `added 4`. Head: `added 4` (the record
  names a directive, not a Rexx body), which would make that sentence false. Base the same.
- `x14n`, the extent of the `x14s` entry ("A CALL INSIDE A METHOD'S ARGUMENT LIST"): a `::routine
  foo` inside a function's argument list (`id(foo())`, evaluated through `eval_call` with no site,
  per `compile.rs:1415`'s `native_shape(expr, None)`), inside a `CALL`'s argument list, and in a
  concatenation, with `addRoutine('FOO')` after pass 1. Oracle `old` on every line. Head and base:
  `2 new` on the two argument-list lines; the concatenation not predicted.
- `idw`: the committed identity witness as a probe; `idweq`: the same with its `distinct objects`
  control written with `=`. Predicted: `idw` SAME on three descriptors (`loadExternalRoutine 1 1 4`,
  `distinct objects 0`), base `loadExternalRoutine 0 0 4`. `idweq`: oracle `distinct objects 1`
  (15-digit addresses at `DIGITS 9`, as `gc2` measured), head `0`, so the control line is what goes
  red if the comparison is ever numeric again.
- `idrexx`: the boundary `668c73827` draws ("for a routine outside the REXX package"):
  `loadExternalRoutine` of `LIBRARY REXX FILESPEC` twice. Oracle: not predicted with confidence
  (`PackageManager::loadRoutine` goes through `loadLibrary('REXX')`, so `1` if the internal package
  resolves as a `LibraryPackage` does). Head and base: `0` (`native_load_external`'s `rexx` arm
  builds a fresh object, unchanged by the round).

Ran:
- `k13`, `q1`: **as predicted**. `k13` head SAME (`dynamic 2 added 2`, `method arg 2 added 2`), base
  `2 ext 2`/`2 ext2 2`. `q1`: oracle `fn late` / `call late`; head and base `fn raised SYNTAX 43` /
  `call raised SYNTAX 43`, rc 0. The namespace-qualified call still resolves before its arguments,
  as the report says it left it; that is the C-1 class in the one call form the round did not move,
  and no exclusions entry records it (Minor below).
- `k14`: SAME on all three (`sub 1 ext 1` / `raised 43.1 Could not find routine "EXT".`).
- `k15`, `k16`: **as predicted**, `k15` head SAME (`2 4`), base `2 file 16`; `k16` SAME on all three.
- `deep1`-`deep3`: head and base both `stopped 11.1 9999` through every op, rc 0, no signal death;
  oracle `20761`, `20762`, `27304` (the crate's activation limit, the same before and after).
- `ar2`: oracle, head and base `added 4`. **So the `ar1` entry's "install_routine accepts only a
  Routine whose record names a Rexx body" is false**: it accepts a `::ROUTINE ... EXTERNAL
  "LIBRARY ..."` directive's object, whose body is a library routine; what it refuses is a record
  with no `routine` directive (`dispatch/package.rs:837-844`), which the shared library object is.
- `x14n`: **as predicted**, oracle `old` on all six lines; head and base `fn arg 2 new` and `call arg
  2 new`, the concatenation `aold` on all three. So the `x14s` entry's heading ("A CALL INSIDE A
  METHOD'S ARGUMENT LIST") understates it: any call nested in another call's argument list, function
  or `CALL`, re-resolves on every pass.
- `idw`, `idweq`: **as predicted**. The witness SAME on head, base `loadExternalRoutine 0 0 4`; with
  `=` the oracle prints `distinct objects 1` and head `0`, so the control line is live.
- `idrexx`: oracle `rexx loadExternalRoutine 1 b.c`; head and base `0 b.c`. A third route to a
  library routine object (the `REXX` package) answers a fresh object per send, pre-existing, and
  neither extended nor recorded by the round (Minor below).
- Exclusions entries (`p/ex/`, the implementer's `fix3/ex` programs copied): `rw3`, `j2`, `j3`,
  `c3`, `b47`, `x14s`, `t6`, `t6b`, `t6c`, `x12`, `x5`, `a9`, `a9m`, `ns1`, `nf2`, `ar1`, `id3`,
  `i2a` all re-run on three sides. **Every transcript the block quotes reproduces** at head, and
  base (`9ff11e465`) is identical to head on each. `j2`/`j3` quote the oracle's stderr with its
  `5 *-* say d~pos()` and `Error 88 running ...` lines elided, where the other entries keep them.
- Sweep (`sweep/summary.txt`): every `t2-rereview-b/p/` and `t2-rereview2/p/` probe with a
  `main.rex` (107, `sig` excluded) on three sides. Head SAME and base DIFF on `o1`, `o2`, `o2r`, `o3`,
  `o3a`, `o3c`; head SAME and base SAME on 76; DIFF on both on 25, each a recorded divergence or a
  raw-`identityHash` printer (`gc2`-`gc4`, `ih`) or `d2`'s own Phase 5 `~items` line. **No probe is
  DIFF at head and SAME at base.**
- Memory test control (`memtest.sh`, `head/` archive, own target): mutant `exit 101`, `20 passed; 1
  failed`, the panic at `library_routine_memory.rs:38` naming
  `/home/moritz/dev/repos/ooRexx/build/lib/librxmath_absent.so`, no `memory allocation` text;
  restored file `cmp`-equal to the worktree's, `21 passed` (`imported_routines_... ok`, 1.37 s).
  **Confirmed as predicted.** `refusal-sites.tsv` across the round: 19 rows differ, column 4 alone
  (`python3` over `git show`), as both messages say.

### Cost (risk 3)

`bench/`: callgrind `Ir`, `target-base` (`9ff11e465` archive) and `target-head` (`cb13f0033`
archive), both built in this review, two rounds, 8 lanes (instruction counts do not depend on
concurrency; rounds agree within 0.3% per program), every rc 0. Per iteration is (200,000 - 100,000
iterations) / 100,000. `A` shapes' arguments compile (`Op::CallArgs`/`Op::CallNamed`, unchanged);
`E` shapes have a call as the argument (`Op::CallExpr`/`Op::Call` outside, `eval_call` inside, the
paths the round changed).

| shape | base r1 | head r1 | base r2 | head r2 | head vs base |
|---|---|---|---|---|---|
| `x = length(i)` | 1001.7 | 998.0 | 1002.0 | 998.1 | -0.4% |
| `x = length(length(i))` | 1825.1 | 1846.2 | 1825.0 | 1846.3 | **+1.2%** |
| `x = f(i)`, label | 3606.8 | 3586.3 | 3606.8 | 3605.1 | -0.6%, -0.05% |
| `x = f(f(i))` | 6366.0 | 6413.5 | 6376.1 | 6416.7 | **+0.7%** |
| `call f i` | 3791.0 | 3767.1 | 3792.3 | 3767.1 | -0.6% |
| `call f f(i)` | 6584.0 | 6645.2 | 6585.9 | 6650.7 | **+1.0%** |
| `x = filespec('N', i)` | 1711.0 | 1707.9 | 1711.0 | 1707.9 | -0.2% |
| `x = filespec('N', length(i))` | 2597.2 | 2656.9 | 2597.0 | 2657.0 | **+2.3%** |
| `x = r(i)`, `::routine` | 3479.0 | 3481.2 | 3478.7 | 3465.8 | +0.1%, -0.4% |
| `x = r(r(i))` | 7367.0 | 7514.2 | 7380.9 | 7472.1 | **+2.0%, +1.2%** |
| `rexxcps` total | 21,269,630,581 | 21,141,787,215 | 21,264,384,575 | 21,142,367,780 | -0.60%, -0.57% |

Attributed with `callgrind_annotate --inclusive=no` over the 200,000 runs, base against head, self
cost: `x = length(length(i))` +4.40M in `run_ops_from::<true>` (the `Op::CallExpr` arm, about +22
per call), `resolve_call` 5.40M replaced by `resolve_fixed_call` 5.00M (the inner call).
`filespec('N', length(i))` the same plus **`invoke_call` +6.60M (+33 per call)**. `f(f(i))` and
`call f f(i)`: `invoke_call` and `invoke_call'2` **+4.20M each (+21 per call, outer and inner)**,
`run_ops_from::<true>'2` -1.6M; `call f f(i)` shows `site_resolution_before_arguments` at 5.60M
(+28 per call) out of line. So every call that evaluates its arguments in `invoke_call` pays about
21 to 33 instructions more, and each `Op::CallExpr` about 22 more; the direct shapes are flat or
cheaper. `rexxcps`'s -0.6% is `run_ops_from::<true>` -51.9M and `::<false>` -10.6M in self cost
spread over every op, with `resolve_call` -16.8M against `resolve_fixed_call` +15.7M: no function
the round added accounts for it (that it is the driver's codegen is inferred). The report's comparison measured external-file calls
alone, at about 664,000 instructions each, where a 20-60 instruction change is under 0.01% and
cannot show.
- `xb1`-`xb4`, the excluded-builtin kind, **crate sides only** (`RXFUNCQUERY` reaches the rxapi
  daemon on the oracle, so the oracle is not run): a raising argument through `Op::CallArgs`,
  `Op::CallExpr`, `Op::Call` and a method argument. Loud either way, so only head against base
  matters: predicted identical, the refusal (rc 120) where the site refuses before the arguments
  (`Op::CallExpr`, unchanged `break 'cold`) and `raised 42.3` where the arguments run first.
  Ran: **as predicted**, head `cmp`-equal to base on all three descriptors in all four (`raised
  42.3`; `xb2` rc 120 `routine "RXFUNCQUERY" is not implemented (Phase 10)`).

Readers of `phase-4-exclusions.txt` (risk 6), read: `licensed_divergences.rs:200-224` collects
`LICENSED DIVERGENCE WITNESS:` lines and asserts set equality with its table; `builtin_status.rs:
590-606` asserts a `KNOWN GAP: <name>` exists for each divergent status row; `coverage.rs`,
`owners.rs`, `bif_assertions.rs`, `keyword_assertions.rs` and `rexx-inventory` restate the builtin
set and owner vocabulary as literals and parse no prose. The new block carries neither marker
(`git diff 9ff11e465 cb13f0033 -- docs/.../phase-4-exclusions.txt | /bin/grep -a -c "WITNESS\|KNOWN
GAP\|DEVIATION"` 0), so every reader's expectation holds by construction, and none of them polices
the block either. Owner lines, read against the plan and the entries they point at: the `Routine~new`
pointer lands on "OWNER: none" (`:4736-4745`); the traceback family exists (`:4747-4760`); the L0
entry records no owner (`:4794-4803`); Task 3 fills every `REXX_VALUE_*` row
`processArguments` handles (plan `:211-225`), so the Unfilled owner is coherent; no surface task
names `~addRoutine`, `setMethod` or namespaces (`/bin/grep -a -n -i` over the plan, nothing).

## Not done

No suite, gate or Miri run (the brief's rule); the implementer's gate table is unverified here. No
mutant of the round's resolution code (the report's C-R3a/C-R3b/C-R3c are unverified here); the
only mutant is the memory test's library name. The M-root entry's claim was not re-measured at
head (no collect-on-every-allocation harness was rebuilt). No wall-clock measurement.

---

## Finding Verdicts (✅/❌/⚠️, file:line, check run)

- ✅ **C-1** (a kept merged library routine resolved before its own arguments merged a shadowing
  routine). `run.rs:3942-4021` (`invoke_call` evaluates, then `settle_after_arguments`),
  `ir/drive.rs:2438-2468` (`site_resolution_before_arguments`), `eval.rs:530-532`, `run.rs:4689-4691`,
  `run.rs:4553-4561`. Oracle order printed for the function, `CALL`, dynamic `CALL`, qualified and
  `CALL ON` forms: only `externalCall`'s search can differ across the arguments. Ran: the second
  re-review's `o1`, `o2`, `o2r`, `o3`, `o3a`, `o3c` head SAME, base DIFF; `k1`, `k2`, `k3`, `k5`, `k6`
  (recursion), `k13` (dynamic `CALL`, method argument), `k15` (library loaded by the argument), `i1`
  (`INTERPRET`), `b1` head SAME, base DIFF on exactly the pass the arguments changed; `e1` (error
  precedence, one evaluation per call, `NOVALUE`/`SYNTAX` traps, every call shape), `e2`, `l1`
  (labels after the site, a label shadowing a builtin, quoted and dynamic forms), `t1`-`t3` (trace
  `i` and `r`, every kind, head `cmp`-equal to base), `k8`-`k11`, `k14`, `k16`, `i2`, `deep1`-`deep3`,
  `xb1`-`xb4` (excluded builtin, crate sides) head identical to base. 107 earlier review probes: no
  probe DIFF at head and SAME at base.
- ✅ **m-1** (identity by `=`). `corpus/lang/library_routine_object_identity.rex:10-18` compares hash
  strings with `==`. Ran `idw` SAME; `idweq` (the control line with `=`) oracle `distinct objects 1`,
  head `0`, so the control reddens a numeric comparison.
- ⚠️ **m-2** (no running gate sees the root). Recorded, not fixed, as ruled
  (`phase-4-exclusions.txt:4974-4982`). The entry's measurement was taken at `9ff11e465`, before
  the root moved into `library_routine_object`, while the block's header (`:4817-4823`) says every
  entry without an earlier-revision note was measured at `273e8f608` (Minor n-2c). Not re-measured.
- ✅ **m-3** (four prose places). `run.rs:185-198`, `ir/drive.rs:2428-2437`, `corpus/phase-8.txt:174-177`
  and `run.rs:3876-3885` now say what the code does; `new_file_executable` (`lib.rs:4583-4589`)
  records the context or the caller for both `Routine~newFile` and `Method~newFile` (read). But unchanged
  neighbours are made false by the round (Minor n-1).
- ⚠️ **m-4** (`loadExternalRoutine` and the directive route). Built for the library route
  (`dispatch.rs:8428-8446`, `environment.rs:2036-2050`): `idw`, `L1`, `L2` head SAME, base DIFF, and
  the retroactive `~package` rule holds for the shared object whichever route made it first (`before
  nil` / `after requires nil` / `after directive pk.cls` on all three sides). The directive route is
  recorded (`:4951-4961`; `id3` reproduces). The `REXX`-package route is neither (`idrexx`: oracle
  `1`, head `0`; Minor n-3).
- ✅ **m-5** (memory test needs the checkout). `tests/library_routine_memory.rs:36-44` calls
  `locate()` and asserts `librxmath.so` with a message naming the path; `tests/support/oracle.rs:153`
  adds `lib_dir`. Ran the control: red at `:38`, message naming `.../lib/librxmath_absent.so`, no run
  of the binary; restored green, 1.37 s. No skip path.

## New Issues In The Fix

### Critical

None found. No silent divergence introduced by the round in any probe run (listed above).

### Important

**I-1. Every call that evaluates its arguments in `invoke_call` got slower, and the report's cost
comparison could not see it** (ran, callgrind, archives of both revisions). Per loop iteration, head
against base: `x = length(length(i))` +1.2% (+21 instructions), `x = f(f(i))` +0.7% (+40 to +47),
`call f f(i)` +1.0% (+61 to +65), `x = filespec('N', length(i))` +2.3% (+60), `x = r(r(i))` +1.2% to
+2.0% (+91 to +147). The forms whose arguments compile are flat or cheaper (`length(i)` -0.4%, `f(i)`
-0.05% to -0.6%, `call f i` -0.6%, `filespec('N', i)` -0.2%, `r(i)` +0.1% to -0.4%). Self cost
attributes it to the round's code: `invoke_call` +21 per label or routine call and +33 per
internal-package routine call (why is read, not measured: the settle after the loop, and for the
internal path the early return it lost), the `Op::CallExpr` arm +22 in `run_ops_from::<true>`, and
`site_resolution_before_arguments` out of line at 28 per `Op::Call`. `rexxcps` is -0.60%/-0.57%,
matching the report, but that is `run_ops_from`'s self cost across every op (-62.5M), not a
function the round added. The report measured external-file calls only (about 664,000 instructions
each), where this cost is below 0.01%. A call as an argument (`length(strip(x))`,
`substr(s, pos(t, s))`) is the path every nested call takes, including the inner call itself, which
runs through `eval_call`. Cost, not correctness: fix it (for instance keep the settled kinds off the
after-arguments path in `invoke_call`, and restore the internal and library-routine early return
after the loop), or record these figures in the report and the perf ledger as accepted.

### Minor

**n-1. Neighbours the round falsified, and a re-committed dangling pointer** (read).
- `run.rs:118-119`: `Resolved` is "decided in one place before any argument is evaluated". Routine
  kinds are now decided after the arguments, by `resolve_routine_call` or `settle_after_arguments`.
- `ir/drive.rs:1497-1503`: `Op::Call` runs "through the same `Interp::resolve_call` and
  `Interp::invoke_named_call` that `step`'s own `Call` arm reaches". Neither calls `resolve_call`
  now; its only callers are `run_call_args` and `run_call_named` (`drive.rs:267`, `:311`).
- `ir/drive.rs:666-672`: "The one thing this does that `EvalExpr` does not is skip `resolve_call`."
  `eval_call` no longer calls it, and a kept Step 3 answer is looked up again after the arguments
  on this op too.
- `ir/drive.rs:1522-1528`, rewritten by `c55caa62f`: "`Interp::resolved_after_arguments` has the
  citation." Its doc (`run.rs:4636`) has none. `eval.rs:528-529`'s "has the C++ citation and the two
  measurements" is the same pointer, carried unchanged.

**n-2. The exclusions block** (ran `ar2`, `x14n`; read).
- (a) `:4948`, "install_routine accepts only a Routine whose record names a Rexx body", is false.
  `ar2`: `~addRoutine` of a `::ROUTINE sq EXTERNAL "LIBRARY rxmath RxCalcSqrt"` directive's object is
  `added 4` on all three sides. `install_routine` (`dispatch/package.rs:837-844`) accepts any record
  naming a `::ROUTINE` directive and refuses one with none, which the shared library object is.
- (b) `:4875`, "A CALL INSIDE A METHOD'S ARGUMENT LIST", understates the extent. `x14n`: `id(foo())`
  and `call id foo()` answer `2 new` on head and base against the oracle's `2 old`. Any call nested
  in another call's argument list is resolved again on every pass.
- (c) `:4822`, "the others were measured on this revision alone", is false for the root entry
  (`:4974`), whose measurement is the second re-review's at `9ff11e465`.
- (d) `:4842-4852` (`j2`/`j3`) quote the oracle's stderr without its `5 *-* say d~pos()` and
  `Error 88 running ... line 5:  Invalid argument.` lines, where the neighbouring entries keep theirs.
- (e) `3f48151ce`'s message says every divergence the task and its rounds recorded "now has an
  entry". The task report's own Concern 6 (`call .context~package~loadPackage(...)` rendering
  `rexx-exec: 35.1` rc 120 against `Error 35` rc 221; `.K~define` of a source-built `Method` refusing
  naming Phase 5), fix round 2's `u1` (`StringTable~hasIndex` refusing naming Phase 5) and Concern 3's
  unmeasured name collisions have none (`/bin/grep -a -n -i` over the file for `35.1`, `~define`,
  `hasIndex`, `collision`: the only `hasIndex` hit is `.local`'s L2 entry).
- Every quoted transcript itself reproduces (18 programs re-run on three sides; base identical to
  head on each), and the readers' expectations hold (the block adds no parsed marker).

**n-3. Unrecorded remainders of the round's own classes** (ran; both pre-existing at `9ff11e465`,
rc 0 when trapped).
- `q1`: a namespace-qualified call is still resolved before its arguments. `NS:late(add('LATE'))`,
  where `add` does `addPublicRoutine('LATE')` on the namespace package: oracle `fn late` / `call late`,
  head and base `raised SYNTAX 43` twice. The report says it left this form alone; nothing tracked
  records it.
- `idrexx`: `loadExternalRoutine` of `LIBRARY REXX FILESPEC` twice: oracle `1`, head and base `0`.
  `668c73827` draws its boundary at the `REXX` package, and the exclusions block records the
  directive route only.

## Out-of-Scope Observations

- **A `::routine` whose call raised stays kept** (`k12`): an internal routine `sub:` calls `foo()`,
  which raises 42.3 (trapped); `addRoutine('FOO')`; `sub:` again. Oracle `sub 2 added 2`, head and base
  `pass2 raised 42.3`. The oracle's `setField(externalTarget, ...)` runs after `externalCall`
  returns (`ExpressionFunction.cpp:210-214`, `CallInstruction.cpp:193-197`), so a raised call is not
  kept; the crate records the answer before the call. Not recorded anywhere.
- **`trace i`/`trace r` of a function call to an external file lacks the `>>>` line** (`t1`, `t2`):
  after `>F>   EXT => "ext 1"` the oracle prints `>>>   "ext 1"`; head and base do not, for
  `x = ext(1)` and `x = ext(id(2))`. The `CALL` forms agree. Same at `9ff11e465`; not checked
  earlier; not recorded.
- **A library routine's `Routine~call` traceback lacks a second line** (`L3`): the trapped
  `TRACEBACK` misses `*-* Compiled method "CALL" with scope "Routine".` as well as the `Compiled
  routine` line `c3`'s entry names. The untrapped report agrees.

## Assessment

**Fix round 3:** Needs another round

No correctness defect found. C-1 is fixed in every call form the round moved, and nothing I ran
moved the wrong way. That covers error precedence, evaluation counts, traps, trace output, labels,
builtins, `INTERPRET`, recursion, nested generation moves and the shared `loadExternalRoutine`
object's retroactive package. m-1 and m-5 are fixed and their controls are live. What remains:
- I-1: the changed call paths cost 21 to 33 instructions more per call (+0.7% to +2.3% on
  call-in-argument loops). The report's benchmarks were too expensive per call to show that. This is
  a decision to fix or to record, not a defect.
- Prose and record-keeping: n-1 to n-3.

If the controller accepts I-1 with its figures recorded, the rest is small enough for the controller
to close without a fresh implementer.

Cleanup: `target-base` and `target-head` under `t2-rereview3/` deleted by explicit path after this
file was written; the worktree was not written (`git status` clean).
