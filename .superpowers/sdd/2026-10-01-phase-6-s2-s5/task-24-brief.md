### Task 24: No refusal names Phase 6

**Files:** every remaining `"Phase 6"` owner, every Message and semaphore method still refused,
`tests/closed_phases.rs`, `rust/corpus/phase-6.txt` (new, from `phase-8.txt`'s Phase 6 witnesses),
exclusions.

- [ ] **Step 1:** Criterion 8, two enumerations, each derived by a committed command and shown
      empty: the S0/S1 command of `phase-6-gate.md` (messages and owners, not constructors only); and,
      by name, every method of the oracle's Message, EventSemaphore and MutexSemaphore tables
      (`memory/Setup.cpp:1055-1076`, `:1325-1356`) checked against this crate's dispatch tables and
      by a probe that it does not answer the generic not-implemented refusal (which names Phase 9,
      `lib.rs:409-417`, so a `"Phase 6"` grep cannot see it). Each one still refused is built here or
      re-homed with its reason. Each recorded divergence (inverted pinned waits, immovable REPLY,
      wrappers blocking on the baton, HALT under nesting, a pinned busy-waiter, stale C writes to
      reallocated lent storage, the `nohup` difference) is an exclusions row with owner none and its
      reason.
- [ ] **Step 2:** `closed_phases` gains Phase 6, and its negative control (`closed_phases.rs:335`)
      counts `"Phase 10"` alone; move the Phase 6 corpus witnesses from `phase-8.txt` to
      `phase-6.txt` (P17). Gates; commit.

