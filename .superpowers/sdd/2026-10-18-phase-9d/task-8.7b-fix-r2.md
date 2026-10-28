# Task 8.7b fix round 2

Phase 9d Task 8.7b (security manager, native API, stack frames, ::REQUIRES install, condition reports) of the ooRexx Rust rewrite. Code HEAD cca0956e0, records HEAD 878d94a13 and later controller commits. You are a fresh implementer for this round.

Read first: task-8.7b-dispatch.md (requirements and rules; follow its build, probe, perf and commit rules), task-8.7b-fix-r1.md (the previous round's brief, including Moritz's ruling: follow rexxref condtra.xml for a condition raised again while its CALL ON trap is delayed), the "Fix round 1" section of task-8.7b-report.md, task-8.7b-rereview-r1.md (findings; probe paths relative to t87b-rr1-scratch/ there), task-6-dispatch-rules.md, global-constraints.md. Never run oracle-crashes.txt entries 64 and 65 or programs shaped like them on the oracle.

Fix:
1. Important 1. NOTREADY raised while its CALL ON trap is delayed is ignored (condtra.xml:278-280, iostrms.xml:665-667), not held and redelivered (probe d1: handler runs once on the oracle and base, twice at head). Change it and its assertions (the delayed_trap_cases NOTREADY half, conditions.rs:1073's row). Then re-derive the debug-pause drop of 848403a01: if it existed only for the NOTREADY loop, remove it; keep only what rexxref supports, and cite it in the report.
2. Important 2. A primitive defined under another name whose own name is hidden (probes n5, n6) gets a frame with scope .NIL, and reading its executable refuses at lib.rs:730 rc 120 where oracle and base end rc 0. Push no level, or the right one, when no primitive is found; witness red at cca0956e0.
3. Important 3. Each redelivery in a chain retains about 10 KB until the chain ends (40 MB at 2000, 218 MB at 20000; an endless self-raising handler is OOM-killed at 2G). Find what each redelivery keeps and stop keeping it; a chain must stay flat in memory, as an ordinary delivery does (37 MB). Witness: peak RSS at two chain lengths, recorded, and a test if one can be made bounded.
4. Minor 1. SIGNAL ON redelivery drops ADDITIONAL; keep it.
5. Minor 3. oracle-crashes.txt entry 64's title is wider than its program; narrow it to the program.
6. Witnesses for two controller Deviations (tests only, Moritz ruled both): (a) an INTERPRET translation failure's condition object PACKAGE is the running package (`.context~package == condition('o')~package` answers 1; the oracle answers 0; report probes/compile-condition/m2.rex); (b) `~traceline` and `condition('o')~traceback` read inside INTERPRET after `do 2; nop; end` carry this crate's indent from block structure, where the oracle is two columns short (report probes/interpret-indent/i1-i3). Pin this crate's answers; name the tests in the report.

The controller fixes the defects.json category Minor. Not in this round: queue candidates.

Perf: callgrind on every program against rexx-base-504ff0743, own delta at most +0.05% libc subtracted per program (pingguard/pingsem SPREAD reported, not failed). Run the workspace test once at the end and name every failure. Commit before gating. Never go idle on a background command: poll in a bounded foreground loop; never pgrep -f or pkill -f.

Report: append a "## Fix round 2" section to task-8.7b-report.md (what changed, witnesses red/green, memory figures, perf table, workspace result, commits), commit it, then SendMessage the team lead: status, commits, one-line tests, concerns. Never dispatch subagents. Never write a numbered Deviation.
