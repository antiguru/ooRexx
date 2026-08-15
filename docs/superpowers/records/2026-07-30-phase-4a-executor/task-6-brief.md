### Task 6: The resolution plan

**Spec:** D16 in full.

**Files:**
- Create: `rust/crates/rexx-exec/src/plan.rs`, `src/activation.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/plan.rs`

**Interfaces:**
- Produces: `Plan { slots: HashMap<Box<[u8]>, usize>, len: usize }`, `build_plan(&CodeBody, &SymbolTable) -> Plan`, and on `Interp` a cache keyed by `(program_id, body_index)` where the loader assigns `program_id`.
- `Activation { plan: Rc<Plan>, extra: HashMap<Box<[u8]>, usize>, frame: SlotFrame, blocks: Vec<Block>, pc: usize, settings: Settings, program: Rc<Program> }`.

**`extra` is not optional and Task 3 already proved why.** The plan is an `Rc`, shared and immutable, built by an upfront pass that never saw a name introduced at run time — and such names exist in 4a: `DROP (v)` names its target at run time, and an interpreted fragment's bindings are visible to the enclosing body's own later clauses (measured: `interpret "newvar = 7"` then `say newvar + 1` prints 8). Resolution is `plan.slot_of(name).or_else(|| extra.get(name))`; allocation writes `extra` and calls `grow_slots`. Task 3 built this; you are inheriting its shape, not inventing one.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn a_tail_piece_and_a_plain_variable_share_one_slot() {
    // b = 2 ; say a.b -> A.2 ; a.2 = 'hit' ; say a.b -> hit
    // An implementer who gives tail pieces their own slots gets A.B.
}

#[test]
fn a_runtime_name_grows_the_frame() {
    // v = 'X' ; x = 1 ; drop (v) ; say x  ->  X
    // X may not appear in the body at all, so the plan cannot have a slot for it.
}

#[test]
fn names_are_keyed_upcased_but_tail_values_are_not() {
    // The two rules live in different decision blocks and are easy to swap.
}
```

- [ ] **Step 2: Run them to watch them fail**

- [ ] **Step 3: Implement the upfront pass**

One walk over the body's AST, collecting every referenced name and every `Tail::Variable` piece from `compound_parts`, assigning dense indices. **Not lazy**: a lazy design threads a "seen this name?" check through every site that touches a variable, which is a different algorithm and the wrong one. Run-time growth is the exception, not the normal path.

`Settings` lives on the `Activation`, inherited from the caller at call time. Measured: an internal call sees the caller's `DIGITS`, changes its own, and the caller is unaffected after `return`.

- [ ] **Step 4: Verify** — `cargo test -p rexx-exec plan`, plus the three oracle transcripts.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/plan.rs rust/crates/rexx-exec/src/activation.rs rust/crates/rexx-exec/tests/plan.rs
git commit -m "Resolve a body's variables once, keyed by name, cached on Interp"
```

---

