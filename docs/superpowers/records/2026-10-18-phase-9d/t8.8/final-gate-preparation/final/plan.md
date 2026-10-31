# Final affected-check reconciliation plan

> For the assigned validator: use the existing root-controlled foreground execution method. No subagents, source changes, Git mutations, builds, tests, config or freeze are authorized by this preparation document. Execution starts only after the settled isolation repair passes review and root supplies the final committed HEAD and execution authorization.

**Goal:** Execute fresh final validation on the settled committed source while preserving all previous failures and their original context.

**Architecture:** Copy the accepted corrected runner into a separate final context, keeping its bounded Cargo-artifact dispatch and one-target-per-action model. Root declined a 151 retained targets and five fresh targets substitute because `rust/CLAUDE.md:76` requires the workspace test against the committed tree. Every default target, doctest and pinning test therefore executes freshly, while the previous results remain historical evidence.

**Tools:** Python, Cargo-owned harnesses, `memcap`, existing delta helper and the fixed `/tmp/codex-p9d-t88-target` cache.

**Binding inputs:** Task 8.8 dispatch, batch header, global constraints, `rust/CLAUDE.md:75–79`, approved final-test repairs and `.superpowers/sdd/2026-10-18-phase-9d/task-8.8-sysfiletree-isolation-plan.md`.

## Current state and readiness

The earlier source was `6b6f3fb460049c6ef63f565ceb78e11b5582f165`, with all 156 default targets executed, 3,801 tests passed, two failed and five existing ignored. Its 15 doctests and seven pinning tests passed, and both corpus profiles matched 1,242/1,242 programs. The two assertion repairs are scoped-review ADDRESSED, while SysFileTree isolation is evolving from BASE `4714f6521`, with launch extraction `0d8d1b29c` already committed. None of those provisional metadata revisions is a final freeze input.

The accepted runner is `/tmp/codex-p9d-t88-final-gate-resume.py`, SHA256 `af2bfb41da1d71930e2b7eb3c09319d16ac685609365dce8aa9a8dc1587f85e4`. Its output/config and attempt 1 remain immutable historical records, including their negative audits and first failures. The proposed final driver, output, config and bridge paths below are currently absent, and preparation has not created or executed them. `/tmp` had 19,568,574,464 bytes free at this inspection, sufficient for the prior 165-second debug-build scale plus changed harnesses and records, but available space must be rechecked before execution without deleting caches.

## Separate final context and narrow changes

* Driver: `/tmp/codex-p9d-t88-final-gate-final.py`.
* Output: `/tmp/codex-p9d-t88-final-gates-final/`.
* Config: `/tmp/codex-p9d-t88-final-final-config.json`.
* Profile attestation: `/tmp/codex-p9d-t88-final-final-profile-completion.json`.
* Source bridge: `/tmp/codex-p9d-t88-final-source-bridge.json`.
* Delta helper: `/tmp/codex-p9d-t88-final-final-suite-delta.py`, copied from accepted helper SHA256 `058a7adb629464b79e2117e57528d4045ae73d032b7bd3d039b31b7dc5b773a5` with only the output-root change.

Copy the accepted runner and retain its exact original bytes, final diff and both hashes before root review. Change only the fixed driver/config/output/helper paths, final test-input provenance bridge and exact expected two-test addition. Preserve the accepted three-path watchdog fix, two explicit added-target keys, four historical renames and strict seven-child nested-summary reconciliation. Do not add retained-target adoption or a generalized mutable-path, mismatch, retry or reuse mechanism.

The six config fields are `source_commit`, `profile_completion_record`, `release_binary`, `release_binary_sha256`, `release_pin_provenance`, and `validation_source_bridge`. Root supplies `source_commit` only after all source, review and runner records are committed, and all referenced files are hashed before freeze. The immutable normal pin stays `/tmp/codex-p9d-t88-pins/8.8-corpus-registration-rexx`, SHA256 `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`, with existing producer provenance attributed to `f65066b04fe1f2355ce64d8be581fa9e79641673`. The attestation cites the existing 1,683-profile manifest and explicit unchanged-production/normal-byte bridge, without claiming a new profiling execution.

The existing provenance helper requires the producer and validator's full Rust trees to match, which is now deliberately false because test inputs changed. Replace only that equality with a reviewed exact bridge: retain producer source/tree identity, require the full 300-entry production/Cargo manifest and bytes to match, require the existing eleven registration inputs to remain unchanged, and enumerate the exact final test-only delta from the earlier validation source with before/after Git identities and current hashes. Require the delta paths to be exactly the settled reviewed assertion/isolation files, including added/moved files, without accepting arbitrary `tests/` changes. Cite both scoped reviews and pure-move proofs in that bridge, and reject any changed production, Cargo, dependency, profile, licence, corpus or protected ooTest input.

The measured normal pin remains distinct from the Cargo-test CLI SHA256 `9c3d09f9f9e39810985b1200007035488a3eba9d00ab6ee7382b6d7aea1b4d06`. Preserve the attempt 1 causal rustix/pty proof and its attribution, the exact resolved rustix features/profile checks and the normal-byte bridge. Fresh release Cargo JSON must again identify that exact test CLI, current suite/corpus harnesses, source paths and profiles before either semantic action receives credit. A changed suite harness hash is expected from reviewed test source, but it is computed from the fresh artifact and pinned for both before/after checks rather than compared to an obsolete expected hash.

## Populations and context safeguards

Fresh metadata must still report 156 default target keys: all 154 historical targets plus `rexx-api::concrete_owners` and `rexx-exec::semantic_boundaries`. Derive the actual default population from metadata and Cargo artifacts, reject removals, unexplained additions, duplicates, unsupported target kinds, custom harnesses or required-feature omissions, and retain exact package working directories and argv. Compare each fresh test-name list against the actual 156-target lists from the earlier validation, allowing only the two reviewed suite additions below, while retaining the historical four-rename proof. Fresh expected total is 3,810 test identities, derived as the earlier 3,808 plus two new test functions, rather than a substitute for listing each target.

* `fixtures::tests::sysfiletree_launches_use_distinct_populated_child_temporary_directories`.
* `fixtures::tests::sysfiletree_temporary_directory_selection_is_exact`.

The suite target should grow from 68 to 70 identities, concurrency remains 63 and dispatch-seam remains six, with all existing names unchanged. All 15 doctest identities and their semantics remain required, comprising four run, ten compile-fail and one compile-only, and the pinning module remains exactly seven tests. The unchanged ignore population is five overall, including one suite diagnostic test and one corpus test, while the outer default executions must have zero filtered/measured tests. If all fresh checks succeed, the expected default outcome is 3,805 passed and five ignored, but only actual populated summaries establish the result.

Each command inherits only the execution environment, without serializing it wholesale. Keep the accepted sanitation of codegen/profile/target-runner overrides and exact existing nonsecret Cargo/package/runtime-library whitelist, including `REXX_CORPUS_GATE`, `REXX_SUITE_GATE`, `REXX_SUITE_TABLE` and `REXX_SUITE_DIAGNOSTICS`. Add only `TMPDIR` to that whitelist because parent/child temporary-directory identity is the new harness contract, recording its absence explicitly when unset. Keep stdout, stderr and status separate, never serialize a broad `CARGO_*` or `REXX_*` prefix, and never lower test/probe coverage or timeouts.

## Bounded execution sequence after approval

Every line below is a separate foreground tool invocation, with no shell loop, leading assignment, redirect, background watcher or external timeout wrapper. Every action has the accepted 535-second deadline, with at most two seconds to reap after the deadline, keeping the foreground command below 540 seconds. Cargo uses `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, `--locked` and jobs four, with an 8 GiB outer memory cap and existing 2 GiB interpreter probes. Use saved `memcap` prefix escalation only when actual sandbox permissions require it, and preserve any first refusal/failure before continuing independent checks.

```text
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py freeze
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py preflight
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py provenance
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py fmt
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py clippy
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py release-build
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py release-corpus
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py release-suite
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py suite-delta
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py metadata
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py build
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py dispatch
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py queue
```

Preflight precedes every build/test stage and checks actual checkout-local native/toolchain prerequisites. The `release-build` action retains the existing preflight-wrapped Cargo no-run JSON command for `corpus` and `suite`, and `clippy` is the full workspace/all-targets command with warnings denied. The release-corpus action must actually match all 1,242 gated programs, and the release-suite action must run only exact `l3_core_suite` with no group selector and fresh dedicated table/diagnostic paths. Keep the suite run even if another independent check failed, and run suite-delta even when release-suite is nonzero.

The full release suite must produce all 402 groups at revision 13178 and all 24,523 per-test keys, with nonzero oracle/crate counts. Require protected SysFileTree source bytes unchanged, both sides 26/26, and recorded distinct populated private child TMPDIR paths/seeds with diagnostic capture before cleanup. Snapshot the same configured static ooTest/framework/driver/oracle/class/sample/native-library roots before/after, extended automatically by the new suite launch/fixture paths under the existing suite directory root. Exclude only repository administrative metadata and mutable run directories, and do not claim to hash arbitrary operating-system, proc or PATH-tool state.

The exact outer join remains `(group, test)` against `docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r4/suite/tests.tsv`. Retain old/new row licence, oracle outcome and crate outcome separately, with zero unexplained key additions/removals and every changed row printed. Also record the same join against the earlier resumed attempt to identify the repaired SysFileTree row explicitly, without treating the old record as a new execution. Stable out-of-scope Phase 10 timeout rows and accepted licences are reported as actual unchanged rows, not deleted or newly licensed.

The debug build is full `cargo test --workspace --no-run --locked -j4 --message-format=json`, and fresh profile records must retain debug assertions and overflow checks. The dispatch command remains explicit `--workspace --lib --bins --tests --no-fail-fast`, after metadata proves parity with default targets, so it captures actual Cargo argv/cwd/environment without running tests or doctests. The queue must emit all 156 separate `target INDEX` commands in actual artifact order, and each is executed once with a 15-second list bound and 510-second full unfiltered run bound. A target's historical record never substitutes for this fresh invocation, and actual test failures do not stop later independent targets.

After the queue, execute each emitted line separately, for example index 0 below, and finish the three dedicated actions only after all 156 target processes stopped.

```text
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py target 0
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py doc
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py pinning
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-final.py audit
```

The example `target 0` is one line of the actual 156-line emitted command file, not permission to omit the other 155. Full concurrency previously took 475.42 seconds, so its 510-second child bound has limited headroom and must not be silently extended if host load causes a timeout. Full collect stress, gated debug corpus, all source/coverage/refusal/owner/closed/phase-ten/representation checks, semantic fixtures and suite fixtures execute in their default harnesses once rather than being repeated through extra selectors. The suite fixture harness runs with `REXX_SUITE_GATE` unset, so its default `l3_core_suite` test contributes no second full semantic suite, which is covered separately by the release action.

## Dependency analysis and retention boundary

Five default targets are directly affected by the repairs or their dynamic scan roots: `rexx-exec::{dispatch_seam,concurrency_tests,suite,ir_recorded}` and `rexx-core::unsafe_sites`. The first three compile changed test source, `ir_recorded` scans top-level sibling `.rs` text at runtime, and `unsafe_sites` recursively scans every workspace Rust source including tests. No other default target imports `suite.rs`; `api_group_tests` and concurrency share unchanged `support/group_runner.rs`, not the extracted suite launch implementation. Existing `include_str!`/`include_bytes!` dependencies point to unchanged production/corpus fixtures, including concrete FFI owners, semantic stdout fixtures and the pinning sleeper input.

Other source-policy scanners cover unchanged `src/` trees or unchanged explicit owner tables, but they still execute freshly under root's committed-workspace requirement. After source settles, use fresh Cargo metadata, artifact profiles, compiler fingerprints and exact `.d` dependency paths to verify that the actual affected closure agrees with this analysis, and expand the declared affected set if shared support changed. Preserve those input/compiler/harness identities for all fresh results without asserting that byte-identical executables alone prove unchanged filesystem inputs. This mapping explains expected rebuild scope and review scope only, and grants no permission to adopt any of the 151 other historical runs as final green results.

The earlier negative audit remains negative and immutable, with its first stop at dispatch target 118 and the separately retained TraceObject and SysFileTree failures. A new final audit may supersede those findings only by linking the reviewed repairs, old failure records and complete fresh positive records on the new frozen HEAD. It must require all 156 fresh target records, exact test and ignore populations, zero failed/filtered/measured tests and timeouts, populated doctest/pinning evidence and all required named gates. If any check fails, retain that first failure, continue independent checks, produce an explicitly negative completeness report and return control to root for any source-changing repair.

## Remaining inputs before execution

* Settled isolation source commit, its scoped review acceptance, exact changed-file manifest and two added-test identities.
* Root's final full HEAD after source, review and runner evidence commits, followed by clean tracked/index checks.
* Root-inspected final driver/helper diff and hashes, actual config/source bridge/profile-attestation hashes, and absence of prior final output/freeze markers.
* Fresh disk-space check and static protected-input identities, followed by explicit root execution/freeze authorization.

No normal production, Cargo or dependency change is authorized by either repair, so unchanged production hashes and normal `d921…` bytes require no new performance cohort. Original performance-budget acceptance and original failure dispositions remain separate root/user decisions. This plan prepares functional validation and does not claim Task 8.8 or phase closure. No final gate was executed during preparation.

Preparation update: the originally proposed helper name already held the immutable attempt1 helper. Root accepted `/tmp/codex-p9d-t88-final-final-suite-delta.py` instead, and its original bytes remain unchanged. Both scoped reviews are now accepted and committed at `dc8b8caf83e933a3115219d352b2e7306dbdfb8c`, and the exact source bridge uses settled Rust tree `ac446500e03c18b5143f2967ca30015e6d64b95f`. Final HEAD/config/profile attestation and execution authorization remain pending.
