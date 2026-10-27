# Task 8.5 fix round 4

Context: Phase 9d Task 8.5 (variables) of the ooRexx Rust rewrite. Read, in order: task-8.5-dispatch.md,
task-6-dispatch-rules.md and global-constraints.md (all in .superpowers/sdd/2026-10-18-phase-9d/), then
task-8.5-report.md's "Fix round 3" section and the re-review task-8.5-rereview-r3.md. Probes are under
docs/superpowers/records/2026-10-18-phase-9d/t8.5/r4/probes/ (records under t8.5/r4/). Append a "Fix round
4" section to task-8.5-report.md.

1. Important 1: a cycle through a stem's default (`s. = >s.; say s.`, and via a variable: sd_via.rex,
   stemdef.rex cases) still spins without a HALT check and ignores SIGTERM, or overflows the stack
   (sd_tr.rex, sd_msg.rex). Controller ruling as for round 3: every chain walk, whether its links are
   references or stem defaults, detects a cycle and raises 11.1 in the same positions `r = >r` does
   (conversions, forwarded messages, operators), and renderings show the reference's or stem's name. No
   path may loop without a HALT check or recurse unboundedly. Add the stem-default shapes to
   a_cyclic_reference_is_insufficient_stack_in_every_conversion (own thread, timeout) and sd1.rex to
   rust/corpus/oracle-crashes.txt in its format (the oracle hangs; never run the oracle on it or on entries
   59 and 60). Keep the acyclic paths at their current cost.
2. Minor 1: move the doc lines at value.rs:1009-1013 back so Referent and Redirect each carry their own.

Perf: callgrind on every program against 6e0bf5e5b (rexx-base-6e0bf5e5b in
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.5/), +0.05% libc subtracted per program; record the run of the
final code commit. Tests: fmt, clippy -D warnings, rexx-exec --lib, collect_stress, signals, the corpus gate
(known mismatch rexxutil_file_date_time.rex), coverage, and the whole suite compared with t8.5/r3/'s tables.
Build under CARGO_TARGET_DIR=/tmp/claude-1000/t85-fix4-target, delete at the end. Do not touch
task-8-batches.md, progress.md, phase-4-exclusions.txt or oracle-defects/. Never go idle on a background
command; never pgrep -f; never pkill -f. rm only literal paths. No subagents. Commit, then report.
