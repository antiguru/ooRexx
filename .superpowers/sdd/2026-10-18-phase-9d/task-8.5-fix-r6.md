# Task 8.5 controller-ruled fix (after round 5)

Re-review: .superpowers/sdd/2026-10-18-phase-9d/task-8.5-rereview-r5.md (read fully). Its probes are copied
to docs/superpowers/records/2026-10-18-phase-9d/t8.5/r6/probes/ (h1.rex, f1/f2/f5.rex, g1-g9.rex,
chains.rex); records under t8.5/r6/. Append a "Controller fix" section to task-8.5-report.md.

Ruling (controller, past round 5): Important 1 is load-bearing (a quadratic regression on a shape linear at
4a6321376), so one more fix, scoped to it:

1. Interp::forwarded (value.rs:1273) must not walk the whole chain at every outermost forward. Detect a
   cycle among the nested forwards instead, with total walking cost O(k log k) or better for a forward
   answered after k hops (the re-review proposes a walk from the receiver when stem_forwarding reaches a
   power of two; any equivalent bound is fine). A cycle made under the outer forward must then end in 11.1
   without consuming the native stack (Minor 1: f1, f2, g1-g7 at round 4's memory, about 20 MB, and g1
   under memcap 1G).
2. Witness: h1.rex's shape as a unit test that times out if quadratic (20000 nodes, bounded thread), and the
   Minor 1 shapes in the cyclic unit test with a peak-memory or depth assertion if one is practical; record
   h1 at 2000/10000/20000 and f1/g1 peak RSS.

Everything round 5 fixed must stay (round 4's and 5's probe sets byte-identical on stdout/stderr/rc).
Perf: callgrind on every program against 6e0bf5e5b (rexx-base-6e0bf5e5b in
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.5/), +0.05% libc subtracted per program, the final code
commit's run recorded. Tests as round 5. Do not touch task-8-batches.md, progress.md, phase-4-exclusions.txt
or oracle-defects/. Never go idle on a background command: poll in the foreground; never pgrep -f. rm only
literal paths. Commit, then report.
