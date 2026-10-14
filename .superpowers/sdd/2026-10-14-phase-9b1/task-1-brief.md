### Task 1: The value-based rule, its Deviations, and the refusal-text test

**Files:** `docs/superpowers/plans/phase-4-exclusions.txt` (Deviation 4 rewritten, Deviation 31
withdrawn into a new Deviation 36, a new Deviation 37 for L12), the `objectName=` refusal site
(find it with `git grep -n 'objectName' rust/crates/rexx-exec/src`), a new corpus witness
`rust/corpus/lang/value_based_identity.rex`, and a new test
`rust/crates/rexx-exec/tests/representation_refusals.rs`.

**Produces:** `representation_refusals.rs` with a `const ALLOWED: &[(&str, &str)]` of
(file:substring, owner) rows that later tasks shrink.

- [ ] **Step 1:** Write Deviation 36, "String and the numbers are value-based", in the format of
      Deviation 33. Cite spec section 1, and list what is unspecified (equal-value identity in both
      directions, `~copy` identity, a WeakReference to a dropped value) and what stays specified.
      Mark Deviation 31 withdrawn, pointing to 36, the way Deviation 8 points to 33. Rewrite
      Deviation 4 to "separately produced equal values may or may not be one object" plus the no-op
      set, scoped to value-based values. Write Deviation 37 for L12 (SysStemSort with a STRING method
      that mutates the stem mid-sort), citing scout-leaks L12 and the spec.
- [ ] **Step 2:** Reword the `objectName=` refusal on a value-based value so it names the rule ("a
      String is value-based") and cites Deviation 36. Re-derive the refusal tables.
- [ ] **Step 3:** Write `value_based_identity.rex`. Use the scout's probes C01-C16 and E01-E20
      (`scout-deviations.md` appendix) for the shapes that agree on both sides:
      - a held value's `identityHash` is stable across reads, an alias, `use arg`, a return, and
        storage in and retrieval from an Array, a Directory and an IdentityTable;
      - a subclass instance and an enhanced string accept `objectName=` and keep their identity
        through the same paths.
      Run it on the oracle first and read the stdout. It must print identical output on both sides.
- [ ] **Step 4:** Write `representation_refusals.rs`. It greps the `rust/crates/*/src` files for
      refusal text containing "did not build", "does not keep" or "is no longer live", and fails on
      any match not in `ALLOWED`. Fill `ALLOWED` from the grep. Tag each row with its owner: Deviation
      32 (permanent), Deviation 36 (permanent), or the Task of this plan that removes it (from
      scout-leaks' code sites). Commit the grep command in the test's doc comment. Show the test going
      red when a row is deleted.
- [ ] **Step 5:** Per-task check. Commit.

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
