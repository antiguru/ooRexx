### Task 1: A real `INTERPRET`, replacing the Task 3 spike

**Files:**
- Modify: `rust/crates/rexx-exec/src/lib.rs` -- delete `run_program_interpret_spike` (**`:1114`**), the `interpret_spike` field (**`:784`**), its constructor parameter (**`:830`**), and `execute`'s parameter (**`:1169`**); add a `#[cfg(test)] mod tests` for the fragment-lifetime proofs
- Modify: `rust/crates/rexx-exec/src/run.rs` (the `InstructionKind::Interpret` arm)
- Modify: `rust/crates/rexx-exec/tests/spike.rs`
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Interfaces:**
- Consumes: `Fragment { source, body: CodeBody, symbols }`; `Interp::alloc_with`; `step_in_temps_frame`; Task 0's shared owner table.
- Produces: nothing public. No new public item.

**Why:** D1. The spike's public entry point exists only because a private subject needed an integration test, and its own doc records that a `#[cfg(test)] mod tests` inside `lib.rs` could prove the same lifetime with no public surface. That trade is remade here.

**Check the tree before writing anything.** A private fragment-running helper may already do most of what Step 6 describes. Find it and reuse it rather than writing a second one.

**Inherited items this task pays for:**

* **I7.** Delete the spike surface; move `tests/spike.rs`'s three fragment tests into `lib.rs`.
* **I8.** Fragment plans are built, used and dropped, and **stay that way**. Revision 6's `(enclosing body, fragment id)` cache key was withdrawn as "sound and useless": fragment text varies per execution, so every lookup misses while every entry is retained. A cache keyed by fragment *text* is permitted only if this task measures a hit rate on a real program and reports it. Absent that measurement, do not add one.
* **I9.** `Fragment::body.labels` is always empty. Measured and settled in 4a's Task 1: a label inside `INTERPRET` text is error 47.1 both ways. Do not add label handling.
* **I16.** `step_in_temps_frame` is the single chokepoint healing six `push_frame` sites in `eval.rs` that skip their `pop_frame` on the `?` path, and it is the **only** caller of `step` in the crate. 4a's conclusion that `SIGNAL ON SYNTAX` cannot accumulate temps leaks rests entirely on that. **If this task moves execution off that chokepoint, say so in the report** -- Task 7 depends on the analysis.
* **I21.** Allocations go through `Interp::alloc_with`.
* **I22.** `pop_frame`'s truncation semantics are load-bearing. **No assert may be added there** without balancing the six `eval.rs` sites first. A debug tripwire in `step_in_temps_frame` asserting temps balance on the `Ok` path was scheduled in 4a and not built; it needs a `temps_len()` accessor. Building it is welcome; changing `pop_frame` is not.

- [ ] **Step 1: Move the three fragment-lifetime tests into `lib.rs`**

Reproduce each as a `#[test]` in a `#[cfg(test)] mod tests`, driving `Interp` directly. Keep each test's doc comment verbatim: they record why the lifetime is what it is.

- [ ] **Step 2: Change the loud fixtures in `spike.rs` before they break**

At least two tests there use constructs 4b implements and assert `NOT_IMPLEMENTED_EXIT`. One uses `call "sub"` and asserts stderr contains `"CALL"`; find the others by running the suite after Step 6 and reading what fails. This is the **fourth** occurrence in this project of a witness implemented out from under a test.

Replace each fixture with a **message send** (`q~append(1)`), which the spec assigns to Phase 5 outright rather than by ruling. Give the exact expected stderr text in the test, taken from a run. Add a one-line comment saying why a message send and not `PARSE` or `ADDRESS`: those are 4c's and would break again in weeks.

- [ ] **Step 3: Run the suite to see the moved tests pass before anything else changes**

- [ ] **Step 4: Write the failing `INTERPRET` test**

```rust
#[test]
fn interpret_binds_a_name_the_enclosing_body_never_mentions() {
    // Measured on the oracle in 4a: the binding outlives the fragment.
    let out = run_source(b"interpret \"zork = 42\"\ninterpret \"say zork\"\n");
    assert_eq!(out.stdout, b"42\n");
    assert_eq!(out.exit, 0);
}
```

`run_source` is shorthand: use whatever helper the surrounding tests already use, or drive `Interp` directly.

- [ ] **Step 5: Run it and watch it fail**

- [ ] **Step 6: Implement the `InstructionKind::Interpret` arm**

Evaluate the expression to a string, parse it as a `Fragment`, give it a plan, execute it against the **current** activation: same frame, same slots, same settings. A name the enclosing plan never saw goes through `Plan::slot_of` into `Activation::extra`.

The fragment's instruction list is separate, so `pc` cannot continue into it. Execute through the same bounded sub-loop shape `run_bounded` uses, and forward an unowned `Flow` outward.

**Two escapes must be measured against the oracle before you choose their semantics, and the first revision of this plan got one wrong by asserting it:**

* `LEAVE` naming a loop that encloses the `INTERPRET` instruction, from inside the fragment.
* `RETURN` inside a fragment, both in the main body and inside a called routine.

Measure both, record the transcripts in the report, and implement what the oracle does. If either needs machinery Task 3 has not built, leave it failing loudly and say so -- do not invent a plausible answer.

- [ ] **Step 7: Run the suite**

- [ ] **Step 8: Create `rust/corpus/phase-4b.txt` with this task's witness**

**This task creates the 4b subset file, not Task 10.** Moving `Interpret` in scope requires a witness program in the subset or `every_in_scope_variant_is_witnessed_by_the_phase_4a_subset` (`tests/coverage.rs:528`) fails, and the 4a subset cannot hold it: `phase-4a.txt`'s own header lists `INTERPRET` among the constructs it excludes by definition.

The witness already exists and is already correctly absent from the 4a subset: **`rust/corpus/lang/interpret_dynamic.rex`**. It exercises a dynamic fragment, a fragment that binds a name the enclosing body never mentions, and a fragment inside a `DO` body. List it in `phase-4b.txt` with a header in the same shape as `phase-4a.txt`'s, saying which constructs the 4b subset admits and which it still excludes (the corpus rules in Task 10 are the list).

Then point the harnesses at the union. Task 0 made `read_subset` take a list, but every call site still passes a one-element slice. **Two of them must change and one must not:**

* `every_in_scope_variant_is_witnessed_by_the_phase_4a_subset` (`:538`) reads the **union**, and its name is now wrong -- rename it to say "subsets".
* `collect_stress`'s call site reads the **union**.
* The `EXPECTED_SUBSET` pin (`:516`) stays **`phase-4a.txt` only**. It exists to catch drift in that one file's line list, and widening it would destroy that.

- [ ] **Step 9: Move `Interpret` in scope in `tests/owners.rs`, and update `EXPECTED_OUT_OF_SCOPE` and the four counts Task 0's module doc names**

- [ ] **Step 9: Commit**

---

