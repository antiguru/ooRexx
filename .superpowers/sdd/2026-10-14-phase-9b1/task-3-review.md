# Phase 9b1 Task 3 review (485c13720..3536722be)

Reviewer binary: release `rexx-run` built from `git archive 3536722be` into a scratch tree with its
own `CARGO_TARGET_DIR` (sha256 prefix `663d721e97c39d49`, `Compiling rexx-exec` seen). The tree and
target dir were deleted afterwards. Probes, the comparison script and saved outputs are in
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-3-review/` (`probes/`, `fsloop/`). Every oracle
run used the standard wrapper from a fresh `mktemp -d` dir, and every crate run used `memcap 4G`.
Stdout, stderr and status were compared separately.

## Spec compliance: PASS

- Step 1 (L10): the row is a field of the copy's own body. `NativeObject.bound` is
  `Bound::RoutineRow(usize)` (`rexx-core/src/body.rs:1029-1038`). `~copy` clones the body, and
  `rexx_routine_object` reads it (`install.rs:2373-2381`). The side map `rexx_routine_objects` is
  deleted (`lib.rs`), along with its re-record in `object_protocol.rs:658`. The exclusions row is
  removed. `add2.rex` (9b task-9 probes) is IDENTICAL, rc 0/0.
- Step 2 (L11): `push_internal_level` and `pop_internal_level` (`run/call.rs:591-647`),
  `LiveLevel::Internal` and `build_internal_frame_at` (`dispatch/context.rs:936-962`). extcall was
  measured.
- Step 3: no `ALLOWED` row was tagged Task 3, and the `OWNERS` entry is removed
  (`representation_refusals.rs:131`).

## Checks run

1. L10 oracle parity. `l10a.rex` covers `==` and `\==` against the original, class, rendering,
   `objectname`, `source~items`, `package`, `call`, `addRoutine`, `findRoutine`,
   `routines[]`, a call by name, `addPublicRoutine`, `findPublicRoutine`, `publicRoutines[]`, a copy
   of a copy, and adding that copy. It is IDENTICAL, rc 0/0. `l10b.rex` passes a row copy to
   `setMethod` (97.2) and to `define` (93.974), then copies and adds again. IDENTICAL. `Routine` has
   no `NAME` method on either side (97.1 on both).
2. L11 fields. `l11a.rex` prints type, name, line, target, arguments, executable identity and
   traceLine for a CALL, a function call, `r~call`, a `::ROUTINE EXTERNAL` directive, and a
   SysStemSort nested inside the callback (5 levels). Every line matches the oracle except the
   `METHOD CALL` frame for `r~call`, which is the queued native-method ruling. `l11c.rex` covers
   SysStemSort inside `INTERPRET`, in the main program and in an internal CALL. It is IDENTICAL, so
   the order Internal, then Fragment, then Activation is right. `err1.rex` traps a failing
   `filespec` and a failing `SysSleep` and prints traceback, stackFrames and executable. IDENTICAL.
3. Readers of the retired by-name map. `rexx_routine_executable`, `rexx_routine_executables` and
   `Annotated::RexxRoutine` have no remaining references, and the build proves it. `Package::Rexx` is
   built only in `blame_internal_routine` (`dispatch.rs:3860`), which now passes the row. The
   install-time row at `dispatch.rs:1695` is an access scope, not a frame. `frame_record`
   (`environment/route.rs:335-338`) still answers `.nil` for every non-activation level. That is
   report finding 5, not a regression.
4. Slot sharing (25d47538e). `set_scope` is called only on objects built with `method_class()`
   (`identities.rs:175`, `:303`), in `method_new_scope` (`:364`, reached only through
   `scoped_method`, which needs `executable_sources[..].installed`, and a row object's record has
   `installed: None`, `install.rs:2015-2023`), and in `clone_executable` with a caller-given scope.
   `set_routine_row` is called only on a fresh `routine_class()` object (`identities.rs:1061`). No
   path writes both kinds to one object. `l10b.rex` shows that a row copy offered as a method is
   refused before any scope write, as on the oracle. `scope()` on a Routine answers `None` as it
   did before, and the trace arm reads `scope()` only. Sound, and not observable.
5. GC. `collect_stress` at HEAD plus one review test: exit 0, 39 passed. The review test runs
   `gc1.rex` under collect-every-allocation: a SysStemSort STRING callback that allocates 60 strings
   around `.context~stackFrames`, holds the frames in `.local`, and reads the routine frame's heap
   argument `'longstem' || 'name.'`, its executable and its traceLine after 50 more allocations.
   It had 462 collections and output identical to the oracle's.
6. Perf. `callgrind.sh -B 0.5 -j 9 -p "extcall fibcall startup"`, pin against the reviewer binary:
   exit 0. extcall -13.3316 (-libc -2.0270), fibcall -3.4469 (-2.4130), startup +0.2869 (+0.3918),
   spreads 0.0002% or less. This matches the report.
7. Mutations. Each was applied as an exact edit, rebuilt (`Compiling rexx-exec` seen), checked by
   witness against the oracle, and reverted by restoring the file from `git show` and `cmp`.
   - The push replaced by the bare length: `stackframes_native_routine` red (rc 159).
   - `added_routine` copy arms set to `return None`: `routine_object_add_copy` red (rc 168).
   - The copy drops its row (`clone_executable` overwrites `RoutineRow` with a scope):
     `routine_object_add_copy` red (rc 168).
   - `build_internal_frame_at` given context depth 0: `stackframes_native_routine` red (context
     line).
   - The live frame given no executable: `stackframes_native_routine` red (rc 120).
   - The root loop given `.take(0)`: `an_added_routine_object_survives_in_its_importer` red, panic
     `a live value` at `dispatch.rs:1710`.
   - The root loop with only `merged_public_routines` kept: **every test green**. See I1.

## Findings

### Important

- **I1. Two of the three new root tables have no red test** (`lib.rs:3144-3157`). With `routines`
  and `package_public_routines` dropped from the loop, `an_added_routine_object_survives_in_its_importer`
  stays green, and so do two review stress tests: `root2.rex`, a copy added only to the main
  package, and `root3.rex`, copies added to a `.Package~new` package and called through
  `findRoutine`, `findPublicRoutine` and an importer. Both match the oracle and both pass under
  collect-every-allocation. The global constraint says every new root gets a test that goes red
  with it deleted. Either the entries are already held elsewhere, likely through the `.ROUTINES`
  table that `install_routine` also writes (`dispatch/package.rs:966-968`), in which case the two
  tables should come out of the loop with a comment saying what holds them. Or a shape exists that
  needs them, and a test should pin it.
- **I2. The changed hot path is not measured by any budget program** (`run/call.rs:591-647`).
  extcall calls a library routine (`TestIntArg`, the `native_handles` path), and none of the
  eleven programs calls an internal row. The reviewer's measurement of a 200k-iteration
  `filespec('N', ...)` loop (`fsloop.rex`), whole process, against the Task 2 head
  `rexx-run-9b1-t2c`: 537092239 to 561287895 Ir, **+4.51%**. Against the pin: +4.18%.
  `run_internal_as` inclusive grows by 24.2M, which is about 121 Ir per call, and its self cost
  goes from 8.8M to 11.6M plus the inlined `Vec` work. Allocation counts are unchanged, so
  "nothing is allocated after warm-up" holds. The cost is copying the name bytes and arguments,
  plus `running_activation()`, on every internal routine call. The gate as written passes. This
  needs a ruling: accept it, or record the level without copying (row plus a name flag, with
  arguments read from the caller's stack).

### Minor

- **M1.** `body.rs:1088`: "The row of the embedding's own routine table" should name
  `INTERNAL_ROUTINES`. Nothing embeds anything here.
- **M2.** `pop_internal_level` runs before a parked body (`Started::Entered`) has finished, so a
  parked internal routine is missing from a stackFrames read during the park. It is not
  observable from the same activity, and not measured.
- **M3.** Concern 3, `probes/q2.rex`, is **not covered** by `oracle-crashes.txt` entry 37. That
  entry lists only errors `StemClass::sort` raises itself (40.922, 40.923, 40.924). q2's error is a
  93.900 raised by a user `STRING` method that the sort's comparison sends. Entry 37 needs an added
  shape: the q2 program, "an untrapped error raised by a method the sort sends (an element's
  STRING)", SIGSEGV rc 139, 1 run of 1, 2026-10-14, Phase 9b1 Task 3 probe q2, and the crate's
  answer of rc 163 with `Compiled routine "SYSSTEMSORT"` in the traceback (reviewer, crate only).
  If that cannot be argued to be the same defect, it needs a new entry 49 with the same
  content. Not run on the oracle.

### Strengths

The row lives in the object, and the extra size was taken back by sharing the slot, with a size
assert that a mutation turns into E0080. The executable identity is now the row's own object on
every path, which also fixes q3's `ROUTINE SYSSLEEP 0`. The witnesses read every field and both
call shapes, and each fix has a mutation that turns its witness red. The report is accurate about
which programs' first-form growth was malloc and which was memcmp.

## Verdicts

- Spec compliance: **PASS**.
- Task quality: **Needs fixes** (I1). I2 needs a controller ruling.
