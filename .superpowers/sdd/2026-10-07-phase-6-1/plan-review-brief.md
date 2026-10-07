# Plan review: Phase 6.1

Plan: `docs/superpowers/plans/2026-10-07-phase-6-1.md` at `b89f33392`. Spec it implements:
`docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md` (rulings R1-R8 bind). Evidence: the
scout reports and spec reviews beside this file. Rules: `scout-common.md` beside this file (read-only;
runs only where a finding needs one; no subagents).

Review as the controller who will dispatch each task to an implementer who sees only that task's text:
- spec coverage: every spec decision, staging row and exit criterion maps to a task step; name gaps;
- each task's text agrees with itself and with its neighbours (files one task creates and another
  touches; interfaces produced and consumed; order: Task 1 before 6, Task 4 before 7, Task 5 before 7,
  Task 8 before 9 before 10 before 11);
- file:line citations at `b89f33392` (sample at least ten), names that do not exist;
- steps an implementer could not execute without guessing (missing probe text, missing expected output,
  an undefined measurement), and checks that could not fail;
- Review Focus: each line has its test in a task.

Write `.superpowers/sdd/2026-10-07-phase-6-1/plan-review.md`: verdict (dispatch / dispatch after fixes /
rework), findings BLOCKER / IMPORTANT / MINOR with the plan's text, evidence, fix, and a table of
pairs of tasks sharing a file or interface with what you found. Questions only Moritz can answer, if
any, with a recommendation. Prose minimal, no em-dashes. Return verdict, counts, path.
