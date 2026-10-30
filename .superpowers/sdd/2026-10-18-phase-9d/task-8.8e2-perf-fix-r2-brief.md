# Task 8.8e2: Performance repair round 2

## Remaining measured gap

The first repair recovers most of the executable-index miss cost, but ordinary-send counts remain above the own-delta budget. Against the original e1 counts, the completed H medians show dispatch at +0.12091429%, class dispatch at +0.13069467%, and sendloop at +0.18086132%. These cross-source comparisons include intervening f work, whose attribution limits remain explicit. [The residual diagnosis](task-8.8e2-perf-r1-residual-diagnosis.md) identifies four remaining miss-path compare/branch instructions and additional constructor code-generation differences.

## Narrow candidate

Move the unchanged executable-retention block immediately after `start.apply`, preserving both adjacent operations and their comments. The package application operates only on the local callee and cannot send, yield, or collect interpreter objects. Retention must still happen before flags/push, invocation tracing, guards, or user code, using the exact resolved Method and existing activation root. Do not change registry bounds, table ownership, retention eligibility, public flags or caller numeric semantics to obtain a cheaper miss.

The candidate aims to restore constructor/application adjacency and reduce discriminator/copy bookkeeping. The diagnosis observes a 472-byte callee copy in H that the original e1 binary constructs directly in the push slot. These observations motivate an experiment, not a promise that source order forces specific machine code or meets the budget. If the candidate does not produce a meaningful code-generation improvement, report that result instead of widening into an index redesign.

## Verification and handoff

The controller supplies the committed BASE and pre-repair release pin after the current f repair is reviewed. Preserve that pin, commit before verification, and run the exact identity/redefinition/alias/GC/nil-scope controls plus relevant package numeric/TRACE-start controls. Add a focused package-options preservation witness only if existing coverage does not exercise the reordered application path. Follow the common capped foreground workflow, formatting and affected all-target clippy, without repeating unrelated broad suites.

Write `task-8.8e2-perf-fix-r2-report.md`, append the result to the original e2 report, and retain exact commands/statuses and source/build/compiler hashes under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/runtime/e2/perf-fix-r2/`. Preserve a distinct release pin for scoped review and matched all-program evaluation. No new unsafe, dependency, side map, semantic ruling or global performance framework is authorized, and M6/final gates remain separate. Do not spawn agents.
