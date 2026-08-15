### Task 1: an `IF`'s condition compiles

**Files:**
* Modify: `rust/crates/rexx-exec/src/ir/mod.rs` (the new `Op::Condition`, its doc, the render arm's sibling in `golden.rs`)
* Modify: `rust/crates/rexx-exec/src/ir/compile.rs` (the `InstructionKind::If` arm, `Root::of`)
* Modify: `rust/crates/rexx-exec/src/ir/drive.rs` (the region arm, and the outer loop's not-driven arm)
* Modify: `rust/crates/rexx-exec/src/ir/golden.rs` (the render arm)
* Modify: `rust/crates/rexx-exec/src/run.rs` (`eval_condition`'s split, `chunk_node_at`'s new slot arm)
* Modify: `rust/crates/rexx-exec/src/ir/golden_tests.rs` (the pinned `IF` streams)
* Create: `rust/crates/rexx-exec/tests/ir_dual_cases/conditions`

**Interfaces:**
* Consumes: `native_shape(expr, Some(NodePath::ROOT))`, `push_native`, `push_echo`, `close_region`, `Registers::{mark, alloc, release}`, `Interp::chunk_node_at`.
* Produces: `Op::Condition { index: u32, reg: u16 }`; `Interp::condition_value(&mut self, value: ObjRef, trace: ConditionTrace<'_>, checked: bool, raise: fn(&[u8]) -> Raised) -> Result<bool, Failure>`.

- [ ] **Step 1: split `eval_condition` at the evaluation**

`run.rs`.
Everything past `let value = self.eval(code, condition)?;` moves into a function both engines enter.
`checked` is the comma list's own answer: `eval_logical_list` has already validated every element, so its result is read back rather than re-checked, and re-checking it would report 34.6 as 34.1.

```rust
fn eval_condition(
    &mut self,
    code: &Code<'_>,
    condition: &Expr,
    trace: ConditionTrace<'_>,
    raise: fn(&[u8]) -> Raised,
) -> Result<bool, Failure> {
    let value = self.eval(code, condition)?;
    let checked = matches!(condition.kind, ExprKind::Logical(_));
    self.condition_value(value, trace, checked, raise)
}

pub(crate) fn condition_value(
    &mut self,
    value: ObjRef,
    trace: ConditionTrace<'_>,
    checked: bool,
    raise: fn(&[u8]) -> Raised,
) -> Result<bool, Failure> {
    // body of the old eval_condition from `let frame = self.roots.push_frame();`
}
```

The frame push and pop stay inside `condition_value` unchanged, for both callers.
A register is already a root, so the compiled caller's frame is redundant and harmless; one implementation is the point.

- [ ] **Step 2: run the split under the existing suite**

Run: `cd rust && memcap 8G cargo test --workspace --no-fail-fast`
Expected: the same pass count as before the edit, no failures. This step changes no behaviour.

- [ ] **Step 3: commit the split**

- [ ] **Step 4: add `Op::Condition`**

`ir/mod.rs`, beside `Op::JumpUnless`.

```rust
/// Validates the condition value in `reg` and leaves the logical value
/// `Op::JumpUnless` reads back in the same register.
Condition { index: u32, reg: u16 },
```

The doc comment states: what it replaces (the tail of the `Op::EvalExpr` an `IF`'s condition used to be, whose own `eval_chunk_expr` arm stays for the shapes that decline), that it reads and writes one register because the value it validates is the value it replaces, and that a native condition is never a comma list -- `native_shape` has no `ExprKind::Logical` arm -- which is what licenses passing `checked: false`.
Confirm `size_of::<Op>() == 16` still holds, by building unpiped.

- [ ] **Step 5: emit it, natively or not at all**

`compile.rs`, the `InstructionKind::If` arm. `push_value` is not used here, because the fallback is not the same op: a condition that declines stays one `Op::EvalExpr` doing the whole job, validation included, and adding a `Condition` after it would trace and validate twice.

```rust
if native_shape(condition, Some(NodePath::ROOT)) {
    push_native(
        &mut ops, consts, registers, hints, calls, plan, condition,
        instruction_index(index)?, 0, Some(NodePath::ROOT), dst,
    )?;
    ops.push(Op::Condition { index: instruction_index(index)?, reg: dst });
} else {
    ops.push(Op::EvalExpr { index: instruction_index(index)?, slot: 0, dst });
}
```

- [ ] **Step 6: drive it**

`drive.rs`, in the region loop beside `Op::JumpUnless`, and a not-driven arm in the outer loop's list.

```rust
Op::Condition { index, reg } => {
    debug_assert!(
        chunk.holds_register(*reg),
        "op reads register {reg} outside the region the chunk reserved"
    );
    debug_assert_names_the_clause(code, *index, clause, "Condition");
    let value = self.roots.temp_at(registers, *reg as usize);
    let indent = self.clause_state.current_value_indent;
    let holds = match self.condition_value(
        value,
        ConditionTrace::Result(indent),
        false,
        raised_if_not_logical,
    ) {
        Ok(holds) => holds,
        Err(failure) => break 'region Err(failure),
    };
    // In range unconditionally: `SMALL_INT_MAX` is far above one.
    let logical = ObjRef::small_int(i64::from(holds)).unwrap_or(ObjRef::NIL);
    self.roots.set_temp(registers, *reg as usize, logical);
}
```

`indent` is read live rather than compiled in, for the reason `eval_if_condition` reads it live: a nested activation moves it.

- [ ] **Step 7: address a call inside a condition**

`run.rs`, `chunk_node_at`: add the `(InstructionKind::If { condition, .. }, 0)` arm, so `Op::CallExpr`'s descent resolves for `if length(s) > 3 then`.
Without it every condition holding a call reaches `Loud::call_op_off_its_node`.
`chunk_node_at`'s doc says its slot arms are a subset of `eval_chunk_expr`'s; that stays true, since `eval_chunk_expr` keeps its own `If` arm for the declining shapes.

- [ ] **Step 8: render it**

`golden.rs`: `Condition index=N reg=R`.
`Root::of` in `compile.rs` and any exhaustive match over `Op` gain their arm; there is no catch-all in either.

- [ ] **Step 9: update the pinned streams**

`golden_tests.rs`: the `IF` expectations now carry the condition's own ops and their echoes, and every jump target past the insertion point moves.
**The doc comments above them are part of this step**: a comment that explains a target by counting ops, or that says an `IF`'s region is two ops, is falsified by this change and must be corrected rather than left.
Add one expectation for a declining condition (`if .nil then nop`) that still renders `EvalExpr` and no `Condition`, which is what pins the fallback.

- [ ] **Step 10: the differential cases**

`tests/ir_dual_cases/conditions`, every expected block captured from the oracle by running the stanza's program under the wrapper above, from a fresh empty directory:

* a true and a false condition, plain
* `if 'x' then nop` -- 34.1, and the message must still name `IF`
* `if 1, 'x' then nop` -- 34.6, the comma list's own raiser, which pins that the fallback did not change
* `trace r` over an `IF`, for the `>>>` line and its indent
* `trace i` over an `IF` whose condition is an operator over a call, for the intermediate lines and their order
* an `IF` inside a routine called from a loop, for the live indent

- [ ] **Step 11: gates**

Run, from `rust/`: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `memcap 8G cargo test --workspace --no-fail-fast`.
Expected: exit 0, no failures, and a run count larger than before by the cases added.

- [ ] **Step 12: commit**

---

