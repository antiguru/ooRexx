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
Task 3: dispatched (implementer opus, BASE 2b59b029d); Review Focus section appended to brief.
Task 3: implementer DONE_WITH_CONCERNS (a6c147e62, 28aa77b29, ef1357a27). Groups 44/44.
Task 3 review: spec OK; quality needs fixes. I1 new oracle crash (SysDumpVariables over var whose STRING raises: SIGSEGV even under SIGNAL ON SYNTAX, 1 run). I2 report's 999999999 justification false. Minors: sort writes back by tail not by captured element; dump buffered not line-by-line; sort values rooted only via coarse reqstr latch; 40.919/40.921 claim unrun. Out of scope: class-side ::method string not sent.
Ruling: I1 -> repeat oracle run 3x, oracle-crashes entry 38, crate-only pin with reason.
Ruling: 999999999 keeps 40.924 with no check; wording: oracle's 5.0 is its eager new_array(bounds*3) under the memory cap (5.0 at 1 GiB, 40.924 at 4 GiB), so the crate matches the oracle's unpressured answer -- that IS the OOM licence, stated with the measurement -- if wrong, a stem-size check would be needed.
Ruling: dump writes line by line as the oracle does -- interleaving with a STRING method's own output is observable -- if wrong, cost is one buffered write.
Ruling: sort's value Vec gets an explicit hold, not a comment -- a reqstr latch refinement would silently unroot it; prose is not a control -- if wrong, a redundant root.
Ruling: sort write-back by element identity -> queued to 9b1 (needs element identity) -- if wrong, a mid-sort stem-emptying STRING method diverges until 9b1.
Ruling: class-side ::method string not sent -> queued (pre-existing, reqstr.rs Instance gate).
Task 3: fix round 1 dispatched (resume b3-impl).
Task 3: fix round 1 at d6c1b2d85, 0009afbc5; re-review dispatched (sonnet).
Task 3: complete (a6c147e62, 28aa77b29, ef1357a27, d6c1b2d85, 0009afbc5). Re-review approved.
Ruling: plan and gate record said 'Moritz ruling' for controller rulings (Task 2 fix rounds); corrected to 'controller ruling' -- attribution must be true.
Task 4: dispatched (implementer opus, BASE 72550bcda). Carry-in item 6 (samples rows suite-9d-set.txt:206-208) handed to Task 4.
Task 4: implementer DONE_WITH_CONCERNS (92d2868fc, 0dccdb3b7, bf532a205, 309d16dce, 1cbce6cde).
Task 4 review: spec flag (qualify_name widened into QUALIFY/DIRECTORY/.File/.Stream/stream BIFs/redirects inside the task, against "mechanical batches" constraint) -- all probe-match the oracle; quality: Dublin DST-gap divergence unlicensed, in-program TZ (VALUE ENVIRONMENT) ignored by date-time routines; minors: unneeded push_temp files.rs:663, entry-39 test fixed dir/no cleanup guard, datetime_zone lossy compare, .File absolutePath raw bytes unwitnessed. Perf: dirread +0.39%, sayloop +0.19% vs pin; stream loop -22%.
Ruling: ratify the qualify_name widening -- correct, oracle-matching, the tilde group needed it -- add witness lines from the reviewer's probes (copied to task-4-review-probes/) -- if wrong, cost is a revert of a correct change.
Ruling: date-time routines read the zone from the program's ENVIRONMENT TZ and pick the gap candidate as glibc mktime(tm_isdst=-1) does, via a pure-Rust TZif reader that exposes is_dst (a crate is fine, builds are online; no unsafe) -- matches oracle on both divergences; per-process TZ was a stance, not a licence -- if it does not fit in the round, fall back to a Deviation entry naming both and report it.
Ruling: drop push_temp at files.rs:663 (cannot be load-bearing: no allocation between text_built and slot write) -- Task 3's hold differed (values crossed allocations) -- if wrong, collect_stress catches.
Ruling: entry-39 test stays loud on a shared fs (this machine's gate); give it a unique dir and a cleanup guard. datetime_zone compares bytes. Witness .File absolutePath on 'ff'x name.
Task 4: fix round 1 dispatched (resume b4-impl).
Queued: lossy path sites (except SysFileTree's, which goes to the Task 5 dispatch).
Task 4: fix round 1 at ab305b53e, ff222442b (tz-rs 0.7.3 added); re-review dispatched (sonnet).
Task 4: complete (92d2868fc..1cbce6cde, ab305b53e, ff222442b). Re-review approved.
