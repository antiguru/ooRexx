# `drop w.1` on an unset stem: SYMBOL('W.') diverges

Found by Phase 6 Task 7's implementer (2026-09-30), not fixed: after `drop w.1` on a stem never
assigned, the oracle answers `symbol('W.')` = VAR and the Rust interpreter LIT.
Repro and oracle command: ask for them in .superpowers/sdd/2026-09-29-phase-6-s0-s1/task-7-report.md.
