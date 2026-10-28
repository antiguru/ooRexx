# Task 8.6 fix round 1

Context: Phase 9d Task 8.6 (streams, paths, commands) of the ooRexx Rust rewrite, checkout
/home/moritz/dev/repos/ooRexx-rust-rewrite. You are a fresh implementer for the fix round. Read, in order:
task-8.6-dispatch.md, task-6-dispatch-rules.md and global-constraints.md (all in
.superpowers/sdd/2026-10-18-phase-9d/), the report task-8.6-report.md, then the review task-8.6-review.md.
The review's probes are copied to docs/superpowers/records/2026-10-18-phase-9d/t8.6/r1/probes/ (with
run3.sh, its three-way runner); records under t8.6/r1/. Append a "Fix round 1" section to the report.

Moritz's rulings (2026-10-28) on the report's candidates: A becomes a sentence in Deviation 40 (a write error
on a buffered standard output is answered by the stream writes after the hand-over that met it); B a new
numbered Deviation (a standard descriptor closed when rexx starts is open on /dev/null). The controller
writes both; you write nothing in phase-4-exclusions.txt.

1. Important 1: a held write on an APPEND stream must land at the end of the file when it is handed to the
   descriptor (open with O_APPEND, or seek to the end at hand-over), so appends by a command or another
   stream object in between are kept (append.rex, append2.rex: oracle cmd1|b1|b2|). Witness red at HEAD
   8ed7774e1.
2. Important 2: `.stdout~open('write')`, `stream('STDOUT','c','open write')` and `.stderr~open('write')`
   keep the .local object's writes in order with SAY and trace and never lose them at program end (reo1-reo6
   shapes; base 0fd10a074 gave the rule's answer). The oracle loses them too: that is an oracle bug, not
   copied. Witness each shape.
3. Minor 1: keep Box<NativeState> at its base size (box StreamState or equivalent); show peak RSS for 200k
   MutableBuffer instances back at base.
4. Minor 3: a program STDOUT stream's FLUSH hands over and answers the write error (`ERROR:n`) as the oracle
   does, so candidate A's sentence needs no FLUSH clause. Witness with a closed pipe and /dev/full.
5. Minor 2: pingguard's deterministic +0.0521% libc subtracted is over the +0.05% limit. Bring it under (item
   3 may do it); if it stays over, bisect and report the cause.

Perf: callgrind on every program against 0fd10a074 (rexx-base-0fd10a074 in
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.6/), +0.05% libc subtracted per program, the final code commit's
run recorded. Tests as the dispatch lists, including the workspace test once at the end. Do not touch
task-8-batches.md, progress.md, phase-4-exclusions.txt or oracle-defects/. Never go idle on a background
command: poll in the foreground in a bounded loop; never pgrep -f or pkill -f. rm only literal paths. No
subagents. Commit, then reply with only status, shas, one-line test summary, concerns.
