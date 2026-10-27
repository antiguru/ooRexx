# Task 8.5 fix round 5 (the last round before the controller rules)

Re-review: .superpowers/sdd/2026-10-18-phase-9d/task-8.5-rereview-r4.md (read fully; probes under
t85-rereview4-scratch/, copy what you use to docs/superpowers/records/2026-10-18-phase-9d/t8.5/r5/probes/).
Append a "Fix round 5" section to task-8.5-report.md.

1. Important 1: an acyclic chain of Stem subclass instances is quadratic (instance_conversion and the
   stem_subclass_* renderings walk the whole chain once per link; 20000 links: 2.56 s against 0.0003 s at
   4a6321376; 100000 links killed). Make every chain walk linear in the chain's length, one walk per
   conversion. Witness: a unit test that would time out if quadratic (a long subclass chain, bounded wall
   time on its own thread), and the re-review's timing probe at 20000 and 100000 links recorded.
2. Minor 1: forwarding through a reference takes the reference end; only what needs the whole chain asks
   for its end. `s. = >s.; q = >s.; say q~items` (and ~allIndexes, ~put, ~hasIndex) answers as `s.~items`
   does, not 11.1. Witness these.
3. Minor 2: name in the report every acyclic answer that moved (q~string, r~string, TRACE R of r), with the
   rexxref line it follows.

Queued by the controller, not yours: a stem's own string value with an array default (t.~string, TRACE of a
stem).

Perf: callgrind on every program against 6e0bf5e5b (rexx-base-6e0bf5e5b in
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.5/), +0.05% libc subtracted per program, the final code
commit's run recorded. Tests as round 4. Do not touch task-8-batches.md, progress.md, phase-4-exclusions.txt
or oracle-defects/. Never go idle on a background command: poll in the foreground; never pgrep -f. rm only
literal paths. Commit, then report.
