**CLOSED 2026-10-30 by Phase 9d Task 8.8f (3979b7e31).**

# Number max/min with an object argument

`numeric digits 5; say 5~max(k)`, `k`'s makeString `1234567890`: the oracle raises `LOSTDIGITS 1234567890`; this crate sends DEFAULTNAME and raises 93.904 found "DNK". The argument's string value is the rule's. Probe docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr2/ld/l20.rex. Pre-existing (8.7c re-review r2, task-8.7c-rereview-r2.md queue candidate 3).

## Task 8.8f evidence

MAX and MIN convert object arguments once, preserving required-string failure and argument errors. The whole-value prepass stops when quietness is impossible and retains ordinary arguments by reference. Converted and plain whole-number controls follow Deviation 44, including its LOSTDIGITS difference from the oracle. Missing or nonnumeric earlier arguments prevent later conversion sends. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8f-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/`.
