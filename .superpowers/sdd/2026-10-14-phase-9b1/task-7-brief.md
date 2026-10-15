### Task 7: Executable and message maps, and the collector's trigger

**Files:** `lib.rs` (the Method/Routine and Message groups in scout-leaks' side-map section),
`scheduler.rs` (`message_ids`), `rexx-core/src/heap.rs` (allocation charge near `:172`).

- [ ] **Step 1:** Each map in scout-leaks' Method/Routine group and Message group becomes a field of
      its object, or is keyed by ObjRef and pruned at sweep (spec section 5, states 1 and 2). Record
      which state each map takes, and why. Memory witness: scout-leaks L9's `msgloop` at 20000 and
      80000 iterations: the 80000-iteration peak is within 10% of the 20000-iteration peak, with the
      oracle's figures recorded beside them.
- [ ] **Step 2:** The allocation charge counts a `::CLASS` class, a package and a program, as it
      already counts a `~subclass` class. Witness: a loop that only defines classes through
      `Package~new` collects (a WeakReference to a dropped one clears within the oracle's bound).
      Close `.superpowers/sdd/queued/2026-10-12-gc-trigger-blind-to-class-weight.md` in the report.
- [ ] **Step 3:** Measure alloc, hashnew, startup and pingmsg against the pin. Per-task check, and
      shrink `ALLOWED`. Commit.

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
