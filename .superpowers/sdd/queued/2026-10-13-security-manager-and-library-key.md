**CLOSED 2026-10-28 by Phase 9d Task 8.7b (5053fbd9b).**
**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** no probe recorded; two oracle measurements to make first.

# Two findings from 9b Task 9 (not RexxUtil)

Source: .superpowers/sdd/2026-10-13-phase-9b/task-9-report.md, concern 5.
1. The crate's security manager sees `setSecurityManager` take effect in the activation that is
   already running; the oracle's does not. Measure the oracle's boundary (next activation? next
   call?) and match it, with a witness.
2. `libraries.rs` keys its package table `REXXUTIL` where the oracle keys `rexxutil`. The table models
   unload order. Measure whether any observable (unload order, package name, events) differs.
