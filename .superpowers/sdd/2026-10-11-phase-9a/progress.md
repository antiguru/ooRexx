# SDD ledger — plan: docs/superpowers/plans/2026-10-11-phase-9a.md

Spec: docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md (agreed 2026-10-11).
Plan base: 180e5b41e. Moritz 2026-10-11: "Run task 1 sdd" -- Task 1 only, then report.

## Pre-flight scan

| Rows | Produces / consumes | Found |
|---|---|---|
| T1-T2 | T1 makes `skip`/`without_rxfuncquery` byte-exact, moves `mark_starts`/`started` to `group_runner.rs`; T2 consumes them | consistent |
| T1-T3 | T3's whole run uses T1's byte-exact copies | consistent |
| T2-T3 | T2's `GroupRun.expected` is filled by T3's `expected_set`; `REXX_SUITE_GROUPS` used by T3-T9 | consistent |
| T3-T4 | T3's owner check for "Phase 10, other reason" needs the D3 reasons T4 adds | **conflict**: T3 precedes T4 |
| T4-T5..T8 | each refusal task refreshes `refusal-sites.tsv` after T4's owner parameter exists | consistent if sequential |
| T5-T8 | both touch `class_protocol.rs` (T5 `:774` metaclass NEW, T8 subclass NEW) | sequential, no conflict |
| T1 self | test (byte comparison) matches code (byte functions); perf pin outside tree | consistent |
| T2 self | fixture tests specified before code; `GroupRun` named | consistent |
| T3 self | seeds files from a run whose Environment rows wait for Moritz | consistent (candidates only) |
| T4 self | enum in `rexx-core`; `EXCLUDED` holds names as strings | consistent |
| T5-T9 self | witnesses then implementation | consistent |
| T10 self | close command and suite | consistent |

Ruling: T3's "Phase 10, other reason" check accepts a captured refusal naming Phase 10 until T4 lands, and T4's last step tightens it to require a D3 reason — the plan orders T3 before T4 and the check cannot see reasons that do not exist yet — if wrong, T3 is re-run after T4 with the strict check.
Task 1: dispatched (implementer sonnet, BASE 180e5b41e)
Task 1: implementer DONE, bee47022f; pin sha256 c97bf40a…
Task 1: review approved (sonnet); ⚠️ resolved by controller: pin-build.log has a `Compiling rexx-exec` line, gate record names `git archive a98238be5`.
Task 1: minor (deferred): dispatch/library/tests.rs:177 says collect_stress "reads phase-8.txt last", stale since phase-9.txt appended.
Task 1: minor (deferred): skip compares method names through from_utf8_lossy; non-ASCII quoted names could collide (not live).
Task 1: minor (deferred): group_runner.rs:435 `contains` panics on an empty needle via windows(0) (no caller passes one).
Task 1: complete (commits 180e5b4..bee4702, review clean)
Task 2: dispatched (implementer opus, BASE bee47022f)
Task 2: Ruling: the suite's started marker is a file outside the copy, written and closed per test on both sides, not whole_groups' stderr line — rexx-run hands buffered stderr to fd 2 only at reads, idles and exit, so a KILLed run loses its markers and a timeout cannot rename the hung test (implementer measured rc 137 with no stderr; SIGTERM raises HALT, trapped by the framework) — if wrong, the harness reverts to stderr markers once output is unbuffered.
Task 2: implementer DONE, 93c691bcd (31 suite tests; Package_Options 54 blocked, C2X 34/34, samples blocked at load)
Task 2: controller ran api_group_tests (24 passed) and concurrency_tests ungated (52 passed) at 93c691bcd, which the implementer had only compiled.
Task 2: review approved (opus), no Critical/Important. Spec amended: bin beside the copy.
Task 2: minor (deferred, carried into Task 3's dispatch since Task 3 edits suite.rs): (1) early-end detection reads stderr text before exit status, so a child rexx's refusal on inherited stderr would end a normal run (suite.rs:665-682; live once 9c adds -e); (2) `timeout` without --foreground escapes kill_group, grandchildren can hold pipes (suite.rs:884-908); (3) PATH test checks the env vector, not the child's PATH (suite.rs:1216-1243); (6) blocked_at_load overrides earlier per-test blocks (suite.rs:411).
Task 2: minor (deferred): (4) run_group_on_crate uses only the floor; Task 3 must call run_group; (5) bound truncated to whole seconds; (7) CapKind::label repeats the limit constants; (8) empty REXX_SUITE_GROUPS runs nothing silently; (9) the revision test needs svn in a plain cargo test; (10) Ended's doc names run_crate_process.
Task 2: complete (commits bee4702..93c691b, review clean)
Task 3: dispatched (implementer opus, BASE 8a149e8b0; carries Task 2 minors 1,2,3,6 and the T3/T4 Phase 10 ruling)
Task 3: implementer DONE_WITH_CONCERNS, 717ab354b..f82bca323 (+ report c161c39ee). Whole run at 2415e1733: 328 s, 324/402 groups green, 78 red (expected). Enumeration matched the oracle's started set in all 402 groups.
Task 3: Ruling: the four socket samples (scclient, scserver, sfclient, sfserver) run one group at a time in the harness, not as Environment rows — they pass alone on the oracle and fail only on fixed-port collisions (EADDRINUSE) at eight groups at once, so a row would license a harness artefact — if wrong, they become Environment rows by Moritz's ruling and the serial rule goes.
Task 3: review (opus) needs fixes: Important 1 refusal row certifies itself (judge must require the captured refusal). Minors 2-6.
Task 3: Ruling: the crate-side bin gets the `*.cls` Rexx library files ooRexx/build/bin holds, so Package.TEST_PACKAGE_FINDPROGRAM_REXXTRY sees mime.cls on both sides — the files are Rexx source the distribution installs beside rexx, not oracle binaries, and the asymmetry is a harness artefact — if wrong, the test becomes an Environment candidate for Moritz.
Task 3: minor (deferred): (3) Phase 10 Rxapi owner label covers reaching_rxapi, R1 group and refusal rows; the equality covers only the first (note for the gate record); (5) row existence check accepts any class's method, use source_test_names_in; (6) owner-check fixtures written with the code, mutants as evidence.
Task 3: fix round 1 dispatched (resume implementer): items 1, socket-serial ruling, mime.cls ruling, minor 4.
Task 3: fix round 1 implementer DONE, c0e99bd1d; concern: committed baseline lacks '# groups all', so the DIFFERING cross-check needs a fresh full run (Task 10's whole run provides it). Re-review dispatched (sonnet).
Task 3: fix round 1/5 (4 addressed, 0 open; commits c161c39..c0e99bd)
Task 3: minor (deferred): a refusal row on a test that refuses at load reads as "refusal not captured" (blocked_at_load is not keyed by test); no such row exists yet.
Task 3: complete (commits 8a149e8..c0e99bd, review clean after fix round 1)
