**CLOSED 2026-10-30 by Phase 9d Task 8.8e1 (probe agrees).**

# yaml.cls reaches the op_not_driven GUARD

Found 2026-10-13 by the Phase 9a final review (`.superpowers/sdd/2026-10-11-phase-9a/final-review.md`, I4).
Pure-Rexx `extensions/yaml/yaml.cls` reaches the `op_not_driven` GUARD refusal ("pass boundary with no
loop open"); the review reproduces it in five lines (its repro is in the review). Pre-existing at the
Phase 9a base; 9a's unblocking exposed it, and the extensions group's Phase 10 Extension row hides it
from the suite gate. It is an IR driver defect, not an extension one.

Owner: Phase 9d (engine divergence), unless 9b/9c reach it first.


## Task 8.8e1 evidence

Both reductions agree, and TEST35_DIRECTIVES and TEST47_MULTILINEFLOWCOLLECTIONS pass in the full YAML group with both class copies. The private suite harness overlay changes only the selected crate-side yaml.cls source. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8e1-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1/`.
