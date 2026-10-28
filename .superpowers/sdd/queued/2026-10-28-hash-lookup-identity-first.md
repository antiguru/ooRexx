**OPEN, queued 2026-10-28 by the controller; proposed 9d (Task 8.7c).**

# A hash collection's lookup sends == to the identical index object

Found by the Phase 9d Task 8.7b review (Queue candidates); predates the task. A `.k` with a constant `hashCode` and a printing `==`: `t[k] = 2` and `t[k]` after `t[k] = 1` print `== called` once here and never on the oracle, which compares identity first.
Probe: docs/superpowers/records/2026-10-18-phase-9d/t8.7b/review-probes/hash-identity-first-idq.rex. Derive the rule from rexxref (collection index equality, hashCode) before following the oracle; keep the untraced lookup's cost within budget.
