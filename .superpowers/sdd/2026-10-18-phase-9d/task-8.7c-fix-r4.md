# Task 8.7c fix round 4

Phase 9d Task 8.7c (variables and streams residuals) of the ooRexx Rust rewrite. Code HEAD cd070fd13, records HEAD af4aa4ac5 (later commits are controller records only). You are a fresh implementer for this round; the round is narrow.

Read first: task-8.7c-fix-r3.md (round 3's brief), the "Fix round 3" section of task-8.7c-report.md, task-8.7c-rereview-r2.md (this round's finding R2-1; its probes are in docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr2/), and the build, probe, perf and commit rules of task-8.7c-dispatch.md, task-6-dispatch-rules.md and global-constraints.md. Never run oracle-crashes.txt entries 64 to 67 or anything shaped like them on the oracle.

1. R2-1. `to_text`'s and `text_len_inner`'s reference arms (`reference_text`, value.rs:518-528, called from :308 and :776) go back to unsent, as at d6720f350. Renderings that must send keep going through the propagating path round 3 added in `object_string_value_text`. Every round 3 witness and the rr1 probes (i1b/g, i1d/k, i1f/n, i1g/p, i1c/l) stay as round 3 has them. Witness c2/once.rex, c2/oncesig.rex and n2c/u.rex, each matching the oracle (stdout, stderr and rc), red at cd070fd13. The re-reviewer's mutant `n` (review-probes-rr2/mutants/) is the reference shape. Leave c2/onceparse.rex (PARSE) as it is: it predates the round and is queued.
2. Perf: every program within +0.05% libc subtracted of rexx-base-97eee8a9a. Callgrind on every program. pingguard/pingsem SPREAD flags are reported, not failed; a deterministic delta over the budget is a finding.

Run the workspace test once at the end and name every failure. Commit before gating. Never go idle on a background command: poll in a bounded foreground loop. Never pgrep -f or pkill -f; no nohup, bash -c or sh -c.

Report: append a "## Fix round 4" section to task-8.7c-report.md (the change, witnesses red and green, the perf table, the workspace result, commits), commit it, then SendMessage to "main" with status, commits, one-line tests and concerns. Never dispatch subagents. Never write a numbered Deviation. Do not edit defects.json or phase-4-exclusions.txt.
