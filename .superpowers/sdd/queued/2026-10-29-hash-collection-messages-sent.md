**OPEN, queued 2026-10-29 by the controller; proposed 9d (Task 8.8).**

# Which messages a hash collection sends

Found by Phase 9d Task 8.7c (report Queue candidates 1; confirmed by its review); predates the task. `Table~remove`, `Set~remove` and `Set~removeItem` send the index's `HASHCODE` twice where the oracle sends it once; `Bag~hasItem` and `Relation~allAt`, `~hasItem`, `~items` and `~removeItem` scan with `==` where the oracle hashes the index and finds an empty bucket. Moritz's 2026-10-29 ruling ("Keep rexxref, Deviation": the index's `==` is sent, as rexxref says) stands; match which other messages are sent and how often.
Probe: docs/superpowers/records/2026-10-18-phase-9d/t8.7c/probes/primframe/f4.rex (the rows outside the 8.7c witness).
