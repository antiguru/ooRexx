# Phase 9b1 Task 3 report: L10 and L11

Status: DONE_WITH_CONCERNS (concerns are findings outside the brief, listed at the end).

Commits: `0bce0e10c` (L10 and L11, witnesses, exclusions row), `25d47538e` (the routine row shares
the scope slot, for the budget). The records commit follows this report.

## What changed

**L10.** A `Routine` object's internal row is now a field of its body: `NativeObject` holds one
`Bound` slot, either a Method's scope or a Routine's row (`rexx-core/src/body.rs`). A copy clones
the body, so it runs the same row. The `rexx_routine_objects` side map (ObjRef to row) is deleted,
and `Interp::rexx_routine_object` reads the body.
`~addRoutine` (`dispatch/package.rs` `install_routine`) now goes through `Interp::added_routine`
(`environment/identities.rs`). A `::ROUTINE`'s object files as `Installed`, as before. A library or
internal row's own object files as `Library(code)` or `Internal(index)`. Any other object over a row
(a copy) files as the new `MergedRoutine::LibraryObject(code, object)` or
`InternalObject(index, object)`, so `findRoutine` answers the object that was added and a call runs
the row. `Interp::object_roots` roots those objects from `routines`, `package_public_routines` and
`merged_public_routines`. A merge copies an entry into an importer, and nothing else holds the copy
once its own package replaces it (probe `imp.rex`, oracle `importer 0 Routine 0 0`).
The library case is included because the exclusions row the brief removes names it
(`p~addRoutine('SQ2', p~findRoutine('RXCALCSQRT'))`, oracle `added 4`, crate 88.914 before).

**L11.** `run_internal_as` (`run/call.rs`) pushes an `InternalLevel` (caller activation, row, and
offsets into two reused buffers on `Activity`, `internal_names` and `internal_arguments`) before
the body runs and truncates after. After warm-up nothing is allocated, and no per-call struct is
moved. `live_levels` lists `LiveLevel::Internal(row)` before an activation's native and fragment
levels. `build_internal_frame_at` builds the frame with `build_native_level_frame`, whose
`method: bool` became `caller: Option<usize>` so that a live routine frame's context is its calling
activation's, not depth 0's.
**The frame's executable is the row's own `Routine` object** (`internal_routine_object`), live or
blamed. Measured, oracle rc 0 (probes `q1.rex`, `q3.rex`): the native frame's executable is the
same object `.Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysStemSort')` answers. That holds
whether the call is by name, through `Routine~call` or through a `::ROUTINE vv EXTERNAL` directive,
and for a failing `SysSleep` and `ww` too. The crate answered a different, per-name object before
(`q3`: `ROUTINE SYSSLEEP 0` against the oracle's `1`). So `blame_internal_routine` takes the row,
and the by-name map `rexx_routine_executables` with `rexx_routine_executable` and
`Annotated::RexxRoutine` are deleted.

**Refusal tables.** `representation_refusals.rs` had no `ALLOWED` row tagged "Task 3" at
`485c13720`. Its OWNERS entry "Task 3" is removed. No Loud site changed, so `refusal-sites.tsv` was
not re-derived (`refusal_sites` 5 passed).

**Exclusions.** The row "Package~addRoutine REFUSES A LIBRARY Routine OBJECT" is removed from
`docs/superpowers/plans/phase-4-exclusions.txt`.

**Outside the brief's file list:** `rexx-core/src/body.rs` (the body field the dispatch asked for),
`activity.rs`, `dispatch.rs`, `dispatch/context.rs`, `install.rs`, `lib.rs`, `run/call.rs`,
`dispatch/object_protocol.rs` (its copy path no longer re-records the row).

## Witnesses (corpus, `phase-9.txt`)

- `corpus/lang/routine_object_add_copy.rex` (+ `.env` for rxmath): an internal row's copy, a
  library row's own object and its copy, the importer case, and a `Method` refused at 88.914.
- `corpus/lang/stackframes_native_routine.rex`: a live frame for `SysStemSort` and for a directive
  bound to it. It reports type, name, line, invocation and target `.nil`, the arguments (the first a
  heap string built in the call), the context, the executable class and source, the trace line and
  the string. It then checks that the executable is the `loadExternalRoutine` object, and that a
  trapped error from the callback lists the native frame once in `TRACEBACK` and `STACKFRAMES`.

Both are byte-identical to the oracle on stdout, stderr and status (rc 0/0), run from a fresh
directory. Sourceline files were generated with the driver in `sourceline_oracle.rs`.

## TDD evidence

RED, at `485c13720`'s binary, oracle first:
- `l10.rex`: oracle `copy added 1 0`, crate `copy 88.914`. `add2.rex`'s last line is the same.
- `p6.rex`: oracle `added 4 1 / copy added 5 1 / lib copy added 7`, crate `s1 88.914 / s2 88.914 /
  s3 88.914`.
- `l11.rex`: the crate lacks the oracle's `ROUTINE  SYSSTEMSORT Routine 0` line, and `l11b.rex`
  answers `items 2` where the oracle answers `items 3`.
- `q3.rex`: the executable identity, as above.

GREEN: every probe above and both witnesses are identical to the oracle at `0bce0e10c` and again at
`25d47538e`. The one remaining `q1.rex` difference is the missing `CALL` method frame (finding 1).

Mutations, each reverted by copying the saved file back:
- `push_internal_level` replaced by the bare length: `stackframes_native_routine.rex` red (rc 159,
  `items 2`).
- The copy arms of `added_routine` replaced by `return None`: `routine_object_add_copy.rex` red
  (rc 168, 88.914).
- The routine-table root loop given `.take(0)`: `collect_stress`'s new
  `an_added_routine_object_survives_in_its_importer` red, panic `a live value` at
  `dispatch.rs:1710`.
- `size_of::<NativeObject>() < 104`: E0080, so the size is exactly 104.

`internal_arguments` gets no root. A running call's arguments are its caller's temps. The stress
subset reads a heap-string argument back after allocations and matches. No root exists for a red
test to delete.

## Checks

From `rust/`, `...` = `CARGO_INCREMENTAL=0 timeout 600 memcap 8G`. Exits are read unpiped
(redirect to a log, then `echo $?`).

| command | tree | exit | result |
|---|---|---|---|
| `cargo fmt --all --check` | 25d47538e | 0 | |
| `... cargo clippy -j 4 -p rexx-core -p rexx-exec --all-targets -- -D warnings` | 25d47538e | 0 | `Checking rexx-core` .. `rexx-exec` printed |
| `... cargo test -j 4 -p rexx-core --no-fail-fast` | 25d47538e | 0 | 13 binaries, no failures |
| `... cargo test -j 4 -p rexx-exec --no-fail-fast --lib --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test licensed_divergences` | 25d47538e | 0 | lib 1121, closed_phases 8, licensed_divergences 22, owners 6, phase_ten_reasons 5, refusal_dispositions 3, refusal_sites 5, representation_refusals 3 passed |
| `... cargo test -j 4 -p rexx-parse --test sourceline_oracle` | 25d47538e | 0 | 1 passed |
| `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 900 memcap 8G cargo test -j 4 --release --no-fail-fast -p rexx-exec --test corpus --test collect_stress` | 25d47538e | 0 | collect_stress 38 passed, corpus 29 passed 1 ignored, `1054 of 1054 matching`, 5m32s |
| `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Package,base/class/StackFrame,base/class/Routine,base/rexxutil/SysStemSort REXX_SUITE_TABLE=<records>/suite/table.tsv ... cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite -- --nocapture` | 25d47538e | 101 | SysStemSort green. The rest are red on the same tests as the 9b close2 run (`docs/superpowers/records/2026-10-13-phase-9b/close2/suite/verdicts.tsv`). Package: TESTRECLAIM01 TEST_NEW01 TEST_PROLOG. Routine: TEST_NEW_ARRAY_FROM_FILE TEST_NEW_FILE_COMPILED. StackFrame: TESTNATIVECODEFRAME |

The first gated run at `0bce0e10c`'s tree, before the `.env` sidecar existed, was red. The crate
could not load rxmath (98.903), and the program joined the zero-collection list. Adding
`routine_object_add_copy.env` fixed both, and that run was exit 0 with the same counts as above.
Suite records: `docs/superpowers/records/2026-10-14-phase-9b1/task-3/suite/`.

## Budget (callgrind whole process, `-B 0.5`, pin `rexx-run` sha `c4b4e8c4...`)

Final (`25d47538e`, head `rexx-run-9b1-t3b` sha `a9a76e90...`), exit 0:

| program | head d% | head -libc d% |
|---|---|---|
| fibcall | -3.4469 | -2.4130 |
| fibfunc | -3.6143 | -2.4623 |
| extcall | -13.3316 | -2.0270 |
| dispatch | -0.2472 | -0.0715 |
| dispatchclass | -3.9384 | -2.6560 |
| sendloop | -0.1004 | -0.1069 |
| varlookup | +0.0028 | +0.0017 |
| pkglocal | -2.6189 | -3.7206 |
| startup | +0.2872 | +0.3920 |
| alloc | +0.2407 | +0.2725 |
| hashnew | +0.2990 | +0.4350 |

Spreads 0.0000 to 0.0001%. Against Task 2's head (`rexx-run-9b1-t2c`, `task-2/callgrind-fix1a`
and `callgrind-fix1b`), whole process: fibcall +0.0020% (8795416637 against 8795241007), every other
program at or below it.

Command, from `rust/`: `timeout 600 bench-programs/callgrind.sh -B 0.5 -j 11 -p "fibcall fibfunc
extcall dispatch dispatchclass sendloop varlookup pkglocal startup alloc hashnew" -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-3b pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run
head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t3b`. Outputs in
`docs/superpowers/records/2026-10-14-phase-9b1/task-3/callgrind/`, raw `.cg` in `cg-9b1/task-3b/`.

First form (`0bce0e10c`, a separate `Option<usize>` field, head `rexx-run-9b1-t3` sha `48be4314...`):
exit 0, but hashnew was +0.4926% against the pin. Records are in `task-3/callgrind-first/` and raw
`.cg` in `cg-9b1/task-3/`. Against Task 2's head, every -libc count was within 0.04%. The
whole-process growth was in libc:
- hashnew: `_int_malloc` +2.7M and `__libc_malloc2` +2.3M. The field took `NativeObject` from 104
  to 120 bytes and every Directory, StringTable and Table box into the next malloc size class.
- alloc: `__memcmp_avx2_movbe` +24M, collect_now -2.9M.
- sendloop: memcmp +40M.
- pkglocal: memcmp +16M.

The memcmp call counts were unchanged, so those were per-call cost changes from layout. Sharing the
scope slot (`25d47538e`) removed all of them. `25d47538e`'s commit message says the cost was "all of
it in malloc". That is true of hashnew only. alloc's growth was memcmp.

## Findings for the controller

1. **Native *method* frames are missing from a live `.context~stackFrames`.** It is the sibling of
   L11, and an ooTest test is red on it. `base/class/StackFrame` TESTNATIVECODEFRAME expects
   `comparator~stackframes[2]` to be the `SORTWITH` METHOD frame (target, arguments, executable
   `.Array~method("SORTWITH")`). `q1.rex` shows the same for `Routine~call`: the oracle lists a
   `CALL` method frame between the routine and the program (4 items), and the crate does not (3).
   scout-leaks L11's "Reach: ... None found through a native callback" is wrong for methods.
   Closing it needs a live record per primitive method send, which is the send hot path, so I did
   not widen into it.
2. **`filespec` does not send `STRING` to an object argument.** `p1.rex`: the oracle raises 88.909
   ("Argument 2 must have a string value"), rc 168. The crate answers the default name `a K`, rc 0.
3. **`call r~call('s.')` reports a different 35.1 insert.** The oracle has
   `Incorrect expression detected at "~"`, the crate `... at "&1"` (rc 221 both).
4. **Oracle crash.** `probes/q2.rex`: an untrapped SYNTAX raised by a `STRING` method that
   `SysStemSort` sent ends in SIGSEGV, rc 139 (1 run). It looks like a case of
   `oracle-crashes.txt` entry 37, but that entry names only errors the sort raises itself. The crate
   answers rc 163 with the native frame in the report.
5. `Interp::frame_record` (TraceObject caller frames, the spawner frame) still answers `.nil` as
   the executable for every non-activation level, the new internal level included. Not measured
   against the oracle.
