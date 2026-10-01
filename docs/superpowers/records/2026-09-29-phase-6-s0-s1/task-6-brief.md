### Task 6: `ObjRef` becomes `!Send` and `!Sync`; the countdown reload is bounded; S0 closes

**Files:** `rust/crates/rexx-core/src/` (the `ObjRef` definition), `rust/crates/rexx-exec/src/clause.rs`
(`NO_DEADLINE_SPACING` at `:35`, `countdown_reached` at `:282-297`), `phase-6-perf.md`.

- [ ] **Step 1:** Add a phantom raw-pointer marker to `ObjRef` so it is neither `Send` nor `Sync`.
      Add a compile-fail check (a doc test with `compile_fail`, or a `static_assertions`-free trait
      probe) that `ObjRef: Send` does not hold. `cargo check --workspace --all-targets` must pass
      unchanged (review 1 showed nothing sends one today).
- [ ] **Step 2:** Replace the `u32::MAX` reload with `CLAUSES_PER_CHECK` (1024) when there is no
      deadline, so the cold path runs every 1024 clauses always (spec section 4). The cold path does
      nothing new yet beyond reloading. Measure its cost on every program.
- [ ] **Step 3:** **S0 gate:** every Task 2 program's running total at most +0.3% beyond its noise
      band, wall clock within ±4%; record the table in `phase-6-perf.md`. Over budget: up to three
      rounds, then stop for a ruling.
- [ ] **Step 4:** Gates; commit.

