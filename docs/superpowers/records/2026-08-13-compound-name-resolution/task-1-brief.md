### Task 1: the split, computed once

**Files:**
* Modify: `rust/crates/rexx-exec/src/plan.rs` (the new table, filled by the pass that already walks these names)
* Modify: `rust/crates/rexx-exec/src/stem.rs` (`tail_key` reads the table)
* Modify: `rust/crates/rexx-exec/src/eval.rs` (the two stem-name sites, `:407` and `:457`)

**Interfaces:**
* Consumes: `Plan::note_compound_name`, which already splits every compound name and assigns each piece a slot; `rexx_parse::compound_parts`; `Tail`.
* Produces: a per-compound entry on `Plan`, addressed by the compound's own `SymbolId`, holding the stem name and the tail pieces in source order, with each piece marked constant or variable.

- [ ] **Step 1: decide how the table is addressed, by measuring rather than by taste**

`by_symbol` is a `HashMap<SymbolId, usize>`, and the profile shows `hash_one::<&SymbolId>` already costing about 1%.
A `Vec` indexed by `SymbolId` costs an index and no hash, and is correct only if ids are dense from zero within one body.
**Establish which it is from `rexx-parse`'s interner before choosing**, and say in the report what you found and what you chose. Do not assume density; do not assume sparsity.

- [ ] **Step 2: fill the table in the pass that already walks these names**

`note_compound_name` has the split in hand and throws it away. It needs the compound's `SymbolId` to key the entry -- check its callers, since one of them is `note_variable_ref`, and say in the report whether every caller has an id or whether only some do.
A name reached without an id is the case to handle explicitly rather than to leave to a fallback nobody named.

- [ ] **Step 3: read the table in `tail_key`, keeping its behaviour exactly**

`tail_key` must still: join pieces with `.`, take a constant piece verbatim and case-sensitively, read a variable piece's *current value* through the ordinary variable path, derive an unset name's own spelling, and render with `to_text`.
Only the split moves. Slot resolution stays by name in this task -- that is Task 2 -- so `read_by_name` is still what a variable piece goes through.

- [ ] **Step 4: the two stem sites in `eval.rs`** use the table's stem name rather than splitting again.

- [ ] **Step 5: the fallback, named**

A body reached without a plan entry -- an `INTERPRET` fragment's own compound, whose ids belong to a different `SymbolTable` -- must still work.
`fragment_plan` translates a fragment's ids to enclosing slots through the *text*, so a fragment's compound cannot be keyed by the enclosing plan's ids.
Decide what happens there, implement it, and pin it with a case that runs an `INTERPRET` containing a compound reference. **An `INTERPRET` in a loop is the shape most likely to be wrong.**

- [ ] **Step 6: prove the answers did not move**

The corpus sweep and `ir_dual` are the net. Beyond them, capture from the oracle: a compound with a constant tail, with a variable tail, with several tails, with an unset tail variable, with a tail variable whose value changes between references, a bare stem, `DROP` of a compound, and a compound built inside an `INTERPRET`.

- [ ] **Step 7: gates, then commit.** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `memcap 8G cargo test --workspace --no-fail-fast`.

- [ ] **Step 8: measure this task alone**

`perf stat -e instructions:u` over `rexxcps` at a fixed count, before and after, one run each, reported in the task report.
Instructions rather than wall clock: entry 27 records that the wall clock on these axes moves by more than this whole plan is worth, from layout alone, while the instruction counter's arm-internal noise is at or below 0.005%.

---

