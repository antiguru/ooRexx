### Task 7: The prebuilt test extensions, and the group partition

**Files:** `docs/superpowers/plans/phase-8-gate.md`, a derived test beside `rexx-api/tests/load.rs`

- [ ] **Step 1:** Record, with the commands, that `api/` and `testbinaries/` are byte-identical
      between the oracle checkout and this tree, and that the oracle's `build/lib` holds the six
      library products and its `build/bin` the `rexxinstance` and `provoke_locks` executables. **This is what "compile unchanged against frozen headers" is
      witnessed by**, and the gate document says so, and says that a build succeeding says the
      headers are compatible, not that the entry points behind them work.
- [ ] **Step 2:** A test that reads each API group's `EXTERNAL "LIBRARY <name>"` and `rxfuncadd`
      directives, and each named library's `NEEDED` entries followed transitively and its
      undefined `Rexx*` symbols, and asserts the partition against a list the test itself holds:
      the groups whose libraries need no interpreter library are exactly `METHOD`, `CONVERSION`
      and `FUNCTION`, and every other group maps to the phase its imports belong to (embedding to
      Phase 9, the RXAPI registries to Phase 10). Derived, so a group that changes its library
      fails the test. Task 8 runs the groups that list names.
- [ ] **Step 3:** The re-homing is already recorded (`313807bc5`, `3bb1d34d2`); make the test's
      partition and those records agree, and where they do not, correct the records. Commit.

---

