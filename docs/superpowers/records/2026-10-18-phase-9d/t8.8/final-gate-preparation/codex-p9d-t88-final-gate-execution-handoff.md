# Task 8.8 final gate execution handoff

## State and authorization

No final gate, build, test, or freeze action has run. The final config and freeze record are absent, and profiling remains incomplete under the parent's control. The reviewed Q pin at `/tmp/codex-p9d-t88-pins/8.8f-perf-r2-rexx` hashes to `22c317baa3ed111935a6577ac33097e428a53b64eb64b4697af2b85664e3f8eb`, matching the supplied pin. Parent-supplied runtime source is `cd418bd304e4bd1fedd339cb090ae861b1a460d8`, while inspected metadata HEAD was `55655b4885a208d6ff45b9a080943b80b2b168ee`; final validation must use the later settled, committed HEAD authorized by root.

## Owned draft corrections

Exact pre-review files, current hashes, and unified diffs are retained under `/tmp/codex-p9d-t88-final-gate-preparation-review/`. The original drafts are `/tmp/codex-p9d-t88-final-gate-before-preparation-review.py` and `/tmp/codex-p9d-t88-final-suite-delta-before-preparation-review.py`. Static review found that a post-command freeze failure could prevent writing the command's actual result, so the driver now retains that descriptor before reporting the freeze failure. Timeout signaling also tolerates a child that exits between timeout detection and SIGTERM, and the suite join now validates companion-table schemas and revisions rather than accepting any parseable TSV.

The strict historical-name guard now reconciles only these explicit source-proven replacements. Each requires the historical name to exist in the retained population, to be absent from the fresh list, and its replacement to exist in that same fresh target. Every other removed name still blocks validation, and the emitted population record retains the exact reconciliation. No broad prefix replacement, basename match, or count-only allowance was introduced.

| Target | Historical name | Current name and evidence |
| --- | --- | --- |
| `rexx-exec` library | `run::tests::an_executable_rebuilt_under_the_nil_scope_answers` | `run::tests::an_executable_retained_under_the_nil_scope_answers`, `src/run/tests.rs`: Entry rooting changes the explicit rebuild assertion from one to zero while preserving the nil-scope program result. |
| `rexx-exec` library | `scheduler::tests::pool::a_spawn_failure_leaves_ready_activities_to_the_next_baton_holder` | `scheduler::tests::pool::a_disabled_pool_leaves_ready_activities_to_the_next_baton_holder`, `src/scheduler/tests/pool.rs`: Disabled-pool coverage now uses configured bound zero, with actual spawn-failure coverage added separately in the same default library population. |
| `suite` integration | `fixtures::tests::the_suite_revision_is_the_recorded_one` | `fixtures::tests::the_recorded_suite_revision_is_a_revision_number`, `tests/suite/fixtures/tests.rs`: The default check validates recorded syntax, while the actual-checkout revision contract remains in the full release suite gate. |
| `concurrency_tests` integration | `group_runs::whole_groups::sim_gate::calibration` | `group_runs::whole_groups::sim_gate::calibration::calibration`, `tests/group_runs/whole_groups/sim_gate/calibration.rs`: The function moved into a named child module. |

The RAISE, PARSE trigger, hash-removal, assignment, and USE extractions add production modules without moving their tests into those modules. The input, condition, and method-row inline test modules moved into adjacent `tests.rs` files while retaining their `tests` module names. A read-only scan of historical library-test function names found only the two library replacements above absent from current declarations. Fresh libtest listings remain the authoritative check and can expose another specific reconciliation need without weakening the guard.

## M6 and default-population coverage

The default workspace still excludes feature `pinning`, so its successful default library run cannot cover the retained M6 witnesses. A single final `pinning` action now runs `cargo test --locked -j4 -p rexx-exec --lib --features pinning scheduler::tests::mutants:: -- --test-threads=1 --nocapture`, with no mutation or restoration rerun. It requires every current `#[test]` in that named module, including the REPLY witness and RETURN twin, to appear and pass, then writes `pinning-verified.json` for the final audit. The historical slice's seven passing feature tests remain prior evidence, while this one final-source action accounts for the feature obligation after Q's later production repair.

Fresh default selection remains all historical targets plus the explicitly reconciled `semantic_boundaries` integration target. Counts derive from fresh metadata, Cargo JSON artifacts, and complete libtest lists, rather than a fixed final cardinality. Cargo dispatch capture is explicitly nonexecuting and is never gate evidence, with package cwd, argv, runtime-library environment, executable hash, target parity, and separate Cargo doctests guarded as before. The only recorded environment keys are the existing exact nonsecret whitelist, and no registry-token prefix is serialized.

## Prerequisites and disk

Read-only path inspection found every file/directory required by `rust/scripts/test_preflight.py`, without executing that gate. The later real preflight must still pass, and the full suite must still validate its actual ooTest revision. The inspected Q file hash matches the supplied immutable pin, while the final config and profiling-completion record must be supplied only after the parent settles and commits all reports. The config is frozen with an empty evidence-output allowlist, fixed HEAD, no tracked changes, and unchanged full index staging identity.

At the initial inspection, `/tmp` had 1,059,414,016 bytes free and the workspace filesystem had 775,546,081,280 bytes free. The active target allocated 9,589,870,592 bytes, including 8,109,694,976 debug bytes and 1,061,367,808 release bytes, and the retained prerequisite cache allocated 6,653,358,080 bytes. Historical selected prerequisite binaries alone occupy 5,505,060,864 allocated bytes, with the old whole cache providing an approximately 6.65 GB cold-build footprint reference rather than a measured final increment. Current debug outputs include approximately 144 MB execution-library/unit artifacts and approximately 130 MB integration binaries, so several parallel links plus profiling growth cannot be treated as safe within a few hundred MB.

Root authorized relocation of exactly `/tmp/codex-p9d-t88-prereq-target` to the already ignored worktree-local `.superpowers/cache/codex-p9d-t88-prereq-target`, retaining the original path through a symlink. That would release approximately 6.65 GB of tmpfs allocation while preserving bytes, original executable path strings, modes, modification timestamps, symlinks, and internal hardlink groups. Pre/post manifests and operation records belong to `/tmp/codex-p9d-t88-prereq-cache-relocation/`; device, inode and ctime changes must remain explicit, and hash/traversal reads change access times on this tmpfs. The relocation is currently pending the user's treatment of the inaccessible systemd `/proc` link, so no space gain is claimed.

## Minimal execution after root authorization

First finish profiling, resolve relocation, commit all source/evidence updates, and obtain root's explicit execution authorization. The parent writes `/tmp/codex-p9d-t88-final-gate-config.json` with the final settled full `source_commit`, completed `profile_completion_record`, immutable `release_binary`, and `release_binary_sha256`. The completion record must identify that same final commit, declare `profiling_complete: true`, and list `active_profile_commands: []`. No tracked report or index staging may change between `freeze` and the last action, and all raw outputs remain under `/tmp` until release of that freeze.

Run each following line as its own foreground command, preserving the saved approval prefix. `queue` emits every fresh default-target command to `/tmp/codex-p9d-t88-final-gates/target-commands.txt`; execute those emitted lines individually after `queue` and before `doc`, continuing independent targets after a failure. Internal action and child deadlines remain below 540 seconds, and the driver separately records stdout, stderr, exact command, cwd, allowed environment, timeout, status, and source pin. Do not add a timeout shell prefix, leading environment assignments, redirects, or a shell loop around the whole queue.

```text
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py freeze
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py preflight
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py provenance
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py fmt
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py clippy
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py release-build
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py release-corpus
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py release-suite
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py suite-delta
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py metadata
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py build
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py dispatch
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py queue
```

After every emitted target command has completed, run the remaining actions below. These do not repeat slice-focused checks or prior mutation controls, and the M6 feature module is accounted for once on the final source. The suite diagnostics preserve the first SysFileTree/stderr-fixture failure rather than licensing or retrying it until green. Only a fresh failure needing field-level reduction justifies the separate existing `sysfiletree-diagnostic` action.

```text
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py doc
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py pinning
memcap 8G python3 /tmp/codex-p9d-t88-final-gate-draft.py audit
```

## Remaining blockers

Root has not authorized final execution, matched P-to-Q profiling has not been accepted as complete, and the settled final commit/config do not yet exist. Relocation is pending the explicit `/proc` choice, and actual final build/link disk increments and bounded run durations remain unmeasured. Cargo runner capture and final fresh test populations have only been statically inspected, with exact fresh parity still mandatory before crediting execution. Preparation is ready for those decisions, without claiming any final gate passed.
