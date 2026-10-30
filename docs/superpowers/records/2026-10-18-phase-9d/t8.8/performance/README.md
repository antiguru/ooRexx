# Original slice performance

The original four own-BASE comparisons completed 594 successful profiles across two cohorts. Every benchmark program ran in three rotating rounds, without a `-p` subset or shortened workload. [Independent verification](original-verification.json) checked the program enumeration, actual profiled arguments, raw summaries, stream hashes, pin hashes, and instrument hashes. All interpreter statuses were zero, non-TIMED stdout agreed, stderr was empty, and no scheduling SPREAD flag occurred. The additional e2-to-f comparison is tracked separately in [additional-f-inputs.json](additional-f-inputs.json), with the superseded unprofiled f pin preserved in [held-f-inputs.json](held-f-inputs.json).

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
