### Task 3: a plain `WHEN`'s condition compiles

**Files:**
* Modify: `rust/crates/rexx-exec/src/ir/mod.rs` (`Op::Condition` gains its keyword tag)
* Modify: `rust/crates/rexx-exec/src/ir/compile.rs` (the listed-`WHEN` arm)
* Modify: `rust/crates/rexx-exec/src/ir/drive.rs`, `golden.rs`, `golden_tests.rs`
* Modify: `rust/crates/rexx-exec/src/run.rs` (`chunk_node_at`'s `When` slot arm)

**Interfaces:**
* Consumes: Task 1's `Op::Condition` and `Interp::condition_value`.
* Produces: `Op::Condition { index, reg, keyword: ConditionKeyword }`, where `ConditionKeyword` is `If` or `When` and selects the raiser (34.1 or 34.2).

- [ ] **Step 1: tag the op with its keyword**

The raiser is the only difference between an `IF`'s condition and a plain `WHEN`'s: `scan_when`'s `When` arm is `eval_condition` with `raised_when_not_logical`, and its trace is the same `ConditionTrace::Result(indent)`.
A tag rather than a second op, because a second op would be the same arm twice.
Check the width holds, unpiped.

- [ ] **Step 2: emit it for a listed `WHEN` whose condition is native**

`compile.rs`, the `InstructionKind::When { .. } | InstructionKind::WhenCase { .. } if when_info[index].is_some()` arm.
**Only `When` promotes.** A `WhenCase` compares its values against the `SELECT CASE`'s text through `test_case_when`, which is not a condition and stays on `Op::WhenTest`; so does a `When` whose condition declines, and so does an absorbed `WHEN`, which has no `when_info` entry and falls to `Op::Generic` as it does today.

- [ ] **Step 3: address a call inside a `WHEN`'s condition**

`run.rs`, `chunk_node_at`: the `(InstructionKind::When { condition, .. }, 0)` arm.
`eval_chunk_expr` has no `When` arm today, because a `WHEN` reached the driver through `Op::WhenTest` rather than `Op::EvalExpr`; the subset rule `chunk_node_at`'s doc states is about slots *that function evaluates*, so adding an arm here without one there needs that doc corrected rather than quietly broken.

- [ ] **Step 4: pin the streams, add the differential cases, run the gates, commit**

Cases: a matching and a non-matching `WHEN`, `when 'x' then` for 34.2, a `SELECT CASE` whose `WHEN` is a `WhenCase` (unchanged), an absorbed `WHEN` (unchanged), and `trace r`/`trace i` over each.

---

