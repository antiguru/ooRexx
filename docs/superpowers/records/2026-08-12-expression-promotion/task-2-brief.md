### Task 2: the prefix operators as a native op

**Files:**

* Modify: `rust/crates/rexx-exec/src/eval.rs`
* Modify: `rust/crates/rexx-exec/src/ir/mod.rs`
* Modify: `rust/crates/rexx-exec/src/ir/compile.rs`
* Modify: `rust/crates/rexx-exec/src/ir/drive.rs`
* Modify: `rust/crates/rexx-exec/src/ir/golden.rs`
* Modify: `rust/crates/rexx-exec/src/ir/golden_tests.rs`
* Modify: `rust/crates/rexx-exec/src/ir/corpus_shape_tests.rs`
* Modify: `rust/crates/rexx-exec/tests/ir_dual_cases/operators`

**Interfaces:**

* Consumes: Task 1's `Op::Binary`, and its widened `native_shape`.
* Produces: `Interp::apply_prefix(&mut self, op: PrefixOp, value: ObjRef) -> Result<ObjRef, Failure>`, `Op::Prefix { op: PrefixOp, src: u16, dst: u16 }`, `Op::TracePrefix { op: PrefixOp, src: u16 }`.

**The split.**
`Interp::eval_prefix` has the same seam, with one operand:

```rust
let frame = self.roots.push_frame();
let value = self.eval(code, operand)?;
self.roots.push_temp(value);
let result = /* the operator's own work */;
self.roots.pop_frame(frame);
Ok(result)
```

The operator's own work becomes `apply_prefix`, keeping the existing `match op` body verbatim and the measured facts in its doc comment.
`eval_prefix` keeps the prologue and calls it.
State the same rooting contract as Task 1: `apply_prefix` allocates, so its argument is rooted by the caller.

**Why the trace op is separate from `Op::TraceOperator`.**
A prefix operator does **not** trace `>O>`.
`trace_intermediate`'s `ExprKind::Prefix` arm calls `trace_prefix_op` with the spelling `+`, `-` or `\`, which is a different line from `echo_operator`'s.
So the echo needs its own op carrying a `PrefixOp`.

**The promotion.**
`native_shape` gains `ExprKind::Prefix { operand, .. } => native_shape(operand)`.
`push_native` gains a `Prefix` arm: emit the operand into `dst`, then `Op::Prefix { op, src: dst, dst }`, then `Op::TracePrefix { op, src: dst }`.
The operand lands in `dst` itself for the reason a binary operator's left operand does, and it is read before `dst` is written.

`compile.rs` has a family of adjacency assertions (`assert_literal_echoes_follow_their_load`, `assert_read_echoes_follow_their_load`, `assert_operator_echoes_follow_their_op`).
Add `assert_prefix_echoes_follow_their_op` in the same shape, checking position, register **and** operator -- an echo behind the wrong op lands in the right place with the wrong tag in it -- and call it beside the others.

**Steps:**

- [ ] **Step 1: write the failing golden test**

```rust
/// A prefix operator compiles to its own op and its own echo, and the echo
/// carries the operator: `\` and `-` trace different lines from one value.
#[test]
fn a_prefix_operator_compiles_to_a_native_op_and_its_own_echo() {
    /* assert_eq! against the full rendered stream of `za = -zb` */
}
```

- [ ] **Step 2: run it and watch it fail** -- `cargo test -p rexx-exec a_prefix_operator_compiles`, reading the run count.

- [ ] **Step 3: split `eval_prefix`, add `apply_prefix`, run the workspace green.**

- [ ] **Step 4: add both ops, drive them, extend `golden.rs`, `corpus_shape_tests.rs` and the adjacency assertion.**

`Root` gains a `Prefix` variant; `root_of` and `native` restate the widened set independently.

- [ ] **Step 5: add prefix stanzas to `tests/ir_dual_cases/operators`**, under Task 1's rules for that file -- oracle-captured bytes, a fresh empty directory, no `REWRITE` -- covering `+`, `-` and `\`, a prefix applied to an operator's result, a prefix applied to a prefix, `\` on a non-logical value, and `-` on a non-numeric value, with a `trace i` stanza so the `>P>` line's position is compared.

- [ ] **Step 6: gates, as Task 1 Step 7.**

- [ ] **Step 7: commit.**

---

