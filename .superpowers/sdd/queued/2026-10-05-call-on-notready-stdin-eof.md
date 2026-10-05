# CALL ON NOTREADY at end of the default input stream is Error 43.1

Found by the Phase 6 S2-S5 Task 21 re-review 4 (N-3). Pre-existing (present before P66 at bf76afa1d).

`call on notready` then `v = linein()` (or `parse linein`, `charin()`) at end of input: the oracle
prints the handler's output and rc 0; ours ends rc 213 with `Error 43.1: Could not find routine
"NOTREADY"` raised from the monitor's `UNKNOWN`. Same in a method and in a started activity. SIGNAL ON
NOTREADY works. The CALL trap is looked up in the activation that raised the condition (the monitor's
UNKNOWN) rather than the caller's. Probes nt1-nt6 in `.superpowers/sdd/2026-10-01-phase-6-s2-s5/task-21-rereview-4.md`.
