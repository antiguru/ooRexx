**OPEN, queued 2026-10-29 by the controller; proposed 9d (Task 8.8).**

# Stem [] does not take its indexes' string values

collclasses.xml Stem `[]`: it "concatenates the index string values". `a.KM = 'km-tail'; say a.[kk]` with `kk`'s `makeString` answering `KM`: the oracle answers `km-tail`, the crate `A.a KMAKE`. Under SIGNAL ON NOSTRING, `a.[nn]` with no `makeString`: the oracle raises NOSTRING, the crate answers `A.a NSTR`. A reference index and a stem index the same. Probes docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr1/r1a/a.rex, r1b/c.rex, r1c/d.rex. Present at e620097f0 (8.7c re-review r1, task-8.7c-rereview-r1.md queue candidate 1).
