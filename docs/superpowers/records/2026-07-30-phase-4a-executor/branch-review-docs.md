# Branch review: documents slice

STATUS: DONE

Scope: plan 2026-07-30-phase-4a-executor.md, spec 2026-07-30-phase-4a-executor-design.md,
phase-4-exclusions.txt, phase-4a-gate.md, rust/corpus/README.md, commit messages 9f68662a..HEAD.

Method: every number re-derived from the artifact it claims to describe (awk over TRACE.testGroup, oracle probes for the CONCATENATION split and DO OVER order, fresh runs of the corpus/assertions/collect-stress/workspace suites), every citation checked by containment (grep for the named function, confirm the line falls inside it), never by reading the cited line alone.

## Verdict

The documents are in better shape than this project's history predicts.
Every load-bearing number I could re-derive reproduced exactly, every C++ and Rust citation I checked is contained where it claims to be, and the stack-cost lineage reads as correct-for-its-commit with method and arithmetic attached.
Two findings matter for 4b/4c planning: a spec Risks requirement (`TRACE ?`) that never reached any task body and now covers a real, measured, unrecorded stderr divergence (F1), and a numeric correction that reached the plan but not the governing spec, leaving the spec carrying both a superseded figure and a wrong mechanism (F2).

---

## Findings, by consequence

### F1 — HIGH. `TRACE ?`: a spec requirement dropped between spec and plan, now a real, measured, unrecorded divergence

The spec's Risks section (2026-07-30-phase-4a-executor-design.md:525) says:
"the plan measures what the oracle does with `?` and no tty, and 4a either reproduces that or fails loudly. Silently ignoring `?` is the exact shape the failing-loudly rule exists to prevent."

*The plan never carried this into any task.* No task body mentions `TRACE ?` or interactive debug; Task 13's steps do not; the task-13 report and review do not; the gate document does not; the exclusions file does not.
This is exactly the slice-3 failure mode the project has hit five times: a requirement stated only where task-brief extraction never looks (a spec Risks bullet).

*The implementation silently skips `?`.* `trace.rs::mode_from_setting` (rust/crates/rexx-exec/src/trace.rs:130) skips any leading `?`s, "a debug-pause toggle this non-interactive runtime has nothing to toggle, so simply skipped rather than tracked". That is a decision the doc comment justifies from the C++ *setting parser* (`TraceSetting::parseTraceSetting`), which only classifies the string; the pause behaviour lives elsewhere and was never measured.

*Measured today, and they diverge.* `trace ?r; x = 1; say x` with stdin from /dev/null:

* Oracle: emits `+++ "LINUX COMMAND /abs/path.rex"` and `+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++` on stderr, then continues (EOF answers each pause), rc 0.
* Ours (rust/target/debug/rexx-run): plain `trace r` output, no banner, no `+++` line, rc 0.

Same stdout, same exit code, divergent stderr — a corpus program containing `trace ?r` would fail criterion 1, and nothing records this: not KNOWN GAPS, not DEVIATIONS, no loud failure, no witness.
Per the exclusions file's own asymmetric rule, adding a KNOWN GAPS row needs no permission; someone should add it (or decide it is a deviation) before 4b/4c planning treats the trace surface as closed except for the Controlled pair.
Note criterion 3's honest framing ("shapes near [the five witnesses] have kept turning up divergences") predicted precisely this; it is now the second open trace gap, not the first.

### F2 — HIGH. The CONCATENATION 56/332 correction never reached the spec, and left a false remnant in the plan

The correct split is **106 silent / 282 loud**, re-derived here, not re-read: I ran all 388 CONCATENATION assertSame rows under the oracle with `a`..`g` unset and byte-compared each result to its expected value — 106 match, 282 do not.
Composition, also measured: all 56 rows containing `==`/`\==` are silent, *plus 50 non-strict rows* whose expected value happens to equal the all-distinct pattern (e.g. line 66's `(a=e)...` expecting `0 0 0 0 1 0 0`).

* **The spec still carries the superseded claim with a wrong mechanism.** 2026-07-30-phase-4a-executor-design.md:459: "Measured, that is the strict `==` and `\==` rows, 56 of the 388. The other 332 use non-strict `=` and depend on real blank-padding equalities ... so they fail visibly instead."
  Both the figure and the mechanism ("the silent set = the strict rows") are wrong.
  Commit 4f282fdc ("Correct the CONCATENATION silent/loud split: 106 and 282, not 56 and 332") touched *only the plan* — one line, one file.
  The spec is the document 4b/4c are told governs, and the assertion-table work continues into those phases.
* **The plan's own corrected paragraph contradicts itself one sentence later.** 2026-07-30-phase-4a-executor.md:989 now says 106/282; line 991 still says "a reader who checks it, finds 332 loud failures, and concludes the hazard was imagined".
  A checker finds 282, not 332.
  The sentence exists to armour the paragraph against exactly that checker, and it now hands the checker a second wrong number.

### F3 — MEDIUM. rust/corpus/README.md's catalogue omits six programs, four of them phase-4a subset members

The tables under "Current programs" / "Phase 4a additions" list 32 of the 38 `lang/*.rex` files.
Absent: `no_trailing_newline.rex` (subset member), `mutation_controlled_order.rex`, `mutation_digits_at_render.rex`, `mutation_form_at_render.rex` (subset members, added at gate time), `gate_variants.rex`, `keyword_as_variable.rex`.
The count-rot the spec warned about (Risks, "hard-codes 24 programs") was fixed — the README now says derive N, never assert it — but the same document has since acquired catalogue rot: its per-program purpose table, which is what the "Covers" column is for, no longer covers the corpus.
The three mutation witnesses' purposes (each tied to a named mutation, transcript captured from the oracle, verified to fail under the mutation) currently live only in the gate document and the mutation script — a reader editing one of them from the README alone has no warning of what it pins.

### F4 — LOW-MEDIUM. phase-4-exclusions.txt's framing header describes a file two sections ago

Line 4: "Two sections, and the distinction between them is the whole point of the file."
The file now has five (EXCLUSIONS whole, EXCLUSIONS partial, DEVIATIONS, EXPRKIND OWNERSHIP, KNOWN GAPS), and line 12's "the gate asserts the SET of both sections" under-describes what is pinned: EXPRKIND OWNERSHIP is set-asserted too (its own text says so, and coverage.rs/loud.rs do), while KNOWN GAPS is deliberately not.
Each later section states its own rule locally, so the harm is bounded, but the header is the first thing a 4b editor reads and it is wrong about the file's shape.

Filing check (priority 5): nothing is filed under a clearly wrong heading.
One borderline row, flagged for a call rather than asserted as wrong: QUEUED's partial-exclusion row says cross-process sharing "will never match" and self-describes as "a caveat on a delivered builtin rather than undelivered work" — a permanent difference is the definition of a DEVIATION, and parking it under EXCLUSIONS keeps it out of the deviations review cycle.
The row is self-aware about the tension; whether it moves is a plan-amendment call, not a reviewer's.

### F5 — LOW. mutate-4a.sh's mutation 4 label does not describe its own edit

The label says "Controlled::order evaluated in fixed To/By/For order" (the spec's criterion 6 wording); the applied edit is `for entry in ctrl.order.iter().rev()` — *reversed written order*, a different mutation.
The gate document describes the reversal accurately ("evaluated in reverse rather than written order"), so the durable record is right; the script's own label is the wrong claim, and it is the one a future editor of the script reads first.

### F6 — INFORMATIONAL. The gate's "3,863 total collections" has no reproducing instrument

collect_stress.rs sums `stress.collections` and asserts per-program > 0 and total > 0, but never reports the total, so 3,863 cannot be re-derived from any committed instrument.
The criterion's anti-vacuity devices are real and re-ran green here (the per-program assertion is in the test, and the suite passes on the current tree); only the decorative aggregate is unreproducible.
Low consequence; nothing builds on the specific figure.

### F7 — RESOLVED HERE. The gate's parked Controlled-loop worry is parity, not a defect

phase-4a-gate.md's "What went wrong" parks: `do i = 1e10 to 1e10 by 1` "appears to loop forever ... Not reproduced against the oracle, not filed, not fixed."
Measured today with a 5 s timeout on both sides: **both interpreters loop forever** (oracle converts the timeout SIGTERM to HALT, 4.1; ours is killed).
The digits-limited `1E+10 + 1 = 1E+10` behaviour is Rexx semantics, the same class as the documented `BY 0` forever-loop, so this is parity of a forever-loop, not a rewrite defect.
The gate bullet can be closed with this measurement; until someone does, it reads as an open lead.

---

## Numbers re-derived (all reproduce)

| figure | claimed in | re-derived |
|---|---|---|
| 342 trace lines, 34 blocks, 128 `*-*`, 214 value/marker | spec D17 | awk over TRACE.testGroup, anchored lowercase `::resource`, end at `::end`: 34/342/128/214 exact |
| 437 lines / 40 blocks case-insensitive; 374 after dropping 3 uppercase `.rex` blocks; 32 uppercase expected lines | spec D17 recount annotation | 342+32+63=437; 342+32=374; 32 exact; the annotation's four-scans-four-answers claim holds |
| 81 builtins, `builtinTable[]` at BuiltinFunctions.cpp:3042 | exclusions, spec, coverage.rs | 81 `&builtin_function_` entries; table opens at :3042 |
| 15 whole + 3 partial = 18 excluded; 66 = 81 − 15 | exclusions, gate, coverage.rs | file lists 11+4 names; coverage.rs asserts 18/81/66 identically |
| 29 subset programs; 29 of 29 matching (STRICT) | gate criterion 1 | phase-4a.txt has 29 non-comment lines; `REXX_CORPUS_GATE=1 cargo test --test corpus` re-run: 29 of 29 |
| 4,259 rows / 4,224 matching / 35 blocked; all 35 unblocked by Phase 5 | gate criterion 2 | `cargo test --test assertions` re-run: 4224 of 4259, 35 RUNTIME-BLOCKED; grep: 35 of 35 EXEMPT rows say "Phase 5" |
| 4,269 assertSame in base/expressions; CONCATENATION 388; PRECEDENCE 1,226 | spec | grep -c per file: 4,269 / 388 / 1,226 |
| CONCATENATION silent/loud 106/282 | plan Task 15 | oracle run over all 388 rows with unset vars: 106/282; also 56/0 among strict rows (see F2) |
| enum variants 40/15/6/3/6/4/32 (InstructionKind/ExprKind/LoopKind/PrefixOp/EndStyle/Trace/Operator) | gate criterion 1 | counted in ast.rs/token.rs: all exact |
| splits 20/9/4/6/1 (InstructionKind) and 9/6 (ExprKind); 26 loud witnesses | gate criteria 1 and 5 | EXPECTED_OUT_OF_SCOPE rows: 1×P7 + 9×4b + 4×4c + 6×P5 = 20; 6 ExprKind; `variant_counts_match_the_audited_split` asserts the same |
| 19 trace prefixes at RexxActivation.hpp:90-110 | spec, gate, trace_oracle.rs | enum runs :92 (CLAUSE) to :110 (INVOCATION_EXIT), 19 values 0-18; committed table = 9 spec prefixes + `>E>` bonus + 9 named as 4b-or-later |
| eval stack-cost lineage 820, 850, 783/784, 1600, 1840 | plan Task 11, lib.rs:195-250, eval.rs | lib.rs carries all five values, each framed correct-for-its-commit, latest (1840) with verbatim method output and the ≈291,777 arithmetic; plan stops at 1600 but instructs "re-measure, do not quote a predecessor's number", which is the right guard |
| 824 passed / 0 failed / 4 ignored, whole workspace | gate | `cargo test --offline --workspace --no-fail-fast` re-run and summed: 824/0/4 |
| 331 parens vs 341 calls (KNOWN GAPS) vs plan Task 3d's 349 | exclusions vs plan | not a contradiction: 349/350 was pre-Task-3d-counter, 341/342 post; depth_probe.rs:53-55 records both with the transition |
| DO OVER order "1 B 3 2 ZZ 10" | spec D15a, exclusions Deviation 1 | oracle re-run with tails 1,2,3,10,ZZ,B: byte-identical |
| RexxLocalVariables 602 lines; CompoundVariableTable.cpp 545 lines | spec D16, D15a | wc -l: 602 and 545 exact |
| line 71 of CONCATENATION expects `0 0 1 1 0 1 0` | plan Task 15 | read: line 71 is the `(a=c)` row with exactly that expectation |

Not re-derived (stated, not verified): the gate's per-mutation catch rates (25/29 etc.) — re-deriving means running mutate-4a.sh, which edits source, off-limits to a read-only reviewer; the oracle's deep-recursion cliffs (39,900 parens, [34,500, 34,760] calls, 88,800/91,948 sized, 100k/150k/200k terms) and the 16,608,454 cps rexxcps figure — expensive or crash-adjacent probes, and the mid-phase adversarial claims audit (d7b84dc5) already re-measured the oracle-attributed set; 3,863 collections (F6, no instrument); prefix-chain and derive cliffs in KNOWN GAPS (1,150-1,200 / 2,000-2,200, stack artifacts that move with the code).

## Citations checked by containment (all pass)

* `DoBlock::checkControl` at DoBlock.cpp:182 — :182 is the signature; body matches the prose (increment branch traces value then value+by; else branch explicitly skips tracing "We've already traced the initial assignment").
* `RexxString::stringComp` at StringClass.cpp:795 — :795 is the signature.
* `LanguageParser::resolveCalls` at LanguageParser.cpp:1690 — :1690 is the signature.
* `builtinTable[]` at BuiltinFunctions.cpp:3042 — :3042 is the array definition.
* `TRACE_PREFIX_CLAUSE` :92, `TRACE_PREFIX_INVOCATION_EXIT` :110 — exact.
* `NumberStringBase::createdDigits` — field exists (NumberStringClass.hpp:86).
* `CompoundVariableTable::findEntry` — exists (:107 of its .cpp).
* ast.rs:912 (`keyword()` maps both `When` and `WhenCase` to `"WHEN"`), ast.rs:776 (absorption rc-0 note inside `Select::whens` doc), ast.rs:801-815 (the `WhenCase` value-list vs condition-list distinction) — all contained.
* Plan citations lib.rs:418/:424 and eval.rs:70-71 are stale against the current tree (Plan moved to plan.rs:93; eval.rs:70-71 is now the shipped limit's doc) — expected, since those tasks shipped what the citations pointed at; no reader is directed to them any more.

## Priority 3 sweep (decisions outside the place the next reader looks)

Checked the commit list and sampled the decision-bearing commits; the phase has been unusually disciplined about landing decisions in the tree rather than only in messages:

* 63a4092e (DROP (v) newline predicate) — landed as a run.rs comment plus a mutant-killing test, not commit-only.
* 87fe09d8 (NotNumeric collapse, team-lead answer) — reasoning recorded in body.rs's doc.
* The loud.rs/coverage.rs hand-synced owner-table duplication — stated in loud.rs's own module doc (:42-45), in the gate, and the sets are asserted against the exclusions file.
* The two known-equivalent mutants — recorded in mutate-4a.sh's header with task-report citations.
* The clause-echo KNOWN GAPS row carries its own falsification note (Task 10, commit addf89b1) — the exact discipline this project keeps failing at, done right.
* The ExprKind ownership rulings — in the exclusions file per the lead's instruction, asserted by two tests.

The one instance found of the classic failure is F1: the `TRACE ?` requirement lived only in the spec's Risks section, which no task brief extracted, and it silently became an unrecorded divergence.
F7 is a half-instance: a lead recorded only in the gate's postmortem list, now resolved by measurement (parity).

## Priority 4 sweep (true-then, false-now)

* Spec Risks "corpus/README.md hard-codes 24 programs" — fixed during the phase (README now derives N); the spec sentence self-describes as the thing 4a fixes, fine.
* Spec criterion 4 "Heap::alloc_with never collects today, collect has no caller" — true at writing; the gate built the mode and renamed the method, and documents both; the spec's "today" phrasing survives acceptably as the criterion's motivation.
* Spec criterion 3's nine-prefix "measured reachable" list — now known incomplete (`>E>` is reachable); trace_oracle.rs and the gate both record the correction, the spec does not. Minor: the committed table is the durable record.
* "Fourth trace divergence" framing — corrected in place by e158225d; gate criterion 3 now reads correctly (unreproducible by construction, not narrowly triggered).
* Gate's "no CI builds or tests the Rust tree" — still true (.github/workflows has bsd/unix/windows for the C++ tree only).
* Plan checkboxes all remain `- [ ]` although the phase closed — cosmetic, the SDD flow tracks elsewhere.

## What this review did not reach

* The mutation script was not executed (edits source; read-only slice). Its structure was read: nine `run_one` calls matching the spec's list, UNAPPLIED-PATTERN fatal exit, cp-backup/EXIT-trap restore, no `git checkout --`.
* Oracle deep-recursion cliff figures and rexxcps baseline (see the not-re-derived list above).
* The 112 commit messages were read as subjects and sampled as bodies (about a dozen read in full); a body-by-body audit of all 112 was not done.
* The task reports and briefs under .superpowers/sdd/ were consulted only to trace figure provenance; they are another slice's surface.
* Whether `>E>`'s witness (`dotvariable_beyond_the_list.rex`) byte-matches was trusted to the re-run trace_oracle suite (5/5 green in the workspace run) rather than re-captured against the oracle.
