### Task 6: Hash methods on the condition object and `.methods`

**Files:** `dispatch/hash.rs:213` (`refusal_outside`, `not_this_task`), `dispatch/hash/relation.rs`, the
condition-object construction (`Message~errorCondition`, `Activity::reraiseException` equivalent),
`.methods` from `::METHOD` directives, corpus programs.

**Interfaces:** None new.

- [ ] **Step 1:** Corpus programs: `c~items` on `Message~errorCondition` (oracle `13`), `allIndexes`,
      `hasIndex`, `at` on it; `.methods~items`, `~allIndexes` in a package with `::METHOD`s; the same
      condition object re-raised and read again (identity: `c1 == c2` as the oracle answers).
- [ ] **Step 2:** Make both a store-backed Directory and StringTable, keeping identity under re-raise.
      Delete the refusals reached; any `not_this_task` call site still unreached keeps its refusal and
      is listed in the report. Refresh tables. Per-task check with `Message`, `METHOD`, `Package`
      through the harness. Commit.

