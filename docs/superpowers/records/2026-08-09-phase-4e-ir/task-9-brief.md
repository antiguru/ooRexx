### Task 9: Promote arithmetic, with the patch table as its consumer

**Predicted movement: `compound` only.** 4d-1 measured the small-integer path for `/`, `%`, `//` and `**` at **-51.6% on `compound`** as reverted prototype P1 (`phase-4d-attribution.md:228`).

**`arith` is predicted not to move, and an earlier wording of this line predicting both axes was wrong.** P1 moved `arith` **-0.7%, inside noise**, and `phase-4d-attribution.md:232`-`:235` gives the reason and calls it a prediction rather than a miss: **`arith`'s divisions have non-integer operands, so a small-integer path cannot reach them.** Its 28.7% share "is therefore real and *not* addressable by this fix, which is a fact 4d-2 needs before it schedules one task for both axes" -- and this plan then scheduled one task for both axes anyway. An `arith` that does not move is this task succeeding. If it *does* move, that is a finding worth more than the win, and it needs its mechanism named.

**The -51.6% is a wall-clock figure** (`phase-4d-attribution.md:559`), and this phase measures instructions and cycles. Do not "beat or explain" it on an instrument that cannot reproduce it. Predict on C2's profile share instead -- `compound` 41.5% (`:145`) -- and record the measured instrument figures against that. P1 exceeded its own share because it removed the operand conversion, the result allocation and the root pushes surrounding the division as well, each attributed to its own frame in the profile, so a figure above the share is expected rather than suspicious.

**Files:** `ir/compile.rs`, `ir/drive.rs`, `ir/mod.rs`; tests as before, plus a precondition test.

**Interfaces:**
- Produces: the arithmetic ops and `Chunk`'s parallel `Vec<AtomicU32>` patch table.

**A hard width constraint, landed by Task 8 and stated here so it is a design input rather than a compile error.** `ir/mod.rs` now carries `const _: () = assert!(size_of::<Op>() == 12);`, verified load-bearing (asserting 13 fails the build). An arithmetic op with two source registers, an instruction index and an opcode fits. **Anything wanting a second `u32` beside an index does not, and the answer then is a side table keyed off the op the way `Chunk::consts` already is -- not relaxing the assert.** Widening `Op` costs every op in the stream, including the ones that never quicken.

**D22's rules, and none of them is optional:**

* **The op stream stays immutable**; patch state lives in the parallel atomic table, read **only** inside arms of ops that can specialise, so an op that never quickens pays nothing.
* **A patch slot is a hint that never removes a precondition check.** The specialised path re-validates and falls through.
* **`AtomicU32`, not `Cell<Op>`**, because ooRexx shares routine bodies across activities and `Cell` is not `Sync`. This survives Phase 6.

  **Checked against the tree 2026-08-10, and the reason does not bite yet -- which changes what this task must measure, not what it must build.** Chunks are held as `Rc<crate::ir::Chunk>` in a per-`Interp` map (`lib.rs:1353`), and `Rc` is neither `Send` nor `Sync`, so **nothing is shared across threads today** and `Cell` would compile. The rule stands as forward-compatibility rather than present necessity: converting a hot-path field after Phase 6 means touching it twice, and the atomic is the cheaper bet. **But the justification as written claims a present constraint that does not exist, so this task must price the choice rather than inherit it:** report what the `AtomicU32` load costs against a plain `u32` read on the quickened path, at the same two sizes as everything else. If it is free, say so and the rule needs no defence; if it is not, that number belongs to Phase 6's ledger as a cost it has already been charged. `AtomicU32` appears nowhere in the workspace today, so this task introduces the first one.
* **The schema does not generalise to sends.** A send's precondition is "the lookup would still return this method", and checking that *is* the lookup.

**The guard that is easy to get wrong, and which `ootest` caught before:** an integer fast path needs **both operands** checked against `DIGITS`, not just the result. Rexx rounds the operands before operating: at `DIGITS 3`, `1000 - 25` is `980`, not the exact `975`. The loop version is subtler still -- each step adds to the value the previous step stored *after rounding*, and the exact and rounded sums render identically for one line before parting company. Four hand-written probes missed it; `corpus/lang/loop_control_rounding.rex` is the witness that sees it.

- [ ] **Step 1: Write the precondition test first**

Drive the quickened op with operands outside the small-integer range and assert the general path's answer. This is exit criterion 5's falsification and it must exist before the specialisation does.

- [ ] **Step 2: Add the dual cases**, including `loop_control_rounding.rex` and a `NUMERIC DIGITS 3` subtraction.
- [ ] **Step 3: Golden test, run, fail.**
- [ ] **Step 4: Emit the arithmetic ops without specialisation. Full suites.**
- [ ] **Step 5: Add the patch table and the quickened arm.** Full suites again.
- [ ] **Step 6: Record the prediction you are not measuring, and commit.** Write down what you expect each axis to do and why, in the form Task 11 can check it against. A prediction recorded before the measurement is worth more than one reconstructed after it, and recording it is the whole of what this task owes on performance.

**Measure nothing (2026-08-11, Moritz).** Every performance measurement in this task moves to Task 11, which takes them per-commit through `rexx-arms`. Deferring costs nothing that cannot be recovered: the harness builds and measures any commit after the fact, which is exactly what Task 7-M2 did for Task 7-M's archaeology, and the commits are immutable. **What it does cost is the chance to notice a surprise while the code is still in your hands**, so if you have a reason to believe an axis will move the wrong way, say so in the report rather than leaving it for Task 11 to discover.

**Two things this task must still do, because they are falsifications rather than performance reports.** Both are named in Task 11's list so they cannot be lost: deleting the patch table to confirm the number moves (exit criterion 5 -- a win that survives deleting the table was static specialisation, and a dead `AtomicU32` beside a statically specialised op would otherwise pass), and pricing the atomic load against a plain `u32` read. **Task 11 runs both.** Leave the code in a shape that makes them one build each: the patch table's removal must be a single guard rather than a change threaded through the arms.

---

