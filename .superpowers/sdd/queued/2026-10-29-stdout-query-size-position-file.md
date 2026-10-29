**OPEN, queued 2026-10-29 by the controller; proposed 9d (Task 8.8).**

# .STDOUT's QUERY SIZE and POSITION when standard output is a regular file

Found by Phase 9d Task 8.7c (report Queue candidates 3; confirmed by its review). With standard output redirected to a regular file, `.STDOUT`'s `QUERY SIZE` and `QUERY POSITION` answer the null string here and the file's size and position on the oracle (`out size 34 1`). Derive from rexxref (iostrms.xml, STREAM QUERY) and Deviation 40 (buffered standard output).
Probe: docs/superpowers/records/2026-10-18-phase-9d/t8.7c/probes/stdinq/q5.rex.
