**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** probe still differs: chars() and .stdin~chars answer 1 here and 0 on the oracle.

# chars/lines on standard input answer 1 after a read drained the pipe

Found by Phase 6 S2-S5 Task 21 fix round 4 (concern 5). Pre-existing.

After a read has drained a pipe on standard input, `.stdin~chars` and `lines()` answer 1 here where the
oracle answers 0.
