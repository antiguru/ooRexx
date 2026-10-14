### Task 2: Value defects L1, L2 and L3

**Files:** `rust/crates/rexx-core/src/heap.rs` (weak pass, near `:282`),
`rust/crates/rexx-exec/src/eval.rs` (near `:646`, `:843`), `rust/crates/rexx-exec/src/loops.rs`
(near `:1062`), `rust/crates/rexx-exec/src/dispatch/reqstr.rs` (near `:198`), corpus witnesses.

- [ ] **Step 1, L1:** A WeakReference whose referent is a non-heap (inline or tagged) handle is never
      cleared by the weak pass. Witness: scout-leaks L1's probe. The oracle prints `abc 12346 xxx...`.
      Also add Review Focus 5's shapes. Confirm that the "dropped value never clears" consequence is
      Deviation 36's, and say so in the witness's comment.
- [ ] **Step 2, L2:** User arithmetic and prefix operators on a String subclass instance are sent to
      the receiver before any numeric conversion, and so is the DO setup `+`, as the oracle does.
      Witness: scout-leaks L2's probe (the oracle prints `plus`). Cover binary operators, prefix
      `-`/`+`/`\`, and `DO i = s TO ...` with a subclass `s`. Measure fibcall, fibfunc, dispatch and
      sendloop against the pin, because this is the operator path.
- [ ] **Step 3, L3:** A class-side `STRING` method is sent for `say .k` and `length(.k)`.
      Witness: scout-leaks L3's probe. Close the queued
      `.superpowers/sdd/queued/2026-10-13-class-side-string-method-not-sent.md` in the report.
- [ ] **Step 4:** Per-task check, including `collect_stress`. Commit.

## Review Focus

1. **A package collected while still in use.** A package whose only reference is a running
   activation, a `.context~package` held in a variable, a class or routine defined in it, or another
   activity's frame. Expected: it is never collected while any of these reaches it. Task 5 pins each
   shape under `collect_stress`, and runs one from two activities under the scheduler's simulation
   mode.
2. **Re-requiring after collection.** `::requires 'x.cls'`, drop every reference, collect, then
   require it again. Expected: it loads again, and the oracle's prolog-run count and class identity
   answers hold (measure the oracle first). Task 5 pins it.
3. **A copied package mutated.** `p2 = p~copy` then `p2~addRoutine`, `p2~addClass` and
   `p2~setSecurityManager`. Expected: `p` is unchanged, `p2` changed, as on the oracle. Task 4 pins
   it, including a copy of the REXX package.
4. **A stale program id.** An activation, plan cache entry or Method object whose program was freed,
   for example a Method created in a loop and dropped, while a frame from it is still on a stack
   (`.context~stackFrames` held in a variable). Expected: never a wrong program, never a panic. Task 6
   pins it with a test that reuses freed slots under `collect_stress`.
5. **Value-based identity through collections.** A plain string stored in an IdentityTable, a Set and
   a WeakReference, then read back after collections. Expected: the held value's `identityHash` is
   stable and the WeakReference does not clear while held. Task 1 (witness) and Task 2 (L1) pin it.

---
