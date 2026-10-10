### Task 12: Close

**Files:** `.superpowers/sdd/queued/` (new items), `docs/superpowers/plans/2026-07-27-rust-rewrite.md`
(rows 6.1 and 9), `phase-6-1-gate.md`, the SDD workspace's `bggates.sh`.

- [ ] **Step 1:** Queue, one file each: scout A section 4's silent divergences (concatenation with a
      variable reference, `RAISE NOVALUE` into `CALL ON ANY`, `subclass package`, `subclass rexxinfo`;
      `.context~name` in a `Routine~new` body already has a file); scout B's `cond3` extra traceback
      line; scout D's `Class~enhanced` skipping `NEW` and `'12'~translate('','')`; the shrinker; an
      oracle-judged seeded gate; P52 lending under sim (a pool with seeded delivery).
- [ ] **Step 2:** Roadmap: row 9 names subclass `NEW` on String, Stem, Method, Routine and Message as
      its work (R2; `scout-d-prototype.patch` as the starting point); row 6.1's parenthetical names
      random bounded preemption with opt-in PCT, judged by invariants (R1, R5); row 6.1 marked closed
      with the gate record.
- [ ] **Step 3:** The whole-groups run (`concurrency_tests.rs` `whole_groups`, `REXX_CORPUS_GATE=1`),
      its table in the gate record; each spec criterion 5 row passes or has a reason naming a cause
      outside 6.1; TIME TEST_5 and TEST_11 get a reason.
- [ ] **Step 4:** Cumulative perf on every program against the base; gates G1-G9 via `bggates.sh`
      (`-j 4`); the TSan run of Phase 6 criterion 3 repeated; the seeded gate in release and its debug
      subset. Each spec section 8 criterion recorded with its evidence in the gate record. Commit.
