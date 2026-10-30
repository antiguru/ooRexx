**CLOSED 2026-10-30 by Phase 9d Task 8.8f (3979b7e31).**

# Which messages a hash collection sends

Found by Phase 9d Task 8.7c (report Queue candidates 1; confirmed by its review); predates the task. `Table~remove`, `Set~remove` and `Set~removeItem` send the index's `HASHCODE` twice where the oracle sends it once; `Bag~hasItem` and `Relation~allAt`, `~hasItem`, `~items` and `~removeItem` scan with `==` where the oracle hashes the index and finds an empty bucket. Moritz's 2026-10-29 ruling ("Keep rexxref, Deviation": the index's `==` is sent, as rexxref says) stands; match which other messages are sent and how often.
Probe: docs/superpowers/records/2026-10-18-phase-9d/t8.7c/probes/primframe/f4.rex (the rows outside the 8.7c witness).

## Task 8.8f evidence

Removals reuse the located slot and predecessor, while indexed Relation and Bag operations walk the selected bucket. Collision controls compare indexes before items and preserve Deviation 78 equality sends. Store replacement during HASHCODE or equality discards stale lookup state without recomputing HASHCODE. Moritz approved keeping ITEMS=0 for the separately retained equality-reentry oracle ITEMS=-1 defect as Deviation 84 on 2026-10-30, replying "Yes, 0." See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8f-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/`.
