### Task 8: The side-map sweep

**Files:** whatever maps remain. The output is
`docs/superpowers/records/2026-10-14-phase-9b1/side-maps.tsv`.

- [ ] **Step 1:** Enumerate every map field on `Interp`, `Heap`, `ClassRegistry`, the scheduler and
      the `rexx-api` handle tables, with a committed command (e.g. a `git grep` for `FxHashMap<` and
      `HashMap<` and `Vec<Option<` field declarations). For each, record in `side-maps.tsv`: the
      object it describes, its key type, its state (1, 2 or 3 of spec section 5, or "not per-object"
      with the reason), and the code site that prunes it.
- [ ] **Step 2:** Fix every map that is per-object and in none of the three states. A fix that needs a
      change outside the map's own readers is reported, not widened.
- [ ] **Step 3:** Check the queued `2026-10-02` clock item and the `2026-10-07` SETLOCAL item
      against Phase 6.1's move to the activation, re-probing the REPLY seams on both sides. Close
      each one that is stale, and report each that is not.
- [ ] **Step 4:** Every `ALLOWED` row tagged Task 8 (the freed-handle, body-shape and stack-frame
      guards) is either removed, or shown unreachable from any Rexx program and retagged
      "internal guard" with the argument in the test's comment. A guard reachable from a program is a
      leak, and it is fixed here or reported.
- [ ] **Step 5:** Per-task check. Commit.

