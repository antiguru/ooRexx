# Task 8.8 final gate plan

This is an unexecuted controller plan for the final committed Task 8.8 source. The controller must supply the final full source commit and release pin after the repair owner finishes. No current HEAD, previous successful slice, or prerequisite enumeration is final-gate evidence. Tasks 9 and 10 remain outside this plan.

## Inputs and bounds

The retained prerequisite runner is `/tmp/codex-p9d-t88-prereq-enumeration/driver.py`, with its durable archive at `docs/superpowers/records/2026-10-18-phase-9d/t8.8/prerequisites/raw.tar.gz`. Its completed enumeration at `87357a519cdac3cbfc79810dc58026d525c0c6d9` selected 154 default test binaries, with 3,769 listed tests, and separately completed 15 doctests. Task 8.8f subsequently added the `rexx-exec` integration target `semantic_boundaries`, which is explicitly reconciled as an addition and must run in full. Final binary and test counts come from fresh metadata, Cargo artifacts, and complete listings, while the historical counts remain provenance only.

The owned draft `/tmp/codex-p9d-t88-final-gate-draft.py` reuses the old runner's Cargo JSON selection, package-directory execution, separate descriptors, bounded children, and standalone Cargo doctests. Each invocation runs one action, with an internal 535-second alarm that kills its active process group and allows at most two seconds to reap it. A default target action allows 15 seconds for listing and 510 seconds for its tests, with bounded termination under that action deadline. Run each generated target command as a separate foreground command, never one enclosing shell loop over the whole population.

Every actual execution invocation starts with plain `memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py ACTION`, preserving the approved `memcap` prefix. Do not add leading environment assignments, shell redirects, heredocs, globs, or a `timeout` prefix to these invocations. The driver owns cwd, timeout, output capture, and per-step environment, setting `CARGO_INCREMENTAL=0`, the required target directory, and Cargo jobs internally. It removes inherited `REXX_*`, codegen/profile overrides, build-target overrides, and target runners, then serializes only the exact nonsecret environment whitelist.

Before execution the controller writes `/tmp/codex-p9d-t88-final-gate-config.json` with the following schema and real supplied values. This proposed config is not existing evidence, and its release binary names an immutable retained final pin rather than a mutable Cargo output. The `freeze` action records the config hash and checks that binary's SHA256, while each later action refuses changed config, source, staging, or release pin. No shell export is needed, and credentials do not belong in this config.

```json
{
  "source_commit": "<controller-supplied-final-full-sha>",
  "profile_completion_record": "/tmp/codex-p9d-t88-final-profile-completion.json",
  "release_binary": "<controller-supplied-immutable-release-pin-path>",
  "release_binary_sha256": "<controller-supplied-release-pin-sha256>"
}
```

```bash
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py freeze
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py preflight
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py provenance
```

The preflight must pass with this checkout's real `build/lib`, `ootest`, and `oodocs`, without substitutions from the oracle tree. The full suite additionally validates the actual ooTest SVN revision through its existing `suite_revision()` path. Commit source and its report placeholders before the gates, verify that HEAD equals the supplied SHA, then freeze source/test inputs and index staging until every command finishes. Retain command, cwd, allowed environment, separate stdout/stderr, exit status, timeout flag, elapsed time, and source pin for every command, including failures.

Profiling and source repairs must complete before the blanket freeze begins. The parent first settles and commits its tracked performance/report outputs, then writes the config's `profile_completion_record` as JSON with `source_commit` equal to the final supplied SHA, `profiling_complete: true`, and `active_profile_commands: []`. The `freeze` action copies that completion record, records an empty evidence-output allowlist, and snapshots the full `git ls-files --stage -z` identity before any broad gate. A dirty tracked evidence file blocks freeze initiation without claiming the interpreter source changed, and no concurrent tracked report/performance update or index staging is allowed during validation.

The tracked/index guards run before and after each draft action, checking fixed HEAD, no tracked diff, unchanged staging identity, and no uncommitted Rust inputs. Keep checkout-local native inputs, ooTest/reference sources, and the oracle tree read-only throughout the same interval. Raw gate outputs stay in the owned scratch directory until all commands complete, so reporting does not invalidate the frozen committed tree. The parent releases the blanket freeze before copying or staging final evidence, and the runner has no mutable-path exception framework.

## Formatting, lint, and release gates

Run the following driver actions separately, with no shell environment wrapper. The driver records each child command, exact cwd, whitelisted environment, separate stdout/stderr, raw exit status, and timeout flag under `/tmp/codex-p9d-t88-final-gates/`. Copy the final records into the proposed durable directory `docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-gates/` after the freeze ends. Keep an initial failure and its diagnostics intact if a later repair requires a new run.

```bash
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py fmt
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py clippy
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py release-build
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py release-corpus
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py release-suite
```

The release corpus command runs the whole target with its full gate enabled. Require its completed nonzero test population, the differential's completed program denominator, zero mismatches, and status 0, rather than accepting a Cargo build or an early gate skip. The suite command selects exactly one Rust harness test but covers every ooTest group because `REXX_SUITE_GROUPS` is absent. Require the final release executable/source pin supplied by the controller to match the source under test, and retain the fresh release Cargo JSON executable identities rather than reusing the prerequisite worktree's binaries.

Task 8.8 is not a phase boundary, so the required target directory remains `/tmp/codex-p9d-t88-target` for lint. `rust/CLAUDE.md` separately requires clean-target lint at a phase boundary, which is not established by a same-directory Task 8.8 lint. Do not delete caches to manufacture that condition during another owner's work. A release build exceeding this bound is BLOCKED evidence for the controller, not authority to enlarge the timeout or reduce its target selection.

## Bounded debug workspace gate

The debug obligation is `REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast`, including default tests and Cargo doctests. `[profile.test]` has `opt-level=3` while retaining debug assertions and overflow checks, unlike release. The prerequisite's measured longest binary was `collect_stress` at 313.185 seconds, the default corpus target took 151.246 seconds, and a monolithic workspace command risks exceeding 540 seconds. The draft therefore separates Cargo build, nonexecuting dispatch capture, each actual default test binary, and native Cargo doctests.

```bash
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py metadata
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py build
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py dispatch
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py target 0
```

The metadata action proves that default `test=true` targets have supported library, binary, or integration kinds, without test-enabled examples or custom harnesses. The build uses `cargo test --workspace --no-run --locked -j4 --message-format=json` and requires exact target-key equality against fresh metadata, with every historical target retained and only the explicitly reconciled `semantic_boundaries` addition. Its key is `(manifest parent directory name, target name, target kind, target crate_types)`, so the prior worktree's package URL prefix is not mistaken for a changed package. Every selected artifact must have an empty feature list, a true test profile, debug assertions, overflow checks, and a Cargo-owned executable.

The dispatch action runs explicit `--lib --bins --tests` after that parity proof, using a temporary target runner which records dispatch and exits without executing the executable. **Dispatch is NOT gate evidence, and its zero exit does not mean tests passed.** Its exact capture count and executable set must equal the fresh artifact selection, and it must capture each Cargo package cwd, complete executable argv, executable hash, and whitelisted nonsecret Cargo package/runtime-library variables. It excludes doctests by explicit target selection, reports unsupported targets instead of excluding them, and never copies binaries or reconstructs executable names from directory scans.

After successful dispatch verification, generate the remaining individual controller commands with the following preparation-only command. Execute each emitted line separately and continue independent target commands after a nonzero test result to preserve `--no-fail-fast` coverage. The draft refuses to overwrite a target record and applies the blanket tracked/index freeze around each action after recorded profiling completion. A target action lists the exact binary, verifies a completed libtest summary against that listed population, rejects filtering or timeout, and records its executable hash.

```bash
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py queue
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py doc
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py audit
```

If target 0 already ran, the controller starts the emitted queue at target 1. Workspace completion requires exactly one complete result per fresh artifact, the entire reconciled fresh population covered, zero target timeouts or failures, matching per-binary list/summary counts, and no unexpected filtered tests. Empty unit harnesses are legitimate and must still execute, while the aggregate nonignored test population and the named required gates below must be nonzero. Added tests are included automatically, including every test in the new `semantic_boundaries` target, while removed tests or unexplained target changes stop for explicit controller reconciliation rather than accepting a smaller population.

The doc action invokes `cargo test --workspace --doc --locked -j4 --no-fail-fast -- --test-threads=1`, retaining Cargo/rustdoc ownership of compilation and execution. Require all 15 semantic doctest identities from the retained `doctest-results.json`, allowing line-number movement only. Their modes remain four executed examples, ten compile-fail examples, and one compile-only example, with no ignored example or unvisited doc target. Status 0, exact identities/modes, and completed doctest summaries are separate requirements from the binary tests.

## Coverage mapping and source checks

Each following obligation is satisfied by its single unfiltered default-profile target execution above. Keep its package, target key, artifact hash, listed test names, actual count, and result in a final coverage table. Do not rerun a successful binary solely to give it another gate label. Release corpus and release suite remain distinct release obligations, while collect stress and debug corpus must retain debug assertions.

| Obligation | Default-profile target |
| --- | --- |
| Source-line behavior and oracle expectations | `rexx-parse`: `sourceline`, `sourceline_oracle` |
| Admitted keyword/directive and witness coverage | `rexx-exec`: `coverage` |
| Refusal tables and source-site derivation | `rexx-exec`: `refusal_dispositions`, `refusal_sites` |
| Ownership and closed-phase policy | `rexx-exec`: `owners`, `closed_phases`, `phase_ten_reasons` |
| Representation policy with unchanged `ALLOWED` | `rexx-exec`: `representation_refusals` |
| Granted unsafe-site inventory | `rexx-core`: `unsafe_sites` |
| Collection on allocation with debug tripwires | `rexx-exec`: `collect_stress` |
| Full differential with debug tripwires | `rexx-exec`: `corpus`, `REXX_CORPUS_GATE=1` |
| Stderr-refusal fixture and suite instrument tests | `rexx-exec`: `suite`, full default target, diagnostics enabled |
| Task 8.8f semantic boundary regressions | `rexx-exec`: `semantic_boundaries`, full newly added target |
| Other default unit/integration regressions | Every remaining fresh Cargo artifact |

No oracle refresh or refusal-table refresh variable is enabled during these checks. In particular `REXX_REFUSAL_SITES_REFRESH`, oracle expectation regeneration, docs-less modes, suite selectors, calibration settings, and simulation overrides remain absent. Compare the final tracked/index freeze identities before and after, confirm `ALLOWED` did not change relative to Task 8.8's supplied source base, and inspect `unsafe_sites` output rather than inferring no new site from compilation. The source owner supplies any missing final/base pin, and all profiling/report updates must precede the blanket freeze.

## Whole-suite join and diagnostics

The exact prior full record is `docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r4/suite/tests.tsv`. A read of that file and its companion tables found revision 13178, `# groups all`, 24,523 unique `(group,test)` keys, and 402 groups. Its schema is `group<TAB>test<TAB>row<TAB>oracle<TAB>crate`, while `table.tsv` has one `(group,side)` row per side and `verdicts.tsv` has one group row. The newly proposed `/tmp/codex-p9d-t88-final-gates/suite/tests.tsv` is emitted beside `REXX_SUITE_TABLE` before the harness's green assertion, so a failing final assertion can still have complete comparison evidence.

Perform a full outer join by the exact case-sensitive `(group,test)` pair, retaining `row`, `oracle`, and `crate` separately on both sides. Emit every change in any of those fields, plus every added or removed key, rather than comparing only aggregate passing counts. Reject duplicate keys, wrong headers, missing revision/full-groups markers, missing output companions, or unequal full-group populations before calling the record complete. Reconcile each added/removed key and each changed verdict explicitly, including improvements, oracle changes, owner changes, and regressions.

The owned `/tmp/codex-p9d-t88-final-suite-delta.py` contains the exact join, invoked only through the driver after the suite command completes. It writes the delta before enforcing population completeness, so a population failure still retains the discrepant keys. The final status and per-group verdicts remain evidence to assess separately from the join. Existing phase failures or SysFileTree history do not authorize a licence or a retry-until-green policy.

```bash
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py suite-delta
```

Enable `REXX_SUITE_DIAGNOSTICS` on both the release whole suite and debug workspace suite target before their first run. The existing hook retains each run's stdout, stderr, status/end reason, elapsed time, GroupRun fields, marker, generated `child.rex` where present, and run/group/interpreter path metadata before cleanup. Preserve these records for SysFileTree `TEST_TREE_DIRECTORY` and `fixtures::tests::a_child_refusing_on_the_inherited_stderr_leaves_the_run_ended_normally`, including the first failure. The prior non-reproductions in `t8.8/timing/sysfiletree/`, `sysfiletree-trace/`, and `stderr/` are context, not final evidence or a Flaky licence.

If a fresh SysFileTree failure needs field-level reduction, retain its failing full-suite records first, then run the existing explicit diagnostic once under a distinct diagnostic directory. This is an additional diagnosis, not a substitute for the failed whole suite, and it does not alter the real ooTest source. A stderr-fixture failure already has its default target output and generated-child diagnostics, so do not rerun the whole target merely to obtain a green result. Bring the retained failing field, command, source pin, and proposed next action to the controller without adding a licence.

```bash
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py sysfiletree-diagnostic
```

## Completion and unresolved concerns

The draft has been written but has not been executed or certified against a final source pin. In particular its Cargo target-runner interception must satisfy the dispatch parity guard before any direct binary execution is credited, and metadata/custom-harness mismatches must stop rather than omit coverage. The controller still needs the final committed source/base hashes, release executable pin, recorded profiling completion and settled tracked outputs, fresh target counts, and measured statuses, and must review the bounded release build/suite duration on that source. This plan makes no claim that final formatting, lint, corpus, suite, stress, workspace tests, or doctests have passed.

The artifact population guards retain historical targets plus the known Task 8.8f addition, include all freshly listed tests, and stop on unexplained removals or additions. Each target writes its exact added/removed-name record, and the final `audit` action checks all fresh artifact results, derived counts, unchanged ignored-test identities, nonzero named obligations including `semantic_boundaries`, and completed doctests. The doctest action compares semantic identities with line-number normalization, while the suite join retains every changed row and population discrepancy. Add an audit result only after reading those actual outputs, then release the source/input and index-staging freeze and copy the raw runner, commands, diagnostics, and summaries into the durable final record.
