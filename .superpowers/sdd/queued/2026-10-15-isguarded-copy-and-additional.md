**CLOSED 2026-10-22 by Phase 9d Task 8.3 (68e7f15d5).**

**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** the review's probes still differ: isGuarded 1 against 0, additional[1] a against the whole string.

# Two divergences found by 9b1 Task 8 review (pre-existing)

1. An enhanced unguarded copy: `.context~executable~isGuarded` answers 1; the oracle answers 0.
2. `raise ... additional("str")` then `condition('o')~additional[1]`: crate answers 'a', oracle the
   whole string. Probes in .superpowers/sdd/2026-10-14-phase-9b1/task-8-review-files/.
