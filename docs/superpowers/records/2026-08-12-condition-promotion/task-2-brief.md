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

