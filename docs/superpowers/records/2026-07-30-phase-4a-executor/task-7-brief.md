### Task 7: Expression evaluation, part one — terms, arithmetic, concatenation

**Spec:** "Expression evaluation", the arithmetic and concatenation groups.

**Files:**
- Create: `rust/crates/rexx-exec/src/eval.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/eval.rs`

**Interfaces:**
- Produces: `fn eval(&mut self, body: &CodeBody, expr: &Expr) -> Result<ObjRef, Raised>`.

- [ ] **Step 1: Write the failing tests** — `Literal`, `Constant` (`say 1e5` is `1E5`), `Variable`, `Stem`, `Compound`, `DotVariable` for the three admissible names, `Prefix` (`+ - \`), arithmetic `+ - * / % // **` through `rexx-num`, and `Abuttal` / `Blank` / `||`.

- [ ] **Step 2: Run to watch them fail**

- [ ] **Step 3: Implement**

Push every intermediate to `RootSet::push_temp` before any allocation that could collect while it is live. A value held only in a Rust local across an allocation is the defect class the root set exists to remove.

Every unimplemented `ExprKind` — `Call`, `QualifiedCall`, `Message`, `ClassResolver`, `List`, `VariableReference`, and any `DotVariable` beyond the three — takes the loud-failure path with the owning sub-phase named.

- [ ] **Step 4: Verify** — plus a `--release` run, because the temps discipline is what `debug_assert`s cannot check.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/eval.rs rust/crates/rexx-exec/tests/eval_arith.rs
git commit -m "Evaluate terms, arithmetic and concatenation"
```

---

