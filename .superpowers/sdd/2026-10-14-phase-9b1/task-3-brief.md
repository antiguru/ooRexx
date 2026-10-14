### Task 3: L10 and L11

**Files:** `rust/crates/rexx-exec/src/environment.rs` (the per-row routine store from 9b Task 9),
`rust/crates/rexx-exec/src/dispatch/native.rs` (native frame), corpus witnesses.

- [ ] **Step 1, L10:** `~addRoutine` accepts a copy of an internal-row Routine object, as the oracle
      does. The copy must carry what makes it resolvable as that row, as a field of the copy and not
      a side map. Witness: probe `add2.rex` (`docs/superpowers/records/2026-10-13-phase-9b/task-9/`).
      Remove the exclusions row "addRoutine REFUSES A LIBRARY Routine OBJECT".
- [ ] **Step 2, L11:** A native routine call (SysStemSort with a user STRING method that reads
      `.context~stackFrames`) shows the native routine's frame in a live `.context~stackFrames`, with
      the oracle's fields. Witness: scout-leaks L11's probe. Measure extcall against the pin, because
      this is the native-call path.
- [ ] **Step 3:** Shrink `ALLOWED` for any site removed. Per-task check. Commit.

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
