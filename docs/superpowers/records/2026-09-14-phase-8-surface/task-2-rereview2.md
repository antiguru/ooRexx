# Task 2 fix round 2 re-review

Fix base `bd64f3197`, head `9ff11e465`. Findings under verification: integration C1, I-A, I-B, m1-m4
(`task-2-rereview-integration.md`) and boundary I-A, M-A-M-D (`task-2-rereview-boundary.md`), against
the ruling in `progress.md` "Task 2 fix round 2: sent". Written first and appended as the review goes.
Scratch: `scratchpad/t2-rereview2/`, a fresh directory per run, three descriptors. Read
`corpus/oracle-crashes.txt` first; nothing below is one of its entries (no `loadExternal*` object is
used as a context anywhere). Nothing registered with rxapi. Read-only on the worktree.

## Log

(appended as it happens)
- Setup. Worktree clean at `9ff11e465`; `cargo build --release --bin rexx-run` "Finished" with no
  compile line, `sha256` `d5d10646...`, the implementer's recorded final-tree binary. `git archive
  bd64f3197 rust api` into `t2-rereview2/base` and `9ff11e465` into `head`, the read-only
  `interpreter/` linked beside each (the build scripts read `CoreClasses.orx` from it); base built
  release in `target-base`. `ab.sh` runs oracle (`o`), head worktree binary (`h`), base (`b`) from
  the same fresh run path, `ulimit -v 1048576` on each, three descriptors.
- Printed, oracle: `ExpressionFunction.cpp:179-215` and `CallInstruction.cpp:159-198` (arguments
  first; `externalTarget` if set, else label, else builtin, else `externalCall(resolvedTarget, ...)`
  then `setField(externalTarget, resolvedTarget)`); `RexxActivation.cpp:3062-3105` (`routine =
  settings.parentCode->findRoutine(target)` at Step 2 is the only write to the out-parameter; Steps
  2a, 2b, 3, 4 leave the null); `RexxCode.hpp:97` (`findRoutine` is `package->findRoutine`);
  `PackageClass.cpp:822-911` (`findLocalRoutine` walks `routines` up `parentPackage`, then
  `findPublicRoutine` walks `publicRoutines`, `mergedPublicRoutines` up `parentPackage`);
  `:1432-1451` `addInstalledRoutine` (both tables with `setEntry`); `DirectiveParser.cpp:2690-2770`
  (every `::ROUTINE` form writes `routines` and, when public, `publicRoutines` too, so public is a
  subset of `routines` at install); `:693-772` `mergeRequired`/`mergeLibrary`; `:1377-1390`
  `addPackage` (returns before merging a package already in `loadedPackages`);
  `ExpressionQualifiedFunction.cpp:148-190`, `CallInstruction.cpp:429-470` (a namespace call runs
  `findNamespace` and `findPublicRoutine` on every execution, nothing kept);
  `ExternalFunctions.cpp:104-135` (Step 3: macro space, `callNativeRoutine`, `callExternalRexx`,
  macro space). **The ruled rule is the oracle's**: kept for good exactly when Step 2 answered.
- Printed, oracle, for risk 3: `LibraryPackage.cpp:256-299` (`loadRoutines` builds one
  `RoutineClass` per table row, `routines` under the row's spelling and `publicRoutines` upper-cased,
  both the same object) and `:410-432` (`resolveRoutine`: exact then caseless, answering that same
  object); `RoutineClass.cpp:501-535` `loadExternalRoutine` -> `PackageManager::loadRoutine`
  (`PackageManager.cpp:400-412`) -> `resolveRoutine`. So `loadExternalRoutine`, a `::ROUTINE ...
  EXTERNAL "LIBRARY lib entry"` and `mergeLibrary` all hand out the **same** object per row.
- Printed, oracle: `BaseExecutable.cpp:250-360`: `Routine~new`/`newFile` **with no context** take
  the calling Rexx frame's package as the source context.
- Crate, read: `CallSite::get` (`ir.rs:458-465`) is the only reader that drops; the four readers
  (`ir/drive.rs:255`, `:304`, `:705`, `:1546`) all pass `self.routine_generation`. Chunks are cached
  per `(BodyKey, ChunkTrace)` (`plan.rs:686-706`); an `INTERPRET` fragment compiles fresh per run
  (`run.rs:7531`), as the oracle translates per run. `namespace_routine` (`lib.rs:3527-3553`) is
  read at each execution (`eval.rs:484`, `run.rs:1361`), so no site keeps a namespace call.
- Reran the reviewer's probes on `o h b`: `x1 x2 x3 x3c x4 p1 p3 x5c x5d r5 m1b` head SAME, base
  DIFF; `x4b p2 r1 x9 x13 x14 q1 n1b x8` SAME on both (`p/*.ab.txt`).

### Attack probes on the kept kinds, predictions written before running

Oracle rule predicted throughout: a site's second pass answers what its first pass found when that
was a Step 2 hit, and a fresh site answers the current lookup. Head predicted SAME unless named.
- `a1` kept `Routine` from the running package, then `addRoutine('FOO')` and a `loadPackage` of
  `pub.cls` exporting `foo`, then a fresh site, then `newFile('r.rex', main)` calling `foo`: `1 main`
  / `2 main` / `fresh added` / `r added`.
- `a2` kept `Routine` found in the parent (`r.rex` newFile'd with main), then
  `mainpkg~addRoutine('FOO')` into the parent; then a second loop whose first pass finds the
  parent's new entry and whose `.context~package~addRoutine` writes `r.rex`'s own: `1 main` / `2
  main` / `fresh parentadded` / `own 1 parentadded` / `own 2 parentadded` / `fresh2 ownadded` / `main
  parentadded`.
- `a3` kept `Routine` from `pub.cls`'s merged public routine, then `addRoutine`: `1 pub` / `2 pub` /
  `fresh added`.
- `a4` kept merged library routine, then `loadPackage` merging a Rexx `RxCalcSqrt` (add-if-absent,
  no change), then a second loop through `addPublicRoutine`: `1 4` / `2 4` / `fresh 4` / `b 1 5` /
  `b 2 5` / `fresh2 added 36`.
- `a5` kept merged library routine reached through `mid.cls`'s merged table, then `addRoutine`: `1 4`
  / `2 4` / `fresh added`.
- `a6` kept merged library routine of the parent's merged table, then `addRoutine` into the parent:
  `1 4` / `2 4` / `fresh parentadded`.
- `a8` `interpret 'say i RxCalcSqrt(16)'` in a loop across `addRoutine`: `1 4` / `2 added` (a fresh
  translation per run). `a8b`, the loop inside one `INTERPRET`: `1 4` / `2 4` / `fresh added`.
- `a9` (`Routine`) and `a9m` (merged library routine): one site inside an internal routine `sub:`,
  called twice, the second call under `trace r`, `addRoutine` between: oracle stdout `1 main` / `2
  main` (and `1 4` / `2 4`). Head: **not predicted with confidence**; if the traced call runs a
  second chunk of the same body, its empty site table answers `2 added`. Base the same as head for
  `a9`; `2 added` for `a9m`.
- `nf1` `.Routine~newFile('r.rex')` with no context, `r.rex` calling main's private `mainr`, then
  `r~package~findRoutine('MAINR')`; `nf2` the same through `.Routine~new('x', 'return mainr()')`:
  oracle `r mainr` / `find a Routine` and `new mainr` / `find a Routine`. Head: not predicted (it
  turns on whether the crate records a parent for a context-less `new`/`newFile`).

Ran (`p/a*.ab.txt`, `p/nf*.ab.txt`):
- `a1`, `a2`, `a3`, `a4`, `a5`, `a6`, `a8`, `a8b`: **every prediction confirmed**, head SAME on three
  descriptors, rc 0. Base SAME on these too (none of them reaches a kept kind the round changed
  across a generation move that base mishandled), so they are not witnesses of the change, only of
  the rule holding in the shapes the brief named: a kept `Routine` from the running package, a
  merged table and a parent, and a kept merged library routine from the running package, a
  transitive merge and a parent, each past `~addRoutine`/`~addPublicRoutine`, a same-name
  `loadPackage` merge, a write into the parent, and a later `newFile` context.
- `a9`, `a9m`: **the uncertain half came out as the divergent guess**. Oracle `1 main` / `2 main`
  and `1 4` / `2 4`; **head `2 added` on both**, stdout and the `>>>` trace line differing, rc 0;
  base the same as head on both. The site the oracle keeps at the instruction is kept here per
  chunk, and a body has a chunk per `ChunkTrace` (`plan.rs:686`), so the traced second call of
  `sub:` resolves again in an empty table. Pre-existing for both kinds (base answers the same), so
  not this round's regression; recorded under out-of-scope below, since the round's rule reads "for
  good" and this is the one shape found where a kept `Routine` is not.
- `nf1`: oracle and head `r mainr` / `find a Routine`; base `find The NIL object` (base's
  `findRoutine` walked no parent). Fixed by `c08f4badd`.
- `nf2`: oracle `new mainr` / `find a Routine` rc 0; **head and base rc 213**, 43.1 `Could not find
  routine "MAINR"`, stdout empty. `.Routine~new(name, source)` with no context gets no parent here,
  where `newFile` with no context does (`lib.rs:4583-4589`) and the oracle gives both the calling
  frame's package (`BaseExecutable.cpp:252-260`). Pre-existing, out of scope.

Writes table, checked against the code (`/bin/grep -rn` over `rexx-exec/src` for `routines`,
`merged_public_routines`, `package_public_routines`, `package_routines`, `package_routine_codes`,
`package_parents`, tests excluded; each hit read): writers of `routines` are `install_directives`
(`lib.rs:3048`) and `install_routine` (`dispatch/package.rs:853`); of `package_public_routines` the
same two (`lib.rs:3051`, `package.rs:859`); of `merged_public_routines` only `merge_routines`
(`lib.rs:4808`, reached from `merge_required` `:3442` and `merge_library` `:4789`); of
`package_routines`/`package_routine_codes` only `register_package_routines` (`:4775-4781`); of
`package_parents` `package_from_source` (`:3744`) and `new_file_executable` (`:4588`). Nothing
removes from any of them. `install_directives` extends `routines` (`:3048`) before
`install_requires` runs any prologue (`:3213`), so the "no code of that package has run" argument
holds for the prologues too. `resolve_call`'s other inputs are static (`internal_routines::lookup`,
`library_bootstrap`) or searched again on entry (`External`, `x8` SAME). **The table is complete and
its generation column is right**; the one write it lists as not a lookup (`.ROUTINES` from Rexx) is
the pre-existing `w1` defect below.

### Lookup walk (risk 2) and identity (risk 3), predictions written before running

- `a9c`, `a9` with no `TRACE`: head and base SAME, `2 main` (the control for `a9`'s reading).
- `d2`, two levels of context: `r1.rex` newFile'd with main, `r2.rex` newFile'd with `r1`, and a
  `.Package~new('lvl2', source, r1)`. main declares `mainr` and `pubr` and requires rxmath LIBRARY
  and `q.cls` (public `frommerged` answering `q`); `r1` declares `rxcalcsqrt` and requires `pub.cls`
  (public `pubr` answering `pub`, `frommerged` answering `pubm`). Oracle: `r2 mainr r1 16 main pubm`,
  `r2 find mainr r1 16 main pubm`, `r2 findPublic mainr`, `r2 tables 0 0 0`, `p2 mainr r1 16 main
  pubm`, `p2 find main pubm r1 16`, `p2 tables 0 0 0`. Head SAME. Base DIFF (`pub` for `pubr`, and
  `findRoutine` through no parent raising on `.nil~call`).
- `ns1`, a namespace call into a context-built package `p` (parent main) added as `NS`: oracle `own
  own`, `findPublicRoutine a Routine`, `ns parent public mainpub`, `ns parent merged frompub` (the
  oracle's `findPublicRoutine` walks `p`'s parent's public and merged tables), `ns parent private
  raised SYNTAX 43`. Head: `findPublicRoutine a Routine`, and **the two parent lines raising
  `SYNTAX 43`** (`namespace_routine` reads the namespace package's own two tables alone); base the
  same but `findPublicRoutine The NIL object`. If so, pre-existing and not this round's walk.
- `id1`, one library routine object asked for every way: oracle `1` on every comparison
  (`findRoutine` twice, `importedRoutines`, `loadExternalRoutine` against `findRoutine` and against
  itself spelled in lower case, `findRoutine` in a second importing package, the `::ROUTINE sq
  EXTERNAL "LIBRARY rxmath RxCalcSqrt"` directive's object) and `calls 4 5 6`. Head: `1`, `1`,
  **`0 0`**, `1`, **`0`**, `calls 4 5 6` (`loadExternalRoutine` builds a fresh object per send,
  `dispatch.rs:8430-8438`, and a directive-bound routine is its own `InstalledRoutine`). Base `0` on
  every library comparison.

Ran:
- `a9c`: **confirmed**, `2 main` on all three. So `a9`'s `2 added` needs the second chunk.
- `d2`: head matched the oracle up to my own `StringTable~items` line (rc 120, Phase 5, loud). `d2b`,
  the same without the `~items` lines: **head SAME** on all three descriptors (`r2 mainr r1 16 main
  pubm`, `r2 find mainr r1 16 main pubm`, `r2 findPublic mainr`, `p2 mainr r1 16 main pubm`, `p2
  find main pubm r1 16`); base `r2 mainr r1 16 pub pubm` then rc 159, 97.1 on `.nil~call`. The walk
  is the oracle's two levels deep, through `newFile` and through `.Package~new`, for a call and for
  `findRoutine`/`findPublicRoutine`.
- `ns1`: **as predicted**, oracle `ns parent public mainpub` / `ns parent merged frompub`, head and
  base `raised SYNTAX 43` on both; head's `findPublicRoutine a Routine` fixed against base's `The NIL
  object`. The namespace call's lookup (`lib.rs:3527-3553`) walks no parent, where the oracle's
  `findPublicRoutine` does. Pre-existing (base the same), untouched by `c08f4badd`; out of scope.
- `id1`: **as predicted**, oracle `1` on every line; head `1` for `findRoutine` twice,
  `importedRoutines` and the second importing package, **`0 0` for `loadExternalRoutine`** and
  **`0` for the directive**; base `0` on every library line. `a15a8603a` made the merged-table
  object one per row; the oracle's object is also the one `loadExternalRoutine` and an `EXTERNAL
  "LIBRARY ..."` directive answer. Pre-existing for those two, rc 0; the ledger's "closes the
  identity concern" is wider than what was built (Minor below).

### `tests/library_routine_memory.rs` (risk 3), predictions written before running

- Read: the test writes `main.rex` under `CARGO_TARGET_TMPDIR`, runs `CARGO_BIN_EXE_rexx-run` through
  `bash -c 'ulimit -v 1048576 && exec "$0" main.rex'` with `LD_LIBRARY_PATH` set to
  `oracle_root().join("lib")` (`tests/support/oracle.rs:57-59`, the hard-coded oracle checkout), and
  asserts `(Some(0), "100000\n..400000\ndone\n", "")`. It is not behind `REXX_CORPUS_GATE`; the only
  other test file that names `oracle_root()` ungated is `ir_recorded.rs`, and there only to test
  whether a library exists (`:1069-1072`). So it needs the oracle checkout's `librxmath.so` on every
  plain `cargo test`, and a machine without that checkout reddens it with rc 43.1-style output
  rather than skipping.
- The test file copied into the `bd64f3197` archive, built `--release` there: predicted red on the
  assertion with `None` for the status (a signal, since `exec` hands the abort to `Command`), stdout
  empty, and stderr `memory allocation of N bytes failed`. That is the reason its doc states.

Ran: the base test binary, **red exactly as predicted**: `left: (None, "", "memory allocation of
545259536 bytes failed\n...")` against the expected triple, `20 passed; 1 failed` (the other 20 are
`tests/support`'s own unit tests, which every binary that says `mod support;` re-runs), 2.19 s wall.

### Rooting under collect-on-every-allocation (risk 3), predictions written before running

Harness `t2-rereview2/harness` (debug, own `target-harness`, links `head/`'s `rexx-exec` and calls
`run_program_collect_every_alloc` with `LD_LIBRARY_PATH` in the invocation's environment), each
program from a fresh copy. `gc1`: take `findRoutine('RXCALCSQRT')~identityHash` and drop the object,
allocate and send `importedRoutines` fifty times, then compare the hash against a fresh
`findRoutine` and the last table's entry, and call both (oracle `same 1 1` / `call 4 5`).
- Head: `gc1`, the four new witnesses, `id1`, `d2b`, `a4`, `a6`, `p3`, `x4`, `m1b` each exit 0 with
  a non-zero collection count and stdout equal to the release binary's.
- Mutant **M-root** (in `head/` only, file backed up first): delete the
  `self.roots.add_global(&library_routine_root_key(code), object);` statement in
  `merged_routine_object`. Predicted: `gc1` red under the harness (a panic at a dead handle, or an
  answer other than `same 1 1` / `call 4 5`), and `library_routine_object_identity` **green**, since
  its first object stays in the variable `a` for every comparison. If `gc1` stays green the harness
  cannot see the rooting and the head run above proves nothing about it.

Ran, head under the harness: `gc1`, the four witnesses, `id1`, `d2b`, `a4`, `a6`, `p3`, `x4`, `m1b`
all harness exit 0, rc 0, collections 11 to 165, stderr empty, stdout `cmp`-equal to the oracle's
and the release binary's **except `id1`**, which printed `loadExternalRoutine 0 1` under the harness
against `0 0` in release. Chased before believing it (`gc2`-`gc4`, `ih`):
- **`identityHash` compared with `=` is not an identity test on either side.** Raw values (`ih`):
  the oracle's are 15-digit addresses (`-139684902648961`), so `=` at `NUMERIC DIGITS 9` calls
  distinct objects near each other equal: `gc2` oracle `object 1 1` for two `.object~new`s, and
  `loaded 1 1 1`, `new 1 1` likewise. The crate's are handle bits: small in a run with no
  collection (`1132`), but after one they carry high bits (`gc4` under the harness: `raw 756
  51539608304 51539608312`), and `=` then calls those two distinct objects equal too. So `id1`'s
  `0 1` is the comparison, not a dead handle (`gc3` under the harness: `d 760`, `e 764`, distinct
  and stable).
- The oracle's identities, re-read from raw values: `findRoutine` twice, `importedRoutines` and
  **`loadExternalRoutine` twice all print `-139684902648961`**, the same object. So `id1`'s
  conclusion stands on the raw values, not on its `=` lines.
- `gc1s`, `gc1` with `==` and `~class`: oracle and head `same 1 1` / `class The Routine class The
  Routine class` / `call 4 5`; base `same 0 0`; head under the harness the same as release, 171
  collections. The M-root prediction above is re-pointed at `gc1s` (written before the mutant was
  built): red there, green on the identity witness.

Ran, M-root (`add_global` deleted in `head/`'s `environment.rs`, harness rebuilt):
- **`gc1s` stayed green** (`same 1 1` / `class ...` / `call 4 5`, 171 collections): **that half of
  the prediction is falsified**.
- `gc1`, the same program with `=`, **red**: rc 120, stdout empty, `rexx-exec: a message send to a
  value whose object is no longer live is not implemented (Phase 5)`, 160 collections. So the
  harness sees a missing root when the freed slot has been reused before the next ask, and not
  otherwise; `gc1s`'s extra literal shifts the allocation sequence. `gc1` green at head is the
  rooting evidence.
- `library_routine_object_identity` **green** under M-root, as predicted: the object stays in `a`.
- File restored from the backup, `cmp`-equal to the worktree's, harness rebuilt; `gc1` green again.
- No committed test runs a shape that frees the object between two asks: the identity witness keeps
  it in a variable, and `collect_stress`'s L0 test, the one runner of `phase-8.txt` under this
  mode, is red at the base for its pre-existing `dispatch.rs:1510` panic. So the root is right and
  unwitnessed (Minor below).
- Correction to the last bullet, from running every `library_*`/`external_*` row of `phase-8.txt`
  (65 programs, `w8/`) under the harness at head and under M-root: **`library_routine_imported.rex`
  goes red under M-root** (rc 120, the same dead-handle refusal) and nothing else changes rc (the
  stderr hashes differ only by the run directory in the path). So a committed witness does see the
  root in this mode. But two rows **panic at head under the harness** (`dispatch.rs:1510`, `a live
  value`, the known L0 failure): `library_routine_argument_errors` (`phase-8.txt:139`) and
  `library_routine_rexx_package` (`:157`), and `on_interpreter_thread` re-raises a panic
  (`lib.rs:5879-5881`, `resume_unwind`), so `collect_stress`'s L0 loop stops at `:139` and never runs
  `library_routine_imported` (`:186`). No gate the tree runs today fails without the root.

Ran, the memory test at head and its margins (predictions: green; cost and margin not predicted):
- The head test binary (`headtest/` archive, own `target-head`, release): `1 passed`, 1.41 s wall
  twice. Base 2.19 s red (above).
- Margin: the test's program through the head release binary under decreasing `ulimit -v`:
  1048576, 900000, 800000, 750000 KiB rc 0 with every line; **700000 rc 134**, `memory allocation
  of 10 bytes failed`, stdout empty; a `say 'x'` program is rc 0 at 700000. So about 300 MiB of the
  cap is headroom. It caps address space, not RSS, so it does not depend on the machine's memory;
  it would move with the collector's working set or the interpreter thread's stack reservation.
- Debug: a debug `rexx-run` of the head tree (`target-headdbg`) on the same program under the cap:
  rc 0, every line, **11.78 s**, 128,180 kB peak RSS. So the workspace debug gate pays about 12 s
  for it.
- It depends on the oracle checkout: `LD_LIBRARY_PATH` is `oracle_root().join("lib")`, ungated.
  The in-crate library tests beside it load the worktree's own `build/lib`
  (`dispatch/library.rs:531-539`), so this is the first plain `cargo test` that needs
  `/home/moritz/dev/repos/ooRexx/build/lib/librxmath.so`.

### `.ROUTINES` from Rexx (risk 4), predictions written before running

`e7cb210d9` (the task's base) built release from its own archive in `target-task`, run as side `m`.
- `rw1`, the implementer's `w1` copied (a site keeping the external file `ext`, then
  `.routines~put(r, 'EXT')`, then a fresh site): oracle `1 ext 1` / `2 put 2` / `fresh put 3`; head,
  `bd64f3197` and `e7cb210d9` all `2 ext 2` / `fresh ext 3`.
- `rw2`, no file and no kept site: `.routines~put(r, 'FOO')`, `hasIndex`, then `foo(1)` trapped:
  oracle `items 1` / `call put 1`; head, base and `e7cb210d9` either `raised SYNTAX 43` after an
  `items` line, or a loud Phase 5 refusal at `hasIndex` (the crate's `StringTable~hasIndex`, `u1`).

Ran: `rw1` oracle `2 put 2` / `fresh put 3`; head and base `2 ext 2` / `fresh ext 3`, as predicted;
**`e7cb210d9` rc 213 on the second pass**, `43.1 Could not find routine ""` (the round-1 empty-name
defect I1, which reaches the kept site first), so `rw1` cannot show the `.ROUTINES` half at the task
base. `rw2` was malformed: with no `::ROUTINE` in the program the oracle's `.ROUTINES` is unset and
all four sides raise the same 97.1 on `PUT`. Prediction for `rw3` (a `::routine bar` so the table
exists; three separate sites, no loop): oracle `first ext 1` / `second put 2` / `foo putfoo 3`;
head, base and `e7cb210d9` `first ext 1` / `second ext 2` / `raised SYNTAX 43`.

Ran `rw3`: **as predicted on all four sides**: oracle `second put 2` / `foo putfoo 3`; head, base and
`e7cb210d9` `second ext 2` / `raised SYNTAX 43`, rc 0. **Pre-existing at the task's base.** Where it
is recorded: the report's "Recorded, not fixed (fix round 2)" and "Concerns", and the ledger's
round-2 entry. Both sit under `.superpowers/`, which `.gitignore:30` ignores (`git check-ignore -v`),
so nothing tracked records it yet: not the plan (`2026-09-14-phase-8-surface.md` has no deferred
list), not `phase-4-exclusions.txt`, not a roadmap row. The same holds for the round's other
recorded-not-fixed items (`x14s`, `o1`, `t6`, `x12`) and for `nf2`, `ns1` and `a9` here.

Prediction for `ar1`, before running (the shared object handed to `~addRoutine`): oracle `added 4`;
head and base `raised SYNTAX 88` (`install_routine` accepts only a `Routine` with an
`InstalledRoutine` record, and `record_loaded_executable` gives none).

Predictions for two merge neighbours behind `merge_routines`, before running (whether a crate table
can hold a name the oracle's `findRoutine` lacks, which the kept-for-good rule would then keep):
- `q3`: `loadPackage('pk.cls')`, `p~addPublicRoutine('NEWP')`, `addPackage(p)` a second time, then
  `newp()` trapped: oracle `raised SYNTAX 43` (`addPackage` returns before merging a package already
  in `loadedPackages`, `PackageClass.cpp:1377-1386`). Head and base: not predicted with confidence.
- `q4`: main requires `a.cls`; `a~addPublicRoutine('LATE')` afterwards; `late()` trapped; then
  `loadPackage('b.cls')` where `b.cls` requires `a.cls`, and `late()` again: oracle `raised SYNTAX
  43` (main's merged table is a copy taken at the merge), then `after b late` (b's merge from `a`
  now carries it). Head and base SAME.

Prediction, before running, for the kept-for-good rule composed with the recorded `o1` order (the
crate resolves a call before its arguments run, `ExpressionFunction.cpp:182-192` runs them first):
- `o2`: `r.rex` newFile'd with main (which requires rxmath LIBRARY) runs `say i RxCalcSqrt(step(16))`
  three times, and `step` does `loadPackage('pub.cls')` (a Rexx public `RxCalcSqrt`) on its first
  call. Oracle: the arguments run first, so Step 2 finds `r.rex`'s newly merged routine and keeps
  it: `1 pk 16` / `2 pk 16` / `3 pk 16`. **Head: `1 4` / `2 4` / `3 4`**: the site resolved to the
  parent's merged library routine before `step` ran, and that kind is now kept for good. Base: `1
  4` / `2 pk 16` / `3 pk 16` (the merge moved the generation and base dropped the kind).
- `o2r`, the same with a `::routine foo` in main and `step` doing `addRoutine('FOO')`: oracle `1
  added 16` / `2 added 16` / `3 added 16`; head and base `1 main 16` / `2 main 16` / `3 main 16`
  (`Routine` was kept for good at base too).
- `o3`, the same event reached the ordinary way: the argument is a call of an external file
  `helper.rex` that exports a public `RxCalcSqrt`, which `callExternalRexx` merges into `r.rex`'s
  package (`RexxActivation.cpp:3149`): oracle `1 helper 16` / `2 helper 16` / `3 helper 16`; head
  `1 4` / `2 4` / `3 4`; base `1 4` / `2 helper 16` / `3 helper 16`.
- Scope, not predicted: `o3c` (`call RxCalcSqrt helper(16)`) and `o3a` (`x = RxCalcSqrt(helper(16))`).

Ran:
- `ar1`: **as predicted**, oracle `added 4`; head and base `raised SYNTAX 88 88.914`. Pre-existing.
- `q3`, `q4`: SAME on all three (`raised SYNTAX 43`; then `after b late`). The crate's merge tables
  hold nothing the oracle's `findRoutine` lacks in either shape.
- `o2`: **confirmed, a regression**: oracle `1 pk 16` / `2 pk 16` / `3 pk 16`; **head `1 4` / `2 4`
  / `3 4`**; base `1 4` / `2 pk 16` / `3 pk 16`. rc 0, stderr empty on all three.
- `o2r`: as predicted, head and base `main 16` on every pass against the oracle's `added 16`
  (pre-existing for `Routine`).
- `o3`: **confirmed**, the same three answers with the event being an ordinary external-file call in
  the argument list. `o3c` (`CALL`) and `o3a` (assignment): the same, head `4` on every pass where
  the oracle and base answer `helper 16` from the second pass on.
- Citations printed in both C++ trees: `RexxActivation.cpp:3069`, `ThreadContextStubs.cpp:1948`,
  `RexxErrorCodes.h:456` (`49000`), `InterpreterInstanceStubs.cpp:106` (`:105` blank) and `:84`,
  `Version.cpp:103-106` (`getLanguageLevel` returns `REXX_CURRENT_LANGUAGE_LEVEL`),
  `LibraryPackage.cpp:279-286`, `PackageClass.cpp:897`, `NumberStringClass.cpp:704`,
  `RexxActivation.cpp:2877-2879` (`.ROUTINES`). `rexx-num/src/lib.rs:793-821` read against
  `double_of`'s new doc: exponential when `adjusted >= digits` (the integer part would be padded) or
  `exponent <= -(2 * digits + 1)` (more zeros after the point than digits), so the doc is right.
  `docs/superpowers/specs/2026-09-14-phase-8-native-api.md:107-115` says the in-crate tests open
  the worktree's `build/lib`, "a second build from identical sources that the oracle never runs",
  which the new helper doc repeats; `build/lib/liborxmethod.so` exists there and NEEDs `libc.so.6`
  alone. `refusal-sites.tsv` across `eab19ac3b` and `c08f4badd`: column 4 of 19 rows each and nothing
  else (`python3` over `git show`), as both messages say. `corpus.rs:82-100` runs the crate in
  process, as the memory test's doc says. `rexx-api`'s
  `a_refusing_entry_records_itself_and_returns` at head: 1 passed.

## Not done

No suite, gate, Miri or benchmark run (the brief's rule); the implementer's gate table and cost
figures are unverified here. No mutant beyond M-root. The `o2`/`o3` fix suggested below is not
built or run.

---

## Finding Verdicts (✅/❌/⚠️, file:line, check run)

**Integration re-review**
- ✅ **C1** (a kept external file survives `~addRoutine`). `dispatch/package.rs:850` moves the
  generation; `run.rs:179-187` keeps `External` only until it moves. Ran: `x3`, `x3c` head SAME
  (`2 added`), base `2 ext 2`; witness `library_routine_site_add_routine` head SAME under
  collect-on-every-allocation too.
- ⚠️ **I-A** (a kept merged library routine re-resolved). `run.rs:179-187` drops
  `MergedLibraryRoutine` from the set, which is the oracle's Step 2 rule (`RexxActivation.cpp:3069`
  printed). Ran: `x4`, `p3`, `a4`, `a5`, `a6` head SAME, and `a1`-`a3` for `Routine` from the running
  package, a merged table and a parent past `~addRoutine`, a same-name merge, a parent write and a
  later `newFile` context. **But keeping it for good exposed a new stale answer** when the site's
  first resolution predates a merge its own arguments perform (new Critical below).
- ✅ **I-B** (parent's routines before merged). `environment.rs:1996-2023` walks `routines` up the
  chain, then merged tables up the chain; `run.rs:3847-3851` and `environment.rs:1977-1990` both read
  it. Ran: `x5c`, `x5d`, `r5`, `p1`, `nf1` head SAME, base DIFF; `d2b` (two levels, through `newFile`
  and `.Package~new`, call and `findRoutine`/`findPublicRoutine`) head SAME, base DIFF; `r1`, `p2`,
  `q3`, `q4` SAME. The public-routines table is safely skipped: every `::ROUTINE` form and
  `addInstalledRoutine` write `routines` too (`DirectiveParser.cpp:2690-2770`,
  `PackageClass.cpp:1432-1451`; crate `lib.rs:2888-2890`, `package.rs:853-862`).
- ⚠️ **m1** (invalidation prose). The false sentences are gone; the replacements carry three
  imprecisions (Minor m-3 below).
- ✅ **m2** (`importedRoutines` leak and identity). `environment.rs:2029-2046`, `lib.rs:1653-1657`.
  Ran: the new test red at `bd64f3197` exactly as its doc says (`None`, empty stdout, `memory
  allocation of 545259536 bytes failed`), green at head in 1.41 s; `gc1`/`gc1s` head SAME, base
  `same 0 0`; rooting live under collect-on-every-allocation (M-root reddens `gc1` and
  `library_routine_imported`). The identity closed is the merged-table one only (Minor m-4).
- ✅ **m3** (`double_of` doc). `dispatch/library.rs:443-446` matches `Number::format`
  (`rexx-num/src/lib.rs:810-813`, read).
- ✅ **m4** (build directory text). `dispatch/library.rs:531-539`, `run/tests.rs` four `expect`s and two
  docs; matches the D5 amendment (`2026-09-14-phase-8-native-api.md:107-115`, printed).

**Boundary re-review**
- ✅ **I-A** (49). `layout.rs:691-693` `failing 49` with `RexxErrorCodes.h:456` cited; `ffi.rs:828`
  asserts 49; no other `DisplayCondition` pin in `rust/` (`grep`). Ran the unit test: 1 passed;
  printed `:1948` and `:456` in both trees.
- ✅ **M-A**. `ffi.rs:137` cites `:106`, which is `InterpreterInstance::interfaceVector` (printed).
- ✅ **M-B**. `load.rs:54-56` names the stub at `:84` through `Interpreter::getLanguageLevel`, which
  returns `REXX_CURRENT_LANGUAGE_LEVEL` (`Version.cpp:103-106`, printed).
- ✅ **M-C**. `invoke.rs:138-139` now covers a condition raised before or after the refused member.
- ✅ **M-D**. `load.rs:258-261` states the divergence and cites `LibraryPackage.cpp:279-286`
  (printed: `RegisteredRoutine` for classic, `NativeRoutine` otherwise).

**The writes table** (report, item 1): ✅ complete and right, checked against every writer
(`grep` over `rexx-exec/src`, each read; log above). **The `.ROUTINES` concern** (risk 4): ✅
pre-existing at `e7cb210d9` (`rw3`, all three crate revisions `second ext 2` / `raised SYNTAX 43`
against the oracle's `second put 2` / `foo putfoo 3`); ⚠️ recorded only under the ignored
`.superpowers/` tree (out of scope below).

## New Issues In The Fix

### Critical

**C-1. A merged library routine kept for good is the one resolved before a shadowing merge its own
arguments performed** (ran). `run.rs:179-187` now keeps `MergedLibraryRoutine` for good, and the site's
first resolution answers what the lookup held before the call's arguments ran (the recorded `o1`
shape: head and base both answer the pre-merge routine on the first pass). The oracle evaluates the
arguments, then calls `externalCall`, then keeps what Step 2 found (`ExpressionFunction.cpp:182-213`,
`CallInstruction.cpp:166-196`). `o3`: `r.rex`, built by `.Routine~newFile('r.rex', .context~package)`
from a main that requires rxmath LIBRARY, loops `say i RxCalcSqrt(helper(16))`, where `helper.rex`
is an external file exporting a public `RxCalcSqrt` (merged into `r.rex` by the call, as
`callExternalRexx` does): **oracle `1 helper 16` / `2 helper 16` / `3 helper 16`; head `1 4` / `2 4`
/ `3 4`; base `1 4` / `2 helper 16` / `3 helper 16`.** rc 0, stderr empty. The same through `CALL`
(`o3c`), an assignment (`o3a`), and `loadPackage` inside the argument (`o2`). Every pass after the
first was right at `bd64f3197` and is stale at head, which is the brief's definition of Critical.
The `Routine` analogue (`o2r`) was already stale at base. **Fix direction, not built:** keep for
good only a resolution that is current once the arguments have run: when `routine_generation`
moved between resolving and calling, resolve again before the call and remember that answer. That
closes `o2`, `o2r` and `o3` and the resolution half of `o1` together. Witness: `o3` (with `o3c`),
first-pass line included.

### Important

None found.

### Minor

**m-1. `library_routine_object_identity.rex:7-13` tests identity with `=` on `identityHash`, which
is not an identity test** (ran). At `NUMERIC DIGITS 9`, `=` compares about nine significant digits.
The oracle's hashes are 15-digit addresses, so `gc2` prints `object 1 1` for two distinct
`.object~new`s on the oracle. The crate's are handle bits, exact only until a collection sets high
bits: under collect-on-every-allocation `gc4` prints `raw 756 51539608304 51539608312`, and `=`
calls the last two equal. The expected `1`s are right today, re-read from raw values (`ih`: oracle
`-139684902648961` for `findRoutine` twice, `importedRoutines` and `loadExternalRoutine` twice). But
the witness proves it only while no collection happens before its comparisons. Use `==`.

**m-2. No gate that runs sees the new root** (ran). With `add_global` deleted (M-root), the identity
witness stays green (its object stays in `a`). `library_routine_imported.rex` goes red (rc 120,
dead-handle refusal), but only under collect-on-every-allocation. `collect_stress`'s L0 loop
panics first at `library_routine_argument_errors` (`phase-8.txt:139`, `dispatch.rs:1510`, the known
L0 failure, reproduced at head), and `on_interpreter_thread` re-raises it (`lib.rs:5879-5881`). So
it never reaches `:186`. The release corpus does not collect there. `gc1` is a single-program
witness that reddens under M-root.

**m-3. The rewritten rule prose is imprecise in three places, plus a retained false clause** (read;
`a9`, `o3` ran).
- `run.rs:177-178`, "kept while no routine table they come after has been written": the generation
  is global (`lib.rs:4822-4824`), so these kinds are dropped by any routine-table write in any
  package or any library load.
- `ir/drive.rs:1536-1540`, "a kept answer the oracle would look up again once a routine table has
  been written": the oracle looks them up on every call.
- `phase-8.txt:174-176`, "A call site keeps a routine findRoutine found for good": false for a body
  run under a second trace setting (`a9`, out of scope) and for C-1's first resolution.
- `run.rs:3842-3846`, kept under the rewritten first sentence: "The parent step is what a `newFile`
  executable resolves through, and only that route has one" is false, because
  `package_from_source` records a parent too (`lib.rs:3743-3745`; `d2b` resolves through one).

**m-4. The identity concern is closed for merged-table objects only** (ran, `id1` and raw values in
`ih`). The oracle's `loadExternalRoutine` and a `::ROUTINE ... EXTERNAL "LIBRARY rxmath RxCalcSqrt"`
answer the same object as `findRoutine` (`LibraryPackage.cpp:410-432`, `PackageManager.cpp:400-412`,
printed; raw hashes equal). Head answers a fresh object per `loadExternalRoutine` (`dispatch.rs:8430-8438`)
and the directive's own. Pre-existing, rc 0. But the ledger's "closes the identity concern" is wider
than `a15a8603a` built; record the remainder.

**m-5. `tests/library_routine_memory.rs:56` needs the oracle checkout on every plain `cargo test`**
(read, ran). `LD_LIBRARY_PATH` is `oracle_root().join("lib")` with no `REXX_CORPUS_GATE` check. The
tests beside it that load extensions use the worktree's `build/lib`. The cost is fine: 1.41 s
release, 11.78 s debug. It does not depend on the machine's memory (`ulimit -v` caps address space);
headroom is about 300 MiB (rc 0 at 750000 KiB, rc 134 at 700000, where `say 'x'` still runs). Gate
it, or skip when the library is absent, or say in its doc that it needs the checkout.

## Out-of-Scope Observations

- **A kept resolution is per chunk, and a body has a chunk per `ChunkTrace`** (`plan.rs:686-706`).
  `a9`/`a9m`: an internal routine called twice, the second time under `trace r`, re-resolves after
  `~addRoutine`. Oracle `2 main` / `2 4`; head and base `2 added` on both, stdout and the `>>>` line.
  The control `a9c` (no trace) is SAME. Pre-existing for both kept kinds.
- **A namespace call walks no parent** (`lib.rs:3527-3553`). `ns1`: oracle `NS:mainpub()` `mainpub` and
  `NS:frompub` `frompub` through a context-built package's parent; head and base 43. Pre-existing.
- **`.Routine~new(name, source)` with no context gets no parent** (`nf2`: oracle `new mainr`, head and
  base 43.1 rc 213). `newFile` with no context does get one (`lib.rs:4583-4589`). The oracle gives
  both the caller's package (`BaseExecutable.cpp:252-260`).
- **`~addRoutine` refuses a library `Routine` object** (`ar1`: oracle `added 4`, head and base
  88.914).
- **Where "recorded, not fixed" lives.** `.ROUTINES` from Rexx (`rw3`, pre-existing at `e7cb210d9`),
  `x14s`, `o1`, `t6`, `x12`, and this review's `a9`, `ns1`, `nf2`, `ar1` are recorded only in
  `task-2-report.md`, `progress.md` and this file. All three are under `.superpowers/`, which
  `.gitignore:30` excludes. No tracked file names them yet. C-1 makes `o1` matter more.

## Assessment

**Fix round 2:** Needs another round

C1, I-B, the boundary items and three of the four integration minors are fixed and hold past their
witnesses. That includes two levels of context, both kept kinds against every event the brief
named, the writes table read against every writer, and the new root under collect-on-every-allocation.
The kept-for-good rule is the oracle's, but it is only safe for a resolution taken after the
arguments ran. Making `MergedLibraryRoutine` permanent turned the recorded `o1` first-pass error
into a permanent one: `o3` answers the library routine on every pass where the oracle and
`bd64f3197` answer the helper's routine from the second pass on (C-1, rc 0). The fix is small and
closes the `Routine` analogue too. m-1 to m-5 are prose, witness strength and test placement.

Cleanup: `target-base`, `target-harness`, `target-head`, `target-headdbg`, `target-task` under
`t2-rereview2/` deleted by explicit path (1.5 GB together); `/tmp` 30% used after. The worktree was
not written (`git status` clean).
