# Slice performance

All accepted comparisons in this report completed 1,485 independently verified profiles across six cohorts, with 33 programs and three rotating rounds per cohort. The [final verification](q-complete-verification.json) reports no invalid profiles or driver diagnostics, and [final analysis](q-analysis.json) retains every original and repaired own-cost result. The additional comparisons below use immutable accepted pins, preserving the original failures as evidence rather than replacing their records. All interpreter statuses were zero, non-TIMED stdout agreed, stderr was empty, and no SPREAD flag occurred.

| Additional adjacent comparison | Profiles in cohort | Programs over +0.05% | Failing programs |
| --- | --- | --- | --- |
| e2 to corrected f | 198 | 3 | dirread +0.36465390%, parse +0.22055592%, pingmsg +0.07291944% |
| Corrected f to registry H | 198 | 0 | None |
| Registry H to numeric A | 297 shared | 0 | None |
| Numeric A to f-performance | Same 297 | 2 | parse +0.34815184%, rexxcps +0.08961594% |
| f-performance P to PARSE repair Q | 198 | 0 | None |

The [corrected f report](e2-f/README.md), [registry repair report](f-registry/README.md), and [numeric and f-performance report](h-a-fperf/README.md) retain complete tables and accepted pin provenance. Registry H improves ordinary sends directly, but its residual own cost relative to original e1 remains +0.12091429% on dispatch, +0.13069467% on dispatchclass, and +0.18086132% on sendloop. Numeric A reduces arithmetic by -0.30308812% against H, with a cross-source residual of +0.04647734% against pre-8.8. These residuals span intervening source changes, so negative repair deltas do not establish elimination of an original regression or isolate its mechanism.

The [Q report](p-q/README.md) separates the passing direct Q/P repair from residual comparisons with retained historical medians. Q still exceeds +0.05% on twelve programs relative to both pre-8.8 and e1, with alloc worst at approximately +0.73786%. Relative to original e2, Q still exceeds the budget on parse at +0.17209251%, while comparison with corrected F has no residual OVER. These results require explicit budget disposition and do not establish original f-budget or global-budget acceptance.

The original four own-BASE comparisons completed 594 successful profiles across two cohorts. Every benchmark program ran in three rotating rounds, without a `-p` subset or shortened workload. [Independent verification](original-verification.json) checked the program enumeration, actual profiled arguments, raw summaries, stream hashes, pin hashes, and instrument hashes. All interpreter statuses were zero, non-TIMED stdout agreed, stderr was empty, and no scheduling SPREAD flag occurred. The superseded unprofiled f pin remains preserved in [held-f-inputs.json](held-f-inputs.json).

| Slice | Own BASE | Programs | Programs over +0.05% | Worst own delta |
| --- | --- | --- | --- | --- |
| 8.8a | Pre-8.8 | 33 | 1 | arith +0.34824337% |
| 8.8c | a+b | 33 | 0 | define +0.00245833% |
| 8.8e1 | c | 33 | 0 | define +0.00126121% |
| 8.8e2 | e1 | 33 | 12 | sendloop +1.01245264% |

The arithmetic regression has [focused existing-profile attribution](arith-attribution-r1/README.md), with 53.5 million new `whole_value` self instructions offset by 13.5 million fewer `arith_general_body` self instructions. The e2 results below exceed the own-cost budget, and their full instruction counts and spreads remain in [8.8e2-own-delta.tsv](c-e1-e2/8.8e2-own-delta.tsv). Both original reporting commands exited one because they found whole-process OVER flags, with their stdout and stderr retained in each cohort directory. The cumulative arithmetic flag on c inherits a's regression, while c's adjacent a+b-to-c comparison passes every own-cost gate.

| e2 program | Own delta |
| --- | --- |
| alloc | +0.73673786% |
| churn | +0.13298402% |
| define | +0.23346786% |
| dirread | +0.06086231% |
| dispatch | +0.67687242% |
| dispatchclass | +0.73155584% |
| hashnew | +0.23820545% |
| heapshape | +0.35532172% |
| sendloop | +1.01245264% |
| pingguard | +0.05221187% |
| pingmsg | +0.23968174% |
| pingsem | +0.49532412% |

[Inputs](inputs.json) preserve the exact original script, helper, program, library, and pin hashes used for the original comparisons. [Pin provenance](pin-provenance.json) copies the source and build metadata, including the c record originally retained under `/tmp`. [Measurement method](method.md) records the capped wrapper diff, checkpoint extraction, rotating order, scheduling changes, separate memory caps, and deadline behavior. Batch JSON files retain exact commands, statuses, timings, and source snapshots, while [the raw-profile manifest](original-profile-manifest.tsv) hashes every retained profile.

Own cost subtracts only libc and the loader, so it still includes other libraries and compiler-generated code. Inlining and unidentified source lines limit attribution, and inclusive function totals overlap their callees. These measurements compare frozen binary transitions and do not establish the performance of unmeasured semantic or failure paths. This worker changed no interpreter source or shared Git index or HEAD, staged and committed no metadata, and retained raw profiles under `/tmp/codex-p9d-t88-performance`.
