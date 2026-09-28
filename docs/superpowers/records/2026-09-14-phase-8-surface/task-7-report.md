# Task 7 report -- the prebuilt test extensions, and the group partition

Implementer: surface-7b (finishing a stopped predecessor's uncommitted draft). BASE `c750719b7`.

## Status

DONE_WITH_CONCERNS (gates green at `844ed9e0a`; concerns below).

## What the draft had, and what was kept

The draft `rust/crates/rexx-exec/tests/api_group_partition.rs` was kept in shape: the group and
package reading (`EXTERNAL "LIBRARY ..."`, `rxfuncadd`, `loadPackage`), the transitive `NEEDED` walk
through `readelf -d` stopping at `librexx.so`/`librexxapi.so`, the `readelf --dyn-syms` import scan,
the import classes, and the in-test negative control. Checked against the tree before keeping: the
`loadPackage(` match does not catch `LoadPackageFromData(` (the next byte differs); `external`
inside identifiers such as `TestExternalFunction` is skipped because no whitespace follows;
`METHOD.testGroup:3086`'s quoted `::routine ... EXTERNAL 'LIBRARY orxmethod ...'` adds only
`orxmethod`, which `METHODPackage.cls` binds anyway; a missing `readelf` panics, never skips.

Wrong in the draft, and changed:

* `PARTITION` held the `INVOCATIONTester.cls` groups at 9, and `phase_of` took the first class that
  matched (embedding before registries), so it derived 9 too: the two agreed with each other and
  with the stale records, against the ruling. Now `phase_of` takes the maximum over a per-class map
  `IMPORT_PHASES` (embedding 9, registries 10), and those groups are 10.
* Phase 8 was "no interpreter library and no import"; now it is "no interpreter library", as Step 2
  words it, with an assertion that such a group imports nothing either.
* The in-test control expected `METHOD` at 9 after the rewrite to `orxinvocation`; now 10. Its
  scratch copy moved from a fixed `CARGO_TARGET_TMPDIR/api_group_partition`, which the draft had
  left behind (deleted), to a per-process directory removed after the derivation.

Not read by the test, noted: `INVOCATION.testGroup:906` and `ProcessInvocation.testGroup:887` set
`tester~loadLibrary = "rxmath"`, an option of the interpreter instance the tester creates, not a
directive; the brief names only `EXTERNAL "LIBRARY"` and `rxfuncadd`.

## Rulings applied

* S3: the test is `rexx-exec/tests/api_group_partition.rs`, using `tests/support/oracle.rs`'s
  `oracle_root()`.
* Ruling (Task 7): a group's owner is the latest phase any of its transitive imports needs, from a
  per-import-class map (embedding to 9, RXAPI registries to 10). `liborxexits.so` imports
  `RexxRegisterExitDll` (rxapi, `rexxapi/client/LocalRegistrationManager.cpp:70`, checked: the
  `ClientMessage ... REGISTER_LIBRARY` line of `registerCallback`), so the `INVOCATIONTester.cls`
  groups are Phase 10's.
* Step 3: records corrected where they disagreed.
* Memory and the variable pool (`RexxAllocateMemory`, `RexxFreeMemory`, `RexxVariablePool`) stay
  in the draft's `NEUTRAL` class, naming no phase. Every library that imports one also imports a
  registry, so the classification changes no row; a group importing only neutral symbols panics.

## Step 1 -- the frozen-header witness

Recorded in `docs/superpowers/plans/phase-8-gate.md` section 9 with its commands: `diff -rq` of
`api/` and of `testbinaries/` against the oracle checkout, both empty, exit 0; each target
`testbinaries/CMakeLists.txt` declares (derived by grep from that file) has its product, the
libraries in `build/lib` and `rexxinstance` and `provoke_locks` in `build/bin`. The section says a
build succeeding witnesses compatible headers, not working entry points.

Found on the way: the products are dated 2026-08-05 16:02 and the oracle's `testbinaries/
{CMakeLists.txt,provoke_locks.cpp,provoke_locks.rex}` have mtime 2026-08-22, later. The oracle's
`git status` for `api testbinaries` is empty and its last commit touching them is `334ab5460`
(2026-08-02), so the content is the committed content; recorded as a caveat in the gate section.

Also: `testbinaries/CMakeLists.txt` links `orxmethod` and `orxfunction` against `rexx rexxapi`, yet
their `NEEDED` is `libc.so.6` alone; the linker dropped the unused libraries. The derivation reads the
binary, so it sees what a `dlopen` would map.

## Step 2 -- the derived partition test

`cargo test -p rexx-exec --test api_group_partition`: 22 passed (the two test functions and the
`support` module's own unit tests). Derived and asserted: `METHOD`, `CONVERSION`, `FUNCTION` at 8;
`CLASSIC`, `RexxStart`, `ProcessRexxStart`, `INVOCATION`, `ProcessInvocation` at 10; no group at
9. `cargo fmt --all --check` exit 0; `cargo clippy -j 4 -p rexx-exec --test api_group_partition --
-D warnings` exit 0.

## Negative control: prediction, then result

Predictions written 2026-09-28 before any control ran.

* **NC1 (the brief's control, scratch copy, never `ootest/`).** Copy `ootest/ooRexx/API` to
  `scratchpad/surface-7/nc1/API`, rewrite `oo/FUNCTIONPackage.cls`'s `LIBRARY orxfunction` to
  `LIBRARY orxclassic` in the copy only, and point a scratch copy of the test's `api_groups_dir()`
  at it (the test file is restored byte-for-byte from a backup afterwards, checked with `cmp`).
  Prediction: `each_api_group_belongs_to_the_phase_its_libraries_import_from` goes red, and the
  derived side of the `assert_eq!` differs from the expected side in exactly one row, `FUNCTION`,
  which derives as `10` (liborxclassic.so NEEDs librexx.so.4 and imports the queue and macro-space
  registries) against the expected `8`. The in-test control
  `a_group_whose_library_changes_changes_phase` copies from `api_groups_dir()`, so under the
  redirect it copies the already-mutated tree, and its `moved` map still expects `FUNCTION` at 8:
  prediction, it is red too, on the `FUNCTION` row alone.
* **NC2 (the owner ruling is live).** Replace `phase.max(...)` with `phase.or(...)` in
  `phase_of`, so the first classified import in name order decides. Prediction: red, and the
  derived map differs in exactly the groups that load `INVOCATIONTester.cls` (`RexxStart`,
  `ProcessRexxStart`, `INVOCATION`, `ProcessInvocation`), each deriving `9` because
  `RexxCreateInterpreter` sorts before every registry import of liborxexits.so; `CLASSIC` stays
  `10`, since its first classified import in name order (`RexxAddMacro`) is a registry.

Results.

* **NC1: as predicted.** Both test functions red. Main test: derived
  `{"CLASSIC": 10, "CONVERSION": 8, "FUNCTION": 10, "INVOCATION": 10, "METHOD": 8,
  "ProcessInvocation": 10, "ProcessRexxStart": 10, "RexxStart": 10}` against expected `FUNCTION: 8`,
  and its bindings print `"FUNCTION": (10, {"orxclassic"})`. In-test control: the same `FUNCTION: 10`
  against 8, `METHOD` at 10 as its own rewrite wants. The test file was restored from the backup and
  `cmp` was silent; the rerun after restoring passed 22.
* **NC2: as predicted.** Main test red with `INVOCATION`, `ProcessInvocation`, `ProcessRexxStart`
  and `RexxStart` at 9 and every other row unchanged, `CLASSIC` 10. Not predicted but consistent:
  the in-test control went red too, `METHOD` at 9 against 10, for the same reason. Restored and
  re-run: 22 passed, with a `Compiling rexx-exec` line.

## Step 3 -- record corrections

Each said the `INVOCATIONTester.cls` groups were Phase 9's; each now says Phase 10's, with the date
and why, and points at the test:

* `docs/superpowers/plans/2026-07-27-rust-rewrite.md` (`313807bc5`): row 8's "the other five are rows
  9 and 10's" now says every other API group is row 10's; row 9's sentence claiming those groups is
  replaced by one saying no API group is Phase 9's and where they went; row 10 now claims every
  group but `METHOD`, `CONVERSION` and `FUNCTION`, with the `orxinvocation` chain and its registry
  imports beside the `CLASSIC` evidence.
* `docs/superpowers/plans/phase-4-exclusions.txt` (`3bb1d34d2`): the RE-HOMED sentence.
* `docs/superpowers/plans/phase-8-gate.md` section 7's "Refusals this phase re-homed" bullet, which
  repeated the Phase 9 claim.
* `docs/superpowers/plans/2026-09-14-phase-8-surface.md`'s "group partition was wrong" bullet, the
  plan's own statement of the same claim.

Left as written: the dated records under `docs/superpowers/records/` and `docs/superpowers/specs/`,
which record what was believed then. Found by `/bin/grep -a -rln -e ProcessRexxStart -e
ProcessInvocation docs rust .superpowers`.

Stray file: `ootest/address-with.tmp` (empty, mtime 2026-07-27 17:51) is
`ooRexx/base/keyword/ADDRESS.testGroup:50`'s `::constant temporaryFilename "address-with.tmp"`,
a relative name, so it lands in the working directory of whatever ran that group; a run with
`ootest/` as its working directory on 2026-07-27 left it. Left alone.

## Commits

* `844ed9e0a` Derive the API group partition, and move the invocation groups to Phase 10 (the
  test, `phase-8-gate.md` sections 7 and 9, the roadmap, `phase-4-exclusions.txt`, the surface
  plan). This report is not in it: `.superpowers` is gitignored here, and `git add` refused it.

## Gate status lines

`scratchpad/surface-7/gates.sh` (surface-6's with `S` changed), run in the background after the
commit, tree untouched until `finished`:

    844ed9e0a81b54c936d9045b62b96886e36e58a8
    started 2026-09-28T06:26:44+02:00
    G1 fmt exit 0
    G2 clippy(empty target) exit 0
    G3 release build (test --no-run) exit 0
    G4 release test exit 0
    G4 Compiling lines: 0
    G5 debug build (test --no-run) exit 0
    G6 debug test exit 0
    G6 Compiling lines: 0
    844ed9e0a81b54c936d9045b62b96886e36e58a8
    finished 2026-09-28T06:40:11+02:00

`git status --short` empty at the end. Summed `test result` lines: G4 2731 passed / 0 failed, G6
2732 / 0; both partition tests `ok` in each. No re-run was needed.

## Concerns

* The frozen-header witness rests on the oracle build of 2026-08-05 matching today's committed
  sources; three oracle `testbinaries/` files have a later mtime (2026-08-22) with no content change
  recorded by git. Recorded in the gate section; not re-proved by rebuilding (forbidden).
* `NEUTRAL` (memory, variable pool) is a classification the ruling did not name; it changes no
  row today.
* The dated records under `docs/superpowers/records/` and `specs/` still say Phase 9, deliberately.
* The task report is outside git, per the ignore rule.
