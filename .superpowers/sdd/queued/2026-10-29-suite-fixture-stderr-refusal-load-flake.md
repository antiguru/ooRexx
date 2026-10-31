**OPEN FOLLOW-UP, approved by Moritz 2026-10-31 ("lgtm"); outside Task 8.8 closure.**

# suite fixtures test fails once under load

`fixtures::a_child_refusing_on_the_inherited_stderr_leaves_the_run_ended_normally` (rexx-bench-suite or suite crate) failed once in the Phase 9d Task 8.7c fix round 2 workspace run and passed 3 of 3 alone (docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r2/close-workspace-tests.txt). Find the timing dependence and fix it, or add it to the known-flake list with its cause.

Re-review r1 of 8.7c (task-8.7c-rereview-r1.md) did not reproduce it: alone and with the fixtures module, plain and under lib-test load, every run passed. See also 2026-10-29-grandchild-sleeps-process-wide.md.

## Task 8.8c bounded diagnosis

The targeted fixture passed in independent bounded runs with full per-run records under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/stderr/`. Each retained second run has one iteration, TEST_A started and passed, no failed/blocked entries, no load/early-end block, normal exit 0 and a Tests ran summary. The records include the exact child.rex and started marker plus canonical run, group and copied-interpreter paths before cleanup, while stderr still contains the expected child's refusal. This remains open because the missing pass did not recur and no assertion, incomplete-run or path-ownership cause was established.

The controller owns the next pending full-suite/workspace reproduction with REXX_SUITE_DIAGNOSTICS set. A future failed run must be classified from its GroupRun, summary and actual child status before changing end_reason or the fixture. The existing refusal expectations remain intact and the complete fixture target also passes. No Flaky licence or known-failure entry was added.

## Final Task 8.8 workspace observation

The fixture passed its first complete default-target execution in the resumed validation and its single fresh final execution at `b51a9466a5ccfa365bcf74e232f6e24c23fb9765`. The final suite harness passed 69 tests with one existing ignored diagnostic, with raw channels and fixture diagnostics retained under `docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-gates/final/`. These observations do not establish the historical missing-pass cause, so this issue remains open without a known-failure entry or Flaky licence. Root asked Moritz whether to retain this as an open follow-up or require its cause before Task 8.8 closure, and that disposition remains unanswered.

## Approved follow-up disposition

Moritz answered "lgtm" on 2026-10-31 to the explicit proposal to accept the instruction-cost residuals and retain this investigation as an open follow-up. This approval removes the unresolved cause from Task 8.8 closure requirements while leaving the investigation open. The controller owns the follow-up, retaining the existing diagnostic capture and requiring a future recurrence to be classified from raw channels, GroupRun, summary and child status before any fix or licence proposal. No cause, known-failure entry or Flaky licence is inferred from the passing final run.
