### Task 1: `rexx-parse` gives the main body a `CodeBody`

**Spec:** "The borrow shape", the paragraph beginning "One `rexx-parse` change is required".

**Files:**
- Modify: `rust/crates/rexx-parse/src/lib.rs`
- Modify: `rust/crates/rexx-parse/tests/program.rs`, and any test that names `Program::instructions` or `Program::labels`

**Interfaces:**
- Produces: `Program { source, main: CodeBody, directives, symbols }` and `Fragment { source, body: CodeBody, symbols }`.
- `CodeBody { instructions: Vec<Instruction>, labels: BTreeMap<Box<[u8]>, usize> }` is unchanged and keeps its `Clone, PartialEq, Eq, Debug, Default` derives.

**Why:** `fn eval(&mut self, body: &CodeBody, …)` cannot be called for the body 4a actually runs. `Program` holds `instructions` and `labels` as sibling fields and `Fragment` has no label table at all, so there is no borrowed `CodeBody` view to hand the evaluator, and behind an `Rc` you cannot make one without cloning both vectors per call.

- [ ] **Step 1: Measure whether an `INTERPRET` fragment may contain a label**

Run, wrapped: a program whose body is `interpret "lab: nop"` and a second whose body is `interpret "signal lab; lab: nop"`. Record the exact error number and text if either is rejected.

Expected from Phase 3's own note: a label inside `INTERPRET` text is error 47.1, so `Fragment`'s label table is always empty. **Confirm it rather than assuming it**, and put the transcript in the task report. If it turns out labels are legal, say so and stop: the `Fragment` field is then load-bearing and 4b needs to know.

- [ ] **Step 2: Change the two structs**

`Program::instructions` and `Program::labels` become `Program::main: CodeBody`. `Fragment::instructions` becomes `Fragment::body: CodeBody`. Keep every doc comment: move each field's comment onto the corresponding `CodeBody` field's use site, and do not drop the paragraph explaining why labels are keyed by value rather than by `SymbolId`.

In `parse_program` and `parse_interpret`, `parsed.main` already *is* a `CodeBody`, so both become a move rather than a field split.

- [ ] **Step 3: Update callers and tests**

`cargo test -p rexx-parse` names every caller. Prefer `program.main.instructions` over destructuring, so the diff stays mechanical.

- [ ] **Step 4: Verify**

Run: `cd rust && cargo test -p rexx-parse && cargo clippy -p rexx-parse --all-targets -- -D warnings && cargo fmt --check`
Expected: the same test count as before the change, zero failures. A changed count means a test was lost, not that the change worked.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-parse
git commit -m "Give the main program body a CodeBody, so the executor can borrow one"
```

---

