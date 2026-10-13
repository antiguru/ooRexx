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
