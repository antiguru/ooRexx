### Task 11: Close S1

**Files:** `phase-6-perf.md`, `phase-6-pinning.md`, `docs/superpowers/plans/phase-6-gate.md` (new,
the S0/S1 section).

- [ ] **Step 1:** **S1 gate:** every Task 2 program's running total against the Phase 6 base at most
      +0.3% beyond its noise band, measured with fresh builds of the base and HEAD in their own target
      directories with `Compiling` lines, and wall clock within ±4%; record the table. Over budget:
      the three-round rule, then stop and put the figures to Moritz.
- [ ] **Step 2:** Re-run the Task 3 counter over the concurrency tests; record, per kind, the would-be
      pinned parks that remain and compare with Task 3's baseline. This is the input to the S2 plan.
- [ ] **Step 3:** Record the cap-lifted recursion depth for calls and sends, and the Error 11 witnesses.
- [ ] **Step 4:** Write `phase-6-gate.md`'s S0/S1 section from the gate run, quoting its status lines;
      gates; commit.
