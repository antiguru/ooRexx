# Phase 9b Task 9 report: package and library sites

Status: all three steps are done, and so is the Task 7 follow-up the controller assigned (rulings of 2026-10-13: identity is (a) for REXX and rexxutil, HANDLE is (B), and the rooting bug is chased here).

Commits:
- 6869f47ac: Step 3, classic-style routine rows keep their refusal as a GUARD
- baeb16fdc: Step 1, `LIBRARY rexxutil` and `LIBRARY REXX` name the internal packages
- 9c9becde7: Step 1, refusal-sites line numbers
- 0ed17d225: report and records for Steps 1 and 3
- 249246d13: Phase 9b Task 7 follow-up, `security_entry` roots the info directory
- 1b6212fe1: the two install-time rexxutil witnesses collect nothing under stress
- 8739071a1: Step 1, one Routine object per internal package row
- 52bcef6ce: Step 2, `HANDLE:0`, `HANDLE:1` and `HANDLE:2` through the standard streams

Probes and gate outputs are in `docs/superpowers/records/2026-10-13-phase-9b/task-9/`. `probes/both.sh FILE [STDIN]` runs the oracle and the crate, each under `memcap 4G` from a fresh empty directory. The oracle side also runs under `ulimit -v 1048576`.

## Step 1: `LIBRARY rexxutil`

### Oracle measurements (before the change)

| probe | oracle | crate at BASE |
|---|---|---|
| `req4.rex`: `::requires 'rexxutil' library`, then `importedRoutines` | StringTable, 55 items; `SYSSLEEP`, `SYSADDREXXMACRO` and `SYSFORK` present, `FILESPEC` absent; `findRoutine` == the `importedRoutines` entry; `findPublicRoutine` non-nil; `routines` 0, `publicRoutines` 0; package name `REXX` | 98.903 rc 158 |
| `req1.rex` | `importedPackages` 0 | 98.903 rc 158 |
| `reqrexx.rex`: `::requires 'REXX' library` | imports BEEP, DIRECTORY, FILESPEC | 98.903 rc 158 |
| `req3n.rex`: `p~loadLibrary('rexxutil')`, `('REXXUTIL')`, `('REXX')` | 1, 0, 1 | 0, 0, 0 |
| `ext1.rex`: `loadExternalRoutine('x','LIBRARY rexxutil SysSleep')` | a Routine whose package is REXX; `syssleep` found caselessly; `SysSleepX` and `LIBRARY REXXUTIL` answer .nil; `loadExternalMethod` answers .nil | 98.903 at the program's `::routine rr external 'LIBRARY rexxutil SysSleep'` |
| `ext2.rex`, `attr1-3.rex`: `::method`/`::attribute` over `LIBRARY rexxutil` | 90.998 rc 166 (`GETSysSleep`, `GETx`, `x`) | 98.903 rc 158 |
| `ext3.rex`: `::routine rr external 'LIBRARY rexxutil SysNoSuch'` | 90.999 rc 166 | 98.903 rc 158 |
| `sec3.rex` against `sec4.rex`: a security manager set before a `::routine` runs `SysSleep` | without the requires, `event CALL SYSSLEEP`; with it, no event (a merged routine is found at `externalCall` step 2, before `callObjectFunctionExit` asks the manager) | events in both |
| `err3.rex` against `err4.rex`: `call 'SysSleep' 'abc'` trapped | traceback names `SYSSLEEP` without the requires and `SysSleep` with it | - |

No security-manager event comes from the library load itself: `RexxActivation::loadLibrary` (`execution/RexxActivation.cpp:3278`) only merges.

### What changed

- `internal_routines::Package::named`: `REXX` and `rexxutil`, byte for byte (`GlobalNames.h`). `package_routine(package, name)` replaces `rexx_package_routine`.
- `::requires X library` with an internal name merges that package's rows as `MergedRoutine::Internal(index)`. These resolve to the new `Resolved::MergedInternal`, which is found and run as `MergedLibraryRoutine` is: no security checkpoint, run under the name the call wrote.
- `EXTERNAL 'LIBRARY rexxutil ...'` on a `::ROUTINE` binds the row the way `LIBRARY REXX` does (`internal_routine_entry`). On a `::METHOD` or `::ATTRIBUTE` it goes through the `REXX` method path, with `package_entry_point` answering none for `REXXUTIL`.
- `loadExternalRoutine` and `loadExternalMethod` use the same lookups.
- `Package~loadLibrary` answers 1 for both internal names.
- `merged_routine_object` for an internal row answers one object per row. 8739071a1 extends this to every path (see "Step 1 completed").

### Witnesses (`rust/corpus/phase-9.txt`, each with its `sourceline_oracle/*.txt`)

All four are identical to the oracle on stdout, stderr and status (`both.sh` runs above; the corpus gate below):
- `lang/rexxutil_library_requires.rex`: the merge, `importedRoutines`, `findRoutine`/`findPublicRoutine` identity and package, `Routine~call`, `loadLibrary` 1 1 0, no manager event, and the blame name as written for a quoted call and for a symbol call.
- `lang/rexxutil_library_external.rex`: the directive form, `loadExternalRoutine` caseless, missing, `REXXUTIL`, and `loadExternalMethod`.
- `lang/rexxutil_library_method.rex`: 90.998 rc 166.
- `lang/rexxutil_library_routine_missing.rex`: 90.999 rc 166.

The exclusions entry under "PHASE 8 CLOSES" is now CLOSED, and its pointer at row 521 is updated.

## Step 3: classic-style rows

`git grep --no-index -n -I -e REXX_CLASSIC_ROUTINE -e ROUTINE_CLASSIC_STYLE -- ootest testbinaries extensions samples`, with the output in `records/.../task-9/classic-search.txt`. Every hit is in `extensions/platform/windows/rxwinsys/rxwinsys.cpp:4227-4235`, and the command exits 0. The same file holds the listing of every `RexxRoutineEntry` table in those trees, and none of the others carries a classic row. `ootest` is gitignored here, which is why the command needs `--no-index`.

The refusal is now `Loud::classic_style_routine()` with no owner. It has a GUARD row in `refusal-dispositions.tsv` naming the search, and a `refusal-sites.tsv` row whose witness is the adjacent typed row (`say RxCalcSqrt(4)` over `::requires 'rxmath' library`: 2, rc 0 on both, with the crate run under `LD_LIBRARY_PATH` set to the oracle's lib). The exclusions paragraph that put classic rows under Phase 10 is split: `REGISTERED` stays Phase 10, and classic rows are the GUARD.

## Checks (all foreground, from `rust/`, unpiped exit statuses)

| check | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | exit 0 |
| clippy | `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec -p rexx-parse --all-targets -- -D warnings` | exit 0 |
| tables | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --test internal_routines --test package_requires --test refusal_sites --test refusal_dispositions --test closed_phases --test owners --test phase_ten_reasons --test native_entries` | all ok after the refresh at 9c9becde7 (refusal_sites 5/5) |
| sourceline | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse --test sourceline_oracle` | 1 passed |
| corpus | `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G timeout 580 cargo test -j 4 --release -p rexx-exec --test corpus` | exit 0, "1046 of 1046 matching" (`gates/corpus.out`), `requires_manager_rename.rex` among them |
| collect_stress | `CARGO_INCREMENTAL=0 memcap 8G timeout 580 cargo test -j 4 --release -p rexx-exec --test collect_stress` | at baeb16fdc, exit 101: 36 passed, 1 failed, and the one failure is pre-existing (below, fixed in 249246d13) |
| suite | `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Package,base/directives/ROUTINE,base/directives/METHOD,base/directives/ATTRIBUTE,base/class/MethodArgs,base/security.manager/SecurityManager,base/rexxutil/SysFileTree,base/rexxutil/platform/unix/tilde ... cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` | exit 101. 5 groups green. Red tests: Package TEST_NEW01, TEST_PROLOG; ATTRIBUTE TESTMISPLACEDCLASSMETHOD; SecurityManager's 5. All are in `suite-9d-set.txt`, plus Package TESTRECLAIM01 (the package-as-object item, 9a ledger) (`gates/verdicts.tsv`) |

Which groups the suite run covers: no ooTest file uses rexxutil as a library. `git grep --no-index -n -i -I -e rexxutil -- ootest` outside the rexxutil group directories finds only comments and the directory listing. `git grep --no-index -n -i -I -e rxfuncadd -- ootest ':!ootest/.svn'` finds only `API/classic/CLASSIC.testGroup`'s `orxclassic1` calls. So I ran the groups that load libraries, found by `git grep --no-index -l -i -I -E "library['\"]? *$|loadLibrary|external +['\"]LIBRARY" -- ootest/ooRexx`, limited to `base/`, plus SecurityManager for Task 7's `load_requires`.

### collect_stress: pre-existing failure

`lang/requires_manager_rename.rex` (Task 7 fix round 1's witness) fails `the_l0_subset_passes_again_under_collect_on_every_allocation`. The plain run exits 0. Under stress it prints only `-- an existing file renamed to another`, then stops at exit 120 with `rexx-exec: a message send to a value whose object is no longer live is not implemented`. bf71af676 (BASE), built from `git archive` with its own target dir and with `interpreter`, `api` and `common` symlinked in, gives byte-identical output (`gates/base-collect.out` against `gates/collect.out`). The four new witnesses pass under stress. Reported to the controller.

## Task 7 follow-up: the unrooted info directory (249246d13)

`Interp::security_check` pops the root frame its send ran under and answers the info directory unrooted. `security_entry` then allocated its index string (`self.text(name)`) before reading the directory, so collect-on-every-allocation freed the directory first. Every caller has this shape (CALL, METHOD, STREAM, COMMAND, the environment lookups, REQUIRES), and every caller reads the directory through `security_entry` first. So the fix is one `push_temp(directory)` there, the way the surrounding code roots its temps.

- Before: `the_l0_subset_passes_again_under_collect_on_every_allocation` fails at bf71af676 and at baeb16fdc with `requires_manager_rename.rex` at stress exit 120 (`gates/base-collect.out`, `gates/collect.out`).
- After: the same test passes that program. The only failure left was the zero-collection list, which drifted by the two new install-time witnesses (`gates/collect-fix.out`). 1b6212fe1 adds them, and the run is then 37 of 37, exit 0 (`gates/collect-fix2.out`). The existing L0 subset run covers the defect, so no new test was added.

## Step 1 completed: one Routine object per row (8739071a1)

`internal_routine_objects` is a `Vec<Option<ObjRef>>` indexed by `INTERNAL_ROUTINES` position. Each object is rooted under `internal_routine_root_key` and carries the annotation key `Annotated::InternalRoutine(index)`. These paths answer from it:
- `loadExternalRoutine` (`construct.rs`);
- a `::ROUTINE ... EXTERNAL 'LIBRARY REXX|rexxutil ...'` directive's `.ROUTINES` entry (`environment.rs`), which serves `findRoutine`, `findPublicRoutine` and `~routines` for directives;
- the merged rows from `::requires ... library` (`merged_routine_object`).

`::ANNOTATE ROUTINE` on any directive bound to a row writes the row object's table, as `LibraryRoutine` does. `~addRoutine` files the object under the first directive bound to it. That keeps `p~addRoutine('FF2', p~findRoutine('ff'))` working: unification had turned it into 88.914, and `add1.rex` measured that before the fix. A written `.ROUTINES` entry holding a row object resolves as `MergedInternal`.

Witness `corpus/lang/rexxutil_library_identity.rex` holds every equality, for both packages, and is identical on both sides. It covers:
- `loadExternalRoutine` twice, and caseless;
- a directive against `loadExternalRoutine`, and two directives;
- the `~routines` table, `importedRoutines`, `findRoutine` and `findPublicRoutine`;
- another package's `importedRoutines`;
- `copy`, which is a new object on both sides;
- annotations from two directives in one table;
- `addRoutine` of a directive's object.

The probes behind it are `id1.rex`, `id2.rex`, `id3.rex`, `ann1.rex` and `add1.rex`. `id2.rex` also shows that `StackFrame~executable` of a failing internal routine is a different object from the row's, on both sides, so `rexx_routine_executables` stays separate. The exclusions row "loadExternalRoutine OF A LIBRARY REXX ROUTINE ANSWERS A NEW OBJECT EACH TIME" is deleted now that it is green.

## Step 2: HANDLE streams (52bcef6ce)

- `handle_set` converts its argument as the oracle's `int` does: a whole number in the int32 range. Otherwise it raises 88.907, reported lineless against REXX (`Error 88 running REXX:`, `h6.rex`). Sixteen spellings match (`h4.rex`).
- `0`, `1` and `2` set `standard` and `handle`, so reads and writes take the standard-stream paths. Any other descriptor raises `Loud::handle_descriptor` (no owner, DEVIATION row citing Deviation 35).
- Unopened: `QUERY STREAMTYPE` answers `UNKNOWN`, and `QUERY HANDLE` and `QUERY EXISTS` answer ''.
- `open` parses the file options (93 on a bad one, as `handleOpen` does), answers `READY:`, and from then on reports the name as written for `qualify` and `QUERY EXISTS`.
- Opened: `QUERY HANDLE` answers 0, 1 or 2, and the stream type comes from the Invocation's `standard_transient`.
- Close answers `READY:`, then UNKNOWN.

Deviation 35 is in the exclusions file after 34, with the oracle's answers per shape. The phase-8-gate paragraph and `oracle-crashes.txt` entry 12's last paragraph no longer say HANDLE is refused.

- Agreeing shapes: `corpus/lang/stream_handle_names.rex`, identical on both sides with read shapes only. It covers init, unopened queries, open, opened queries and close for `HANDLE:0`, `handle:1` and `HANDLE:2`, ten descriptor spellings, and a bad open option. A second open is not in it, because the oracle's second open goes through a file named `HANDLE:0` (ERROR:2).
- Crate-only: `crates/rexx-exec/tests/stream_handle.rs`, 6 tests naming Deviation 35:
  - reads through stdin;
  - writes to stdout and stderr;
  - `QUERY HANDLE` after an open;
  - a second open;
  - the stream type from the Invocation;
  - `HANDLE:7` refused, with `HANDLE:2` beside it made.

  A mutant mapping `0` to stdout turned 3 of them red, and the mapping was restored (`grep -c` 1).

Probes `h1.rex` to `h8.rex` run read shapes only on the oracle. One of them, `h5.rex`, opens `HANDLE:1` with `both nobuffer`, which writes nothing (oracle rc 163 at its last line, the bad-option 93).

## Checks after the rulings (from `rust/`, foreground)

| check | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy -j 4 -p rexx-exec -p rexx-core -p rexx-parse --all-targets -- -D warnings` | exit 0 |
| `cargo test -j 4 -p rexx-exec --test refusal_sites --test refusal_dispositions --test closed_phases --test owners --test phase_ten_reasons --test native_entries --test licensed_divergences --test internal_routines --test package_requires --test stream_handle` | all ok (native_entries 24, stream_handle 6); refusal-sites refreshed in 52bcef6ce |
| `cargo test -j 4 -p rexx-parse --test sourceline_oracle`, `cargo test -j 4 -p rexx-core` | ok |
| `REXX_CORPUS_GATE=1 ... cargo test -j 4 --release -p rexx-exec --test corpus` | exit 0, "1048 of 1048 matching" (`gates/corpus3.out`) |
| `cargo test -j 4 --release -p rexx-exec --lib` | 1116 passed, exit 0 (`gates/lib2.out`) |
| `cargo test -j 4 --release -p rexx-exec --test collect_stress` | 37 passed, exit 0 (`gates/collect4.out`) |
| suite: Stream, STREAM, CHARS, LINES, Package, ROUTINE, METHOD, ATTRIBUTE, MethodArgs, SecurityManager, SysFileTree, tilde | exit 101. 6 green. Every red test is in `suite-9d-set.txt` except Package TESTRECLAIM01 (package-as-object, 9a ledger) (`gates/suite2/verdicts.tsv`) |

There is no callgrind run, per the dispatch.

## Other findings, not this task's

- `call r~call 'abc'` is 35.1 on both sides, but the message here is `Incorrect expression detected at "&1".` where the oracle substitutes `"~"` (`id3.rex` before its fix).
- `.Stream~new('STDIN')`, the `std_set` stream, diverges before it is opened (`h3.rex`): `QUERY STREAMTYPE` is PERSISTENT against the oracle's UNKNOWN, and `QUERY HANDLE` is 0 against ''. `chars` and `lines` answer 1 1 against 24 3, and `qualify` and `QUERY EXISTS` after an open answer the path and '' against `STDIN`. HANDLE streams answer these as the oracle does. STDIN was left as it is.
- `time_support.rs`'s `whole_number_argument` raises 88.907 without `lineless`/`internal_package`. If the oracle reports those against REXX too, as it does for `handle_set`, they differ. Not measured.

- A security manager set with `.context~package~setSecurityManager` is seen by the running activation here and not by the oracle, which captures it when an activation starts (`sec1.rex`: the oracle sends no event, the crate sends `CALL SYSSLEEP`). This is pre-existing and does not depend on this task's change.
- `libraries.rs`'s `PackageTable::new` puts `REXXUTIL`, while the oracle's key is `rexxutil` (`GlobalNames.h:201`). That table models unload order by bucket, so the key's hash may place later libraries differently. Not measured.

## Fix round 1

Commits:
- 2722d0049: I1, `~addRoutine` files a row's object as its row
- 01e1c2e2a: M3, `security_check` roots the info directory it answers
- 70f39437e: M2, Deviation 35's chars and lines cell states its condition

**I1.** `Interp::routines` and `package_public_routines` now hold `MergedRoutine` rather than `InstalledRoutine`. `install_routine` files a `Routine` with a directive record as `Installed`, and files an internal package row's own object (`rexx_routine_object` is `Some` and the store holds this object) as `Internal(index)`. Lookups through those tables (`find_routine`, `find_written_routine`, `merge_required`, `namespace_routine`, `~routines`, `~publicRoutines`) read the `MergedRoutine` as stored. The `get_or_insert` of a first directive in `environment.rs` is gone. `rexxutil_library_identity.rex` now opens with probe a6's shapes on both sides, with no reflection before them:
- `addRoutine` of `loadExternalRoutine(SysSleep)` (a1);
- `addPublicRoutine` of an imported-only `findRoutine('SysVersion')` (a2);
- `addRoutine` of REXX `Filespec`;
- `addRoutine` before and after `z~findRoutine('zz')` (a6).

All of them are identical to the oracle (`both.sh` run and corpus gate below). Probe `add2.rex` also covers an importer through `addPackage`, which agrees, and a copy of a row's object. The copy is still refused, 88.914, where the oracle adds it. That refusal comes from BASE (a copy carries no directive record), and the exclusions row "Package~addRoutine REFUSES A LIBRARY Routine OBJECT" now says so.

**I2.** The identity witness's first lines take a row object's `identityHash` and drop the object. They then allocate 50 strings and ask `loadExternalRoutine` again, comparing the hash and calling the new answer. As a mutant, `add_global` was deleted from `internal_routine_object` and `cargo test -j 4 --release -p rexx-exec --test collect_stress` was run: exit 101, with `the_l0_subset_passes_again_under_collect_on_every_allocation` failing on `rexxutil_library_identity.rex` at stress exit 120, stdout empty and stderr `rexx-exec: a message send to a value whose object is no longer live is not implemented` (`gates/fix1/collect-mutant.out`). The line was restored by an exact edit, and `git diff --stat -- crates/` printed nothing.

**M3.** `security_check` pushes the directory as a temp of the caller's frame after `pop_frame`, and its doc says so. `security_entry` no longer pushes.

**M2.** The cell now reads "the standard input's answer: 1 0 in h3.rex, two lines left".

Checks (foreground, from `rust/`, under memcap):

| check | result |
|---|---|
| `cargo fmt --all --check`, `cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0, exit 0 |
| `cargo test -j 4 -p rexx-exec --test package_requires --test refusal_sites --test refusal_dispositions --test closed_phases --test owners --test stream_handle --test internal_routines --test native_entries`, and `-p rexx-parse --test sourceline_oracle` | all ok |
| `cargo test -j 4 --release -p rexx-exec --test collect_stress` | 37 passed, exit 0 (`gates/fix1/collect-fr1.out`) |
| `REXX_CORPUS_GATE=1 cargo test -j 4 --release -p rexx-exec --test corpus` | exit 0, 1048 of 1048 matching (`gates/fix1/corpus-fr1.out`) |
| `cargo test -j 4 --release -p rexx-exec --lib` | 1116 passed, exit 0 |
| suite: Package, ROUTINE, MethodArgs, SecurityManager, ADDRESS | exit 101. 3 green. The red tests are the 9d set plus Package TESTRECLAIM01, the same as before (`gates/fix1/verdicts.tsv`) |
