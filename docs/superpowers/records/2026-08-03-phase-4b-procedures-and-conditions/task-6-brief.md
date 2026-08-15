### Task 6: `SIGNAL` to a label, and `SIGNAL VALUE`

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs` (the `Signal` arm, and `Flow` if a variant is needed)
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Why:** `SIGNAL` needs only the label table `CodeBody` already carries. It is placed here rather than first because it unblocks nothing.

`Signal` has three forms. `Label(Box<[u8]>)` and `Value(Expr)` are this task's; `Trap(ConditionTrap)` is **Task 7's** and keeps failing loudly, with its own witness row from Task 0. **The witness program for the in-scope tag must use `SIGNAL label`, not `SIGNAL ON`.**

**Do not assume `Flow`'s existing variants suffice.** `SIGNAL` out of a `DO`, a `SELECT` or an `INTERPRET` fragment must unwind the block stack, and `SIGNAL` from inside a routine has to be measured before its semantics are chosen. `pop_search_frame` and the `Flow` forwarding do a similar shape for `LEAVE`; reuse where it fits, and add a variant where it does not.

**Three things checked in the tree at `92e80218`, so you do not have to find them:**

* **`Flow::Goto(usize)` already exists**, alongside `Next`, `Exit`, `Return`, `Leave` and `Iterate`. A same-body `SIGNAL` is very likely a `Goto`; the cases that may not be are the ones the measurements below are for.
* **Label resolution must go through the running *activation's* body**, not the body being stepped -- `run.rs:2153-2154` reads `activation_body.labels`. That distinction is a Task 3 fix: inside an `INTERPRET` fragment the two differ and a fragment's label table is always empty, so every `CALL` inside a fragment was unresolvable until it was corrected. `SIGNAL` resolves labels the same way and will hit the same trap if it reaches for the stepped body.
* **`Signal`'s witness set is already arm-grained** from Task 0 -- `loud.rs:160` maps `"Signal"` to `["Signal", "Signal::Trap"]` with its own witness row at `:244`. `Signal::Trap` stays loud until Task 7. **Do not delete that row**, and make sure your in-scope witness program uses `SIGNAL label`, not `SIGNAL ON`, or the coverage witness passes for the wrong reason.

- [ ] **Step 1: Measure, and put the transcripts in the report**

At minimum: `SIGNAL` out of a nested block; `SIGNAL` to a label not in the current body; `SIGNAL` from inside a called routine to a label in the caller's body; `SIGNAL` out of an `INTERPRET` fragment; `SIGNAL VALUE` where the value is not a label.

- [ ] **Step 2: Write the failing tests from those transcripts, asserting exact stdout, stderr and rc**

- [ ] **Step 3: Implement `Signal::Label` and `Signal::Value`**

- [ ] **Step 4: Run the suite, update `tests/owners.rs`, commit**

---

