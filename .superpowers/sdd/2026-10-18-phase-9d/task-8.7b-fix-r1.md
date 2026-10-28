# Task 8.7b fix round 1

Phase 9d Task 8.7b (security manager, native API, stack frames, ::REQUIRES install, condition reports) of the ooRexx Rust rewrite. Code HEAD 5053fbd9b, records HEAD 04d155abc. You are a fresh implementer for this round.

Read first: task-8.7b-dispatch.md (requirements and rules; follow its build, probe, perf and commit rules), task-8.7b-report.md (what was done), task-8.7b-review.md (findings and probes; probe paths there are relative to t87b-review-scratch/ in this directory), task-6-dispatch-rules.md, global-constraints.md.

Fix:
1. Important 1. `primitive_level` (dispatch.rs:401) names the frame's method by looking it up by name on the receiver, so a subclass override of SORT, []=, REQUEST or class NEW forwarding to super names the override (`scope MYARR` where the oracle says `Array`). Name the method actually running. Witness: the review's f2 and rn/rn2.rex shapes, red at 5053fbd9b.
2. Important 2. A security manager set during the main program's ::REQUIRES install is asked by main's later ::REQUIRES and its commands; main's activation began before the set, so by the task's own rule it must not be (probe sm1/). Fix and witness, red at 5053fbd9b.
3. Ruled by Moritz 2026-10-28 ("Follow rexxref, Deviation"): a condition raised again while its CALL ON handler runs is delayed and redelivered as rexxref condtra.xml (CALL ON, delayed state) says: on RETURN from the handler the handler is called again immediately, or at a CALL ON for it re-issued in the handler, immediately. Implement for every CALL ON condition this applies to (not ERROR and FAILURE, per the text), witness with the report's probes/user-raised/u2.rex and u3.rex shapes, and close 2026-10-22-user-raised-in-own-handler.md. The controller writes the Deviation; you write the derived rule, the oracle's answer and the witness names into the report.
4. Minor 1. install.rs:498: the doc names one of `push_installing_level`'s two None cases; state both.
5. Minor 3. The native-method-frames closing line: say which rows of frames/f1.rex agree at base.

Not in this round: Minor 2 (recorded), the review's queue candidates, the INTERPRET PACKAGE and Deviation 0 texts (the controller's).

Perf: callgrind on every program against rexx-base-504ff0743, own delta at most +0.05% libc subtracted per program (pingguard/pingsem SPREAD reported, not failed). startup already sits at +0.047% from the hash put's level lookup: whatever this round adds must keep startup and every program within +0.05% of base; move the lookup off the hot path if needed. Run the workspace test once at the end and name every failure. Commit before gating. Never go idle on a background command: poll in a bounded foreground loop; never pgrep -f or pkill -f.

Report: append a "## Fix round 1" section to task-8.7b-report.md (what changed, witnesses red/green, perf table, workspace result, commits), commit it, then SendMessage the team lead: status, commits, one-line tests, concerns. Never dispatch subagents. Never write a numbered Deviation.
