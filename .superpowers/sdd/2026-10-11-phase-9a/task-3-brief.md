### Task 3: The row files, their checks, the first whole run

**Files:** `docs/superpowers/plans/phase-9-exclusions.txt`, `rust/corpus/suite-licences.txt`,
`rust/corpus/suite-not-run.txt` (all new), `suite.rs` (the checks and the gate test),
`concurrency_tests.rs` (the `DIFFERING` cross-check), `phase-9a-gate.md`,
`rust/corpus/phase-9d-set.txt` (new), `rust/CLAUDE.md:13`.

**Interfaces:** Produces `fn expected_set(dir, group) -> BTreeSet<String>` (the widened
`source_test_names`, per this plan's ruling), the row-file parser, and the gate test
`l3_core_suite` that judges D1.

- [ ] **Step 1:** Widen `source_test_names` to follow a test class that inherits from, or
      subclasses, another `ooTestCase` class in the same file; test it on a fixture with each shape.
- [ ] **Step 2:** Row-file format, one row per test: `group<TAB>test<TAB>owner<TAB>reason`, owner one
      of `Phase 10 Rxapi`, `Phase 10 <D3 reason>`, `Deviation N`, `Environment`, `Flaky`, `NotRun`.
      The owner checks of spec section 2, each with a failing fixture row first: Rxapi rows equal
      `reaching_rxapi`'s output over the suite; other Phase 10 rows sit in a Phase 10 group or their
      captured refusal names Phase 10 with a D3 reason; Deviation N exists under `DEVIATIONS --
      permanent, chosen` in `phase-4-exclusions.txt`; Environment rows cite a committed oracle run
      record under `rust/corpus/suite-oracle-runs/` and carry `ruled: Moritz <date>`; Flaky rows cite a
      `common.txt` entry; NotRun rows carry `restrictOS`, `test type <name>` or another stated cause
      that the check verifies against the group file.
- [ ] **Step 3:** The denominator check (started equals expected minus rows) and D1's green rule as
      the gate test `l3_core_suite`, run only under `REXX_SUITE_GATE=1`. The `DIFFERING` cross-check:
      every test named in a `DIFFERING` key's failing list or as its `last started` is fixed or has a
      row.
- [ ] **Step 4:** Seed the files from a whole run (`REXX_SUITE_GATE=1`, all groups, under `memcap
      8G`, committed first, run in the background with a status file): the Phase 10 groups and
      Rxapi tests, `RexxQueue` and `RXQUEUE` tests, `Macrospace`, the four `common.txt` entries, the
      `restrictOS` and skipped-type groups (`bug2061_newline_misalignment`, `chapter7/Section4`). The
      two socket samples and any other oracle-side failure go to the gate record as **candidates** for
      Environment rows, listed for Moritz's ruling; they are not added without it. Find whether the
      harness copy causes the four `STREAM`/`Stream` relative-file oracle failures (scout A section 1)
      and fix a harness cause.
- [ ] **Step 5:** Commit the 9d set: the TSV's `tests_wrong` tests minus the 52 witnesses spec
      section 3 names, re-derived against this run, with the command in the file's header.
- [ ] **Step 6:** Correct `rust/CLAUDE.md:13` ("That separate `-O3` build is gone from this
      machine") with scout B's finding, quoting `scout-b-report.md` section 2.
- [ ] **Step 7:** The run's table in the gate record (`## Baseline suite run`), with the commit, wall
      time and command. Per-task check. Commit.

