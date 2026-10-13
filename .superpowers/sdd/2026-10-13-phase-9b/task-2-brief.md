### Task 2: The 9b gate-group file

**Files:** `rust/corpus/suite-9b-groups.txt` (new), `rust/crates/rexx-exec/tests/suite.rs` (a test that
the file matches its derivation).

- [ ] **Step 1:** Derive the gate groups as spec section 4 says: `grep -rlwi` over `ootest/ooRexx` for
      each routine name in section 4's list plus `SysFileTree` and `SysFileXXX`, mapped to `dir/group`
      names, minus groups the row files make Phase 10's. The file's header carries the exact command.
- [ ] **Step 2:** A test re-runs the derivation and asserts the file equals it, so a renamed or added
      ooTest group cannot leave the file stale.
- [ ] **Step 3:** Run the harness over the file's groups at HEAD and record the starting verdicts in
      `docs/superpowers/plans/phase-9b-gate.md` (new), with the command and wall time. Commit.

