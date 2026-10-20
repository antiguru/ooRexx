**CLOSED, ruled by Moritz 2026-10-20 ("Write-through, no Dev"): no Deviation.** rexxref expects buffering and offers NOBUFFER and FLUSH to make writes visible (funct.xml:4331-4335, 4390-4391; iostrms.xml:495-500), but never requires written data to stay hidden, so both interpreters follow it. The crate keeps writing through. Measured by Phase 9d Task 6.15 (`docs/superpowers/records/2026-10-18-phase-9d/t6.15/probes/lineout-visibility.rex`).

# LINEOUT to a named stream is buffered on the oracle

Found 2026-10-20 (`task-6.13-rereview-r2.md`, Minor 5, probes `buf` and `noclose`); it predates
Task 6.13. The oracle keeps `lineout` output to a named stream in its buffer until the stream is
closed, so a program that reads the file mid-run, or another process, sees it empty. The crate
writes through. This sits beside Deviation 49 (positions follow the documentation, buffered or
not); whether rexxref says anything about when written data becomes visible decides whether this
is a gap or a Deviation candidate.
