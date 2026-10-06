### Task 25: The concurrency tests and pinned waits

**Files:** `tests/concurrency_tests.rs` (Task 1's runner), `phase-6-pinning.md`, `phase-6-gate.md`
(S5 section), roadmap row 6.

- [ ] **Step 1:** Criterion 1: every test of `phase-6-pinning.md`'s derived list runs on both sides
      through Task 1's runner (one test per run), then each group whole (the runner extended to
      whole-group runs here); each passes on this crate and agrees with the oracle, normally and
      under `EveryOpportunity` with the same result in both modes; differences are recorded with a
      reason or fixed.
- [ ] **Step 2:** Criterion 9: the pinning report over the corpus and the criterion-1 tests, normally
      and under `EveryOpportunity`: every pinned park, deferred slice, immovable REPLY and inverted
      wait by kind; no inverted-wait refusal and no hang in either mode.
- [ ] **Step 3:** Criterion 10: every *(verify)* item of spec section 6 settled with its command.
- [ ] **Step 4:** Write `phase-6-gate.md`'s S5 section quoting each criterion's evidence; gates;
      commit.

