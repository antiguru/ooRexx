# SDD ledger — plan: docs/superpowers/plans/2026-10-13-phase-9b.md

Spec: docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md section 4. Plan base: f9105f68f. Moritz 2026-10-13: "Lgtm" -- run 9b with SDD.

## Pre-flight scan

| Rows | Produces / consumes | Found |
|---|---|---|
| T1-T3..T9 | T1's whole-process callgrind column and the durable pin; later tasks measure against them | consistent (T1 first) |
| T2-T3..T9 | T2's suite-9b-groups.txt names each task's harness groups; T7/T8 cite "the groups Task 2's file names" | consistent |
| T2-T10 | T10's criterion "every Task 2 group passes" reads T2's file | consistent |
| T3-T4-T6-T7-T8 | all edit builtin/rexxutil.rs (804 lines); each may split a family file out | sequential, no conflict; first split defines the module pattern |
| T7-T8 | T7 SysFork/SysWait concurrency; T8 semaphores; both touch scheduler-visible blocking | sequential |
| T4-T5 | both touch file helpers (path_argument, matches) | sequential |
| T9 self | handle_set option 3 from the 9a ledger ruling | consistent |
| T1 self | N1 measured against the pin T1 itself builds (Step 2 before Step 3) | consistent |
| T2 self | derivation test re-runs grep over ootest (read-only) | consistent |
| T3-T9 self | witness first, then implement; groups re-derived | consistent |
| T10 self | close command and suite | consistent |

No conflicts found.
Task 1: dispatched (implementer opus, BASE f9105f68f)
Task 1: implementer DONE_WITH_CONCERNS at 0368229d1 (5 concerns in report). Review package review-f9105f68f..0368229d1.diff. Reviewer dispatched (opus).
Task 1 review: spec OK; quality I1 (m2 .local Owed refusal regresses native FindClass('STDQUE'): oracle 91.999 rc 165, HEAD rc 120). M1 class_named drops raised condition (queued). M2 /tmp binary paths in cg-extcall2 (accepted, sha256 recorded).
Ruling: .local step of native FindClass answers None for an owed entry -- only .STDQUE is owed after bootstrap and it is a queue, FindClass keeps only classes, so the refusal guards nothing; brief wording "as package_find_class does" was the cause -- if wrong, a future owed .local class would answer NULL silently (package-local Owed arm stays loud).
Ruling: carry-in item 6 (samples rows in suite-9d-set.txt) owned by Task 4 (plan line 179, samples/samples through harness) -- if wrong, close gate catches stale rows.
Task 1: fix round 1 dispatched (resume b1-impl).
Task 1: fix round 1 at 307771ede; scoped re-review dispatched (sonnet).
Task 1: complete (0368229d1, 307771ede). Re-review approved.
Task 2: dispatched (implementer sonnet, BASE 61280aaeb)
Task 2: implementer DONE_WITH_CONCERNS (a0abcc77c, ed0923fcc). Baseline 16/18 groups red.
Ruling: "Phase 10's group" = every expected test has an Owner::Phase10 row (only Macrospace drops) -- partial-Phase10 groups still judge their other tests -- if wrong, the gate counts a group 10 owns, visible at close.
Ruling: incidental-hit groups (File, ADDRESS, SecurityManager, tilde) stay -- spec derivation is mechanical, hand-pruning defeats the re-derivation test -- if wrong, close gate judges extra groups, cost is work not wrongness.
Task 2 review: spec OK; quality I1 SecurityManager's 5 red tests have no 9b owner (they are suite-9d-set.txt:200-204, 9d), so gate record "remaining red groups are the 9b work" is false and Task 10 "every Task 2 group passes" unmeetable. Minors: header "path order" is byte order; "29 s"/"exit 101" not in committed output; header revision unchecked; FILEEXISTS_NIL_INVALID double-owned (Task 4 and 9d set).
Ruling: 9b close criterion = every test in a Task 2 group passes OR is listed in suite-9d-set.txt -- the 9d set is the carried list by construction; 9b judges only what 9b owns -- if wrong, a 9b-owned failure hides behind the 9d list (the reviewer found only one overlap, ruled next).
Ruling: SysFileXXX TEST_FILEEXISTS_NIL_INVALID is Task 4's; Task 4 removes its suite-9d-set.txt line when green -- the file routines are 9b's -- if wrong, 9d re-does a test 9b already fixed.
Task 2: fix round 1 dispatched (resume b2-impl).
Task 2: fix round 1 at 82c8bfabc; re-review dispatched (sonnet).
Task 2 re-review 1: owner table wrong for SecurityManager TEST_CALL_PASS_THREE_ARGS (SysSearchPath, Task 4, not Task 7); 29 s figure uncited. Fix round 2 dispatched (resume b2-impl).
Task 2: fix round 2 at 1138fbfa6 (docs only, 2 open findings). Ruling: controller read the diff instead of a third re-review -- both findings are a table row and a deleted figure, verifiable by reading -- if wrong, close review catches a doc error.
Task 2: complete (a0abcc77c, ed0923fcc, 82c8bfabc, 1138fbfa6).
