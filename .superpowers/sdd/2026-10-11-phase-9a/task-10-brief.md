### Task 10: 9a close

**Files:** `phase-9a-gate.md`, `.superpowers/sdd/queued/` (items found, one file each).

- [ ] **Step 1:** Commit, then run in the background with a status file:
      `REXX_CORPUS_GATE=1 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast`
      (the sub-phase close command, spec section 9) and the whole suite (`REXX_SUITE_GATE=1`).
- [ ] **Step 2:** The gate record, each spec 9a gate item with its evidence: the suite table and
      wall time; every section 3 refusal's witness; the row-file test, the D3 test; the denominator
      check on every group with no test blocked by a refusal 9b or 9c owns, the blocked tests listed
      with their refusal text; `Package` ends; the close command's exit status. The Environment
      candidates listed for Moritz. Commit.
