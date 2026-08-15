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
