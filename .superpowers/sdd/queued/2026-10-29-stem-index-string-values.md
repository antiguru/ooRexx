**CLOSED 2026-10-30 by Phase 9d Task 8.8f (3979b7e31).**

# Stem [] does not take its indexes' string values

collclasses.xml Stem `[]`: it "concatenates the index string values". `a.KM = 'km-tail'; say a.[kk]` with `kk`'s `makeString` answering `KM`: the oracle answers `km-tail`, the crate `A.a KMAKE`. Under SIGNAL ON NOSTRING, `a.[nn]` with no `makeString`: the oracle raises NOSTRING, the crate answers `A.a NSTR`. A reference index and a stem index the same. Probes docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr1/r1a/a.rex, r1b/c.rex, r1c/d.rex. Present at e620097f0 (8.7c re-review r1, task-8.7c-rereview-r1.md queue candidate 1).

## Task 8.8f evidence

Stem index positions use required string values left to right, including omitted and leading-empty positions. Reading, writing, removal and hasIndex share the converted tail. NOSTRING and raising conversion failures propagate at the operation. Reference and stem-default controls retain Deviations 72 and 77. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8f-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/`.
