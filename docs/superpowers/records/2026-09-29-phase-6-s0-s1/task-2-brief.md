### Task 2: Benchmarks, the measurement script and the Phase 6 base

**Files:** `rust/bench-programs/` (new programs and README rows), a committed script under
`rust/crates/rexx-bench/` or `rust/bench-programs/` (the plan-time choice goes in the report),
`docs/superpowers/plans/phase-6-perf.md` (new).

- [ ] **Step 1:** Commit benchmark programs: a recursive fib(22) by internal CALL, the same by
      function call, a send loop (a `::METHOD` called in a loop, enough iterations for about 1 s),
      and an extension-call loop through `liborxfunction` (a routine from `testbinaries/orxfunction.cpp`
      called in a loop, loaded with `::REQUIRES ... LIBRARY`). Each runs identically on the oracle
      (record the comparison).
- [ ] **Step 2:** Commit a callgrind script generalising
      `docs/superpowers/records/2026-09-23-driver-spikes/clause-overhead-files/meas.sh` and its
      `sumobj.py`: it takes two or more binaries, runs every program interleaved under
      `valgrind --tool=callgrind --compress-strings=no`, subtracts libc and ld-linux, and prints one
      table. No hard-coded scratch or worktree paths.
- [ ] **Step 3:** Build the Phase 6 base (this task's parent commit) in its own target directory and
      three layout controls (dead code of increasing size, each its own target directory), measure
      every program, and record in `phase-6-perf.md`: the base figures, each control's delta and the
      noise band per program, plus five interleaved wall-clock runs per program. Quote every
      command.
- [ ] **Step 4:** Gates; commit.

