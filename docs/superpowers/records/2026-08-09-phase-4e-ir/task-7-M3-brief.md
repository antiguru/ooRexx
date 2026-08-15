### Task 7-M3: Land the enter/leave split -- BLOCKS TASKS 9 AND 10

**Decided 2026-08-11 by Moritz, on Task 7-M2's measurement.** The spike takes `varlookup` to **0.98143 on instructions and 0.96988 on cycles** and `alloc4c` to **0.99581**, removing 152 instructions per `varlookup` pass -- about twice what the whole of Task 7-M removed. It is the first time in this phase that an axis with a promoted clause has come in under 1.0, and it does so on both instruments.

**This is not chasing a gate.** Criterion 4's floor is withdrawn and stays withdrawn; nothing here re-aims at it. It is taking a measured win on the shape the phase ships, and the reason it lands *now* rather than later is that no patch of the spike survives -- the rebuild costs the same whenever it happens, so doing it first means Tasks 9 and 10 build on the final shape and record their figures against it, instead of having those figures invalidated by a restructure underneath them.

**The shape, from 7-M2's report, which is your specification for it.** `in_clause` and `in_stepped_clause_with` are both *defined in terms of* an enter/leave pair rather than being replaced by one: `enter_clause`/`leave_clause` in `clause.rs`, `enter_stepped_clause`/`leave_stepped_clause` in `run.rs`, **each with exactly one body**. That is what makes the sharing rule hold by construction rather than by inspection -- one implementation with two entry shapes, not two implementations that agree. The driver then calls the pair directly and runs a region's ops in its own loop, with no callee between them.

**The failure path is the part that looked hard and is not.** 7-M2 measured it: the clause's `Result` travels into the leave as a value, which is exactly what `in_clause`'s epilogue already did with it. If your implementation finds this hard, you have diverged from the shape above -- stop and say so rather than inventing a second mechanism for it.

**The trade, recorded so it is not discovered as a regression.** A body whose clauses are all `Op::Generic` pays for this: `emptyloop` goes **1.06596 to 1.08185**, 24 instructions per pass. That is accepted. It must appear in your report as a result, not be omitted because the headline is good -- reporting only the flattering instrument is the failure this sub-phase exists to correct.

**This restructures the clause boundary, which is where every Critical defect in this phase has been.** Four of them, and each was found by asking where a construct's boundaries fall rather than what it does. The boundary carries: the `*-*` clause echo, `SIGL`, the `DATE`/`TIME` clock invalidation, the `>I>` trace-entry decay, `current_value_indent`, the GC temps frame and its watermark tripwire, failure-site resolution, and `CALL ON` trap delivery. **A trap queued by a delivered handler is the case that has caught implementations twice**, because `in_clause` delivers at most one trap and does not re-check, so the last member boundary is not the last boundary with work.

- [ ] **Step 1: Rebuild the split.** No behaviour change is intended anywhere. Run the full suite before touching a benchmark.
- [ ] **Step 2: Run the whole workspace suite in all four combinations** -- dev, dev+STRICT, release, release+STRICT -- plus the dual-engine sweep and the corpus. The dual sweep is the instrument that sees a dropped trace line, and a clause-boundary change is exactly what would drop one.
- [ ] **Step 3: Exercise the boundary cases directly**, not only through the suite: a `CALL ON` handler delivered at a promoted clause's boundary; a handler whose own `RAISE` leaves a second trap queued behind it; `LEAVE` naming a `SELECT` from inside `OTHERWISE`; and a `SIGNAL ON NOVALUE` whose `SIGL` must be the failing clause's own line. Each on both engines, compared as bytes.
- [ ] **Step 4: Measure through `rexx-arms`**, both instruments, both arms, two sizes, one sitting. **Reproducing 7-M2's figures is a check on both the harness and the rebuild:** expect `varlookup` about 0.98143/0.96988, `alloc4c` about 0.99581, `emptyloop` about 1.08185. A rebuild that lands a materially different number has diverged from the spiked shape and the difference needs naming before it is accepted.
- [ ] **Step 5: Write the baseline file, record predicted against measured, commit.**

**Do not box `Raised` here.** 7-M2 measured it: absolute speed improves on both arms and every ratio gets worse, `varlookup`'s cycles going 1.00890 to 1.16911. It is a separate decision on different evidence and it is not part of this task.

---

