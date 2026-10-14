### Task 5: Packages are collected

**Files:** `rexx-core` (trace and free for the Package body), `lib.rs` (package roots, the requires
cache), `install.rs`.

- [ ] **Step 1:** The collector traces a Package body's fields and frees an unreachable package. The
      roots that keep a package alive are its activations' frames, objects defined in it (classes,
      routines, methods), and anything holding its ObjRef. Remove the global root that holds every
      package today (`Interp::package_object` and the root registration near `lib.rs:3009`/`:3022`).
- [ ] **Step 2:** The requires cache holds packages weakly, as `PackageManager::addToRequiresCache`
      (`interpreter/package/PackageManager.cpp:921`) does. Witness Review Focus 2 against the oracle.
- [ ] **Step 3:** Review Focus 1's shapes under `collect_stress`, plus one under the scheduler's
      simulation mode (see `concurrency_tests`' S2 list).
- [ ] **Step 4:** `base/class/Package` `TESTRECLAIM01` passes within its own wait. Run the group
      through the harness and show the test passed, not timed out.
- [ ] **Step 5:** Memory: 80000 `Package~new('p', 'return 1')` in a loop peaks within 2x of the
      oracle's RSS. Record both with `/usr/bin/time -f %M` on the interpreter itself, not on a
      wrapper (`time -v timeout <bin>` measures timeout), and record the command.
- [ ] **Step 6:** Shrink `ALLOWED`. Per-task check, plus the Task 4 groups. Measure startup, alloc
      and hashnew against the pin. Commit.

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
