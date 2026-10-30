**CLOSED 2026-10-30 by Phase 9d Task 8.8f (3979b7e31).**

# Which messages a hash collection sends

Found by Phase 9d Task 8.7c (report Queue candidates 1; confirmed by its review); predates the task. `Table~remove`, `Set~remove` and `Set~removeItem` send the index's `HASHCODE` twice where the oracle sends it once; `Bag~hasItem` and `Relation~allAt`, `~hasItem`, `~items` and `~removeItem` scan with `==` where the oracle hashes the index and finds an empty bucket. Moritz's 2026-10-29 ruling ("Keep rexxref, Deviation": the index's `==` is sent, as rexxref says) stands; match which other messages are sent and how often.
Probe: docs/superpowers/records/2026-10-18-phase-9d/t8.7c/probes/primframe/f4.rex (the rows outside the 8.7c witness).

## Task 8.8f evidence

Removals reuse the located slot and predecessor, while indexed Relation and Bag operations walk the selected bucket. Collision controls compare indexes before items and preserve Deviation 78 equality sends. Lookup paths select their store after HASHCODE without recomputing it, and removals and Relation snapshot operations validate the current locations they use. HASITEM instead roots the contents selected for its operation and follows that contents' live chain through equality callbacks, including growth that detaches those arrays from the receiver. EMPTY clears the active arrays in place, so both EMPTY alone and EMPTY followed by growth remain visible to the active lookup without a geometry heuristic.

Moritz approved keeping ITEMS=0 for the separately retained equality-reentry oracle ITEMS=-1 defect as Deviation 84 on 2026-10-30, replying "Yes, 0." That ruling concerns the count defect, independently of the observable-message corrections. The final-review wave adds no semantic departure. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8f-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/`.

## Final review correction

The final-review wave supersedes the earlier blanket description of equality replacement handling for HASITEM. Its growth and mixed EMPTY/growth witnesses each preserve one HASHCODE and 18 equality sends, matching the oracle rather than restarting into all 41 current entries. Bag HASITEM compares its optional index before hashing and then performs the index-only lookup, without Relation's extra item comparison. The normal and actual collection-on-allocation controls also preserve independent copies and suppliers for Table, IdentityTable, Relation, Bag, Directory and StringTable. See `task-8.8-final-fix-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/fix/` for the superseding source and evidence.
