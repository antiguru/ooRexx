# Resumed validation result

Validation executed completely on frozen HEAD `6b6f3fb460049c6ef63f565ceb78e11b5582f165` and failed. All commands stopped before root released the source, index and input freeze at `2026-10-31T01:08:59.035441+00:00`. The final guard verified the frozen source, index, config and normal pin before release. No source or index changes, semantic retries, timeout extensions, profiles or cleanup occurred in this validation.

## Actual results

| Check | Result |
| --- | --- |
| Preflight, provenance, fmt, workspace clippy | Passed |
| Release artifacts and actual-schema provenance | Passed, distinct normal and test variants verified |
| Release gated corpus | 1,242/1,242 matching, 34 passed and one existing ignored |
| Full release suite | Failed, 402 groups and one red group, revision 13178 and full population |
| Exact baseline suite delta | 24,523 old/new keys, 402 groups, two changed rows, zero added/removed |
| Fresh default metadata/build/dispatch | 156 targets, all 154 historical targets plus two explicitly authorized additions |
| Default target executions | 154 targets passed, two failed; 3,801 tests passed, two failed, five existing ignored |
| Full collect stress with debug assertions | 79/79 passed in 240.26 seconds |
| Debug gated corpus | 1,242/1,242 matching, 34 passed and one existing ignored |
| Semantic boundaries | Five tests passed, all 42 fixture programs traversed with normal and collect-on-every-allocation comparisons |
| Cargo doctests | All 15 identities passed: four run, ten compile-fail and one compile-only |
| Pinning module | All seven tests passed, outside the default population |
| Final audit | Failed at default target 118; no successful audit marker created |

## Retained failures

* Full release suite: `base/rexxutil/SysFileTree TEST_TREE_DIRECTORY` failed on the crate side and passed on the oracle side. `release-suite.{json,stdout,stderr}` and `suite-diagnostics/` preserve the single execution. The bounded static-input manifests matched exactly before and after: 1,057 entries, SHA256 `109c412b19e118ae6804fbcc1fd96f55d43c08d51fd3d924a90814b32b97dfa7`. This scope covers configured static inputs, not arbitrary operating-system or proc state.
* Default target 118: `the_seam_token_is_named_only_by_the_dispatch_module` failed because the fixed allowlist omits `dispatch/hash/removal.rs`, `dispatch/method_entry.rs` and `dispatch/stream/query.rs`. The harness completed with five passed and one failed. Raw evidence is `debug-118-dispatch_seam.{json,stdout,stderr}`.
* Default target 154: `group_runs::the_outcome_table_of_the_trace_object_group` expected three test names and received ten all-passing test names. The harness completed within its bound in 475.42 seconds with 62 passed and one failed. Raw evidence is `debug-154-concurrency_tests.{json,stdout,stderr}`.

The exact delta against `docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r4/suite/tests.tsv` is retained in `suite/delta-vs-t8.7c-fix-r4.tsv`. Bag `TEST_HASITEM_VALUE_NOTEQUAL_INDEX` changed from crate failed to passed with its existing Deviation(45), and SysFileTree `TEST_TREE_DIRECTORY` changed from crate passed to failed with no row. Both oracle outcomes remain passed and no table keys were added or removed. The inherited-stderr refusal fixture passed on its single first default execution, while its historical disposition remains a separate root decision.

## Completeness and runner history

`independent-completeness.json` accounts for all 156 Cargo-owned default artifacts and every retained execution record. It verifies recorded source and dispatch contexts, executable byte hashes, complete listed populations, all four explicit renames, no unexplained test removals, and unchanged ignored populations. It records zero timeouts, filtered default tests or measured tests while retaining both failed records. These checks establish execution completeness and do not convert the failed final audit into a pass.

The resumed runner ended at SHA256 `af2bfb41da1d71930e2b7eb3c09319d16ac685609365dce8aa9a8dc1587f85e4`. Root-authorized corrections and exact before/after evidence are retained in `watchdog-path-correction/`, `concrete-owners-population-correction/` and `nested-summary-correction/`. The library's original driver assertion failure remains preserved, and metadata-only reconciliation accounts for seven known successful self-reexecution summaries plus its unique final 1,239-test summary without rerunning tests. The initial missing-watchdog launcher, exclusive-descriptor refusal and command-path typo started no semantic children and remain separately recorded.

## Artifact identities and handoff

The immutable measured normal pin remains `/tmp/codex-p9d-t88-pins/8.8-corpus-registration-rexx`, SHA256 `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`. The verified Cargo-test CLI is separately identified by SHA256 `9c3d09f9f9e39810985b1200007035488a3eba9d00ab6ee7382b6d7aea1b4d06` and the exact rustix feature set including `pty`. Current provenance and fresh harness identities are in `release-artifacts-verified.json`, with the causal one-variable proof explicitly attributed to attempt 1. Config and delta-helper identities, stopped-command state, failure totals and release authorization are in `freeze-release.json`.

Root owns source and index after the recorded release, and no further actions or retries were run. Attempt 1 remains unchanged under its separate output paths and durable records. This result does not claim Task 8.8 or phase completion. Acceptance of original performance-budget residuals and the two original failure dispositions remains separate.
