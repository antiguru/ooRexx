# rexxcps +0.09% Ir between Task 6 fix rounds 1 and 2, unexplained

Found by the Phase 6.1 Task 6 re-review (`task-6-rereview.md`, Perf: round 2 against round 1 +0.0919% on rexxcps). Queued by Phase 6.1 Task 12 fix round 1 (2026-10-10). Not a divergence; a perf delta nobody attributed.

The re-review's own build of round 2 (`5495d5a08`) measured rexxcps 17,792,493,520 Ir against 17,784,651,245 at the Task 6 base (`fe4956b36`), +0.0441%, and +0.0919% against round 1's build. Round 2 changed the pauses after LEAVE and ITERATE and how typed lines trap, both debug-pause paths; rexxcps runs no debug pause. At the 6.1 close (`ab4bc780e`) rexxcps is +0.2694% against base61, inside the +0.5% budget (`.superpowers/sdd/2026-10-07-phase-6-1/task-12-evidence/cg-close2/table.txt`). No probe: a cgdiff of `5495d5a08^` against `5495d5a08` on rexxcps would attribute it.

Suspected site: the IR driver's debug-pause arms (`ir/drive.rs`), touched by `5495d5a08`.
