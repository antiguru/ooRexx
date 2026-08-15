### Task 10: Promote the call forms

**Both routes, not one.** `eval_call`'s own doc says shared resolution "is what stops `CALL length 'abc'` and `say length('abc')` answering differently"; promoting one route and not the other splits that deliberately-shared path across two engines.

**Files:** `ir/compile.rs`, `ir/drive.rs`, `run.rs`; tests as before.

**Interfaces:**
- Consumes: `resolve_and_run_call`.
- Produces: the `resolve`/`invoke` split -- `Interp::resolve_call(&mut self, name: &[u8], search_labels: bool) -> Result<Resolved, Failure>` and the invocation half, both called by `eval.rs` and the tree-walker uncached.

**D24, amended after the spike (`0409634d`) and the amendment is the load-bearing part.** The seam already exists on the classic path: resolution was already a distinct phase producing `Resolved`, upstream of argument evaluation, and a call-site cache over it is expressible, correct and load-bearing there. But it is correct **because it needs no guard**, which is exactly the property a send cache lacks. So the classic case validates the seam and **not** the caching discipline. Do not carry a conclusion about send caching out of this task.

**Resolution goes against the running activation's body, not against `code.body`, and the two differ inside an `INTERPRET` fragment.** A fragment's `labels` is always empty -- a label in interpreted text is error 47.1 -- so searching `code.body` makes every `CALL` inside a fragment unresolvable. Measured on the oracle: `interpret "call sub"` runs the enclosing program's `sub:`. The spike's first version got this wrong and passed every test that had no `INTERPRET` in it.

- [ ] **Step 1: Add the dual cases**, including `interpret "call sub"`, a `CALL ON` handler, an external routine, a built-in reached both ways, and a `PROCEDURE EXPOSE` callee.
- [ ] **Step 2: Split `resolve` from `invoke`**, from `3573`'s shape on the spike branch. No behaviour change; run the full suite and expect no diff.
- [ ] **Step 3: Golden test for the compiled call, run, fail.**
- [ ] **Step 4: Emit the call ops with the resolution cache keyed on op position.**
- [ ] **Step 5: Suites, record the prediction, commit.** **Measure nothing (2026-08-11, Moritz)** -- Task 11 takes this task's figures per-commit through `rexx-arms`. Record what you expect and why, and say so if you have reason to think an axis moves the wrong way.

---

