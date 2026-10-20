**OPEN: found by the Phase 9d Task 6.13 round-2 re-review, not yet ruled.** The oracle buffers LINEOUT to a named stream until close; the crate writes through.

# LINEOUT to a named stream is buffered on the oracle

Found 2026-10-20 (`task-6.13-rereview-r2.md`, Minor 5, probes `buf` and `noclose`); it predates
Task 6.13. The oracle keeps `lineout` output to a named stream in its buffer until the stream is
closed, so a program that reads the file mid-run, or another process, sees it empty. The crate
writes through. This sits beside Deviation 49 (positions follow the documentation, buffered or
not); whether rexxref says anything about when written data becomes visible decides whether this
is a gap or a Deviation candidate.
