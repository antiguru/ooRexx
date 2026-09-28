# Task 7 review: the prebuilt test extensions, and the group partition

Reviewer: review-7b. Range `c750719b7..844ed9e0a`, HEAD `844ed9e0a`, clean tree throughout. Built the
test into my own `CARGO_TARGET_DIR` under `$S = /tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/review-s7/`
(`Compiling rexx-exec` present, both against the real worktree and against a scratch copy of the
repo built for mutation testing). Both scratch copies and their target dirs were deleted after use;
`$S/predictions.md` (the two independent mutation predictions, written before running either) is the
only file left. `git status --short` in the worktree was empty before, during and after this review.

## Verdict

**Spec compliance: met. Quality: changes requested — one Critical finding (C1); no Important or
Minor findings.** Step 1's byte-identity and build-product claims all reproduced exactly, including
the mtime caveat. Step 2's derivation reproduced independently, from my own `readelf`/`grep`
commands with no reference to the test's code, and agrees with the test's partition row for row: the
three no-interpreter-library groups are exactly METHOD, CONVERSION and FUNCTION, the four
`INVOCATIONTester.cls` groups and CLASSIC are all Phase 10's, and no group is Phase 9's. Both
mutations I designed (a group's library changing, and an import class's target phase moving) killed
the test exactly as predicted. Quality checks (no unsafe, no new dependency, no em-dash, no set-size
language, loud panic when `readelf` is absent) all passed. Step 3's record corrections are true where
made, but incomplete: one sentence from the exact commit (`3bb1d34d2`) the brief named for
correction still calls an `orxinvocation`-based group "a Phase 9 group's."

## C1 (Critical): a stale Phase 9 claim survives in the same file, from the same commit, that Task 7 corrected

`docs/superpowers/plans/phase-4-exclusions.txt:4919`:

    first shipped consumer is orxinvocation.cpp:473-478, a Phase 9 group's.

`git blame -L 4919,4919 -- docs/superpowers/plans/phase-4-exclusions.txt` names `3bb1d34d2c`
(2026-09-15) as the commit that added this line — the same commit the brief calls out by name
("The re-homing is already recorded (`313807bc5`, `3bb1d34d2`); make the test's partition and those
records agree, and where they do not, correct the records"), and the same file whose earlier
RE-HOMED paragraph (originally lines 4618-4631, also from `3bb1d34d2`) this commit did correct. This
sentence sits roughly 300 lines later, under a different bullet ("THE PACKAGE LOADER AND UNLOADER
HOOKS NEVER RUN HERE"), and `844ed9e0a`'s diff never touches it (`git show 844ed9e0a -- docs/superpowers/plans/phase-4-exclusions.txt`
has no hunk anywhere near line 4919). `orxinvocation`-based groups are now Phase 10's per this same
commit's own corrected partition, so "a Phase 9 group's" is now false.

Found by: `/bin/grep -a -n "Phase 9" docs/superpowers/plans/phase-4-exclusions.txt`, then
`git blame -L 4919,4919 -- docs/superpowers/plans/phase-4-exclusions.txt`, then
`git show 3bb1d34d2 -- docs/superpowers/plans/phase-4-exclusions.txt` to confirm the line's origin.
I also grepped every other `docs/superpowers/plans/*.md`/`*.txt` file and `rust/crates` for
`"phase 9"` near `orxinvocation`/`INVOCATION`/`ProcessInvocation`/`ProcessRexxStart` and found no
second miss; this is the only uncorrected instance.

Fix: change "a Phase 9 group's" to "a Phase 10 group's" (or otherwise reflect the corrected
partition) at that line.

## Verification detail

**Step 1 (byte-identity and build products).** Reran from the worktree root against
`/home/moritz/dev/repos/ooRexx`:

    diff -rq api /home/moritz/dev/repos/ooRexx/api            # empty, exit 0
    diff -rq testbinaries /home/moritz/dev/repos/ooRexx/testbinaries  # empty, exit 0

Every `generate_test_library`/`add_library`/`add_executable` target named in `testbinaries/CMakeLists.txt`
has its product in the oracle's `build/lib`/`build/bin`. The oracle's six libraries and
`liborxexits.so`, `libhostemu.so`, `librxregexp.so`, `librxsock.so`, `librxunixsys.so`,
`librexxapi{,1,2,3}.so`, `librexx.so` all carry mtime `2026-08-05 16:02:2x`; `testbinaries/CMakeLists.txt`,
`provoke_locks.cpp` and `provoke_locks.rex` in the oracle checkout carry `2026-08-22 21:21:11`, later,
exactly as the report flags; the oracle's `git status --short api testbinaries` is empty and
`git log -1 -- api testbinaries` there is `334ab5460`, matching the report. Also confirmed:
`testbinaries/CMakeLists.txt:61` links `orxfunction`/`orxmethod`/`orxclassic` against `rexx rexxapi`
via the shared `generate_test_library` macro, yet `liborxmethod.so` and `liborxfunction.so`'s
`NEEDED` is `libc.so.6` alone (the linker dropped the unused libraries), matching the report's note.

**Step 2 (the partition), derived independently.** From the eight `.testGroup` files under
`ootest/ooRexx/API` and their `EXTERNAL "LIBRARY ..."`/`rxfuncadd` directives (own `grep`, not the
test's parser):

| group | library(ies) | own `readelf -d`/`--dyn-syms` result |
|---|---|---|
| METHOD, CONVERSION | `orxmethod` | NEEDED = `libc.so.6` only, no `Rexx*` import |
| FUNCTION | `orxfunction` | NEEDED = `libc.so.6` only, no `Rexx*` import |
| CLASSIC | `orxclassic`, `orxclassic1` (via `rxfuncadd` at `CLASSIC.testGroup:822,861,875`) | NEEDs `librexx.so.4`/`librexxapi.so.4` directly; imports only the queue/macro-space/function/subcom registries, no embedding call |
| RexxStart, ProcessRexxStart, INVOCATION, ProcessInvocation | `orxinvocation` (via `INVOCATIONTester.cls`) | NEEDs `liborxexits.so`, which NEEDs `librexx.so.4`/`librexxapi.so.4` and imports both `RexxCreateInterpreter`/`RexxStart` (embedding) and `RexxRegisterExitDll`, `RexxRegisterExitExe`, `RexxDeregisterExit`, `RexxQueryExit`, `RexxRegisterSubcomExe`, `RexxDeregisterSubcom` (registries) |

This reproduces the test's `PARTITION` exactly: METHOD/CONVERSION/FUNCTION need no interpreter
library (Phase 8), CLASSIC and the four `INVOCATIONTester.cls` groups are Phase 10's (registries
present in every case, and where embedding is also present it is dominated by the registries), and
no group is Phase 9's. Also checked `CONVERSIONPackage.cls`'s directives bind `orxmethod`, not
`orxconversion` (there is no such library) — the group name and its bound library differ, and the
test gets this right. Confirmed the `RexxRegisterExitDll` citation:
`rexxapi/client/RegistrationAPI.cpp:277` calls `registrationManager.registerCallback`, whose body at
`rexxapi/client/LocalRegistrationManager.cpp:70` is the `ClientMessage(RegistrationManager,
REGISTER_LIBRARY, ...)` line the report and gate section 9 cite — real RXAPI-daemon traffic, not
just a link-time detail.

Also checked, not required by the brief: `INVOCATION.testGroup:906`/`ProcessInvocation.testGroup:887`
set `tester~loadLibrary = "rxmath"` at runtime (an interpreter-instance option, not a directive the
test's grammar covers) — `librxmath.so`'s own `NEEDED` is `libm.so.6`/`libc.so.6` and it imports no
`Rexx*` symbol, so including it would not have changed any row; the omission the report notes is
correctly inert.

**Mutation, two independent predictions (`$S/predictions.md`), written before running either.**

*Mutation A — a group's library changes.* Copied `ootest/ooRexx/API` to a scratch tree, rewrote every
`LIBRARY orxmethod` (26 occurrences, confirmed by `grep -c`) to `LIBRARY orxclassic` in
`oo/CONVERSIONPackage.cls` alone (confirmed no other file differs, `diff -rq`), and pointed a scratch
copy of the test's `api_groups_dir()` at it. Predicted red on exactly the CONVERSION row, deriving
10 against the recorded 8, with the in-test control also red on the same row. Result: exactly that —
`bindings: {..., "CONVERSION": (10, {"orxclassic"}), ...}`, both test functions failed, differing
from `expected()` only on CONVERSION.

*Mutation B — an import class moves.* In a scratch copy of the test source, changed
`IMPORT_PHASES` from `[(EMBEDDING, 9), (REGISTRIES, 10)]` to `[(EMBEDDING, 9), (REGISTRIES, 5)]`,
leaving `PARTITION`, `ootest/`, and the oracle build untouched. Predicted red on CLASSIC (now 5, no
embedding import to dominate) and the four `INVOCATIONTester.cls` groups (now 9, embedding
dominating the demoted registries), METHOD/CONVERSION/FUNCTION unaffected (they never reach
`IMPORT_PHASES`). Result: exactly that —
`{"CLASSIC": 5, ..., "INVOCATION": 9, "ProcessInvocation": 9, "ProcessRexxStart": 9, "RexxStart": 9}`,
METHOD/CONVERSION/FUNCTION unchanged at 8; the in-test control additionally failed on METHOD (9
against its own expected 10), as follows from the same demotion.

Both scratch copies of the test source were restored from a saved original and checked with `cmp`
before deletion; the tracked file and the rest of the worktree were never edited (`git status
--short` empty throughout, confirmed after each mutation).

**Loud failure when `readelf` is missing.** Ran the already-built test binary directly with
`PATH` pointed at an empty directory: `each_api_group_belongs_to_the_phase_its_libraries_import_from`
panics at the `readelf could not be run: No such file or directory (os error 2)` message
(`api_group_partition.rs:193`) and the test harness reports it as `FAILED`, not skipped or silently
passed.

**Loading discipline.** `grep -an -e dlopen -e libloading -e 'Library::new'` over the test file
finds nothing: it never loads anything, only invokes `readelf` as a subprocess and reads its
stdout — consistent with the global constraint against mapping `librexx.so`/`librexxapi.so`.

**Quality.** No `unsafe` in the test file. No dependency change (`git diff --stat` on
`Cargo.toml`/`Cargo.lock` between the two commits is empty). No em-dash. No comment states a set's
size (each phase-membership comment names the members or the rule, never a count). Comments are
short and each one I checked lands on the code beside it.

## The testbinaries mtime and NEUTRAL concerns

Both of the implementer's flagged concerns are acceptable as recorded, not blocking:

* **The 2026-08-22 mtime on three `testbinaries/` files** is caveated in the gate section with the
  evidence that would contradict it (a content change) absent: oracle `git status` is empty and the
  last commit touching those paths predates the later mtime. A touch-without-edit (`git checkout`
  re-materializing the same blob, or a bare `touch`) is consistent with everything observed, and
  Global Constraints forbid rebuilding the oracle to settle it further, so recording the caveat
  rather than resolving it is the correct call.
* **`NEUTRAL`'s dead branch** (no group today is neutral-only) is honestly reported as such, its
  panic path exists and is exercised by nothing right now, and I confirmed by reading `phase_of` and
  by direct `readelf --dyn-syms` on `liborxexits.so`/`liborxclassic.so`/`liborxclassic1.so` that no
  library's `Rexx*` imports are neutral-only. This is a true, static fact about the oracle's current
  builds, not a gap in the test's classification.

## Scope not covered by this task

C1's fix is a one-line prose correction in a plan document, not a test or code defect; it does not
require reopening any gate or rerunning any of the derivations above.
