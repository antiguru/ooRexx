# Condition promotion implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compile the expressions that decide control flow -- an `IF`'s condition, a `DO`/`LOOP` header's values, a plain `WHEN`'s condition -- to native ops instead of `Op::EvalExpr`, and give `RETURN`/`EXIT`/`PUSH`/`QUEUE` an op of their own.

**Architecture:** Tasks 1-4 of the expression-promotion plan widened the *value* slots (an assignment's, a `SAY`'s) and left every *condition* slot on `Op::EvalExpr`, which runs the tree-walker's `eval` for the whole expression.
Each task here reuses the seam those tasks established: `native_shape` decides, `push_native` emits, and the tail that a slot's value still owes -- validation, the `>>>` line, the `Flow` -- becomes one op that both engines enter through one shared `Interp` half.

**Tech Stack:** Rust 2024, `rexx-exec` (`ir/compile.rs`, `ir/drive.rs`, `ir/mod.rs`, `run.rs`, `eval.rs`), `datadriven` case files, the `rexx-bench-suite` harness.

## Why these four, in this order

Measured 2026-08-12 on `samples/rexxcps.rex`, with a temporary per-clause counter in the driver's `Op::Clause`, `Op::Generic` and `Op::EvalExpr` arms, one run at `count=2`/`averaging=2` (the program self-calibrates, so the counts are proportions rather than a fixed workload).
The instrumented build was made in a scratch `CARGO_TARGET_DIR` and the tree restored afterwards; `git status` was clean before this plan was written.

| clause kind | executions | how it runs today |
|---|---:|---|
| `IF` | 375,427 | promoted, and **every one** runs an `Op::EvalExpr` for its condition |
| assignment | 308,403 | promoted, no `eval` |
| `PARSE` | 187,714 | `Op::Generic` |
| `WHEN` | 165,924 | promoted, and its condition runs `eval` inside `scan_when` |
| `THEN` | 117,322 | `Op::Generic` |
| `DO` | 95,542 | promoted, and its header runs 97,222 `Op::EvalExpr` |
| `ITERATE` | 46,928 | `Op::Generic` |
| `SELECT` | 46,928 | promoted |
| `TRACE` | 23,472 | `Op::Generic` |
| `CALL` | 23,470 | promoted |
| `ADDRESS`, `RETURN`, label | 23,464 each | `Op::Generic` |

`IF`, `WHEN` and the `DO` header are the three largest remaining `eval` entries, and they are one seam.
`PARSE` and `THEN` are larger than `RETURN` but are not expression work: the survey's rule (`2026-08-12-remaining-promotion-survey.md`) is that promoting an instruction whose body is one `exec_*` call saves one dispatch, and neither has an expression a native op could replace.
`RETURN` is here because it was asked for and is the `SAY` template with a different tail; its measured share is the last row of that table.

**The comma list is not in this plan, and the reason first given for that was wrong.**
The reason that holds is that nothing measures it: it occurs in no benchmark program and nowhere in `rexxcps`, and a depth-tracking scan of the corpus found it in one program.
So there is no axis to state a prediction against.

The reason first given -- that its compiled form needs a forward jump inside a clause region, which a region cannot take -- is false, and this plan's own commit message carries it.
A false element **is** the branch: the false target is one value for the whole list (`if_targets(..).false_target`, already resolved through `PatchKind::Enter`), so each element's false exit ends the region exactly as `Op::JumpUnless` does, and no jump inside a region is needed.
Measured on the oracle 2026-08-12, `trace r` over `if 1, 1 then nop` prints `>>>` for each element and once more for the list's own result, and over `if 0, 1 then nop` prints the false element's line and the list result's; an op that ends the region can emit both before it branches.

Two further corrections to what that paragraph rested on.
`&` and `|` do **not** short-circuit -- measured, `if 0 & (1/0) then nop` raises 42.3 on the oracle, and both already compile to `Op::Binary` -- so the comma list is the only short-circuiting construct in the language and the only thing that would ever have wanted such a jump.
`Op::Jump`'s in-region arm is unreachable today, because `compile` emits `Op::Jump` only outside a region.

## Global constraints

These bind every task in this plan, in full.

* **The two engines must agree byte for byte**, on stdout, stderr and exit status, for every program in the corpus, with `MAX_EVAL_DEPTH` the one accepted divergence (`docs/superpowers/plans/phase-4f-record.md`). A promoted expression enters `eval` fewer times than the tree-walker does, which moves only that limit.
* **`const _: () = assert!(size_of::<Op>() == 16)` in `ir/mod.rs` stays.** A variant that trips it is a design error in this plan -- report it rather than widening the assertion. To check a width, build **unpiped** and read the whole diagnostic list: a build read through `head` once reported an absence that was not there, and the wrong explanation for it reached a plan file and a commit message.
* **The oracle at `/home/moritz/dev/repos/ooRexx/` is read-only.** Every invocation is wrapped `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )`, run from a fresh empty directory with absolute paths, because the scratchpad is on the oracle's search path. Three programs crash it and must never be run: `select; when 1 = 0 then; when 2 = 2 then nop; end`; `say date('M','0','D')`; any `NUMERIC DIGITS` above 1000.
* **A comment may not name the size of a set** (`rust/CLAUDE.md`). Name the set; a measurement keeps its numbers.
* **No em-dashes in comments or markdown**; use `--`. Markdown is one sentence per line, `*` bullets, first-word-only capitalisation.
* **Never `git checkout --`**, never `git add -A` for a restore, never `git reset --hard`, never force-push. Back up with `cp`, restore from the backup, verify with `sha256sum -c`, and rebuild afterwards.
* **Run cargo from `rust/`, never the repository root.** `cargo fmt --all --check` (not `--edition`), `cargo clippy --workspace --all-targets -- -D warnings`, `memcap 8G cargo test --workspace --no-fail-fast`. `cargo test <name>` exits 0 when it matches nothing, so read the run count. Never read a cargo exit code from a pipeline.
* **A new `ir_dual` case goes in a `datadriven` file** under `rust/crates/rexx-exec/tests/ir_dual_cases/`, with the expected bytes captured from the oracle, not written by hand.
* **Scratch files never in the repository.** Use the session scratchpad.

---

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

* `if 1, 'x' then nop` -- 34.6, the comma list's own raiser, and the only comma-list condition run on both engines anywhere in the tree
* `trace i` over an `IF` whose condition is an operator over a call, for the intermediate lines and their order
* `trace r` over a declining condition, for the lines the one `Op::EvalExpr` still owes when no `Op::Condition` follows it: `if .nil then nop`, which traces and then refuses, and `if 1, 1 then nop`, which traces and does not

**This list is shorter than the one Task 1 was dispatched with, and the four rows that went were measured out rather than argued out.**
The dispatched list also asked for a plain true and false condition, `if 'x' then nop` for 34.1, an `IF` under `trace r`, and an `IF` inside a routine called from a loop for the live indent.
Each was written, captured from the oracle, and then held out of the directory while the mutation it was supposed to catch was re-run over the whole workspace: the 34.1 row's mutation still reddened on `BRANCH_CASES`' own `if 'x' then say 'y'`, the live-indent row's still reddened on a `trace-settings` stanza, and the plain and `trace r` rows are the same shapes as rows those two tables already hold.
`rust/CLAUDE.md`'s rule is that a row which can fail is not a row that adds coverage, so they were deleted rather than kept.
The `trace r` rows above replaced them, for a gap the dispatched list did not cover at all: a **declining** condition under trace, which is where a second validating op behind the fallback would print a line the oracle does not.

- [ ] **Step 11: gates**

Run, from `rust/`: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `memcap 8G cargo test --workspace --no-fail-fast`.
Expected: exit 0, no failures, and a run count larger than before by the cases added.

- [ ] **Step 12: commit**

---

### Task 2: a `DO`/`LOOP` header's values compile

**Files:**
* Modify: `rust/crates/rexx-exec/src/ir/compile.rs` (the loop-header emission)
* Modify: `rust/crates/rexx-exec/src/run.rs` (`chunk_node_at`'s header slot arm)
* Modify: `rust/crates/rexx-exec/src/ir/golden_tests.rs`
* Modify: `rust/crates/rexx-exec/tests/ir_dual_cases/` (a loop-header stanza set)

**Interfaces:**
* Consumes: Task 1's unchanged `push_native` seam, `loop_header_slot(body, slot)`.
* Produces: nothing new. `Op::LoopHeaderValue` already carries the validation and the `>K>` line, and is what a header slot's value still owes.

- [ ] **Step 1: emit natively where the slot's expression allows**

`compile.rs`, where the header emits `Op::EvalExpr { index, slot, dst }` per slot.
Same choice as Task 1 Step 5, with `slot` rather than `0`, and with `Op::LoopHeaderValue` following exactly as it does today.
The fallback stays one `Op::EvalExpr` for that slot; a header with one declining expression keeps native ops for its other slots, because each slot is a separate decision.

- [ ] **Step 2: address a call inside a header**

`run.rs`, `chunk_node_at`: the `(InstructionKind::Do(body) | InstructionKind::Loop(body), slot)` arm, resolving through `loop_header_slot` exactly as `eval_chunk_expr` does.

- [ ] **Step 3: pin the streams**

`golden_tests.rs`: `do i = 1 to n` renders its bound's ops; `do i = 1 to length(s)` renders a `CallExpr` addressed at the header slot.

- [ ] **Step 4: the differential cases**

Oracle-captured, under `trace r` and `trace i`: a counted loop, a `to`/`by`/`for` combination, a `DO FOREVER`, and a header whose bound is a call.
The `>K>` order per header value is the property most at risk and must have a case.

- [ ] **Step 5: gates, then commit** (same commands as Task 1 Step 11)

---

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

### Task 4: `RETURN`, `EXIT`, `PUSH` and `QUEUE` get an op

**Files:**
* Modify: `rust/crates/rexx-exec/src/ir/mod.rs`, `compile.rs`, `drive.rs`, `golden.rs`, `golden_tests.rs`
* Modify: `rust/crates/rexx-exec/src/run.rs` (the shared halves)
* Modify: `rust/crates/rexx-exec/tests/ir_dual_cases/`

**Interfaces:**
* Consumes: `Op::Say`'s compile arm and driver arm as the template, including its `src: Option<u16>` for the bare form.
* Produces: `Op::Return { index, src: Option<u16> }` and its three siblings, or one op with a tag if the four tails differ only by which `Flow` they answer -- the implementer decides from the code and says which in the report.

- [ ] **Step 1: read the four `step` arms and choose**

`RETURN`'s arm is `eval`, root, `result_text` then `trace_result`, then `Flow::Return(value)`.
The four differ in their tail; if that is all, one op with a tag is the honest shape, and the doc says so.
`RETURN` with no expression leaves `RESULT` unset and `RETURN ''` sets it, which is why `src` is an `Option` rather than a sentinel register.

- [ ] **Step 2: split the tail into a shared half**, exactly as Task 1 Step 1 split `eval_condition`: the trace and the `Flow` in one `Interp` function both engines enter.

- [ ] **Step 3: emit, drive, render, pin, and add the differential cases**

The driver arm ends the region with `RegionEnd::Flowed(flow)`, which is what `Op::Call`'s arm already does.
Cases: a bare `RETURN`, `RETURN` with an expression, `RETURN` from a routine whose caller reads `RESULT`, `EXIT` with and without a value, `PUSH`/`QUEUE` and a `PARSE PULL` that reads them back, each under `trace r`.

- [ ] **Step 4: gates, then commit**

---

## Measurement, after all four land

The accept rule (plan section 164) is unchanged and applies once, to the whole plan: wall clock, `ulimit -v 8388608`, `REXX_ENGINE=ir`, a fresh empty working directory per run, arm order rotated, and the host-idle gate of six consecutive five-second `/proc/stat` samples at or above 90% idle.

**The axis that matters here is `rexxcps`**, which `rexx-bench-suite` already measures interleaved against the oracle and reports as clauses per second sampled per run.
None of the existing loop axes contains an `IF`, a `WHEN` or a call in a loop header, so none of them can show this work; they are the control, and a change in them is layout rather than the mechanism.
Record the result in `docs/superpowers/plans/phase-4f-record.md` as the next entry, including the axes that did not move.

## Deliberately out of scope

* The comma list, for the reason at the top.
* `WHILE`/`UNTIL` conditions, which reach `eval_condition` with `ConditionTrace::Keyword` and are a fifth task of the same shape once Task 3's tag exists.
* `PARSE` and `THEN`, which execute often and hold no expression a native op replaces.
* A call's arguments, which stay on the node; the survey has the design.
