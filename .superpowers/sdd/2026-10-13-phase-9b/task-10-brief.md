### Task 10: 9b close

**Files:** `docs/superpowers/plans/phase-9b-gate.md`, `docs/superpowers/records/2026-10-13-phase-9b/`.

- [ ] **Step 1:** Commit, then the controller runs the sub-phase close command
      (`REXX_CORPUS_GATE=1 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast`)
      and the whole suite (`REXX_SUITE_GATE=1`), writing outputs to a durable directory.
- [ ] **Step 2:** The gate record, each spec 9b criterion with its evidence: every test in a Task 2 group
      passes under the harness, or is listed in `rust/corpus/suite-9d-set.txt` (the carried list, so 9b judges
      only what it owns), or carries a row the exclusion test accepts; no `"Phase 10"` site in RexxUtil lacks a
      D3 reason; the close command's exit status; the whole-process callgrind table against the pin. The
      suite outputs copied to the records directory. Commit.
