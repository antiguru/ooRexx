### Task 10: The seeded gate

**Files:** `tests/support/group_runner.rs` (`SwitchMode::Sim`, `run_crate_within` `:425-476`, stop
stripping `REXX_SWITCH_MODE` for children `:443`), `tests/concurrency_tests.rs` (beside
`whole_groups` `:3357-3507`), new `rust/corpus/sim-gate.tsv` (per group part: k, the seed count and the
debug subset), new `rust/corpus/sim-exempt.tsv`, new committed oracle outcome sets
(`rust/corpus/sim-oracle/`), `phase-6-1-gate.md`.

**Interfaces:** `REXX_SIM_SEEDS`, `REXX_SIM_POLICY`, `REXX_SIM_ONLY=group:part:seed:policy`,
`REXX_SIM_ORACLE_REFRESH=1`; seeds `seed_i = splitmix(hash(group, part), i)`.

- [ ] **Step 1: Calibrate.** One seed per group part of Phase 6 criterion 1 and per
      `corpus/phase-6.txt` program (the scheduler tests' inline programs are not in the gate: they
      pick their mode in code, `scheduler/tests.rs:347`; record that), in both builds: record each part's contended-step count (its k)
      and wall time; set the seed count and the debug subset so the gate's wall time fits a budget
      stated in the gate record. The policy of seed i is the mix's entry i mod 7: `pre:1`, `pre:2`,
      `pre:3`, `pct:3`, `uniform:0.01`, `uniform:0.2`, `uniform:1`, written per part in
      `sim-gate.tsv`. Populate `rust/corpus/sim-oracle/` from the harness's oracle runs (5, 30 when
      unsettled) under `REXX_SIM_ORACLE_REFRESH=1`.
- [ ] **Step 2: Judge.** Red on: a panic, a `scheduler_inconsistency` or other invariant refusal, a
      determinism breach, a hang past the run deadline, a design-limit refusal in a program whose
      oracle twin does not deadlock, or (not for `pct`) a check the program makes failing (a nonzero
      ooTest failure or error count, or a gate program's assertion line) where no committed oracle
      outcome of that part has it (R6). Each such red found while building the gate goes to Moritz
      case by case: a defect to fix in this task (witnessed per the Global Constraints' found-defect rule), or a `sim-exempt.tsv` row with its reason and
      evidence (P46's MutexSemaphore TEST_EXCLUSION deadlock is the first row). Mask `rexx-sim:` lines,
      including a child's.
- [ ] **Step 3: Report.** Each seeded outcome is compared with the part's committed oracle set
      (read-only; it grows only under `REXX_SIM_ORACLE_REFRESH=1`); differences go to the run's
      report file, never red; `fail=wait:K` runs are excluded. The replay line for a red is the harness
      command with `REXX_SIM_ONLY`, plus profile and stack size.
- [ ] **Step 4: Self-test.** A sample of parts under one seed and `pre:2`, the second run in a fresh process;
      identical trace hash and outcome.
- [ ] **Step 5: The gate can fail.** Revert, one at a time on a scratch branch, the hunk of each S2-S5 ledger fix
      Task 8 I1 (one timer post wakes only the last waiter) and Task 21 N1 (a halt readies an
      already-ready timed waiter), with Task 4 I2 (a pinned busy activity never preempted) as the
      alternate; each sits in a commit with other fixes (`7eafa3b58`; `f9fb61990`/`2e50fc410`), so revert only its hunk and record commit and file; find them with `grep -n 'Task 8\|Task 21\|Task 4' .superpowers/sdd/2026-10-01-phase-6-s2-s5/progress.md`
      and record it. The seeded gate goes red on each; record the reds and replay lines. Run the origin
      failure (REPLY derived under every opportunity, rc 1, `bg/b6efbfd53/logs/g4-test-release.txt:2210-2213`)
      under `uniform:1` and record the result. M11 through the gate: a `sim-gate.tsv` row running a
      group-shaped M11 program under `fail=wait:K` (excluded from the oracle report); with
      `scheduler.rs:1047-1049`'s `sleepers.retain` deleted the gate goes red through the invariant "a
      ready activity holds no park reason"; record the red and its replay line, then restore.
- [ ] **Step 6:** The per-task check (the gate is `REXX_CORPUS_GATE`-gated like `whole_groups`); the
      gate itself in release; record under `## Task 10`. Commit.

