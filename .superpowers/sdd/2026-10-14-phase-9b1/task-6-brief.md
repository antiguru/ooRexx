### Task 6: Program lifetime

**Files:** `lib.rs` (`programs`, the `plans` and `chunks` caches, every ProgramId-keyed table),
`plan.rs`, `install.rs` (`:1729`, `:1864`, `:994`), the collector's sweep hook.

- [ ] **Step 1:** Replace `Interp::programs`' append-only `Vec<Rc<Program>>` with a table whose slots
      are freed. A ProgramId is never reused (a generation counter, or ids that only grow), so a stale
      id misses instead of naming another program. Add a test that frees and allocates programs under
      `collect_stress` and checks that no stale id resolves.
- [ ] **Step 2:** At sweep, a program that no live Package, Method or Routine object holds is freed,
      and every row keyed by its ProgramId (`plans`, `chunks`, `compiled_method_names`,
      `option_writes`, `library_programs`, `untranslated`, and the rest found by
      `git grep -n 'ProgramId' rust/crates/rexx-exec/src/lib.rs`) is pruned. List each table and its
      pruning site in the report.
- [ ] **Step 3:** Review Focus 4: a frame or a held stack frame whose program would otherwise be
      freed keeps it. Pin it under `collect_stress`.
- [ ] **Step 4:** Memory: 80000 `Method~new('m', 'return 1')` in a loop peaks within 2x of the
      oracle's RSS (scout-leaks L8 has the baseline: oracle 20084 kB). Do the same for
      `Routine~new`, source-string `setMethod`, and 80000 `Package~new('p', 'return 1')` (moved from
      Task 5: its remaining 27.3 kB per package is program rows). Record the command.
- [ ] **Step 5:** Measure the full bench set against the pin, because program lookup is on every call
      (`call.rs:966`, `dispatch.rs:3209`). Per-task check. Commit.

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
