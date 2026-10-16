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
