# Task 8.7a fix round 1

Phase 9d Task 8.7a (crash, builtins, collections) of the ooRexx Rust rewrite. Code HEAD 030af45a1, records HEAD 0d76046ca. You are a fresh implementer for this round.

Read first: task-8.7a-dispatch.md (requirements and rules; follow its build, probe, perf and commit rules), task-8.7a-report.md (what was done), task-8.7a-review.md (findings and probes; probe paths there are relative to t87a-review-scratch/ in this directory), task-6-dispatch-rules.md, global-constraints.md.

Fix:
1. Important 1. A traced message result that is a reference is dead before its `>M>` line under TRACE I (`w = o~r(>v)` with `::method r; use arg >x; return >x`), so the stress runner panics rc 101 (assign_evaluated -> result_text -> not_in_arena) and the `say` form ends rc 120 "no longer live". Root the result where a traced message result is handed back to the caller, before anything on the TRACE I path allocates. Witness: the review's `st2/c4o.rex` added to the item's corpus witness (trace_reference_value_rooted.rex, as the review names it) and red under collect_stress at 030af45a1; also cover the `say` form (`c4f.rex`, `c4n2.rex`). Hunt the same shape for routine results, INTERPRET, function calls in an expression, and `>>>`/`>=>` lines; fix every sibling you find red.
2. Minor 1. value.rs: `requested_number` sits under `operand_number`'s doc comment; put each doc on its function.
3. Minor 2 (witness only). Add a test pinning `'kmake'~equals(>k)` and `'a K'~equals(>k)` to this crate's answer under Deviation 72 (the referenced value's string). Do not edit phase-4-exclusions.txt; the controller handles the text with Moritz.
4. New oracle crash: add oracle-crashes.txt entry 63 for `q = >y; trace i; q~value = 3` (review "New oracle defects", `tr2/n5.rex`, `tr/a17.rex`), measured by you under the standard wrapper from a fresh empty directory, in entry 62's shape.

Not in this round: the TRANSLATE KNOWN GAP (Minor 3) and the queue candidates; the controller has them.

Perf: callgrind on every program against rexx-base-81f4bf250, own delta at most +0.05% libc subtracted per program (pingguard/pingsem SPREAD reported, not failed). Commit before gating. Never go idle on a background command: poll in a bounded loop; never pgrep -f or pkill -f.

Report: append a "## Fix round 1" section to task-8.7a-report.md (what changed, witnesses red/green, perf table, commits), commit it, then SendMessage the team lead: status, commits, one-line tests, concerns. Never dispatch subagents.
