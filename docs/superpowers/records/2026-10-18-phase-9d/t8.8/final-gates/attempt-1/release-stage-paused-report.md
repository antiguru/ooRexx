# Release stage complete, debug workspace paused

The candidate remains frozen at `8b7767ed3fda908fffe75181b2c8bd12dd7509f8`, with no live command. Root requested a pause after the whole release suite and exact delta so the already-authorized Deviation 83 witness registration can be repaired before debug execution. No debug metadata, build, dispatch, target, doctest, pinning, or audit action started. Source, HEAD, index, config and tracked reports remain unchanged until root explicitly releases the freeze.

## Actual results

| Action | Actual result |
| --- | --- |
| Freeze, preflight, provenance, fmt, workspace clippy | Status 0 |
| Release compilation | Cargo 0, original verifier 1 due normal/test build-context expectation |
| Metadata-only known-variant reconciliation | Status 0, no repeated compilation |
| Full gated release corpus | Cargo 101, driver 1, 1,241/1,242 matching programs |
| Whole release suite | Cargo 0, driver 0, 402 groups green, 260.4308 seconds |
| Exact suite outer join | Status 0, 24,523 old/new test keys, no added/removed keys, one improved row |

The release corpus's only mismatch is `lang/reference_conversion_raises_in_clause.rex`, with differing stdout, empty stderr and status zero on both sides. Root's implementer identified this as a missed witness registration under the existing Deviation 83 authorization, with its proposal kept separate from the retained raw failure. The Rust corpus harness completed 33 passes, one failed differential and one existing ignored helper. It was not retried, and its exact first descriptors remain in `release-corpus.stdout`, `.stderr`, `.json` and the verified artifact record.

The whole suite produced revision 13178, `# groups all`, complete paired tables and 24,523 unique per-test keys across 402 groups. Every group ended normally on both sides, and all 402 oracle-denominator checks report equal. Both sides expected 23,430 unrowed tests and started 23,935 tests, with 23,932 oracle passes and 23,830 crate passes. Existing rowed failures/refusals/timeouts remain represented, so green does not mean every licensed test passed.

The sole joined change against `t8.7c/fix-r4/suite/tests.tsv` is `base/class/Bag` / `TEST_HASITEM_VALUE_NOTEQUAL_INDEX`. Its row owner remains `Deviation(45)`, the oracle remains passed, and the crate improves from failed to passed. There are no key additions/removals or other owner/oracle/crate verdict changes. `suite/delta-summary.json`, `suite/delta-vs-t8.7c-fix-r4.tsv`, and `release-stage-summary.json` retain the complete counted evidence.

## Original flakes and retained diagnostics

The original SysFileTree group ran once per side, with all 26 tests passing and normal termination on both sides. The full run retained 829 diagnostic bundles containing context, stdout, stderr and marker/path details before cleanup. The separate stderr-refusal fixture was filtered out by the exact `l3_core_suite` selection and remains pending the paused default suite target. No diagnostic was rerun to obtain a green result, and no new Flaky licence or excuse was introduced.

The crate per-test record includes three existing `blocked: timeout after 60 s` rows, retained in `suite/retained-timeout-rows.json` and the associated run diagnostics. The exact outer join confirms those verdicts are unchanged from the prior full record. Original first-failure evidence and actual command statuses remain intact rather than being replaced by the passing suite. Root can inspect the complete per-test table and diagnostics before deciding the next source-only repair.

## Artifact roles and recovery

The immutable measured normal production pin remains SHA256 `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`. The functional release tests use the separately authorized Cargo-test variant `9c3d09f9f9e39810985b1200007035488a3eba9d00ab6ee7382b6d7aea1b4d06`, whose sole relevant graph addition is `rustix/pty`. Fresh CLI, resolved feature/profile, source-tree, normal-pin and harness identity guards passed for both actual release commands. `progress.json` records the accepted runner transition, first verifier failure, release outcomes, pause reason and absence of a live session.

The existing exact environment whitelist omits the nonsecret `REXX_SUITE_GATE` and `REXX_SUITE_TABLE` keys from serialized command metadata, although the accepted driver sets both for `release-suite`. The full nonzero suite denominator, emitted table path and retained accepted source show that the gate actually ran, rather than returning early. No driver/config change was made to correct this logging omission during the requested pause. Functional validation remains incomplete, and performance-budget acceptance remains an independent pending user decision.
