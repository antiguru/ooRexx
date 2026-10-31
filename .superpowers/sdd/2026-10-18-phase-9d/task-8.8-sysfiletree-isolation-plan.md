# SysFileTree child temporary-directory isolation implementation plan

> **For the assigned implementer:** Use the controller's existing execution method and exclusive source/index ownership. The controller supplies the implementation BASE after the active assertion repairs finish. Do not dispatch another owner or reviewer.

**Goal:** Run the unchanged SysFileTree group against a stable, populated private temporary directory on both interpreters.

**Architecture:** Reuse the existing unique group/side work directory and recreated run copy. Only `base/rexxutil/SysFileTree` receives child TMPDIR `run/sysfiletree-tmp`, seeded before execution. Existing preparation resets it on each crate retry, and existing cleanup removes it after diagnostic capture.

**Tech stack:** Existing Rust test harness and filesystem/process APIs, with unchanged protected ooTest input. No runtime or Cargo change.

**Spec:** `/tmp/codex-p9d-t88-sysfiletree-final-diagnosis/diagnosis.md` and `controlled-results.json`. Both interpreters demonstrated stable 1→1 and controlled inter-scan creation 1→2. The historical extra entry's identity remains unproven.

## Constraints

* Exact logical group only; root-level, Windows, and unrelated groups retain their current environments.
* Preserve protected ooTest bytes, all 26 group tests/assertions, existing Rust test identities/comments, public helper signatures, retry policy, caps, and result classification.
* No runtime, Cargo, dependency, profile, licence, or parent-environment mutation. No self-reexecution, sentinel environment variables, new retry framework, sleeps, or race retry loops.
* Commit pure moves before verification, regressions before red execution, and implementation before green execution. Retain first failures and command/source provenance.
* Use the established capped foreground driver, `CARGO_INCREMENTAL=0`, existing target directory, Cargo `-j4`, and bounds at most 520 seconds. Full suite/workspace/profile resumption remains controller-owned.

## Files

* Modify `rust/crates/rexx-exec/tests/suite.rs` for early launch extraction and unchanged public re-exports.
* Create `rust/crates/rexx-exec/tests/suite/launch/mod.rs` for extracted launch functions and two narrow private helpers.
* Modify `rust/crates/rexx-exec/tests/suite/diagnostics.rs` only to record selected TMPDIR and seed names in existing context.
* Add two tests to `rust/crates/rexx-exec/tests/suite/fixtures/tests.rs`, reusing its existing `scratch_run` and settings helpers.
* Create one synthetic fixture: `rust/crates/rexx-exec/tests/suite/fixtures/temporary_directory/base/rexxutil/SysFileTree.testGroup`.

No additional test module hierarchy is needed. The existing fixtures module already uses adjacent `tests.rs`, and existing test names stay unchanged. The launch module follows the integration harness's existing path-module convention. Record the exact file and test population delta at handoff.

## Task 1: Pure launch extraction

Move `run_group_on_oracle`, `run_group_on_oracle_from`, `run_group_on_crate`, `crate_deadline`, `run_group_on_crate_within`, `crate_run_once`, `remove_work`, and `run_group` into `suite/launch/mod.rs`. Preserve their comments, signatures, and parent-level public names through re-exports. Leave preparation, environment construction, marker paths, result recording, judging, and fixture tests in place. Record moved-body equality apart from imports/visibility.

* [ ] Record supplied BASE/status, protected SysFileTree hash, production/Cargo identity, public signatures, and existing test names.
* [ ] Commit the pure move without semantic changes.
* [ ] Run existing `cargo test -j4 -p rexx-exec --test suite fixtures::tests::` through the capped driver with suite gate disabled; retain its actual population and results.

## Task 2: One real-launch regression

Add `sysfiletree_launches_use_distinct_populated_child_temporary_directories`. Use `scratch_run` to create a private source root containing the synthetic fixture at the exact logical group path, then invoke `run_group_on_oracle_from` and `run_group_on_crate_within` sequentially. The fixture requires `.File~temporaryPath` to equal its own run directory's `sysfiletree-tmp`, verifies exactly the seed file and seed directory including kinds/file bytes, and compares File~listFiles with SysFileTree. The old harness must fail this contract because it uses inherited/default TMPDIR instead of that private path.

Before its assertions, the fixture records its absolute TMPDIR and observed names in one fixture-owned observation file outside both enumerated directories and launched work copies. Supply this observation filename through one quoted path literal substituted in the private fixture copy, following the existing private-source-copy pattern; no environment sentinel or public injection interface is needed. The Rust test requires successful GroupRun results, distinct oracle/crate private paths, unchanged `std::env::var_os("TMPDIR")` before/after, removal of observed private work paths on return, and survival of the external observation file. It never writes to or changes the parent's TMPDIR, and the protected group remains untouched.

* [ ] Commit the fixture and regression before running it.
* [ ] Run only this regression and retain its first red output. Confirm failure concerns child TMPDIR/seeds, not fixture syntax or launch infrastructure.

## Task 3: Narrow implementation and selection test

Use `temporary_directory(run: &Path, group: &str) -> Option<PathBuf>` for exact selection/path derivation and `prepare_temporary_directory(run: &Path, group: &str) -> Option<PathBuf>` for creation/seeding. Expose only the path helper to its parent as needed by diagnostics and selection testing; preserve the public helper surface. Seed `stable-file.txt` with exact bytes `b"stable\n"` and an empty ordinary `stable-directory`, without links or timed writers. Reuse owned work/run lifetime rather than adding TempDir, another counter, or global scratch state.

Call preparation immediately after existing `prepare(...)` in oracle launch and `crate_run_once`. The latter already runs per attempt, and `fresh_copy_from` recreates run before seeding, so retries cannot inherit stale private entries. Set oracle TMPDIR only for Some, and set crate TMPDIR after `set_environment`, whose env_clear would otherwise discard it. Leave `crate_side_environment`, unrelated groups, and parent environment unchanged.

Add `sysfiletree_temporary_directory_selection_is_exact`, one table-driven test covering the target, root `SysFileTree`, Windows SysFileTree, and `base/class/Stream`. Require only the exact target to select the private directory, with distinct owned run paths producing distinct results. Extend existing diagnostic context only with the selected path and seed names, preserving raw channels, markers, destinations, and capture-before-cleanup ordering. Existing refusal/retry fixtures exercise preparation reruns; inspect that isolation preparation stays inside that route, without adding a forced-refusal fixture unless a concrete remaining gap is demonstrated to root.

* [ ] Commit implementation before green execution.
* [ ] Run the two added tests, then the complete existing/new fixture module, retaining existing refusal recovery, early-end, marker, child-process, and diagnostic checks.

## Affected verification and handoff

Run the unchanged original group once with `REXX_SUITE_GATE=1`, `REXX_SUITE_GROUPS=base/rexxutil/SysFileTree`, and dedicated table/diagnostic destinations. Use `cargo test -j4 -p rexx-exec --test suite l3_core_suite -- --exact --nocapture` through the capped driver. Require oracle 26/26 and crate 26/26, no new failed/blocked/renamed tests, private populated TMPDIR context, and unchanged protected-source hash. Logs must stay outside the enumerated directories.

* [ ] Run `cargo fmt --all --check` and `cargo clippy -j4 -p rexx-exec --test suite -- -D warnings`; retain failed invocations before correction.
* [ ] Verify exactly two added Rust tests, existing names retained, unchanged production/Cargo/protected inputs, and no licence/row changes.
* [ ] Return committed changes, pure-move proof, red/green records, focused populations, and original-group 26/26 results to root for scoped review. Do not run full-suite, workspace, profiles, or runtime rebuilds here.

## Limits

Isolation prevents external/other-group writers from changing the enumerated directory, while preserving nonempty file/directory metadata coverage. It does not promise an atomic snapshot if the same test deliberately mutates its own directory. Exact selection, both launch routes, parent immutability, unique paths, cleanup, diagnostics, and retry placement are covered by two new tests, existing fixtures, original-group validation, and source inspection. No implementation or verification commands were executed during this plan revision.
