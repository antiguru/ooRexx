**OPEN: Task 8.8d judged five mutants; M6 timer-mode equivalence remains with the controller.**

# 2026-10-07-unjudged-scheduler-mutants

Found by the Phase 6 final review (`.superpowers/sdd/2026-10-01-phase-6-s2-s5/final-review.md`,
"Declined to judge"). Each mutant stayed green with `REXX_CORPUS_GATE=1`; none was shown
observable or unobservable.

* M6, M7: `switch_to` keeps the SLICE request / `slice_deferred`.
* M9, M10, M11: `cancel_wait` keeps `when_parked`, the guard-queue entry, the sleeper.
* M12: `Activity::object_roots` drops `failed_sends`.

Candidates for the first seeds of `2026-10-07-seeded-random-switch-mode`.

## Task 8.8d dispositions

M7 is caught by the existing per-pin activity attribution control with the pinning feature enabled. M9, M10 and M11 are each caught by a separate cancelled-category counter and observable continued activity behavior. M12 records 25 settlement collections, the exact attached condition identity and CODE 42.3 at baseline, then a dead old handle with same-slot generation reuse and missing attachment under the compiled mutant. Exact edits, restored green tests and per-mutant revisions appear in the [Task 8.8d report](../2026-10-18-phase-9d/task-8.8d-report.md).

The controller accepts M6 unreachability only in ordinary seeded mode, where the independent timer is disarmed and generated requests are cleared before yielding. Timer-mode observational equivalence remains unproven because independent ticks can arrive around the activity switch. The next diagnostic must establish incoming-request ownership and a specified observable under real timer-mode transitions, or supply a complete equivalence proof for controller judgement. The production SLICE clear remains intact, and this queue cannot close from the seeded argument alone.
