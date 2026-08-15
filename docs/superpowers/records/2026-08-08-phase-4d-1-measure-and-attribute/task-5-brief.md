### Task 5: D1's Phase 4 re-measurement -- the GC arm, rebuilt

`d1-decision.md:19-22` records the Phase 1 heap result as a debt rather than a pass and says it "must be re-measured at Phase 4 when a real interpreter exists to measure on equal footing". This task is that re-measurement.

**Files:**
* Modify: `rust/crates/rexx-core/benches/heap.rs`, `docs/superpowers/plans/d1-decision.md`

- [ ] **Step 1: Establish that the recorded figure is not reproducible, and say so**

The C++ arm is `TIME('E')` around `GC('F')` on `heapshape.rex` (`d1-decision.md:61`). The Rust arm is `heap.rs`'s `collection`/`full_gc_1m_graph` criterion bench.

Commit `a3178cff` replaced `Body::String(String)` -- the variant D1's risk analysis names -- with `Body::Text`. **So re-running the bench today measures a different representation than the recorded number, and the comparison must be rebuilt rather than re-run.** Record this in `d1-decision.md` rather than quietly quoting a new number against an old one.

- [ ] **Step 2: Rebuild the two arms to be like-for-like**

The C++ arm is a whole-program GC pause on a graph `heapshape.rex` builds. The Rust arm must build a graph of the same shape and count, and time a full collection. Where the shapes cannot be made identical, say which and in which direction the difference cuts.

`heapshape.rex` builds its strings with `"e" || j` rather than a bare literal (`d1-decision.md:67`); preserve that, because it determines whether the strings are distinct heap objects.

- [ ] **Step 3: Measure, and record against parity**

Write the result into `d1-decision.md` as the Phase 4 re-measurement the document schedules, with its date and commit. If it misses parity, say so plainly -- `d1-decision.md:50-56` pre-registers the side byte-arena as the first thing to try, and that recommendation becomes live input to 4d-2 rather than a conclusion drawn here.

- [ ] **Step 4: Commit**

---

