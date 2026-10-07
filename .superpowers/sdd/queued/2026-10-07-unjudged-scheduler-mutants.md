# 2026-10-07-unjudged-scheduler-mutants

Found by the Phase 6 final review (`.superpowers/sdd/2026-10-01-phase-6-s2-s5/final-review.md`,
"Declined to judge"). Each mutant stayed green with `REXX_CORPUS_GATE=1`; none was shown
observable or unobservable.

- M6, M7: `switch_to` keeps the SLICE request / `slice_deferred`.
- M9, M10, M11: `cancel_wait` keeps `when_parked`, the guard-queue entry, the sleeper.
- M12: `Activity::object_roots` drops `failed_sends`.

Candidates for the first seeds of `2026-10-07-seeded-random-switch-mode`.
