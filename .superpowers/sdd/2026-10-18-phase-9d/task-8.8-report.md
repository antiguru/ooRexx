# Task 8.8 validation and pending dispositions

Task 8.8 implementation and final functional validation are complete, while overall task closure awaits two explicit dispositions. The final frozen commit is `b51a9466a5ccfa365bcf74e232f6e24c23fb9765`, with the same Rust tree as reviewed test/harness source `ac446500e03c18b5143f2967ca30015e6d64b95f`. Every final check executed freshly, and all commands stopped before root released the source, index and input freeze. Performance-budget acceptance and the historical stderr-refusal cause investigation remain open, with Tasks 9 and 10 outside this work.

## Final functional results

The complete final run passed all required checks on the committed tree. Root read the successful audit, confirmed the frozen HEAD and clean tracked state, and independently verified all 4,642 retained file hashes before copying them into durable records. The final output contains 156 default targets and 3,810 test identities, with no removed tests, unexpected additions, filtered tests or foreground timeouts. The five existing ignored tests and three deferred Phase 10 suite timeout rows remain explicit parts of their respective populations.

| Check | Actual result |
| --- | --- |
| Preflight, provenance, formatting, workspace clippy | Passed |
| Release and debug gated corpus | 1,242/1,242 matching in each profile |
| Whole release suite | 402 green groups, revision 13178, 24,523 per-test keys |
| Default debug workspace | 156 targets passed, 3,805 tests passed, five existing ignored |
| Full collect stress | 79/79 passed with debug assertions and overflow checks |
| Semantic boundaries | Five tests passed across all 42 fixture programs, normal and collect-on-every-allocation |
| Dispatch-seam policy | 6/6 passed |
| Suite fixture target | 69 passed, one existing ignored |
| Full concurrency | 63/63 passed within its 510-second bound |
| Doctests | 15/15 passed: four run, ten compile-fail, one compile-only |
| Pinning module | 7/7 passed |
| Final completeness audit | Exit 0 |

The exact suite comparison against Task 8.7c fix-r4 retains all 24,523 keys and changes only Bag `TEST_HASITEM_VALUE_NOTEQUAL_INDEX` from crate failed to passed under unchanged Deviation 45. The comparison against the earlier resumed attempt changes only SysFileTree `TEST_TREE_DIRECTORY` from failed to passed, without changing its licence. Both comparisons preserve separate old/new licence, oracle and crate fields, with no key addition or removal. All 1,063 configured static suite-input entries match before, after and through the final guard, without claiming that arbitrary operating-system state was frozen.

The [complete final evidence](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-gates/final/completion-report.md) records exact commands, environments, statuses, populations, runtime profiles and hashes. Final driver `/tmp/codex-p9d-t88-final-gate-final.py` has SHA256 `6e6696a48d592ae937974bab49a0cb8875146bbb555306b7a54681a33e69cdb3`, and each action ran separately in the foreground under `memcap 8G`. The source bridge permits exactly nine reviewed test/harness changes and two added suite tests, while retaining 300 production/Cargo inputs, 11 registration inputs and 553 protected ooTest files. Earlier failed runs, initial launcher errors and negative audits remain unchanged beside their superseding repairs.

## Reviewed final repairs

The resumed default run exposed two stale assertions, whose raw failures remain retained in `t8.8/final-gates/attempt-2`. Test-maintenance implementation `794dbc629` reconciles three explicitly moved dispatch-seam consumer paths and requires all ten existing TraceObject tests to pass with exactly ten total rows. Pure move `5f281e826` preserves comments, public behavior and all 63 concurrency test identities, while negative controls retain the bounded seam and outcome-classification contracts. Independent scoped review approved both findings, and both repaired assertions pass in the complete fresh final run.

SysFileTree separately reproduced a shared-directory count race between File listing and SysFileTree enumeration. Controlled witnesses on both interpreters establish stable 1-to-1 counts and exactly one intervening creation producing 1-to-2, while the precise historical extra writer remains unproven. Harness implementation `6f2c1346e`, following pure move `0d8d1b29c` and red regression `0333f95b1`, prepares a populated private child TMPDIR only for the exact SysFileTree group, separately per interpreter and attempt. Its independent scoped review, real-launch regression and final 26-test/2,014-assertion result per side support closing that queue item without changing protected tests or adding a Flaky licence.

The earlier corpus-registration repair follows already approved Deviation 83 and changes no production behavior. The full original reference-DESCRIPTION program and ruled channels remain covered, while its differential companion retains the unaffected conversion checks. Corpus populations remain 1,242 differential programs, four unfiled programs and 42 semantic fixture programs. Deviations 82, 83 and 84 remain approved and numbered, and the compiled M6 attribution witness remains closed with its production restoration and RETURN control preserved.

## Performance and remaining decisions

The measured production runtime remains `9d919b7c3832c55edc3a0a07974309f092102201`. Normal pin SHA256 is `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`, with separately proven Cargo-test CLI `9c3d09f9f9e39810985b1200007035488a3eba9d00ab6ee7382b6d7aea1b4d06` reflecting development rustix feature unification. All 1,683 retained profiles are verified, and the final test-only changes preserve production bytes, so no new profiling cohort was required or claimed. Direct Final/Q own-cost comparisons improve all 33 programs, while the original-budget acceptance remains a separate obligation.

Ten programs exceed the original BASE/e1 own-cost budget: alloc, churn, dispatch, dispatchclass, heapshape, parse, sendloop, pingguard, pingmsg and pingsem. Worst alloc is +0.73785570% against BASE, and parse remains +0.17203271% against original e2. Whole-process fibfunc +0.12033230% remains separately reported from its own-cost delta of -0.00000809%, without a new source-cause claim. Moritz has been asked whether to accept these recorded correctness costs and defer tuning, and no acceptance has been received.

The historical inherited-stderr refusal fixture passed both complete default executions but its original missing-pass cause remains unproven. Its queue and both inventory disposition columns therefore remain open pending Moritz's choice between retaining an accountable follow-up and requiring the cause before Task 8.8 closure. No known-failure entry, Flaky licence or task-complete checkbox was added from passing observations. Once the two dispositions are explicit, root can reconcile task closure and identify Task 9 as the next separate step without starting Tasks 9 or 10 automatically.
