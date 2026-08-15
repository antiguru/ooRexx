### Task 8: `PUSH`, `QUEUE`, and the in-process queue

**Files:**
- Create: `rust/crates/rexx-exec/src/queue.rs`
- Modify: `rust/crates/rexx-exec/src/lib.rs`, `src/run.rs`
- Modify: `rust/crates/rexx-exec/tests/owners.rs`
- Modify: `docs/superpowers/plans/phase-4-exclusions.txt`

**Why:** I15. The queue is 4b's; 4c's `QUEUED()` and `PARSE PULL` read it.

**This task ships with zero differential coverage, and must say so.** Every construct that can read the queue -- `PULL`, `PARSE PULL`, `QUEUED()` -- is 4c's. So a 4b program containing `PUSH`/`QUEUE` produces byte-identical output whether the queue stores anything or nothing, and `Push | Queue => Ok(Flow::Next)` would satisfy every instrument this plan schedules. Measured: a program whose whole body is `push "X"` / `queue "Y"` produces empty stdout, empty stderr, rc 0.

**So the coverage is a unit test, not a differential.** Test the queue type directly with a `#[cfg(test)] mod tests` beside `queue.rs`, asserting the interleaved order against values measured from the oracle in a **4c-shaped** probe (`push a; queue b; push c` then three pulls). The probe program is not a corpus program and does not go in the subset; it is how you learn the expected order.

**And the exclusions file must carry a KNOWN GAP row** stating that Task 8 shipped with no differential witness, and that its first one is 4c's first `PARSE PULL` corpus program. Without that row, a reader sees a green task and infers coverage that does not exist.

**The cross-process caveat, corrected.** The scoping document says the oracle's `rxapi`-backed queue makes cross-process differential runs impossible, and calls it live rather than theoretical. Measured 2026-08-03: a following separate `rexx` process reports `queued()` as `0`, so the queue is **not** shared across processes on this host. Record the single-program rule anyway -- it is right for a different reason, and depends on the host's `rxapi` state.

- [ ] **Step 1: Measure `PUSH`/`QUEUE` interleaving with a 4c-shaped probe, and record the expected order**

- [ ] **Step 2: Write the failing unit test against the queue type**

- [ ] **Step 3: Implement `queue.rs` and the two arms**

- [ ] **Step 4: Add the KNOWN GAP row and the single-program rule to the exclusions file**

- [ ] **Step 5: Run the suite, update `tests/owners.rs`, commit**

---

