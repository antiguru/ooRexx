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
Moritz 2026-10-12: "Continue with phase 9" -- run the rest of 9a continuously.
Note 2026-10-12: /tmp wiped by a reboot; the perf pin /tmp/claude-1000/p9/pin/rexx-run is gone. Ruling: rebuild it from a98238be5 at the Phase 9 close (the plan's own if-wrong path) and record the new sha256 beside the old -- 9a measures nothing against it -- if the rebuild's sha differs, the gate record names both.
Note 2026-10-12: HEAD 052550e85 merges upstream/master (C++ tree only; nothing under rust/, docs/, .superpowers/). Oracle /home/moritz/dev/repos/ooRexx/build/bin/rexx unchanged (separate checkout).
Task 4: Ruling: a Phase 10 refusal's text carries its reason after the phase name, "(Phase 10 <Variant>" with the Deferral variant name, and suite_rows::refusal_carries requires that exact reason -- spec section 2 says the captured text "names Phase 10 with a D3 reason", which the check cannot see unless the text carries it; Review Focus 5's "only phase names change" reads as "only the phase label changes", the reason being part of the label -- if wrong, the reason leaves the text and the check reads the reason from refusal-sites.tsv instead.
Task 4: dispatched (implementer opus, BASE 052550e85)
Task 4: implementer DONE_WITH_CONCERNS, 598e9f22c. Concerns: (1) Task 3's rust/corpus/phase-9d-set.txt matches the phase-*.txt glob and fails coverage::the_union_reads_every_phase_subset_file and the lib test every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops; (2) handle_set is Phase 9 but needs an unsafe from_raw_fd grant; (3) classic routine is Phase 9 but the oracle reads the queue name from .STDQUE (Rxapi); (4) REFUSING_MEMBERS holds typed Owner, not a string column; (5) extra files derived from refusal text changed; (6) phase-4-exclusions.txt:5284 (R4) has no reason.
Task 4: Ruling: the 9d set file is renamed rust/corpus/suite-9d-set.txt -- the plan's name collides with the corpus subset glob phase-*.txt, and the suite-* family is where the suite's other row files live -- if wrong, the glob gains an exclusion instead.
Task 4: Ruling: concern 3 stands as labelled -- the classic routine's own refusal is Phase 9's; when implemented it reaches .STDQUE, whose refusal is Rxapi, so the test is then blocked by an Rxapi refusal, which is the observable D1 judges -- if wrong, the site is relabelled Rxapi.
Task 4: concern 2 goes to Moritz (unsafe grant is his); it blocks only handle_set's implementation, not Task 4.
Task 4: review dispatched (opus)
Task 4: review (opus): spec ✅, quality needs fixes. Important 1: owned_message and the owed-entry chain take any &'static str, and Owner::phase() returns bare "Phase 10", so a reasonless label bypasses phase_ten_reasons; module doc phase_ten_reasons.rs:15-16 false. Minors: R4 Embedding relabel unpinned; blanking unchecked against production text; REHOME row for a Phase 10 constructor unwritable (OPEN_PHASES "Phase 10" vs label "Phase 10 Rxapi"); duplicate of a rexx-core unit test.
Task 4: fix round 1 dispatched (resume implementer): Important 1 (owned_message takes Option<Owner>), the 9d-set rename ruling, phase-4-exclusions.txt:5284 reason, minors R4 pin, REHOME, duplicate test.
Task 4: fix round 1 implementer DONE_WITH_CONCERNS, 77c6d6569; concern (minor, deferred): no test pins the owners of the two ffi.rs texts (asserts inside extern "C", panic aborts). Re-review dispatched (sonnet).
Task 4: fix round 1/5 (all addressed, 0 open; commits 598e9f2..77c6d65); re-review (sonnet) clean.
Task 4: minor (deferred): the owners of the two ffi.rs texts are unpinned (asserts inside extern "C"); the structural scan covers a bare "Phase 10" only.
Task 4: complete (commits 052550e..77c6d65, review clean after fix round 1)
Task 5: dispatched (implementer opus, BASE 6a8c461fd)
Task 5: implementer NEEDS_CONTEXT + DONE_WITH_CONCERNS, cc7ae4350 (stalled twice waiting on its own background checks; nudged; the dispatch had omitted the foreground-checks line). Message red->green; Object/RexxContext/StackFrame red only on 9d-set tests; samples/samples loads, 29 tests fail (locateSamplePrg looks beside .RexxInfo~executable). method_bodies/introspection_arity red on RexxInfo executable/libraryPath rows.
Task 5: Ruling: the RexxInfo executable/libraryPath table rows get a per-row probe override printing a binary-independent property of the answer (its class and that the named file exists), not the path -- the brief already says the witness prints "a property that holds on both sides" -- if wrong, the rows become a Deviation-licensed diverge.
Task 5: Ruling: the harness hard-links (copy as fallback) rexx-run into bin-rexx instead of symlinking, and gives bin-rexx a sibling `samples` symlink to /home/moritz/dev/repos/ooRexx/build/samples, mirroring the oracle's install layout -- .RexxInfo~executable answers the resolved binary path and locateSamplePrg looks at ../samples beside it; same reasoning as the mime.cls ruling (harness artefact) -- if wrong, samples/samples' tests become Environment candidates for Moritz.
Task 5: Ruling: the five non-mechanical items (OBJECTNAME= without a pool, metaclass NEW, Routine~copy, Package~copy, native StackFrame~executable) leave this batch per the mechanical-batch rule and go to a new Task 8b after Task 8 (metaclass NEW sits with Task 8's subclass NEW) -- they are section 3 refusals and 9a's exit needs their witnesses -- if wrong, they move to 9d.
Task 5: minor (deferred): crate 19.923 substitutes "&1" where the oracle shows "=" (D6 inserts, 9d).
Task 5: fix round 0b (resume implementer): the two rulings above, concern 6 (stale whole_groups rows in concurrency_tests.rs).
Task 5: rulings applied d2f89f559 (samples/samples 27/29, TEST_FACTOR blocked by unbuilt_new, TEST_MAKESTRING by SysTempFileName); concerns: introspection_arity/method_bodies hardcode rust/target/release/rexx-run; new REXX_WHOLE_GROUPS_ONLY knob. Review dispatched (opus).
Task 5: review (opus): spec PASS, quality needs fixes. I1 Supplier~new converts then array_argument (wrong message for MAKEARRAY answering .nil); I2 failure paths of startWith/run/Message~new/callWith/Supplier~new unwitnessed (reviewer probe callers.rex is the witness once I1 fixed); I3 refusal-dispositions.tsv:66 and refusal-sites.tsv:323 say Method copy refuses. Fix round 1 dispatched (resume implementer).
Task 5: minor (deferred): Method~copy side tables never pruned (do forever; c = m~copy grows); startWith error report header 'Error 93:' vs oracle 'Error 93 running FILE line 18:' (predates).
Moritz 2026-10-12, handle_set: option 3 now (HANDLE:0/1/2 through std's stdin/stdout/stderr, no unsafe; other numbers stay a loud refusal), option 1 (per-site from_raw_fd grant) only if a test needs another number. Owner of the implementation: whichever later 9a/9c task takes stream refusals.
handle_set grep: no ooTest file opens a HANDLE: stream (grep -rniI "'handle:\|\"handle:" ootest finds none; the only 'handle:' hit is WinUtils.cls:182, a say). Option 1 not needed for the suite.
Task 5: fix round 1 implementer DONE cd171c821; RexxInfo, SourceFile now green; CHAROUT/Stream/ADDRESS/VarRef red only on baseline-failing tests. Notes: VarRef TEST_COPY blocked by VariableReference~copy (not a Task 5 receiver; goes with Task 8b); TEST_COMMAND_CD refusal relabelled Phase 9 by Task 4. Re-review dispatched (sonnet).
Task 5: fix round 1/5 (all addressed, 0 open; commits d2f89f5..cd171c8); re-review (sonnet) clean.
Task 5: Ruling: VariableReference~copy (VarRef TEST_COPY) joins Task 8b with the other non-mechanical items -- it is a native_method refusal no batch owned -- if wrong, 9d.
Task 5: complete (commits 6a8c461..cd171c8, review clean after fix round 1)
Task 6: dispatched (implementer opus, BASE c515bac3a); groups base/class/Message, base/directives/METHOD, API/oo/METHOD, base/class/Package
Task 6: Ruling: Deviation 8 is withdrawn in Task 6's commit, in Deviation 5's form ('WITHDRAWN 2026-10-12 BY PHASE 9a TASK 6', reason: package_writes, its only program, now agrees byte for byte since .methods/.routines/.resources are store-backed); HASH_ORDERED_STDOUT, stdout_mode and its control leave corpus.rs; support::oracle's stdout_multiset machinery and its own controls stay; no suite row cites Deviation 8 (grep of suite-licences.txt and phase-9-exclusions.txt) -- a licence with no subject is a licence nobody checks -- if wrong, Deviation 8 is restored with an empty list and its control relaxed. The false 'sorted walk' sentences in corpus/lang/package_tables.rex and its sourceline_oracle copy are corrected in both files together.
Task 6: implementer NEEDS_CONTEXT, cecec0830 (crossed with the Deviation 8 ruling). Message, RESOURCE, ROUTINE green; METHOD TESTPACKAGE, RESOURCE TEST_RESOURCE_DEFINED, ROUTINE TESTDOTROUTINES/TESTPACKAGE unblocked.
Task 6: Ruling: the remaining not_this_task stores (Package~local, ~annotations tables, security manager info directory) join Task 8b -- same store change in three more places, outside the batch rule -- if wrong, 9d.
Task 6: minor (deferred): RAISE ... RESULT reports 25.907 found "&1" where the oracle prints "RESULT" (D6 inserts, same family as 19.923, 9d). Condition-object build cost unmeasured (Phase 9 close callgrind).
Task 6: implementer DONE_WITH_CONCERNS, cecec0830 + 6dc1c956c (Deviation 8 withdrawn, sourceline files). Review dispatched (opus).
Task 6: review (opus): spec PASS, quality needs fixes. I1: .ROUTINES writable (put/remove) but CALL does not resolve through it; oracle "after put: bar" then 43.1 rc 213, crate "after put: foo", rc 0 (a refusal turned into a wrong answer). Minors: stale Deviation 8 wording in support/oracle.rs; package_tables.rex "reproduces with .methods alone" sentence likely false; fresh object in build_package_string_table unrooted across attach_annotations (predates).
Task 6: Ruling: writes to .ROUTINES (put, remove and any other mutator) stay a loud Phase 9 refusal, and CALL resolving through the table joins Task 8b -- a refusal is loud, a wrong answer is not; resolution is a new identity rule outside the batch -- if wrong, Task 8b's work lands sooner. .METHODS and .RESOURCES writes: probed on the oracle; if a write there changes any behaviour beyond the table's own contents, the same refusal applies; otherwise writes stay allowed.
Task 6: fix round 1 dispatched (resume implementer).
Task 6: fix round 1 implementer DONE_WITH_CONCERNS, 9e8da28de. .ROUTINES writes refuse (Phase 9); .METHODS/.RESOURCES writes stay allowed (oracle: only readers of the same table see them, and the crate answers them identically). Rooting minor: no change, native_instance roots at environment.rs:1212.
Task 6: Ruling: the implementer's reading stands -- Package~resource and siblings read the same table, and the crate answers them as the oracle does, so no observable beyond the table changes -- if wrong, one line refuses those writes too.
Task 6: minor (deferred): call .routines~put(...) prints 35.1 at "&1" where the oracle prints "~" (D6 inserts, 9d).
Task 6: re-review dispatched (sonnet).
Task 6: fix round 1/5 (all addressed, 0 open; commits 6dc1c95..9e8da28); re-review (sonnet) clean, ~40 crate probes, no mutator gap.
Task 6: minor (deferred, perf-relevant): refuse_routines_write scans package_tables on every StringTable/Directory put/remove/empty (hash.rs ~1245, 1794, 1619); a hot-path cost the Phase 9 close callgrind must look at first; fix shape is a per-object flag, not a scan.
Task 6: complete (commits c515bac..9e8da28, review clean after fix round 1)
Task 7: dispatched (implementer opus, BASE e0f3ee495)
Task 7: implementer DONE_WITH_CONCERNS, 40278c99e. Package_Options 0/54 -> 54/54, green. Oracle crash entry 36 (.object~package~options('T','r') rc 139). Checks overran 5 minutes.
Task 7: minor (deferred, 9d): external CALL under package TRACE R prints no >I>/<I< lines; Routine~new routine's >I> names caller's package; parse error from Routine~new shows "&1" in CONDITION('O')~message (D6).
Task 7: review dispatched (opus); focus: Settings::from_parts unchecked constructor, per-Interp override state vs C++ process statics, PROLOG/NOPROLOG change, hot path.
Task 7: review (opus): spec COMPLIANT, quality fixes. Important 1 (prose): install.rs:263 and lib.rs:1655 false after arm_option_latches writes lostdigits_armed. Minors: introspection-arguments.tsv stale loud list; from_parts doc; three unwitnessed override cases; 35.1 "&1" also in main program (9d, D6).
Task 7: fix round 1 dispatched (resume implementer): Important 1, minors 1-3.
Task 7: fix round 1 implementer DONE 1ff284fd9; re-review dispatched (sonnet).
Task 7: fix round 1/5 (all addressed, 0 open; commits 40278c9..1ff284f); re-review (sonnet) clean.
Task 7: complete (commits e0f3ee4..1ff284f, review clean after fix round 1)
Task 8: dispatched (implementer opus, BASE 023565ff1)
Task 8: stage 1 (String, Stem) committed 5044c382b. Implementer NEEDS_CONTEXT on Class~enhanced running NEW (oracle ClassClass.cpp:1440-1480: dummy "Enhanced Subclass", methods in its instance behaviour, NEW to the dummy, class reset); options A (hook in new_instance), B (oracle's shape), C (cached dummy).
Task 8: Ruling: Class~enhanced moves to Task 8b beside metaclass NEW, ruled by observables not mechanism: oracle-identical answers including an enhancing INIT reached by NEW and a primitive receiver (.array~enhanced(d, 3) answers 0 Array); existing enhanced-object witnesses (setMethod, unsetMethod, instanceMethods, objectName) unchanged; no check added to every allocation unless measured; memory flat across a loop of enhanced calls -- both items need NEW to build into a class other than the one sent to, and one mechanism should serve both -- if wrong, enhanced lands in Task 8 later with option A.
Moritz 2026-10-12: 'You'll need to spawn an agent to figure out the right semantics for enhanced. I don't know.' Dispatched a read-only semantics scout (opus) for enhanced and metaclass NEW; output feeds Task 8b.
Enhanced scout: report .superpowers/sdd/2026-10-11-phase-9a/enhanced-semantics.md. Rules: real "Enhanced Subclass" via subclass() (metaclass NEW and class INIT run for it); NEW sent to the dummy; enhancing methods .nil-scoped in the dummy's behaviour; SUPER in a .nil-scoped method is the owning class (crate answers .NIL, also breaking setMethod methods); metaclass instance INIT runs twice per class. Mechanism: NEW a real send at both levels (class_factory sends NEW to the metaclass, reparents, INIT again); enhanced = class_factory("Enhanced Subclass") + .nil-scoped methods + NEW to dummy + class field reset. Oracle not flat either (~0.47 kB/call).
Ruling: Task 8b's "memory flat across a loop of enhanced calls" criterion is replaced by "no worse per call than the oracle with Method-object sources" -- the oracle grows too, and string-source growth is a separate compile-retention defect (queued 2026-10-12-method-source-compile-retention.md, with 2026-10-12-gc-trigger-blind-to-class-weight.md) -- if wrong, 8b waits for those two. The SUPER-in-.nil-scope fix joins Task 8b (send-path compare, measured).
