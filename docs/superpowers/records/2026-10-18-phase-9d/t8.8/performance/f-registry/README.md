# Registry repair comparison

The accepted registry pin completed all 198 profiles against the corrected f BASE, with every program measured in three rotating rounds. Its SHA-256 is `470dcb41db61ab7a4d64f3276ce28e539d5686d130be245a4ec7b30e6d6725ac`, from runtime source `1233b81da730142057fcf810e324d10d0e55c109`. [Verification](../registry-complete-verification.json) checked all 990 profiles through this cohort, and [direct deltas](8.8h-own-delta.tsv) contain no own-budget OVER. The largest direct increase is churn at +0.03805535%, while outputs and statuses agree and no SPREAD flag occurred. The original reporter exited one on whole-process OVER flags for pingguard and pingsem, retained in [its stderr](original-table.stderr), which are distinct from the own-cost gate.

| Ordinary-send program | H versus matched F | H versus original e1 |
| --- | --- | --- |
| dispatch | -0.55225710% | +0.12091429% |
| dispatchclass | -0.59654662% | +0.13069467% |
| sendloop | -0.82331095% | +0.18086132% |

The negative direct repair deltas do not establish elimination of the original ordinary-send regression. Residual cost against original e1 remains above +0.05% for the programs in the table. [Residual counts and deltas](residual-vs-original.tsv) retain comparisons with original e1, original e2, prior F measurements, and the matched F measurements from this cohort. These comparisons span independent cohorts and intervening source changes, so they do not isolate the registry mechanism from every e2 and f change.
