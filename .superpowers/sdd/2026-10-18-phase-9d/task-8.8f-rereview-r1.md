F1: ADDRESSED. Removal now validates the current location and returns the value actually removed.

F2: ADDRESSED. Relation snapshots now carry their validated store with their selected slots.

## Finding verdicts

* **F1: REMOVEITEM reported a skipped removal.** `rust/crates/rexx-exec/src/dispatch/hash/removal.rs:55` validates backing-array identity and bucket provenance, then re-finds the selected slot and its current predecessor before passing current free-list metadata to removal. `removal.rs:193` and `rust/crates/rexx-exec/src/dispatch/hash/relation.rs:311` propagate the actual removed value and restart when the location no longer belongs to the current store. This handles insertion, overwrite, and same-bucket reuse without returning the captured pre-callback victim. The Table and indexed/unindexed Relation fixtures in `tests/semantic_boundaries/reentry-review-removeitem.rex:1`, `reentry-removeitem-overwrite.rex:1`, `reentry-removeitem-reused.rex:1`, and `reentry-relation-removeitem.rex:1` pin those distinct cases.
* **F2: ALLAT read values from its pre-HASHCODE store.** `rust/crates/rexx-exec/src/dispatch/hash/relation.rs:21` returns the store together with selected slots, and ALLAT/SUPPLIER consume that pair at `relation.rs:88` and `:212`. The search computes HASHCODE once, roots saved indexes, and validates array identity, chain membership, and index occupants after equality changes collection state. Invalidated snapshots restart using the cached hash, so earlier selected slots cannot cross into a replacement store. The separate hash/equality replacement fixtures and `tests/semantic_boundaries/reentry-relation-equality-counts.rex:1` cover refreshed answers and the measured one HASHCODE/two equality sends.

## New breakage in the fix diff

None found. The current-slot rule deliberately allows a callback's replacement or prepended value to be removed, and the retained paired outputs support that behavior. Saved backing arrays are explicitly rooted at `relation.rs:31`, `relation.rs:289`, and `removal.rs:181`, while captured index references are rooted before equality callbacks. These roots conservatively retain arrays across retries, although identity comparisons alone do not require liveness, so their retention and profiling cost remain part of the controller's performance assessment.

## Evidence checked

The reviewed range is `d84ac3917` through `8854b1a42`, with runtime source at `3e1aa9c87`. I read the fix diff in bounded passes and checked the appended original report plus the dedicated fix report. Both changed source hashes and the corrected release pin hash match `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/fix-r1/release-pin.json:1`. No additional source exploration, suite execution, Git mutation, or source edit was needed.

The retained `fix-r1/checks/focused` records report 1,235 library tests and three semantic tests passing. The `boundaries-policy` records report the semantic, coverage, ownership, refusal-site, and representation checks passing before the final saved-array roots. The final `rooted-boundaries`, `rooted-format`, `rooted-lint`, and `release-final` records all report status 0, and final lint stderr contains only normal Cargo progress. The existing semantic harness exercises every added fixture normally and under collection on every allocation.

The `fix-r1/checks/final-probes.stdout:1` record contains all ten oracle/crate stdout, stderr, and status triples, agreeing with `fix-r1/final-comparison.json:1`. The original Table insertion reproduction now removes the victim, overwrite returns replacement, and reused/prepended slots return the actual extra value. ALLAT and indexed SUPPLIER return the new stored value after either HASHCODE or equality replaces the arrays. The explicit equality-reentry witness prints `calls 1 2` on both sides, with status 0 and empty stderr throughout.

## Out-of-scope observations

None newly identified. The previously reported oracle ITEMS=-1 discrepancy remains separately pending, and this round adds no Deviation 84. Full own-BASE profiling and final broad gates remain controller-owned. This scoped verdict does not claim those gates complete.

## Verdict

Fix round: All findings addressed, no new Critical or Important breakage. F1 and F2 are closed for this review round.
