**CLOSED 2026-10-30 by Phase 9d Task 8.8f (3979b7e31).**

# RAISE DESCRIPTION object stored as its unsent rendering

`raise syntax 93.900 description (o)` with a noisy DEFAULTNAME: `condition('d')` answers `a NOISY` here; rexxref (instrc.xml:2866-2874, "evaluates the expression to obtain its character string value. This is the description") and the oracle answer `NOISY`. The rule sends at RAISE, the oracle at `condition('d')`; the timing difference needs a ruling (an untraced RAISE sends nothing on the oracle, docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr2/n2c/u.rex). Probe docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr2/n2b/q9.rex. Pre-existing (8.7c re-review r2, task-8.7c-rereview-r2.md queue candidate 2).

## Task 8.8f evidence

DESCRIPTION obtains its required string when RAISE evaluates it under approved Deviation 83. The timing witness converts before handler entry and repeated CONDITION(D) reads send nothing. A conversion that raises fails at RAISE, including the complete retained unfiled corpus witness. ADDITIONAL and RC keep their separate differential witness. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8f-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/`.
