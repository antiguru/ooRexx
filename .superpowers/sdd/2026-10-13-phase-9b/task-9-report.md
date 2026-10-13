# Phase 9b Task 9 report: package and library sites

Status: Step 1 and Step 3 are done. Step 2 and Step 1's identity half are waiting on the controller's rulings (see "Open").

Commits:
- 6869f47ac: Step 3, classic-style routine rows keep their refusal as a GUARD
- baeb16fdc: Step 1, `LIBRARY rexxutil` and `LIBRARY REXX` name the internal packages
- 9c9becde7: Step 1, refusal-sites line numbers

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
- `merged_routine_object` for an internal row answers one object per row (`internal_routine_objects`, keyed by `INTERNAL_ROUTINES` index and rooted under `internal_routine_root_key`). This is option (a) from the ruling request, applied to merged routines only.

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
| collect_stress | `CARGO_INCREMENTAL=0 memcap 8G timeout 580 cargo test -j 4 --release -p rexx-exec --test collect_stress` | exit 101: 36 passed, 1 failed, and the one failure is pre-existing (below) |
| suite | `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Package,base/directives/ROUTINE,base/directives/METHOD,base/directives/ATTRIBUTE,base/class/MethodArgs,base/security.manager/SecurityManager,base/rexxutil/SysFileTree,base/rexxutil/platform/unix/tilde ... cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` | exit 101. 5 groups green. Red tests: Package TEST_NEW01, TEST_PROLOG; ATTRIBUTE TESTMISPLACEDCLASSMETHOD; SecurityManager's 5. All are in `suite-9d-set.txt`, plus Package TESTRECLAIM01 (the package-as-object item, 9a ledger) (`gates/verdicts.tsv`) |

Which groups the suite run covers: no ooTest file uses rexxutil as a library. `git grep --no-index -n -i -I -e rexxutil -- ootest` outside the rexxutil group directories finds only comments and the directory listing. `git grep --no-index -n -i -I -e rxfuncadd -- ootest ':!ootest/.svn'` finds only `API/classic/CLASSIC.testGroup`'s `orxclassic1` calls. So I ran the groups that load libraries, found by `git grep --no-index -l -i -I -E "library['\"]? *$|loadLibrary|external +['\"]LIBRARY" -- ootest/ooRexx`, limited to `base/`, plus SecurityManager for Task 7's `load_requires`.

### collect_stress: pre-existing failure

`lang/requires_manager_rename.rex` (Task 7 fix round 1's witness) fails `the_l0_subset_passes_again_under_collect_on_every_allocation`. The plain run exits 0. Under stress it prints only `-- an existing file renamed to another`, then stops at exit 120 with `rexx-exec: a message send to a value whose object is no longer live is not implemented`. bf71af676 (BASE), built from `git archive` with its own target dir and with `interpreter`, `api` and `common` symlinked in, gives byte-identical output (`gates/base-collect.out` against `gates/collect.out`). The four new witnesses pass under stress. Reported to the controller.

## Open

1. **Identity (ruling requested).** The oracle keeps one Routine object per internal row, across `loadExternalRoutine`, `::ROUTINE EXTERNAL`, `findRoutine` and `importedRoutines`. It does this for REXX too (`id1.rex`: oracle 1 1 1, crate 0 0 0, recorded with no owner in exclusions as "loadExternalRoutine OF A LIBRARY REXX ROUTINE ANSWERS A NEW OBJECT EACH TIME"). As committed, merged routines share one object per row. `loadExternalRoutine` and the directive form still build their own, so `ext1.rex`'s "same as ler" is 1 on the oracle and 0 here.
2. **Step 2 (ruling requested).** An oracle `HANDLE:n` stream never reads its descriptor. `handleOpen` (`StreamNative.cpp:607`) runs `resetFields()`, whose `fileInfo.reset()` leaves fd -1 and frees the buffer, and that freed buffer is what makes 12a's write crash. Read-only probes `h1.rex`, `h2.rex` and a piped run give the same answers for HANDLE:0, handle:0 and HANDLE:7:
   - open answers `READY:`, and `linein` answers '' with `NOTREADY:EOF`;
   - after open, `query handle` answers 18446744073709551615 and `query exists` answers the name as written;
   - `streamtype` is the descriptor's type sampled at init;
   - `HANDLE:abc` is 88.907.

   Option 3 as ruled would read stdin where the oracle reads nothing. No code yet.

## Other findings, not this task's

- A security manager set with `.context~package~setSecurityManager` is seen by the running activation here and not by the oracle, which captures it when an activation starts (`sec1.rex`: the oracle sends no event, the crate sends `CALL SYSSLEEP`). This is pre-existing and does not depend on this task's change.
- `libraries.rs`'s `PackageTable::new` puts `REXXUTIL`, while the oracle's key is `rexxutil` (`GlobalNames.h:201`). That table models unload order by bucket, so the key's hash may place later libraries differently. Not measured.
