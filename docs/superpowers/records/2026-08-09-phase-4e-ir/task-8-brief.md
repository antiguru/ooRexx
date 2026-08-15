### Task 8: Promote variable access

**Predicted movement:** `varlookup` is the axis; 4d-1 measured assign-by-symbol-id at -10.7% there as a reverted prototype, which is the figure to predict against and to beat or explain.

**What that figure was, stated because the two are easy to conflate.** 4d-1's prototype changed how an assignment resolved its **target**, and `varlookup`'s loop body is `x = x + 1` then `y = x`. This task changes the **read** side, and only where the whole expression is a symbol -- so `y = x` is promoted and `x + 1` stays on `Op::EvalExpr` until Task 9 takes the arithmetic. One of the body's two assignments therefore benefits. Predict against that, not against the whole of -10.7%, and if the change here turns out to be the same change 4d-1 measured, say so and use the figure as it stands.

**Files:** `ir/compile.rs`, `ir/drive.rs`, `ir/mod.rs`; tests as before.

**The trace lines a native read must not drop, which is this task's version of the defect that shipped from the mechanics spike.** `eval.rs` emits a variable's trace line as a *side effect of evaluating it*, so an op that only loads a value silently drops that line and every line after it still matches -- which is exactly how the dropped `>L>` survived. `trace_intermediate`'s own arms say what is owed:

* `ExprKind::Variable(id)` and `ExprKind::Stem(id)`: one `>V>` line, tag the symbol's name, text the value's `to_text`.
* `ExprKind::Compound(id)`: `>C>` **then** `>V>` -- the fully-resolved name first, whether or not the tail resolves, then the value. The resolved name is `compound_parts`'s stem name concatenated with `tail_key`'s output, and it is derived at the read site.

Follow `Op::Const`/`Op::TraceLiteral`'s split exactly: the load op emits nothing, the echo is its own op immediately behind it reading the same register, and the echo op is emitted **unconditionally** rather than under `ChunkTrace`'s decision, because the gate it answers to is `trace_mode().intermediates` and `ChunkTrace` does not carry it. `compile::assert_literal_echoes_follow_their_load` is the precedent for asserting position **and** register; a compound's two lines are ordered against each other as well.

**The dual-engine sweep is what sees a dropped line**, and that is measured rather than assumed: `ir_dual.rs`'s `compare` diffs raw stderr between the arms. Making the echo a no-op must redden it. Run that check -- it is the falsification for this whole section, and it costs one run.

- [ ] **Step 1: Add the dual cases** -- a simple symbol, an uninitialised symbol (which answers its own name), a symbol after `DROP`, a compound, and one introduced by `INTERPRET` so the slot did not exist at compile time. Each under `trace i` as well as untraced, since `trace i` is the only setting that reaches the lines above.
- [ ] **Step 2: Golden test, run, fail.**
- [ ] **Step 3: Emit the load op**, resolving through the plan's slot map at compile time where it is available and falling back to the run-time path where it is not. The fallback is not optional: `INTERPRET` introducing a name, `DROP (v)`, and a first `CALL` in a program that never writes `RESULT` all reach `grow_slots` at run time.
- [ ] **Step 4: Emit the echo op** behind it, with the assertion that pins position and register.
- [ ] **Step 5: Falsify the echo** -- make it a no-op, confirm the dual sweep reddens and names a program, restore.
- [ ] **Step 6: Suites, measure, record, commit.** Instructions **and** cycles, interleaved between the two arms of one build. Wall clock is not an instrument in this phase.

---

