# Two divergences found by 9b1 Task 8 review (pre-existing)

1. An enhanced unguarded copy: `.context~executable~isGuarded` answers 1; the oracle answers 0.
2. `raise ... additional("str")` then `condition('o')~additional[1]`: crate answers 'a', oracle the
   whole string. Probes in .superpowers/sdd/2026-10-14-phase-9b1/task-8-review-files/.
