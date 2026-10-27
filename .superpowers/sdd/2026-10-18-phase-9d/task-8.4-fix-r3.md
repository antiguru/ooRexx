# Task 8.4 fix round 3

Re-review: .superpowers/sdd/2026-10-18-phase-9d/task-8.4-rereview-r2.md (read fully). Its probe sources are in .superpowers/sdd/2026-10-18-phase-9d/t84-rereview2-scratch/src/; copy the ones you use into docs/superpowers/records/2026-10-18-phase-9d/t8.4/r3/probes/ (records go under t8.4/r3/). Append a "Fix round 3" section to task-8.4-report.md.

1. I1: a failing run of the ::CONSTANT method must record its level's site the way the install path does (install.rs:1083-1118: blame_directive, capture_site_frame, seal_site_level), minus the ::CLASS echo, which is install-only. q3, q6, q7, q8, c3, c5 must match the oracle on stdout, stderr and rc (rule: POSITION is where the condition is raised, condtra.xml:769; the level is a method level). Share the code with the install path rather than copying it. Witness: a corpus program (single-threaded) covering the expression's own failure and a failure below it, red at b3875ce42.
2. M1: trace the run's >I> before the guard wait, as an ordinary guarded method does (t1, t2 shapes; control t1c). Witness t1's shape if deterministic, else a unit test run 5 times.
3. M2: correct task-8.4-report.md:212-213 ("another" constant) to the candidate B wording.
4. Add the re-review's new oracle defect (q5d: on the oracle a run replaces the constant's value) to candidate B's measurement in the report, with its probe under r3/probes/.

Perf: callgrind on every program against f0164456d (rexx-base-f0164456d in /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.4/), +0.05% libc-subtracted per program. Tests as before (fmt, clippy, touched binaries, rexx-parse, corpus gate). Do not touch .superpowers/sdd/queued/, queued-inventory.tsv, task-8-batches.md, oracle-defects/ or phase-4-exclusions.txt. Never go idle on a background command. Commit, then send status.
