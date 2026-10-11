# Phase 9 scoping: decisions by Moritz, 2026-10-10

* **`rxqueue` and `rxsubcom` move to Phase 10**, unless the survey shows a Phase 9 item they block.
  Reason: both need RXAPI, and D7 is undecided and executes in Phase 10.
* **"Every benchmark at parity" is struck from Phase 9.** Moritz: "2x is achievable, 1x will be
  difficult as-is." Parity becomes its own work item, outside Phase 9; the spec says where it goes.
* **Phase 9 splits into sub-phases** (suite conformance, binaries, and whatever else the survey
  finds), each with its own spec section and gate, as Phase 4 did.
* **Phase 6.1 scratch deleted.** `/tmp/claude-1000/p61` is removed. The base61 binary was kept at
  `/tmp/claude-1000/base61/rexx-run`, sha256
  `2ea19b3ea2875fada8718eaf5b89f828d5ae07587a74feaf9b34681c879c891e` (unchanged by the copy).
* **Non-macrospace RexxUtil moves into Phase 9.** The `Sys*` routines refused as Phase 10 (scout D
  item 1) are Phase 9's, except the macrospace routines (`SysAddRexxMacro`, `SysDropRexxMacro`,
  `SysQueryRexxMacro`, `SysReorderRexxMacro`, `SysClearRexxMacroSpace`, `SysLoadRexxMacroSpace`,
  `SysSaveRexxMacroSpace`), which need the RXAPI daemon and stay Phase 10's.
* **`.rxo` payload is the checked source** behind the oracle-shaped prefix and a header the oracle
  rejects (3.903). Moritz: "let's not agree on a stable format yet": the format carries no
  compatibility promise, not even between versions of this crate.
* **Parse-error messages fill every insert**, byte-identical to the oracle. This reverses Phase 3's
  2026-07-28 text deferral for secondary messages.
* **The embedding API moves to Phase 10**: `RexxCreateInterpreter`, `RexxStart`, the `librexx.so`
  cdylib and its SONAME, the second-instance rows, `REFUSING_MEMBERS`, and the `AttachThread` and
  `AddCommandEnvironment` panics. `ReleaseLocalReference` stays in Phase 9.
* **Perf:** Phase 9 keeps a non-regression budget of +0.5% Ir on all programs, against a pin taken
  at Phase 9's start. A new optional row after 9 targets every axis at 2x or better, with 1x as the
  stretch goal.
* **The parity exemption covers Phases 9, 10 and 11** (Moritz, 2026-10-11, on spec-review finding 1).
  The Global Constraints parity rule (roadmap :46) is amended so that row 9.1 owns parity, and every
  phase from 9 on carries only the +0.5% non-regression budget.

Controller rulings on the spec review (`spec-review.md`), 2026-10-11:

* Ruling: `RXFUNCQUERY` stays Phase 10's (it reads the RXAPI registry, `rexx-inventory`'s table), so
  `SysGetMessage.TEST_CATALOG_INVALID` and `TEST_GENCAT`, renamed out by `reaching_rxapi`, are not
  9b witnesses -- finding 11 -- wrong costs two catalog tests moving from row 10 back to 9b.
* Ruling: exclusions split into `phase-9-exclusions.txt` (Phase 10 rows, deleted by Phase 10) and a
  permanent `suite-licences.txt` (Deviation, environment and flaky rows); row 10's "no exclusions"
  reads "no exclusions but the licences file" -- finding 13 -- wrong costs a file rename.
* Ruling: D4 stands (`rexx-run` removed, approved by Moritz), as its own 9c task with a perf re-pin
  on `rexx` by zero-change A/B -- finding 10 -- wrong costs restoring a binary name.
* **Row 9.1 measures wall clock**, with cycles as the stand-in where wall clock is too noisy
  (Moritz, 2026-10-11).
* **Checks** (Moritz, 2026-10-11): a per-task check of at most 5 minutes (fmt, clippy on touched
  crates, the touched test binaries and ooTest groups); the workspace test at sub-phase close; G1-G9,
  TSan, the whole-suite gate and all-program callgrind once at the Phase 9 close. Perf drift between
  closes is accepted as a risk. Mechanical additions are batched several to a task.

Controller rulings on the fable review (`spec-review-fable.md`), 2026-10-11:

* Ruling: the suite harness iterates past a run-ending refusal as scout A did (rename the last
  started test out, re-run, cap 60), and the captured refusal text is the owner check -- finding 1 --
  wrong costs a harness redesign before 9a closes.
* Ruling: `ReleaseLocalReference`'s bar is peak RSS at 2,000,000 iterations at most twice that at
  200,000, from a committed C probe built with the system `cc` -- finding 11 -- wrong costs a
  threshold change.
* Ruling: the perf budget is measured once at the close against `rexx-run` at `a98238be5`; the 9c
  A/B sits inside the budget -- finding 3 -- wrong costs the front end's share of the budget.
