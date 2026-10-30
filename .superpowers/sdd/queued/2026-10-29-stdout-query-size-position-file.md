**CLOSED 2026-10-30 by Phase 9d Task 8.8f (3979b7e31).**

# .STDOUT's QUERY SIZE and POSITION when standard output is a regular file

Found by Phase 9d Task 8.7c (report Queue candidates 3; confirmed by its review). With standard output redirected to a regular file, `.STDOUT`'s `QUERY SIZE` and `QUERY POSITION` answer the null string here and the file's size and position on the oracle (`out size 34 1`). Derive from rexxref (iostrms.xml, STREAM QUERY) and Deviation 40 (buffered standard output).
Probe: docs/superpowers/records/2026-10-18-phase-9d/t8.7c/probes/stdinq/q5.rex.

## Task 8.8f evidence

Owned regular-file stdout exposes SIZE, default and WRITE CHAR positions, and SYS through safe descriptor queries. SIZE and SYS hand over pending stdout at the query boundary. A parent-held descriptor verifies the resulting bytes, size and offset, and an in-process Invocation cannot inspect host stdout. Existing transient-output behavior and Deviation 40 remain intact. See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8f-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/`.
