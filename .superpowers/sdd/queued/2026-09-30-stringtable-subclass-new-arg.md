**CLOSED: 1c0e1d769.** closed by Phase 9a Task 8; its witness agrees.

# StringTable subclass ~new refuses a non-length first argument

Found by Phase 6 Task 9's re-review (2026-09-30), pre-existing, not fixed: a StringTable subclass's
~new refuses a first argument that is not a length; the oracle accepts it. Repro: see
.superpowers/sdd/2026-09-29-phase-6-s0-s1/task-9-rereview.md.

Closed 2026-10-12 by Phase 9a Task 8 at `1c0e1d769`: NEW checks no capacity; INIT sizes the store. Witness `rust/corpus/lang/hash_subclass_init_capacity.rex` (`.ks~new('abcdefghij', 'q')`).
