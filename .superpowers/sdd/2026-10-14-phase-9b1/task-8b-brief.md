### Task 8b: The three remaining leaks

**Files:** `rexx-core` (`RootSet::cells`, marking), `rexx-exec` (variable references, `define`,
the MethodId-indexed `special_methods` and `natives` vectors). Source: Task 8 review
(`.superpowers/sdd/2026-10-14-phase-9b1/task-8-review.md`, item 5); the oracle is flat on each.

- [ ] **Step 1, MethodId vectors:** minted identities (`Method~copy`, `Method~new` with
      `setPrivate`, native copies) keep their `special_methods`/`natives` rows in a map beside each
      vector, pruned by `forget_method_identity` (a numeric-id map in state 2, controller ruling
      2026-10-15). Loop witness: 20000 vs 80000 within 10%, oracle beside.
- [ ] **Step 2, retired behaviours:** a behaviour `define` retired on a live class is freed once no
      live instance holds it (a cold pass after the sweep). Loop witness: 5000/20000/80000 defines on
      one class, flat; collect_stress shapes with an instance holding a retired behaviour mid-call.
- [ ] **Step 3, variable-reference cells:** a cell that a `>name` reference created is a heap object
      traced from its holders (VarRef bodies, frame aliases, and every other holder found by grep),
      and is freed with them. Loop witness: `r = >y` at 20000/80000/320000, flat. collect_stress shapes:
      a reference held past its frame, passed as an argument, stored in a collection, held by another
      activity, and used after the variable was dropped. A missed holder is a silent wrong variable,
      so every shape is compared with the oracle.
- [ ] **Step 4:** Full bench set against the pin (`-B 0.5`). Per-task check. Commit.

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
