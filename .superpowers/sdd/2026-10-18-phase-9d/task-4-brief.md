### Task 4: `json`, `yaml` and `dateparser`

**Files:**
* `rust/crates/rexx-exec/tests/support/suite_rows.rs` (`phase_10_group`, `:278-289`).
* `rust/crates/rexx-exec/tests/suite.rs`: the fixture's `PASSING` row at `:1516` and `FAILING` at
  `:1544`.
* `docs/superpowers/plans/phase-9-exclusions.txt`.
* `rust/corpus/refusal-dispositions.tsv`.
* The IR compiler and driver behind `Loud::op_not_driven` (`rexx-exec/src/lib.rs:877`, reached
  from `src/run/loops.rs:1859` and `src/ir/drive.rs:3782`), with `src/ir/` where the compiled op
  comes from.
* `rust/corpus/lang/<name>.rex`, one or more.

**Measured by Task 1:** the two failing yaml tests, `TEST35_DIRECTIVES` and
`TEST47_MULTILINEFLOWCOLLECTIONS`, are `blocked` at the 9c close under `Phase10(Extension)` rows,
so they are not in the 9d set. This task owns them.

**Consumes:** `.superpowers/sdd/queued/2026-10-13-yaml-pass-boundary-guard.md`, and its repro in
`.superpowers/sdd/2026-10-11-phase-9a/final-review.md:80-90`.

- [ ] **Step 1:** Reduce the repro to pure Rexx programs with no `::requires`, one per refusal text
      the two yaml tests reach. Add them as corpus programs, and show each red: the crate answers
      `op_not_driven`, the oracle its own output.
- [ ] **Step 2:** Fix the compiler or driver so that each program matches the oracle. Correct the
      `refusal-dispositions.tsv` GUARD rows that call the site unreachable: delete them if the
      constructor is gone, otherwise reword them with the probes now tried.
- [ ] **Step 3:** Make `phase_10_group` answer `None` for groups under `extensions/json/`,
      `extensions/yaml/` and `extensions/dateparser/`. Move the fixture's json row from `PASSING`
      to `FAILING`, and put an `extensions/rxmath/RxMath` row in its place. Re-derive
      `phase-9-exclusions.txt` as `the_row_files_hold_only_checked_rows` requires.
- [ ] **Step 4:** Run the json, yaml and dateparser groups through the harness until every test
      passes. Run the yaml group once more by hand on the crate side, with
      `extensions/yaml/yaml.cls` copied over the oracle's copy in the run directory that
      `run_group_on_crate_within` (`suite.rs:639`) lays out, and record the result (Review Focus 2).
      Run the perf check, then commit.

