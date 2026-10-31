# Final functional validation

All authorized functional checks passed on frozen commit `b51a9466a5ccfa365bcf74e232f6e24c23fb9765`, whose Rust tree equals reviewed source `ac446500e03c18b5143f2967ca30015e6d64b95f`. Every command stopped before root explicitly released the source, index and input freeze, as recorded in `freeze-release.json`. All executions in this final context were fresh, with no semantic retry, timeout extension, source change or adoption of a historical test run. Earlier failures and negative audits remain unchanged in their separate retained records.

| Gate | Actual result |
| --- | --- |
| Preflight, provenance, fmt, workspace clippy | Passed |
| Release build and artifact/source/feature/profile guards | Passed |
| Gated release corpus | 1,242/1,242 matching, 34 passed and one existing ignored |
| Full release suite | 402 green groups, revision 13178, 24,523 per-test keys |
| Default debug workspace | All 156 targets passed, 3,810 identities: 3,805 passed, five existing ignored, zero failed/filtered/measured |
| Gated debug corpus | 1,242/1,242 matching, 34 passed and one existing ignored |
| Full collect stress | 79/79 passed in 268.40 seconds, debug assertions and overflow checks enabled |
| Repaired dispatch inventory | 6/6 passed |
| Suite fixtures | 69 passed, one existing ignored, including both added temporary-directory tests and the inherited-stderr refusal fixture |
| Full concurrency | 63/63 passed in 486.19 seconds, including TraceObject and its altered-test negative control |
| Doctests | 15/15 passed: four run, ten compile-fail, one compile-only |
| Pinning module | 7/7 passed |
| Final completeness audit | Exit 0 |

Both exact outer joins use `(group, test)` and preserve old/new licence, oracle and crate outcomes. Against t8.7c fix-r4, the only changed row is Bag `TEST_HASITEM_VALUE_NOTEQUAL_INDEX`, crate failed to passed under unchanged Deviation(45). Against the previous resumed attempt, the only changed row is SysFileTree `TEST_TREE_DIRECTORY`, crate failed to passed, with no licence change. Both joins retain 24,523 old/new keys and zero additions or removals, while the three existing Phase10 socketClass timeout-blocked rows remain unchanged.

SysFileTree passed 26/26 tests and 2,014 assertions on each interpreter, in one attempt per side. Its diagnostic contexts record distinct private child TMPDIR paths and both stable seeds. The 1,063 configured static suite inputs matched before, after and through the final guard, with manifest SHA256 `54e34606829f340cb63a4f80ca93f661f8616ab1a08a917969a72d2eeff1005e`. No foreground gate timed out, and this statement does not erase the three existing deferred suite rows or five existing ignored default tests.

`final-completeness-proof.json` links all actual command descriptors, target populations, executable identities, suite rows and diagnostics. The immutable measured normal pin remains SHA256 `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`, separate from Cargo-test CLI SHA256 `9c3d09f9f9e39810985b1200007035488a3eba9d00ab6ee7382b6d7aea1b4d06`. The final strict guard verified unchanged 300 production/Cargo inputs, eleven registration inputs, 553 protected ooTest files, driver/helper/config/bridge identities and every current default executable hash. No new performance measurement was required or claimed, and performance-budget acceptance, original failure dispositions and Task8.8/phase closure remain root/user decisions.
