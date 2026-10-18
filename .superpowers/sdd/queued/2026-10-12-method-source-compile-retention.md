**CLOSED: Phase 9b1 L8.** queued-probes/memcompile/rss.txt: 44992 kB at 50000 and 45160 kB at 100000 here, was 921808 kB; flat.

# Compiled method sources are never reclaimed

Found 2026-10-12 by the Phase 9a enhanced-semantics scout
(`.superpowers/sdd/2026-10-11-phase-9a/enhanced-semantics.md`, R17).

`memcompile.rex`: 50000 iterations of `.method~new('foo', "return 'foo'")`, no `enhanced`.
Crate max RSS 921808 kB, oracle 20324 kB. About 18 kB per compiled source stays live after the
Method object is unreachable. Any program that builds methods or routines from source in a loop
grows without bound.

Owner: unassigned (not Task 8b). Witness: RSS of the crate process itself (not of a wrapper) over
a loop, against the oracle's.
