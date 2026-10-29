**OPEN, queued 2026-10-29 by the controller; proposed Phase 11.1 with its family (debug-typed-raise-untrapped, debug-typed-signal, debug-typed-call-sigl).**

# An EXIT typed at a debug pause is ignored

Found by Phase 9d Task 8.7c fix round 2 (report Concerns 2). A line `exit` typed at an interactive debug pause ends the program on the oracle and is ignored here: `run_debug_fragment` drops the fragment's Flow. Same family as the typed RAISE and typed SIGNAL items.
Probes: docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r2/probes/off/ex.rex, ex2.rex, ex3.rex (each with its .stdin).
