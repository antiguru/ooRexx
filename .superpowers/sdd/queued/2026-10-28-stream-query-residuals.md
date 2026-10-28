**OPEN, queued 2026-10-28 by the controller; proposed 9d (Task 8.7).**

# Stream query residuals: STDIN queries, LINES on a write-only stream, /dev/null as transient

Found by Phase 9d Task 8.6 (task-8.6-report.md, Concerns; task-8.6-review.md, Queue candidates); each
predates the task.
* An open program-made STDIN answers `QUERY SIZE`, `QUERY POSITION`/`SEEK` and `QUERY DATETIME` with the
  null string, where the oracle answers from descriptor 0 (q_stdinq.rex).
* `f = .stream~new('f.txt'); f~open('write'); f~lineout('f1'); say f~lines`: oracle 1, this crate 0.
* `.stream~new('STDIN')~seek('1')` with stdin /dev/null: oracle `1` and READY, this crate 93.958, because a
  character device counts as transient here; with stdin a file the oracle answers `1 READY`, this crate
  `0 ERROR`.
Probes: docs/superpowers/records/2026-10-18-phase-9d/t8.6/r1/probes/ (q_stdinq.rex and the review's shapes
quoted in task-8.6-review.md). Derive each answer from rexxref (iostrms.xml, STREAM QUERY, LINES, SEEK)
before following the oracle; the oracle's /dev/null CHARS/LINES answer is an oracle defect, not copied.
