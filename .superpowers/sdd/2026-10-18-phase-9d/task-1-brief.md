### Task 1: The 9d set re-derived and partitioned

**Files:** `rust/corpus/suite-9d-set.txt`,
`docs/superpowers/records/2026-10-18-phase-9d/clusters.tsv` (new),
`docs/superpowers/records/2026-10-18-phase-9d/clusters.md` (new).

**Produces:** `clusters.tsv`, with columns `group`, `test`, `cluster` and `cause`. `cause` is one
line naming the first divergence.

- [ ] **Step 1:** Re-derive `suite-9d-set.txt` from
      `docs/superpowers/records/2026-10-15-phase-9c/close/suite/tests.tsv`. Use the awk filter in
      the file's header, minus its 9b and 9c witness exclusions: a witness test that still fails
      with no row belongs in the set, and at the 9c close none of them fails. Update the header's
      provenance to that path, the close sha `25197002a` and the new command.
- [ ] **Step 2:** Find each test's cause. The harness keeps no assertion text: `GroupRun` holds name
      sets and blocked texts, and the group's stdout is discarded. So run each group of the set by
      hand, as the harness does (`suite.rs:574` and `:639`, which lay out the run directory), on both
      sides. Keep both sides' stdout and stderr under
      `docs/superpowers/records/2026-10-18-phase-9d/causes/<group>/`, and read the failing
      assertion or refusal text from them.
- [ ] **Step 3:** Measure `samples/scclient` and `samples/sfclient` the same way, and record their
      cause.
- [ ] **Step 4:** Group the tests by cause into clusters named by the shared mechanism, and write
      `clusters.tsv`. Reuse the parent spec's cluster names where they fit. Write `clusters.md`.
      For each cluster it records:
      * its test count, by the command `awk -F'\t' '$3=="<cluster>"' clusters.tsv | wc -l`;
      * the files a fix will likely touch;
      * whether it needs a Deviation ruling, because the oracle is wrong;
      * whether it reaches the byte-name stream paths (Review Focus 3) or INTERPRET and REPLY
        (Review Focus 4).

      Commit.

