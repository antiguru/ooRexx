# Resumed validation preparation

Attempt 1's runner, config, raw outputs and measured results remain unchanged. Its freeze was explicitly released, and the original implementer retains ownership of source/index for the registration-only repair. No Git command, tracked write, resumed config, freeze, build or test was performed during this preparation. New HEAD, new pin/source provenance and root review acceptance are still required before execution.

## Separate execution context

The proposed runner is `/tmp/codex-p9d-t88-final-gate-resume.py`, with output root `/tmp/codex-p9d-t88-final-gates-resume/` and config `/tmp/codex-p9d-t88-final-resume-config.json`. The config keeps `source_commit`, `profile_completion_record`, `release_binary` and `release_binary_sha256`, adding the exact `release_pin_provenance` path once root supplies it. No placeholder config was created, and the new immutable registration pin must hash identically to the measured normal `d9210f4b...` pin. The resumed delta helper has its own filename/output path, so it cannot write into attempt 1's table directory.

The runner preserves source/index/config guards, capped foreground commands, exact Cargo dispatch/population checks, the existing name reconciliations, Cargo doctest semantics and the single pinning-module obligation. It adds only the missing nonsecret `REXX_SUITE_GATE` and `REXX_SUITE_TABLE` keys to serialized command metadata, without relabeling old metadata. The accepted PTY causal proof remains tied to attempt 1's source and authorization, while the current build context is linked separately by identical measured-normal pin bytes and fresh exact test-CLI/features/profile/harness verification. The old context-specific metadata reconciliation action is removed, because the resumed release build must produce its own fresh Cargo artifacts.

## Suite evidence decision

Attempt 1's successful 402-group, 24,523-row execution and delta remain valid historical measurements. They lack a complete pre-execution byte manifest of the untracked ooTest/oracle inputs, so binary identity and SVN revision alone cannot establish the stronger filesystem-input reuse proof requested for the repaired snapshot. Root therefore chose one fresh full-suite execution rather than an adoption/exception mechanism. That repeat is justified by the new validation snapshot and input-provenance gap, without treating existing licensed socketClass timeouts or a desired green result as its reason.

The inspected `l3_core_suite` path reads ooTest enumeration/content, revision and row metadata, cloned framework/drivers, regex class data, oracle binaries/libraries/class files/samples and Cargo-owned executables. It does not directly read `phase-9.txt`, the repaired reference-conversion corpus files, semantic-boundary fixtures, or SOURCELINE oracle records. This excludes those repair inputs from the direct suite call graph, but does not invent missing old external-file hashes. Actual final repair scope and artifacts still need root's handoff/review, and no old execution is presented as a fresh run.

## Bounded configured-input manifest

The new `suite_input_snapshot` runs inside the existing 535-second action deadline before and after the fresh release suite. It records logical paths, file modes, file sizes, modification timestamps, SHA256 bytes, directory topology and symlink targets/resolved paths, rejecting unsupported special files or symlink cycles. Administrative `.git`, `.svn` and `.hg` entries are excluded, while mutable Cargo run directories are outside the selected roots. It writes distinct before/after/comparison records and fails after preserving them if configured static inputs change.

| Input scope | Reason |
| --- | --- |
| Checkout `ootest/`, excluding administrative metadata | Group discovery, framework, root drivers and test assets |
| `extensions/rxregexp/rxregexp.cls` | Explicit class copied by the group runner |
| Checkout `build/lib/` | Required checkout-local native/preflight input root |
| Oracle `/home/moritz/dev/repos/ooRexx/build/bin/` | Oracle executable/tools and copied class files |
| Oracle `build/lib/` | Native loader/library and image input root |
| Oracle `build/samples/` | Sample source layout reached by the copied interpreters |
| `suite-revision.txt`, `suite-licences.txt`, `suite-not-run.txt`, `phase-9-exclusions.txt` | Actual revision and suite-row metadata |
| `suite.rs`, its `suite/` source directory, `tests/support/`, `watchdog.rs` | Compiled suite-driver source context |
| Fresh Cargo `rexx`, `rexxc` and selected suite harness | Actual executable identities |
| Resumed Python driver and delta helper | Owned validation driver identities |

Read-only size inspection found about 55 MB allocated across the ooTest and native/oracle directory roots before adding the selected executables and small driver sources. This bounds the proposed hash scope without executing the snapshot or suite. The manifest explicitly excludes operating-system/proc state and arbitrary PATH tools, so it is not a claim to capture every environmental input. Time, scheduling and existing licensed Phase 10 timeout rows remain actual observed run behavior rather than a new exception policy.

## Execution after handoff and acceptance

Root supplied the reviewed pin producer and actual registration provenance, with matching production/input hashes and identical normal bytes. The runner validates that actual schema as specified in `final-config-and-schema.md`; final metadata/freeze HEAD and a new completion attestation still await root. Only after root accepts the runner and supplies those values may a new config/freeze be created. Prior checks retain their original context, and repair-specific checks are credited from reviewed records rather than repeated for a new timestamp.

Retain attempt 1's preflight/provenance/fmt/clippy evidence and let the reviewed final change list determine which input or Rust-test checks need refreshing, rather than repeating them for a new timestamp. Run fresh release compilation/artifact verification, repaired release corpus, the single fresh full suite and its exact delta. Continue with first-time default workspace metadata/build/dispatch/queue and every selected target separately, then Cargo doctests, the pinning module and audit. Use only plain `memcap 8G python3 /tmp/codex-p9d-t88-final-gate-resume.py ACTION` invocations, without shell loops or leading environment assignments, with exact proposed identities retained in `manifest.json` and the two diff files.
