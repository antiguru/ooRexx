# Final candidate performance

The accepted Q-to-final comparison completed all 198 profiles across 33 programs and three rotating paired rounds. Final runtime source is `9d919b7c3832c55edc3a0a07974309f092102201`, pin SHA256 `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145`, compared with immutable Q `22c317baa3ed111935a6577ac33097e428a53b64eb64b4697af2b85664e3f8eb`. All 33 direct own-cost gates pass, with every direct own delta negative and the largest at strings -0.00000374%. Scoped review authorization is retained in [the accepted manifest](r-inputs.json), with build and review provenance under [the cache provenance index](../provenance.json).

| Comparison | Programs | Own-cost OVER count | Largest measured delta |
| --- | --- | --- | --- |
| Final/Q, direct comparison | 33 | 0 | strings -0.00000374% |
| Final/pre-8.8 BASE, separate cohorts | 33 | 10 | alloc +0.73785570% |
| Final/original e1, separate cohorts | 33 | 10 | alloc +0.73785531% |
| Final/original e2, separate cohorts | 33 | 1 | parse +0.17203271% |
| Final/corrected F, separate cohorts | 33 | 0 | churn +0.03580974% |

All interpreter statuses were zero, non-TIMED stdout agreed, stderr was empty, and no SPREAD flag occurred. The original whole-process table exited one solely for fibfunc at +0.12033230%, while its libc/loader-subtracted own delta was -0.00000809%. [Whole-process category counts](fibfunc-whole-process-evidence.json), [original table stderr](q-r/original-table.stderr), and [output agreement](q-r/output-agreement.tsv) preserve the distinction without assigning a source-level cause. [Independent combined verification](final-all-complete-verification.json) checked all 1,683 profiles, and [historical retention](historical-retention.json) proves the original 1,485 manifest rows remain unchanged.

| Residual OVER program | Final/pre-8.8 BASE | Final/original e1 |
| --- | --- | --- |
| alloc | +0.73785570% | +0.73785531% |
| churn | +0.17088121% | +0.17086080% |
| dispatch | +0.12087342% | +0.12087274% |
| dispatchclass | +0.13063946% | +0.13063906% |
| heapshape | +0.35692912% | +0.35692947% |
| parse | +0.17220282% | +0.17219782% |
| sendloop | +0.18080019% | +0.18079917% |
| pingguard | +0.05284641% | +0.05264537% |
| pingmsg | +0.16231807% | +0.16218848% |
| pingsem | +0.49562447% | +0.49571128% |

The original-budget residuals remain OVER on ten programs against BASE and e1, while original e2 still has parse at +0.17203271%. Define and hashnew no longer exceed those historical budgets, and their direct Q-to-final changes are -0.24382226% and -0.30400469%. The direct repair pass does not establish original f-budget or global-budget acceptance, which remains the controller and user decision. The ten OVER results remain explicit evidence until that disposition is recorded.

Complete tables retain [direct Final/Q](q-r/8.8-final-fix-own-delta.tsv), [Final/BASE](q-r/r-vs-base-residual.tsv), [Final/e1](q-r/r-vs-e1-residual.tsv), [Final/e2](q-r/r-vs-e2-residual.tsv), and [Final/corrected F](q-r/r-vs-f-residual.tsv) medians for all 33 programs. Historical comparisons use different cohorts and complete source revisions with intervening semantic and performance changes. They do not isolate method-entry lookup, PARSE, numeric handling, or another mechanism. The ordinary-send residuals against e1 remain dispatch +0.12087274%, dispatchclass +0.13063906%, and sendloop +0.18079917%.

Batches 23, 24, and 25 completed in 481.393, 390.816, and 201.674 seconds under the unchanged separate memory caps and 525-second foreground deadlines. [Exact commands](batch-commands.json), frozen helper copies, instrument snapshots, authorization, pin provenance, and raw-profile manifests remain in this cache handoff. Successful profiles were never repeated, and the superseded intermediate final-fix pin was never profiled. This worker changed no tracked source, shared Git index, or HEAD, and all raw profiles remain under `/tmp/codex-p9d-t88-performance`.
