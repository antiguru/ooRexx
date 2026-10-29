# Task 8.7c fix round 1

Phase 9d Task 8.7c (variables and streams residuals) of the ooRexx Rust rewrite. Code HEAD e620097f0, records HEAD 5be253e4f. You are a fresh implementer for this round.

Read first: task-8.7c-dispatch.md (requirements and rules; follow its build, probe, perf and commit rules), task-8.7c-report.md (what was done, and its "Deviation candidates" section), task-8.7c-review.md (findings and probes; probe paths there are relative to t87c-review-scratch/ in this directory), task-6-dispatch-rules.md, global-constraints.md, and Deviations 55, 59, 72 and 75 in docs/superpowers/plans/phase-4-exclusions.txt. Never run oracle-crashes.txt entries 64 to 67 or anything shaped like them on the oracle.

Review findings:
1. Important 1. value.rs:474-475 `Err(_) => None` swallows a condition raised by DEFAULTNAME/OBJECTNAME inside a rendering (Array join, trace line, error insert): the oracle raises it (41.1 trapped, rc 158, 98.900; a recursive join Error 11 rc 245), head prints the default name at rc 0. Propagate the condition; witness each review shape red at e620097f0.
2. Important 2. notready_trap_delayed_at (run/condition.rs:1101-1120) also ignores a NOTREADY caught by an internal routine's own new CALL ON NOTREADY while its caller has one queued; base ran its handler, as condtra.xml's Notes give (review n1.rex, crate only). A trap the routine set itself is not delayed by the caller's queue; witness.
3. Minor 1. A DEFAULTNAME answering an Array renders as its join in trace lines and error inserts; render it as `~objectName` does ("an Array").
4. Minor 2. hash.rs leveled_store_at's black_box has no measured effect (review: dirread -0.0223% with or without it); remove it and its false comment. Keep value.rs's black_box (the review's control shows it load-bearing, a code generation effect); its comment states only what was measured.
5. Minor 3. stream.rs:222-226: give unopened_standard its doc block back and standard_input_file its own.

Moritz's rulings (2026-10-29) on the report's candidates; implement 1, 3, 4 and 6, pin 2 and 5. The controller writes every Deviation; put the derived rule, the oracle's answer and the witness names for each in the report.
R1 "Default's value, stem named": a stem's string value is its default's request('STRING') in every required-string position; where that answers none, NOSTRING names the stem, uniformly (the concatenation receiver included). Close 2026-10-27-stem-default-arithmetic-nostring.md.
R2 "Keep rexxref, Deviation": hash lookups send the index's `==` as now; add a test pinning that the identical index's `==` is sent. Close 2026-10-28-hash-lookup-identity-first.md under the Deviation.
R3 "Enforce rexxref": on an APPEND stream a write SEEK, a CHAROUT start or a LINEOUT line within the file's extent as it was at open is refused: 0 and NOTREADY (as Deviation 59's invalid SEEK), CHAROUT answering its residual count (funct.xml:4307-4312). Close 2026-10-28-append-stream-positioned-writes.md.
R4 "Every condition": every condition the debug pause's input read raises takes its default action (SYNTAX excepted as the report's candidate says; derive it from condtra.xml:489-494 and the debug section), not only one whose trap is delayed; update the pinned debug case. Close 2026-10-29-debug-input-traps-off.md.
R5 "New Deviation, oracle bug": the first SEEK of an unopened transient stream stays 93.958; add a test pinning it. Close part 3 of 2026-10-28-stream-query-residuals.md (parts 1 and 2 as the report says).
R6 "Follow rexxref, Deviation": TRACE value lines and a Stem's STRING (its default's string representation) send STRING to an object whose class overrides it (instrc.xml:3697-3699, intro.xml:2656); witness the report's stemnostr/s2.rex and stemarr/b.rex shapes. Untraced cost stays in budget.

Also: the review's queue candidates and new oracle defect stay with the controller.

Perf: callgrind on every program against rexx-base-97eee8a9a, own delta at most +0.05% libc subtracted per program (pingguard/pingsem SPREAD reported, not failed). Run the workspace test once at the end and name every failure. Commit before gating. Never go idle on a background command: poll in a bounded foreground loop; never pgrep -f or pkill -f.

Report: append a "## Fix round 1" section to task-8.7c-report.md (what changed, witnesses red/green, per-ruling derived rule, oracle answer and witness names, perf table, workspace result, commits), commit it, then SendMessage the team lead: status, commits, one-line tests, concerns. Never dispatch subagents. Never write a numbered Deviation. Do not edit defects.json.
