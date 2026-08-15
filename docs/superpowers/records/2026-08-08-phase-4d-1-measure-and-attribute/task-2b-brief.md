### Task 2b: Re-establish the baseline, because five speedups landed after it

Task 2's baseline was committed at `107febcd`. Five speedups landed after it -- `3799692d`, `b6b1d8a9`, `e1d50dda`, `c428ec8a`, `04ab4af6` -- so `perf-baseline.md`'s Phase 4d-1 section no longer describes this interpreter, and every task below that reads it would read a stale number.

**That those speedups landed at all contradicts this phase's own no-optimisation rule** (Global Constraints, `:18`). They were measured and they were the user's call, and the consequence is recorded rather than argued: this unit can produce a *current* baseline and attribution, and cannot produce a pre-optimisation one. The bar-before-optimisation property is partly spent and no document should claim otherwise.

**Staleness is already confirmed and the direction is known.** An indicative re-run on 2026-08-08 gave `arith` 2.60x against the committed 3.52x, `compound` 6.32x against 12.15x, `strings` 10.68x against 13.70x, `varlookup` 4.28x against 23.07x, and an internal cps ratio of 6.21x against 9.33x. **Those figures are not a baseline and must not be quoted as one**: their spreads reached 82.77 per cent on `strings` and 32.84 per cent on `arith`, against the committed baseline's 1.04 to 2.40 per cent, and an interval of 6.32x to 13.37x decides nothing. The machine was not quiet.

**Files:**
* Modify: `docs/superpowers/plans/perf-baseline.md`

- [ ] **Step 1: Re-run the committed harness on a quiet machine**

`./target/release/rexx-bench-suite`, built from `rust/`. Nothing else running -- no background build, no other benchmark, no editor indexing. The harness already interleaves and reports per-side spread; that spread is the check on whether the run is usable.

- [ ] **Step 2: Reject the run if its spread is worse than the committed baseline's**

The committed section's spreads are 1.04 to 2.40 per cent. A run whose spread is materially worse is not a baseline, and re-running it is cheaper than reasoning about which of two bad numbers to trust. Say in the document what spread the accepted run had.

- [ ] **Step 3: Record both figures, with their commits**

Follow the amendment discipline `phase-4c-gate.md` used for its criterion 4: carry **both wordings and the reason**. The old figures stay, marked as measured at `107febcd` and superseded, with the five speedup commits named as what moved them. A reader must be able to see that the numbers changed and why, rather than finding one set silently replaced.

- [ ] **Step 4: Commit**

---

