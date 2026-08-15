### Task 9: The instruction loop — assignment, SAY, DROP, NUMERIC, EXIT, LABEL, NOP

**Spec:** "Control flow", "Output and trace sinks".

**Files:**
- Create: `rust/crates/rexx-exec/src/run.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/run.rs`

**Interfaces:**
- Produces: `enum Flow { Next, Goto(usize), Exit(Option<ObjRef>) }` and `fn step(&mut self, body: &CodeBody, index: usize) -> Result<Flow, Raised>`.

- [ ] **Step 1: Write the failing tests** — assignment to a variable, a stem and a compound; `SAY` of each value kind and of an omitted expression (a blank line); `DROP` of a variable, a tail, a whole stem and the `(v)` indirect form, **using `RootSet::clear_slot`, which Task 2b added expressly for this**; `NUMERIC DIGITS`/`FUZZ`/`FORM` including the `VALUE` spellings; `EXIT` with and without an expression; a `LABEL` as a traced no-op; `NOP`.

- [ ] **Step 2: Run to watch them fail**

- [ ] **Step 3: Implement**

**Upcase an indirect name before resolving it.** `slot_of` deliberately does not upcase — that happens once upstream in `SymbolTable::intern`, before a `SymbolId` exists — but `DROP (v)` never goes through the scanner. Measured: `v = 'x'; x = 1; drop (v); say x` prints `X`, so the *value* is upcased before it names a variable. A resolution path that passes the raw bytes to `slot_of` misses an existing slot and silently allocates a second one for the same variable, which is the aliasing failure Task 2b's `growth_does_not_recycle_a_cleared_slot` exists to prevent, arriving by a different route.

**Do not write `ObjRef::NIL` to mean "dropped".** `x = .nil` is legal Rexx and `.nil` is a value, so the two states are observationally distinct: measured, `y = .nil; drop y; say y` prints `Y`, the derived name, while `x = .nil; say x` prints `The NIL object`. `clear_slot` exists because of exactly this.

`SAY` writes to the output sink, default stdout. Trace goes to the **trace sink, default stderr** — the two are separate descriptors, so their interleaving is not observable and two independently buffered sinks are safe.

- [ ] **Step 4: Verify** — `cargo test`, plus each instruction run under both interpreters through `rexx-run`.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/run.rs rust/crates/rexx-exec/tests/run_basic.rs
git commit -m "The instruction loop, and the seven instructions that do not branch"
```

---

