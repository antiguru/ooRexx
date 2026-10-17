# SDD ledger — plan: docs/superpowers/plans/2026-10-15-phase-9c.md

Spec: docs/superpowers/specs/2026-10-10-phase-9-core-conformance.md (revised 4ab4093b2). Plan reviewed by fable (plan-review.md), fixes e7a983850, Moritz rulings 868281f71.
Moritz 2026-10-16: run Task 1 only, then stop.
Pre-dispatch check Task 1: rexxc oracle binary present; errors.rs:33 Case, :86 cases(), :436 test exist.
Task 1: dispatched (implementer opus, BASE fe4867787).
Task 1: implementer DONE (39fcc7450 code, 1db6c4c75 report). errors 30 pass; PENDING 92 pairs; derive 15.5 s, idempotent.
Task 1 review (sonnet): spec PASS, quality PASS; derive re-run byte-identical; mutations red. Minors: derive.sh truncates multi-line message silently; set -u abort on missing class header; TSV header old counts.
Task 1: fix round 1 dispatched (resume c9c-impl-1) for all three minors.
Task 1: fix round 1 at 021415b17, c567d95ce (all three minors). Controller read the diff: fail inside $(...) exits only the subshell -> controller fix 20fc86a69 (|| exit 1); derive re-run byte-identical, 15.5 s.
Task 1: complete (39fcc7450, 1db6c4c75, 021415b17, c567d95ce, 20fc86a69). Moritz: Task 1 only; stopped here.
Pre-dispatch check Task 2: build.rs:49 generate_errors, expr.rs:338 error, rexx-exec error.rs:1982 From<&ParseError>, token.rs:40-44 sentence exist; error.rs:22 is the impl line (message() inside).
Task 2: dispatched (implementer opus, BASE cdbbc1568).
Ruling (Task 2): expr.rs missing: u16 param and its None/comma_list raises stay on ParseError::new in Task 2; Task 3 Step 1b changes it to Msg<0> with callers and restructures instruction.rs:499 (35.1 with = token) -- keeps Task 2 inside its files; arity-0 subs already render right -- if wrong, 35.1 at :499 stays PENDING one task longer. Plan amended 1c7445742.
Ruling (Task 2): converted sites get their own TSV rows in the converting task (Task 2 adds them now), and PENDING is re-keyed per row (survives insertion) so a pair's converted rows are gated while unconverted rows stay pending -- per-pair PENDING left converted sites unwitnessed -- if wrong, extra rows only. Plan Task 3 Step 2 amended ba860f7de.
Task 2: implementer DONE (fdd1ca33d, 5cd02f326); PENDING 92->81 pairs; ParseError 16->24 B; corpus 1062/1062. Rulings arrived late -> resumed for ruling 2 (rows + per-row PENDING) before review. 47.1 note into plan Task 3 (2ec226bfd).
Task 2: rulings applied 06cf4a4a7 (12 TSV rows, PENDING per row &[&str], M11-M15 red), report aade11443. Label-colon Deviation rows -> plan Task 4 Step 2 (45721cabd). Review dispatched (opus), range cdbbc1568..aade11443 (plan-only commits interleaved).
Task 2 review (opus): spec compliant; quality conditional. Important 1: startup +0.7246% vs pin (OVER), Task 2 share +0.3611% (scan lost inlining into parse, need_variable out of line, eager previous()); interpret +0.149%. Minors: can't-fail unit test, inventory test samples 5, false report sentence.
Ruling: perf fix in Task 2 before Task 3 adds sites -- Task 2 own delta on startup/interpret/parse <= +0.05% -- if wrong, close task pays.
Task 2: fix round 1 dispatched (resume c9c-impl-2).
Task 2: fix round 1 a23fdba9e, dd253ad15, 0f8af4e3a. Task 2 own deltas: startup -0.2739%, interpret -0.0900%, parse -0.0199%; vs pin startup +0.0599%, interpret +0.2012%.
Ruling: ParseError(Box<ParseErrorFields>) accepted over the brief's Option<Box> field -- measured: the field itself cost +0.39% startup; boxing allocates only when an error is built -- if wrong (speculative construction on a hot path), re-review finds it. Plan amended dae4be4eb.
Task 2: scoped re-review dispatched (sonnet).
Task 2 re-review (sonnet): approved, none open; perf figures match summary.tsv; no speculative ParseError construction on valid paths.
Task 2: complete (fdd1ca33d, 5cd02f326, 06cf4a4a7, aade11443, a23fdba9e, dd253ad15, 0f8af4e3a). Moritz: stop after Task 2.
Pre-dispatch check Task 3: instruction.rs:239 error helper, :424 47.1 raise, :499 expr(...,1) 35.1 call, instruction/{loops,address,parse}.rs exist.
Task 3: dispatched (implementer opus, BASE 93de0ca29); carries Task 2 perf lesson (cold error construction; own delta startup/interpret/parse <= +0.05%).
Task 3: implementer DONE (7436ca94e code + 59 rows, b17078946 report). PENDING 299->204. Own deltas startup +0.0094%, interpret -0.0015%, parse +0.0007%. Concerns: (1) oracle's 'unmatched DO OVER loop subkey 29' / 'DO WITH conditional 0' reproduced (internal enum as insert) -> for Moritz; (2) four sites (89.1, 89.2, 20.922, 27.1) oracle passes an unused insert -> condition additional array may differ -> reviewer measures; (3) block.rs:660 -> plan Task 4 (7706d5f9f).
Task 3: review dispatched (opus).
Task 3 review (opus): spec OK; quality needs fixes. Important 1: trapped additional array empty where oracle has 1 item at 89.1, 20.922, 27.1 (unused insert). Concern 1 confirmed oracle bug (IP:1491, :1700 'should really never happen'; enum value as insert). Minors: loops.rs:376-377 wrong subject, :399-400 set size. 61 probes identical; 6 spot mutants red; derive reproduces.
Ruling: Important 1 fixed in Task 3 -- additional array is observable under trap -- if wrong, extra override table only.
Open for Moritz: DO OVER/WITH 49.2 'subkey 29'/'conditional 0' copies an oracle bug (internal enum exposed); crate keeps oracle text until ruled.
Task 3: fix round 1 dispatched (resume c9c-impl-3).
Task 3: fix round 1 6ada7047e, d5cb987d1 (ARITY_OVERRIDES for 20.922, 27.1, 89.1; witness parse_error_unused_insert.rex; corpus 1063/1063). Re-review (sonnet): approved, none open; ~45 extra shapes match.
Task 3: complete (7436ca94e, b17078946, 6ada7047e, d5cb987d1). Moritz: stop after Task 3. Open: DO OVER/WITH subkey ruling.
Moritz 2026-10-16: DO OVER/WITH subkey -> 'a keyword error is better'. Deviation 38 (stream buffering renumbered 39 in plan, d88044e92).
Task 3b (ruling follow-up): dispatched (implementer opus, BASE d88044e92).
Ruling (3b): 25.927 'Unknown keyword following "FOR"; found "TO".' -- only DO-header keyword error is 27.901 whose text names FOREVER (false here); 25.927's text is true -- if wrong, one constant changes.
Task 3b: DONE 90b1412ae (25.927, Deviation 38, do_subkey_deviation exemption pinned to its rows), da84dcc0d report. Mutation (49.2 arm back) reddens 3 tests; corpus gate green; derive reproduces. Controller read the loops.rs diff (error arm only, cold self.error); accepted without separate review (small, ruled, mutation-shown).
Task 3b: complete. Stopped per Moritz.
Pre-dispatch check Task 4: block.rs/directive.rs/lib.rs cites resolve; exclusions note 2026-08-20 has moved from :5205 -> implementer locates by text.
Task 4: dispatched (implementer opus, BASE d5ea1577b).
Ruling (Task 4): 11.1 MAX_EXPR_DEPTH gets a TSV row (50,001 parens); errors.rs parse() runs every program on a sized thread as deep.rs does, no length threshold -- keeps Unreached closed, oracle's answer in the corpus -- if wrong, one long TSV line.
Ruling (Task 4): INTERPRET line inserts fixed here (oracle numbers the string as the clause's line, LanguageParser.cpp:450; crate used line 1) -- widened to run/interpret.rs and source.rs; message line inserts only, other line observables reported not changed -- if wrong, revert one parameter.
Task 4: implementer DONE (dbbcdafb0, e1bfac290 directive ARITY_OVERRIDES, 16e89ac59 INTERPRET line, fb69dafb0 report). corpus 1065/1065; 102 site mutants red; deltas startup -0.0225%, interpret -0.5470% (libc; -libc +0.0197%), parse -0.0015%. Concerns: stale R6 prose at exclusions :2292 KNOWN GAPS row, :2485, NOT_IMPLEMENTED_EXIT bullet; rexx-exec error.rs test change; REPLY half of 2026-10-07 item unprobed.
Task 4: review dispatched (opus).
Task 4 review (opus): spec OK except I1; quality needs fixes. I1: 49.2 UNREACHED NoProgram false (two probes are Deviation 38 shapes, oracle raises 49.2). Minors M1 label-colon exemption names no Deviation; M2 false comment; M3 stale exclusions prose (+ :2463-2483); M4 refusal tables unreported; M5 duplication. Perf: interpret -0.55% is heap-state cost per malloc; read -libc +0.0197%.
Ruling: I1 (b) -- 49.2 entry targets only :453; Deviation 38 shapes stay under do_subkey_deviation -- if wrong, a variant added later.
Task 4: fix round 1 dispatched (resume c9c-impl-4).
Task 4: fix round 1 338e8f5dc, 15b29ae3a (I1 per ruling; M2-M5 done). M1: label-colon divergence ruled in Phase 3 plan :1169/:2557, no Deviation number -> Moritz's call.
Task 4: scoped re-review dispatched (sonnet).
Moritz 2026-10-17: label-colon -> Deviation 39 ('Our error message is clearer'), written by controller 395f39e00; buffering renumbered Deviation 40 in plan.
Task 4 re-review (sonnet): approved, none open (49.2 probes clean on both sides; M3 deletions false-only; coverage red with a row removed).
Task 4: complete (dbbcdafb0, e1bfac290, 16e89ac59, fb69dafb0, 338e8f5dc, 15b29ae3a; Deviation 39 at 395f39e00). Paused per Moritz.
Pre-dispatch check Task 5: invocation.rs:41, lib.rs:3465/:3703, require.rs:30/:57/:129, group_runner.rs:706, suite.rs:298/:1883 resolve; banner ruling is in global-constraints.md.
Task 5: dispatched (implementer opus, BASE 6391ead7c).
Ruling (Task 5): program-path chain widened to bytes here (program_path, ClauseSite::path, package_path, required_paths) -- a lossy boundary is the defect Review Focus 4 targets; compiler enumerates the callers -- if wrong, wider diff than briefed. Partial close of queued lossy-path-sites reported.
Ruling (Task 5): .SYSCARGS consumer widened into environment.rs (leading empty slots per ignored option; entry absent under -e), pinned vs oracle -- otherwise with_*/without_syscargs have no consumer -- if wrong, small revert.
Ruling (Task 5): TEST_PASSINGARGS7 (child rexx writes all stdout then stderr at exit; merged array out of order) red in Task 5, owned by Task 7; Task 7 hands pending stdout over before each stderr hand-over, so shared-pipe order matches the oracle (Deviation 40 loses its interleave clause) -- costs nothing without stderr writes -- if wrong, Moritz overrides the refinement. Plan 1ce4cda11.
Task 5: implementer DONE (629f2d79b, 860fff6fc) before rulings arrived: applied (B) to paths. A/B no OVER; front end +27-28k Ir/run (startup +0.0233%); head vs pin max churn +0.151%. rexx_command 29/30 (PASSINGARGS7). Stale lib test pins found "&1" (from Tasks 2-4).
Task 5: fix round 1 dispatched (resume c9c-impl-5): paths (A), lib test, -o ordering justification, partial A/B re-run.
Task 5: fix round 1 9994f5c1c (paths bytes), 4e2c99aae (A/B re-run: churn +0.1414%, front end 28-30k Ir/run; interpret base -2.20% unexplained). Lib test not fixed by implementer -> controller dc16e526e (oracle 'found "THEN"'), --lib 1143/1143.
Task 5: review dispatched (opus).
Task 5 review (opus): spec FAIL, quality needs fixes. Critical 1: mixed encodings in resolve_search/require search (non-ASCII dirs 43.x, cwd d€ panic require.rs:191, REXX_PATH rpé 3.1) -- regression vs base rexx-run. Important 1: rexx.rs canonicalize resolves symlinks (oracle reports typed path). Minors: wrong interpret explanation (codegen in SymbolTable::intern), false interleave comment, bare tuple. 41 extra shapes clean; mains match; A/B reproduces (front end 17k Ir/run -libc).
Task 5: fix round 2 dispatched (resume c9c-impl-5).
Task 5: fix round 2 1acd33e22, 271a082ef (one bytes search in require::search; symlinks typed path; minors). A/B re-run: <=42 Ir/program -libc. --lib reported 1141 vs 1143 at dc16e526e -> re-review checks.
Task 5: scoped re-review dispatched (sonnet).
Task 5 re-review (sonnet): approved, none open; 17 shapes match; lib 1142 (one justified deletion). Controller fixed the two minors (doc sentence, report count).
Task 5: complete (629f2d79b, 860fff6fc, 9994f5c1c, 4e2c99aae, dc16e526e, 1acd33e22, 271a082ef + records). Stopped per Moritz.
Moritz 2026-10-17: 'copy Python's behavior' (tty line, file/pipe block, stderr line). Refinement (stdout before stderr) withdrawn; TEST_PASSINGARGS7 licensed under Deviation 40 via suite-licences.txt. Plan c8f0420c4.
Pre-dispatch check Task 6: git grep -l rexx-run -- rust lists 29 files at c8f0420c4; task enumerates at start.
Task 6: dispatched (implementer sonnet, BASE c8f0420c4).
Task 6: implementer DONE 257a88fd8 (rexx-run deleted; grep only PINNED.md historical lines; signals thread-name order fixed; scserver/sfserver pass, scclient/sfclient stay: rxsock 98.903). Review dispatched (sonnet).
Task 6 review (sonnet): spec PASS, quality PASS; minors (two false sentences after the rename) fixed by controller (deleted, plus the stale --engine paragraph).
Task 6: complete (257a88fd8 + controller minors). Stopped per Moritz.
