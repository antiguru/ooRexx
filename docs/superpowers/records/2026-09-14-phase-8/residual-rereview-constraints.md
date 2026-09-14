# Re-review of the residual round — addendum to fix-rereview-constraints.md

Everything in `fix-rereview-constraints.md` applies (read-only, a gate run is live in the
worktree's `rust/target/`, build only in a `git archive` copy, the oracle hazards, the rules a
finding may cite, the report contract), with these changes:

* Range: `cf92ff4fb..233d2766d` (X1 `69a579370`, X2 `0e57202cd`, X3 `f83b028a7`, X4 `9b7f77e4c`,
  X5 `e8a6b7667`, Y1 `9c0d44d8e`, Y2 `233d2766d`). Build your copy from `233d2766d`.
* Brief: `residual-fix-brief.md`. Report: `residual-fix-report.md` (its "Follow-up requested by the
  controller" section covers Y1 and Y2). Rulings: `progress.md` from "Adjudication of the re-review
  residuals" on. Findings being fixed: `fix-rereview-boundary.md` (finding 1, minors 3 and 4) and
  `fix-rereview-integration.md` (R1-R5, R7, R8, R10).
* Probes: `scratchpad/residual-fix/`, `scratchpad/rereview-int/`, `scratchpad/boundary/`.
* **Never run `rust/corpus/oracle-crashes.txt` entry 15** (added in this range) or any shape the
  Y1 section of the report records as rc 139 on the oracle.
