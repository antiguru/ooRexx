# Phase 9b Task 9 review (bf71af676..5dfb998ae)

Reviewer: t9-review. Every interpreter run was under `memcap 4G` from a fresh empty directory. Oracle runs used `ulimit -v 1048576`. No HANDLE write was run on the oracle. Mutants were applied in place, reversed by exact edit and confirmed with `git diff --stat`, which showed only the controller's `progress.md`. `target/release` was rebuilt at HEAD afterwards. The temporary probe test was deleted.

## Spec Compliance

❌ One issue. Steps 1, 2 and 3 are implemented, and the rulings (a) and (B) are followed. Step 1's identity is not complete for `~addRoutine` (I1).

⚠️ Cannot verify from the diff: whether the suite's red set is unchanged against BASE. The report's `gates/suite2/verdicts.tsv` was not re-run.

## Strengths

- `Package::named` is byte-exact, so `REXX` and `rexxutil` load and `REXXUTIL` and `rexx` answer 0. `package_routine` takes the place of the REXX-only lookup without duplicating it. `MergedRoutine::Internal` and `Resolved::MergedInternal` follow the existing `MergedLibraryRoutine` shape.
- The identity store is a `Vec<Option<ObjRef>>` indexed by row, rooted by `add_global` (`environment/identities.rs:1027-1041`), as ruled.
- `security_entry`'s single `push_temp(directory)` (`security.rs:1402` in the diff) covers every checkpoint.
- Deviation 35's citations are accurate.

## Checks run

| # | check | result |
|---|---|---|
| 1 | the six new corpus witnesses on both sides, three descriptors | identical on all six (only the side label differs) |
| 1 | `collect_stress the_l0_subset...` at HEAD | ok, 152.9 s, exit 0 |
| 1 | `rexxutil_library_identity.rex` | every equality is 1 on both sides; `copy` is 0 on both |
| 2 | probes a1-a6 (`~addRoutine` shapes, below) | a3 (two directives, plus addRoutine and addPublicRoutine from another package) matches; a1, a2, a5 and a6 diverge (I1) |
| 3 | `grep security_check(` callers | six callers: install.rs:741, dispatch.rs:2277, run/call.rs:557, command.rs:921, environment.rs:98, run.rs:4054. Each passes `info` straight to `security_entry` with no allocation between. No other reader of a manager-returned directory exists. |
| 4 | `grep '^+.*(unsafe\|from_raw_fd\|RawFd)'` over the diff | prose only; no code |
| 4 | Deviation 35 against the oracle source | `handleOpen` is at StreamNative.cpp:607 and calls `resetFields()` (:731). That calls `fileInfo.reset()` and clears `opened_as_handle` (:745). `SysFile::reset` (SysFile.cpp:220-230) flushes, frees the buffer and sets -1. `setHandle` (:2565) calls `SysFile::open(int)`. All accurate. |
| 4 | h1 and h3 (read shapes) on both sides, with stdin from `stdin.txt` | the oracle answers match the Deviation 35 table |
| 4 | interleaving: `say`, `lineout 'HANDLE:1'`, `charout 'HANDLE:1'`, `s~lineout`, `lineout 'STDOUT'`, with stdout redirected to a file | `a b c de f g h rc 0` in program order; HANDLE:2 and STDERR go to stderr in order |
| 4 | stdout closed (`>&-`) and a closed pipe (`\| true`): crate HANDLE:1 and STDOUT; oracle STDOUT only | crate: `lineout` 0 `READY:` for both. Oracle STDOUT: 1 `ERROR:9 Bad file descriptor` and 1 `ERROR:32 Broken pipe` (M1) |
| 5 | `n55.rex`: `::requires 'rexxutil' library`, then `importedRoutines` and `loadLibrary` | identical: StringTable 55, no FILESPEC, the 55 names listed, `loadLibrary` rexxutil/REXX/REXXUTIL/rexx = 1 1 0 0; package name REXX (external witness) |
| 6 | the `classic-search.txt` command, re-run | 9 hits, all in rxwinsys.cpp, exit 0, as committed. `oorexxapi.h:201-206` defines only those two spellings. `REXX_ROUTINE(` with a literal style finds nothing. |
| 7 | mutant M3: `internal_routine_objects[index].filter(\|_\| false)` (no cache) | caught: the identity witness gives all 0, and the requires witness's `stable`/`public` lines give 0 |
| 7 | mutant M1: delete the `add_global` for the row object | **survives** `collect_stress` (ok, 158.4 s) and every committed witness. A 3-line probe under `run_program_collect_every_alloc` catches it: rc 120 "a message send to a value whose object is no longer live". The same probe at HEAD gives rc 0. (I2) |
| 7 | `stream_handle` test binary | 6 passed |

## Issues

### Critical

None.

### Important

**I1. `~addRoutine` of a row object depends on order and diverges from the oracle when no directive has bound the row yet** (`environment.rs:1181-1187`, `dispatch/package.rs` `install_routine`).

`install_routine` requires `executable_sources[obj].routine`. A row object gets that field only when some package's `.ROUTINES` table is built over a directive bound to the row, and that table is built lazily, on reflection. So the same `p~addRoutine('X', u)` on the same object raises or succeeds depending on whether some unrelated `q~findRoutine('zz')` has run. Measured:

- a1: `u = .Routine~loadExternalRoutine('x','LIBRARY rexxutil SysSleep'); p~addRoutine('S2', u)`. Oracle `added 0 1`; crate `88.914`.
- a2: the same with `u = p~findRoutine('SysSleep')` after `::requires 'rexxutil' library`. Oracle `added 0 1`; crate `88.914`.
- a5: the same object after `.Package~new('q', "::routine zz external 'LIBRARY rexxutil SysSleep'")`. Still `88.914`. The REXX package (`FILESPEC`) also gives `88.914`.
- a6: `q` created, then `p~addRoutine` gives `88.914`. Then `q~findRoutine('zz')`, then the identical `p~addRoutine` gives `added`. The oracle answers `added` both times.

"The row object is filed under the first directive bound to it" means: the shared object's `ExecutableRecord.routine` holds the `(program, directive)` of whichever package's routine table was materialised first. It adds no new map, but it uses a directive as a proxy for the row. That proxy is exactly the order dependence above.

Fix: let `install_routine` accept an object for which `rexx_routine_object(obj)` is `Some`, and file it so that a lookup resolves `MergedInternal`. That resolution already exists for written `.ROUTINES` entries. Then drop the `get_or_insert`. Pin a1, a2 and a6 in the identity witness. Part of a1, the REXX `loadExternalRoutine` case, was likely already `88.914` at BASE; the order dependence is new.

**I2. The rooting of the row objects is unwitnessed.** Mutant M1 (no `add_global`) leaves `collect_stress` and all six witnesses green, because every witness holds the object in a variable or a table. A stress-mode program that drops the object and asks again catches it:

```
say .Routine~loadExternalRoutine('x','LIBRARY rexxutil SysSleep')~call(0)
a = .array~new; do i = 1 to 50; a[i] = 'x' || i; end
say .Routine~loadExternalRoutine('x','LIBRARY rexxutil SysSleep')~call(0)
```

Mutant: rc 120. HEAD: rc 0. Add this shape to `rexxutil_library_identity.rex` (or a sibling witness) so that `collect_stress` runs it.

### Minor

**M1. HANDLE:1 inherits STDOUT's swallowed write errors.** With stdout closed, or a pipe whose reader has exited, the crate answers `lineout` 0 `READY:` for both `HANDLE:1` and `STDOUT`. The oracle's STDOUT answers 1 `ERROR:9` / `ERROR:32`. This predates the task (the STDOUT write path is untouched) and is not this task's to fix. It belongs with the queued STDIN/standard-stream item.

**M2. Deviation 35's "1 0, as `.Stream~new('STDIN')`" cell depends on stdin's position.** In h3 the crate's STDIN answers `chars 1 lines 1` and HANDLE:0, read later, answers `1 0` while two lines remain. In h1 the crate answers `lines 1` with two lines left. The cell should say what was measured (h3, after a STDIN read) or drop "as STDIN".

**M3. Each `security_entry` call pushes the directory again.** `command_from_manager` pushes it up to three times per checkpoint. This is harmless, but rooting once in `security_check`, after `pop_frame`, would state the contract where the unrooted answer is made.

## Assessment

**Task quality:** Needs fixes

**Reasoning:** The identity store, the library naming, HANDLE and the security rooting are correct and oracle-identical on every committed witness. But `~addRoutine` of a row object depends on whether an unrelated reflection has run, and diverges from the oracle when none has (I1). The row objects' rooting can be deleted without any gate noticing (I2).
