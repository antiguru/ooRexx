**CLOSED 2026-10-30 by Phase 9d Task 8.8g (3c76dbeeb).**

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

Historical Task 8.8d judgement, withdrawn by the correction below: the controller accepted M6 unreachability only in ordinary seeded mode, where the independent timer is disarmed and generated requests are cleared before yielding. Timer-mode observational equivalence remained unproven because independent ticks can arrive around the activity switch. The proposed next diagnostic was to establish incoming-request ownership and a specified observable under real timer-mode transitions, or supply a complete equivalence proof for controller judgement. The production SLICE clear remained intact, and the seeded argument alone did not close this queue.

## Task 8.8g correction and M6 catch

The universal seeded-unreachability premise omitted the REPLY continuation producer, as documented in the [source proof](../2026-10-18-phase-9d/task-8.8-m6-timer-proof.md). The compiled terminal-REPLY witness at `7662a51a3` records zero baseline deferrals, while deleting only `switch_to`'s SLICE clear at `0ee2685ec` records one incoming `[Native("SORTWITH"), SortComparator]` deferral and fails that attribution assertion. The RETURN twin passes separately in both revisions, and the exact inverse restoration at `3c76dbeeb` restores identical scheduler bytes and passing named tests. This judges M6 by reachable activity attribution, with no claim about forbidden language scheduling cadence or timer-mode observational equivalence.

The [Task 8.8g report](../2026-10-18-phase-9d/task-8.8g-report.md) retains baseline, mutation, restoration and control evidence for M6. The earlier [Task 8.8d report](../2026-10-18-phase-9d/task-8.8d-report.md#scheduler-dispositions) retains restoration commits and separately recorded compiled catches for [M7](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m7-mutant.txt), [M9](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m9-mutant.txt), [M10](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m10-mutant.txt), [M11](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m11-mutant.txt) and [M12](../../../docs/superpowers/records/2026-10-18-phase-9d/t8.8/instruments/m12-mutant.txt). Those earlier reports remain historical records, including the now-withdrawn universal seeded premise. The queue closes from these compiled catches while the production clear and the separate deferred-flag reset remain present.
