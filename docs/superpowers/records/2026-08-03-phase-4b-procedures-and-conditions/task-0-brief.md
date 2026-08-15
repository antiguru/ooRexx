### Task 0: The shared owner table, the subset union, and an owner in every loud message

**This task runs first.** Every later task edits the owner tables, and three later tasks assert a property the tree cannot currently express.

**Files:**
- Create: `rust/crates/rexx-exec/tests/owners.rs`
- Modify: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`
- Modify: `rust/crates/rexx-exec/src/lib.rs` (the two loud-message sites, `:444` and `:468`)
- Modify: `rust/crates/rexx-exec/tests/corpus.rs`, `tests/coverage.rs`, `tests/collect_stress.rs` (the `read_subset` call sites)

**Why:** three separate mechanisms in the tree defeat the later tasks as written.

**Inherited item I36.** `coverage.rs` and `loud.rs` duplicate the owner table **by hand**, because an integration test cannot `mod` another test binary's directory and no shared module was in scope in 4a. 4b and 4c edit both on every variant they deliver, and a divergence between them is caught by nothing.

**The three mechanisms, all verified in the tree:**

1. **`loud.rs`'s witness set is variant-grained and asserted complete.** `assert_witness_set_is_complete` requires exactly one witness per out-of-scope `InstructionKind` variant, "no more and no fewer". So the moment `InstructionKind::Call` moves in scope, the `call sub` witness must be **deleted** -- and `Call::Qualified` (Phase 5's) and `Call::Trap` (Task 7's) are left with no loudness witness anywhere. Same shape for `Signal::Trap` between Tasks 6 and 7. The first revision caught the `coverage.rs` half of this and missed the `loud.rs` half.
2. **No loud message names a phase.** `every_out_of_scope_variant_fails_loudly` compares `exit_code` and nothing else; `Witness::owner` is only printed on failure. Measured, the emitted text is `rexx-exec: CALL is not implemented`. So "must keep failing loudly with owner `Phase 5`" is a property nothing can express, and any test asserting `stderr.contains("4c")` fails today.
3. **`EXPECTED_OUT_OF_SCOPE` (`coverage.rs:606`) and four hardcoded counts** are pinned literals that every variant-flipping task breaks. They are named in no later task.

- [ ] **Step 1: Create `tests/owners.rs` as the single owner table, and have both harnesses read it**

Use `#[path]` or a shared file both `mod`-include. Then add a test asserting the two harnesses see the *same* table, not two lists that happen to agree.

- [ ] **Step 2: Make the witness table arm-grained for the split variants**

`Call::Named` in scope from Task 3; `Call::Dynamic` in scope from Task 3; `Call::Qualified`, `Call::Trap` and `Signal::Trap` each their own witness row with its own owner. A variant whose inner forms have different owners needs one row per form, or implementing the outer variant silently drops coverage of the rest.

- [ ] **Step 3: Give the loud message an owner, and assert it**

Add an `owner: &'static str` to the loud payload sourced from the shared table, so the message reads `rexx-exec: CALL is not implemented (Phase 5)` or similar -- take the exact format from what reads best beside the existing text, and record it here so later tasks assert the same shape. Then make `every_out_of_scope_variant_fails_loudly` assert the emitted stderr contains the witness's owner.

- [ ] **Step 4: Change the `read_subset` call sites to take a list of subset files**

`&[&Path]`, union semantics. There are **two** call sites in `coverage.rs`, not one, plus `corpus.rs` and `collect_stress.rs`. Keep each harness's own copy of the reader -- factoring those together is a separate change.

- [ ] **Step 5: Document `EXPECTED_OUT_OF_SCOPE` and the four hardcoded counts in `tests/owners.rs`'s module doc**

Name each one and say that any task moving a variant in scope must update it. This is the only place a later implementer will look.

- [ ] **Step 6: Run the full suite, commit**

Expected: no behaviour change, all tests green. This task ships zero interpreter functionality by design.

---

