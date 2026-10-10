# A seeded gate judged by the oracle

CLOSED as not wanted by Moritz, 2026-10-10. The seeded gate stays judged by the crate's own invariants with oracle differences reported (R1). A sampled oracle outcome set cannot cover every interleaving the policy reaches, so gating on it would fail correct rare outcomes.

Out of 6.1 scope by spec D9 and ruling R1 ("oracle differences reported, never gating"). Queued by Phase 6.1 Task 12 (2026-10-10).

The seeded gate fails on the crate's own invariants and records oracle differences in the gate record (spec section 4, "Oracle report"). A gate that fails on a difference from the committed oracle outcome set needs that set to cover every legal interleaving the policy reaches, which a sampled oracle set does not (scout C, "Tier-2 noise"). No probe: a design item. R1 frames the end state as a stand-alone crate, so this item may close as not wanted.

Suspected site: the judge in `tests/concurrency_tests.rs` `sim_gate` and the outcome sets in `rust/corpus/sim-oracle/`.
