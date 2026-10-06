### Task 26: The Phase 6 performance gate

**Files:** `phase-6-perf.md` (`## S2-S5 gate`), `phase-6-gate.md` (performance subsection of S5).

- [ ] **Step 1:** Fresh builds of the base `1754a3b5a`, the S1 close head `1a81353e3` and HEAD, each
      in its own target directory from a touched `git archive` tree with a `Compiling` line; three
      layout controls (`layout-pad.py`, measuring noise) against the base;
      `rust/bench-programs/callgrind.sh` over every program. Record the running total against the
      base and the delta against S1's close per program, with the commands and binary hashes. The
      named risks: Task 2's pin depth in every build and Task 12's variable barrier.
- [ ] **Step 2:** Wall clock (`wallclock.sh`, load under 2) recorded, and the extension-call loop's
      wall clock against the oracle (spec section 7). No layout iteration (P19).
- [ ] **Step 3:** Within budget (Global Constraints): record and close. Over budget: at most three
      rounds (a round is one committed candidate measured on every program), then stop and put the
      figures to Moritz.
- [ ] **Step 4:** Update roadmap row 6 (Phase 6 closed, pointing at `phase-6-gate.md`); gates;
      commit.
