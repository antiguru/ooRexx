### Task 7: Package setting writes

**Files:** `dispatch/package.rs:257,311`, `lib.rs:662` (`package_option_write`), corpus programs.

- [ ] **Step 1:** Corpus programs: `Package~options(name, value)` and `~defaultOptions(name, value)`
      for each option name `Package_Options.testGroup` sets (read the group for the names), their read
      back, and their effect on a program loaded afterwards (`digits`, `form`, `fuzz`, `trace`, the
      `'C'` count form `rexx -o` uses).
- [ ] **Step 2:** Implement; delete the constructor. Per-task check with `Package_Options` through the
      harness: every test the oracle passes passes, or is in the 9d set. Commit.

