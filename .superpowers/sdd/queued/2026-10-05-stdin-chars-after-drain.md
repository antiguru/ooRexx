**CLOSED 2026-10-30 by Phase 9d Task 8.8e1 (probe agrees).**

# chars/lines on standard input answer 1 after a read drained the pipe

Found by Phase 6 S2-S5 Task 21 fix round 4 (concern 5). Pre-existing.

After a read has drained a pipe on standard input, `.stdin~chars` and `lines()` answer 1 here where the
oracle answers 0.


## Task 8.8e1 evidence

Durable independent stdout, stderr and status rechecks cover piped EOF and a separately held-open empty pipe. Zero queries alone do not establish EOF. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8e1-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1/`.
