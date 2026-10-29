# Task 8.7b fix round 3

Phase 9d Task 8.7b of the ooRexx Rust rewrite. Code HEAD d1f8b387f, records HEAD 3d23302bc and later controller commits. You are a fresh implementer for this round.

Read first: task-8.7b-dispatch.md (requirements and rules; follow its build, probe, perf and commit rules), task-8.7b-fix-r1.md (Moritz's ruling: follow rexxref condtra.xml for a condition raised again while its CALL ON trap is delayed), the "Fix round 1" and "Fix round 2" sections of task-8.7b-report.md, task-8.7b-rereview-r2.md (findings; probe paths relative to t87b-rr2-scratch/ there), task-6-dispatch-rules.md, global-constraints.md. Never run oracle-crashes.txt entries 64, 65, 66 or anything shaped like them on the oracle; never run the review's p4 shape on the oracle (it segfaults: two conditions of different names raised in one clause under one CALL ON ANY).

Fix:
1. Important 1. Restore the drop of conditions raised by the debug pause's own input read, cited to rexxref condtra.xml:489-494 (Note 4: traps are OFF while that input is processed); the review's dp2 is OOM-killed at d1f8b387f and ends rc 0 at cca0956e0. Witness dp2's shape red at d1f8b387f. Keep NOTREADY ignored while delayed.
2. Minor 1. A SIGNAL ON release sets POSITION and the traceback from the SIGNAL ON clause (condition.rs:127); per rexxref condtra.xml:450-451 and :769 they are the held condition's, as the CALL ON and RETURN releases already give. Witness p8's shape.
3. Minor 2. Controller ruling: under CALL ON ANY, once a condition is queued at a clause the ANY trap is delayed, so a NOTREADY raised later in the same clause is ignored, consistent with the rule the crate applies while a handler runs (review p4, p5). Witness p4's shape (crate only).
4. Add oracle-crashes.txt entry 67 for the review's p4 shape (rc 139, run once by the reviewer; do not run it again), in entry 66's format. Add one sentence to rust/CLAUDE.md's probe bullet on queueing CALL ON conditions: two differently named conditions in one clause are not safe when one CALL ON ANY traps both (entry 67).

Perf: callgrind on every program against rexx-base-504ff0743, own delta at most +0.05% libc subtracted per program (pingguard/pingsem SPREAD reported, not failed). Run the workspace test once at the end and name every failure. Commit before gating. Never go idle on a background command: poll in a bounded foreground loop; never pgrep -f or pkill -f.

Report: append a "## Fix round 3" section to task-8.7b-report.md (what changed, witnesses red/green, perf table, workspace result, commits), commit it, then SendMessage the team lead: status, commits, one-line tests, concerns. Never dispatch subagents. Never write a numbered Deviation. Do not edit defects.json.
