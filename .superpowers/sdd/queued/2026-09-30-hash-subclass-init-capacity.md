# Hash-collection subclass keeps NEW's capacity when its INIT skips super

Found by Phase 6 Task 9's re-review (2026-09-30), pre-existing, not fixed: the oracle sizes a hash
collection in INIT, so a subclass whose INIT does not forward to super differs from this crate, which
sizes in NEW. Repro: ask for it in .superpowers/sdd/2026-09-29-phase-6-s0-s1/task-9-rereview.md.

Closed 2026-10-12 by Phase 9a Task 8 at `1c0e1d769`: INIT sizes the store, and a subclass whose INIT does not forward gets the default one. Witness `rust/corpus/lang/hash_subclass_init_capacity.rex`.
