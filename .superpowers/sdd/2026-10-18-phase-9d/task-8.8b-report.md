# Task 8.8b: Harness safety and binary selection

The selected harness paths are implemented and covered at source revision `ecce67dad`, with lint-only guard cleanup at `b850a9c65`. Byte-input writes now run on one detached private writer while the existing readers and deadline run, preserving early-exit status and stdin EOF. Byte-name matching preserves invalid bytes, empty substring search succeeds, and a present empty suite selector fails visibly. Arity uses Cargo's binary artifact, Grandchild attribution includes its run cwd, and benchmark selection uses the configured or executable-derived target directory.

## Dispositions

| Item | Disposition | Evidence |
| --- | --- | --- |
| oracle-harness-stdin-deadlock | Closed `ecce67dad` | Never-read input times out through run_within_env and run_in_with. Finite output-before-input completes through those methods, run_with, and write_and_wait, preserving distinct complete stdout/stderr and exit 23. Early exit preserves 17. |
| arity-harness-ignores-target-dir | Closed `ecce67dad` | Both arity controls select `/tmp/codex-p9d-t88-target/release/rexx`; complete collection and introspection targets pass without table refresh. Controller accepts Cargo artifact ownership as stale-binary protection; no mtime comparison is added. |
| gate 457 | Closed `ecce67dad` | A quoted FF method stays unchanged while the literal UTF-8 replacement-character method is renamed. Exact rewritten bytes and ordinary ASCII folding pass. |
| gate 458 | Closed `ecce67dad` | Empty/nonempty haystacks and needles follow ordinary substring containment. |
| gate 459 minor 8 | Closed `ecce67dad`, gate remains partial | Absent selection keeps supplied groups, valid selections work, unknown names retain the diagnostic, and empty/whitespace/comma-only selectors fail. |
| grandchild-sleeps-process-wide | Closed `ecce67dad` | Unrelated sleep 41.5 in another cwd is ignored; a live owned sleeper is detected and deterministically killed/waited. Existing Grandchild process-group cleanup passes. |
| gate 472 | Closed `ecce67dad` | Absolute, relative, non-UTF8, fallback, PROFILE/deps, and actual alternate-target controls pass. The runnability check executes the same alternate target interpreter. |

## Regression and verification evidence

Committed red controls expose the named instrument failures before implementation. Byte matching and empty substring search fail with exit 101, selector and sleeper attribution controls fail with exit 101, and each arity target selects the old default-target artifact. Oversized never-reader and output-before-input controls exceed an external 20-second bound and exit 124, while the adjacent early-exit control already succeeds. These are harness failures rather than language-semantic disagreements, so a new semantic corpus witness is not applicable.

Focused byte, selector, fixture-ownership, Grandchild, and arity-selection controls pass after implementation. Complete `oracle_deadline` (29 tests), `input_oracle` (30), `collection_arity` (25), `introspection_arity` (27), and `rexx-bench-suite` (15) targets pass. Formatting and clippy for rexx-exec/rexx-bench with all targets and warnings denied pass. The controller owns the full corpus, full suite and row comparison, workspace, mandatory policy checks, and aggregate performance checks for the batch.

The initial full oracle_deadline run succeeded but emitted a retained-cgroup diagnostic because its existing orphan-output fixture deliberately left a 30-second sleep running after a 10-second reader budget. That run did not record the PID, so only the configured remaining lifetime of approximately 20 seconds is known. The fixture now records `/proc` start identity and checks cwd/command before signaling a pidfd, with a bounded check that distinguishes a terminated zombie from a running process. Its focused rerun recorded PID 305897/start 36392559, passed in 10.02 seconds, and emitted no retained-cgroup diagnostic; [the diagnostic record](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/harness/memcap-diagnostic.txt) retains the original warning.

## Boundaries and rulings

The detached writer is never joined, so a descendant holding stdin cannot extend the bounded Oracle return through writer joining. The existing independent output-reader budget and direct-child termination policy remain in place. `write_and_wait` retains its Output return contract and its pre-existing lack of a child deadline. No interpreter runtime source changed, so the run-path callgrind trigger does not apply to this slice.

Cargo changes test execution cwd to the package directory, so a relative target environment value loses its original invocation directory at runtime. The controller ruled that recognized PROFILE/deps test executables use their actual target root for relative overrides, while absolute overrides remain authoritative and ordinary benchmark binaries resolve relative values against runtime cwd. An actual `cargo test` invocation with relative CARGO_TARGET_DIR selects `/tmp/codex-p9d-t88-target/release/rexx`. A directly launched test executable therefore cannot redirect selection through a relative override; use an absolute override for that case.

Narrow pure moves isolate process I/O, byte matching, selection, fixture ownership, and benchmark target resolution before fixes. Cargo crate roots loaded through explicit paths resolve child modules differently from ordinary module files, which caused the first compile-only failure and was corrected with explicit responsibility roots ending in mod.rs. New regression modules use plain `#[cfg(test)] mod tests;` with adjacent tests.rs files, and existing suite fixture and benchmark test modules remain inline to preserve their paths without a broad suite refactor. Existing comments are preserved, with fixture cleanup and benchmark normalization comments updated to their implemented contracts.

The broad worktree-build precondition remains open, as do gate 459 minors 5/7/9 and its recorded minor 10 disposition at full-gate completion. Wall-clock timeout policy, probabilistic UNSTABLE judgement, and the independent stderr-refusal load flake remain open. No new Deviation is claimed, and RAISE DESCRIPTION timing still needs Moritz's ruling. Tasks 9/10 and Phase 9/10 closure remain outside this slice.

## Commits

| Commit | Purpose |
| --- | --- |
| c9d3680a6 | Pure helper moves and controller's final 8.8a review metadata |
| 2c43f60cf | Regression controls and behavior-preserving input seams |
| 8b1116dd7 | Explicit responsibility module roots after compile diagnostics |
| 928affcfd | Actual benchmark build-target control |
| f7b66267d | Selected safety fixes and controller recovery metadata |
| ad50c78cd | Relative Cargo target regression |
| ecce67dad | Cargo relative-target correction and owned orphan cleanup |
| dd988fc32 | Formatting of orphan cleanup |
| b850a9c65 | Equivalent guard let-chains for clippy |

## Commands and statuses

All commands below ran from the workspace through the recorded foreground wrapper, with Cargo executing from `rust/`. The wrapper sets CARGO_INCREMENTAL=0, CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target, and REXX_CORPUS_GATE=1, then captures output without a pipeline. Its timeout terminates its owned subprocess group, and every build/test uses Cargo -j 4. The exact wrapper is retained as [check.py](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/harness/check.py), and per-command logs use the record name below.

| Record | Exact command | Exit | Seconds |
| --- | --- | --- | --- |
| `red-bytes` | `memcap 8G python3 /tmp/codex-t88b-check.py red-bytes 180 cargo test -j 4 --release -p rexx-exec --test concurrency_tests group_runner::bytes::tests` | 101 | 95.676 |
| `red-bytes-r1` | `memcap 8G python3 /tmp/codex-t88b-check.py red-bytes-r1 180 cargo test -j 4 --release -p rexx-exec --test concurrency_tests group_runner::bytes::tests` | 101 | 79.347 |
| `red-build-harness` | `memcap 8G python3 /tmp/codex-t88b-check.py red-build-harness 300 cargo test -j 4 --release -p rexx-exec --test suite --test oracle_deadline --test collection_arity --test introspection_arity --no-run` | 0 | 26.423 |
| `red-selection` | `memcap 8G python3 /tmp/codex-t88b-check.py red-selection 60 cargo test -j 4 --release -p rexx-exec --test suite selection::tests` | 101 | 0.064 |
| `red-ownership` | `memcap 8G python3 /tmp/codex-t88b-check.py red-ownership 60 cargo test -j 4 --release -p rexx-exec --test suite process_ownership::tests` | 101 | 0.064 |
| `red-arity` | `memcap 8G python3 /tmp/codex-t88b-check.py red-arity 60 cargo test -j 4 --release -p rexx-exec --test collection_arity --test introspection_arity arity_runs_the_cargo_owned_binary` | 101 | 0.064 |
| `red-arity-introspection` | `memcap 8G python3 /tmp/codex-t88b-check.py red-arity-introspection 60 cargo test -j 4 --release -p rexx-exec --test introspection_arity arity_runs_the_cargo_owned_binary` | 101 | 0.064 |
| `red-oracle-deadlines` | `memcap 8G python3 /tmp/codex-t88b-check.py red-oracle-deadlines 20 cargo test -j 4 --release -p rexx-exec --test oracle_deadline oversized_stdin` | 124 | 20.004 |
| `red-output-before-input` | `memcap 8G python3 /tmp/codex-t88b-check.py red-output-before-input 20 cargo test -j 4 --release -p rexx-exec --test oracle_deadline output_before_input` | 124 | 20.004 |
| `red-bench-resolver` | `memcap 8G python3 /tmp/codex-t88b-check.py red-bench-resolver 240 cargo test -j 4 --release -p rexx-bench --bin rexx-bench-suite binary::tests -- --nocapture` | 101 | 27.022 |
| `format` | `memcap 8G python3 /tmp/codex-t88b-check.py format 60 cargo fmt --all` | 0 | 2.269 |
| `green-build-targets` | `memcap 8G python3 /tmp/codex-t88b-check.py green-build-targets 300 cargo test -j 4 --release -p rexx-exec --test concurrency_tests --test suite --test oracle_deadline --test input_oracle --test collection_arity --test introspection_arity --no-run` | 0 | 81.084 |
| `green-bytes` | `memcap 8G python3 /tmp/codex-t88b-check.py green-bytes 90 cargo test -j 4 --release -p rexx-exec --test concurrency_tests group_runner::bytes::tests` | 0 | 0.064 |
| `green-existing-byte-copy` | `memcap 8G python3 /tmp/codex-t88b-check.py green-existing-byte-copy 90 cargo test -j 4 --release -p rexx-exec --test concurrency_tests skip_changes_only_the_renamed_methods_line` | 0 | 0.064 |
| `green-selection` | `memcap 8G python3 /tmp/codex-t88b-check.py green-selection 90 cargo test -j 4 --release -p rexx-exec --test suite selection::tests` | 0 | 0.064 |
| `green-ownership` | `memcap 8G python3 /tmp/codex-t88b-check.py green-ownership 90 cargo test -j 4 --release -p rexx-exec --test suite process_ownership::tests` | 0 | 0.114 |
| `green-grandchild` | `memcap 8G python3 /tmp/codex-t88b-check.py green-grandchild 90 cargo test -j 4 --release -p rexx-exec --test suite fixtures::a_command_a_test_leaves_running_is_killed_with_the_run` | 0 | 0.214 |
| `green-arity-selection` | `memcap 8G python3 /tmp/codex-t88b-check.py green-arity-selection 90 cargo test -j 4 --release -p rexx-exec --test collection_arity --test introspection_arity arity_runs_the_cargo_owned_binary` | 0 | 0.064 |
| `green-oracle-deadline` | `memcap 8G python3 /tmp/codex-t88b-check.py green-oracle-deadline 90 cargo test -j 4 --release -p rexx-exec --test oracle_deadline` | 0 | 10.084 |
| `green-input-oracle` | `memcap 8G python3 /tmp/codex-t88b-check.py green-input-oracle 180 cargo test -j 4 --release -p rexx-exec --test input_oracle` | 0 | 0.816 |
| `red-bench-relative-cargo` | `memcap 8G python3 /tmp/codex-t88b-check.py red-bench-relative-cargo 120 cargo test -j 4 --release -p rexx-bench --bin rexx-bench-suite relative_target_under_cargo_tests` | 101 | 6.980 |
| `format-r1` | `memcap 8G python3 /tmp/codex-t88b-check.py format-r1 60 cargo fmt --all` | 0 | 2.269 |
| `green-bench` | `memcap 8G python3 /tmp/codex-t88b-check.py green-bench 180 cargo test -j 4 --release -p rexx-bench --bin rexx-bench-suite -- --nocapture` | 0 | 7.330 |
| `green-relative-cargo` | `memcap 8G python3 /tmp/codex-t88b-check.py green-relative-cargo 120 env CARGO_TARGET_DIR=../../../../../../tmp/codex-p9d-t88-target cargo test -j 4 --release -p rexx-bench --bin rexx-bench-suite runtime_candidates_use_this_build_target -- --nocapture` | 0 | 0.064 |
| `green-orphan-cleanup` | `memcap 8G python3 /tmp/codex-t88b-check.py green-orphan-cleanup 120 cargo test -j 4 --release -p rexx-exec --test oracle_deadline a_background_process_holding_the_pipe_open -- --nocapture` | 0 | 21.809 |
| `format-r2` | `memcap 8G python3 /tmp/codex-t88b-check.py format-r2 60 cargo fmt --all` | 0 | 2.269 |
| `green-arity-targets` | `memcap 8G python3 /tmp/codex-t88b-check.py green-arity-targets 180 cargo test -j 4 --release -p rexx-exec --test collection_arity --test introspection_arity` | 0 | 18.753 |
| `green-fmt` | `memcap 8G python3 /tmp/codex-t88b-check.py green-fmt 60 cargo fmt --all --check` | 0 | 2.319 |
| `green-clippy` | `memcap 8G python3 /tmp/codex-t88b-check.py green-clippy 300 cargo clippy -j 4 -p rexx-exec -p rexx-bench --all-targets -- -D warnings` | 101 | 10.437 |
| `format-r3` | `memcap 8G python3 /tmp/codex-t88b-check.py format-r3 60 cargo fmt --all` | 0 | 2.269 |
| `green-clippy-r1` | `memcap 8G python3 /tmp/codex-t88b-check.py green-clippy-r1 300 cargo clippy -j 4 -p rexx-exec -p rexx-bench --all-targets -- -D warnings` | 0 | 0.516 |
| `green-fmt-r1` | `memcap 8G python3 /tmp/codex-t88b-check.py green-fmt-r1 60 cargo fmt --all --check` | 0 | 2.319 |
