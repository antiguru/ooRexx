# PARSE repair and current residual costs

The accepted P-to-Q comparison completed all 198 profiles across 33 programs and three rotating rounds. Q/P passes every adjacent own-cost gate, with parse improving by -0.39894332% and rexxcps by -0.10687645%. All interpreter statuses were zero, non-TIMED stdout agreed, stderr was empty, and no SPREAD flag occurred. [Independent verification](../q-complete-verification.json) checked all 1,485 retained profiles and produced raw-manifest SHA256 `583b32c68111ce37e0141eacec04d48094e86776e6d9ebec7983e503c07ccdbd`.

| Comparison | Programs | Own-cost OVER count | Worst residual |
| --- | --- | --- | --- |
| Q/P, direct repair | 33 | 0 | None over +0.05% |
| Q/pre-8.8 BASE, separate cohorts | 33 | 12 | alloc +0.73785973% |
| Q/original e1, separate cohorts | 33 | 12 | alloc +0.73785934% |
| Q/original e2, separate cohorts | 33 | 1 | parse +0.17209251% |
| Q/corrected F, separate cohorts | 33 | 0 | churn +0.03605391% |

The [direct comparison](8.8f-repair-r2-own-delta.tsv) retains three-round median instruction counts, adjacent own deltas, whole deltas, and spreads for every program. The unchanged reporting code exited zero for the complete P/Q whole-process table, with [stdout](original-table.stdout), [stderr](original-table.stderr), and its exact command retained. [Output agreement](output-agreement.tsv) preserves independent stream hashes and the original `heapshape` and `rexxcps` TIMED exceptions. This direct repair pass does not establish acceptance against f's original e2 BASE or the task's original BASE.

| Residual OVER program | Q/pre-8.8 BASE | Q/original e1 |
| --- | --- | --- |
| alloc | +0.73785973% | +0.73785934% |
| churn | +0.17112571% | +0.17110530% |
| define | +0.24656913% | +0.24652314% |
| dispatch | +0.12087866% | +0.12087799% |
| dispatchclass | +0.13064560% | +0.13064520% |
| hashnew | +0.24100422% | +0.24099970% |
| heapshape | +0.35696670% | +0.35696706% |
| parse | +0.17226261% | +0.17225762% |
| sendloop | +0.18080803% | +0.18080702% |
| pingguard | +0.05189598% | +0.05169493% |
| pingmsg | +0.16252796% | +0.16239837% |
| pingsem | +0.49467552% | +0.49476232% |

The current ordinary-send residuals against e1 remain +0.12087799% on dispatch, +0.13064520% on dispatchclass, and +0.18080702% on sendloop. Parse retains an own median of 1,547,905,485 against original e2's 1,545,246,232, producing the +0.17209251% residual OVER. Separate all-33 tables preserve [BASE](q-vs-base-residual.tsv), [e1](q-vs-e1-residual.tsv), [e2](q-vs-e2-residual.tsv), and [corrected F](q-vs-f-residual.tsv) values, with exact counts and source references in [residual analysis](residual-analysis.json). These comparisons use different cohorts and complete source revisions with intervening semantic and performance changes, so they do not isolate method-entry, PARSE, numeric handling, or another mechanism.

Q is runtime source `cd418bd304e4bd1fedd339cb090ae861b1a460d8`, pin SHA256 `22c317baa3ed111935a6577ac33097e428a53b64eb64b4697af2b85664e3f8eb`, compared with immutable P SHA256 `6ed1c08eb03d54c48d1fe3947ce376f09502aaeba34ac221c334b01b1572b86e`. [Accepted inputs](../q-inputs.json) and [pin provenance](../pin-provenance.json) retain the exact source, compiler, build, pin, and original instrument references. Batches 20, 21, and 22 completed in 478.090, 334.515, and 79.079 seconds, preserving all successful records and resuming only missing profiles under the existing separate memory caps and 525-second foreground bounds. The [command ledger](../batch-commands.json) retains exact commands and snapshots, while raw profiles remain exclusively under `/tmp/codex-p9d-t88-performance` without cleanup.

The direct Q/P repair is complete, but original-budget acceptance remains unresolved. The residual method-entry budget request must explicitly cover the ordinary-send costs above +0.05% against e1, and the original f-budget request must explicitly cover parse at +0.17209251% against e2. A global-budget decision must also cover all twelve residual OVER programs against pre-8.8 BASE, rather than treating the passing repair step as a global pass. The controller owns that disposition and staging of these records, and this worker changed no interpreter source or shared Git index or HEAD.
