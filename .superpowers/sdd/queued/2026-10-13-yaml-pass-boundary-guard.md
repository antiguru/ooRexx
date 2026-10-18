**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** no probe of its own; Task 4 consumes it (ruling 2), yaml TEST35_DIRECTIVES was blocked on this refusal at the 9c close.

# yaml.cls reaches the op_not_driven GUARD

Found 2026-10-13 by the Phase 9a final review (`.superpowers/sdd/2026-10-11-phase-9a/final-review.md`, I4).
Pure-Rexx `extensions/yaml/yaml.cls` reaches the `op_not_driven` GUARD refusal ("pass boundary with no
loop open"); the review reproduces it in five lines (its repro is in the review). Pre-existing at the
Phase 9a base; 9a's unblocking exposed it, and the extensions group's Phase 10 Extension row hides it
from the suite gate. It is an IR driver defect, not an extension one.

Owner: Phase 9d (engine divergence), unless 9b/9c reach it first.
