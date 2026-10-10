# A shrinker for simulation decision traces

Out of 6.1 scope by spec D9 ("the shrinker ... queued"); the DST spec review asked for it (`.superpowers/sdd/2026-10-07-phase-6-1/spec-review-dst.md` I6, O10). Queued by Phase 6.1 Task 12 (2026-10-10).

Phase 6.1 ships `sim:trace=FILE` replay and a decision trace per run. A shrinker deletes trace entries (preemptions, order picks, collection points) while the failure stays red, so a found bug reaches its crate test (explicit switch points) from a minimal schedule. No probe: a design item.

Suspected site: the trace reader and replay in the simulation mode (`scheduler.rs` `SimConfig`, the trace file format Task 9 wrote).
