# SDD ledger — plan: docs/superpowers/plans/2026-10-07-phase-6-1.md

Spec: docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md (04b5f0ff8), rulings R1-R8.
Plan: b89f33392. Execution: subagent-driven (Moritz 2026-10-07). Workspace folders kept, never deleted
(Moritz 2026-10-07); committed with git add -f. Full gates only at Task 12 (Moritz 2026-10-04).

## Pre-flight

Scan: plan-review.md (Fable, at b89f33392) is the pre-flight scan; its pair table is the rows below,
with the finding against each and the fix applied to the plan.

| pair / task | produces vs consumes | found, ruling |
|---|---|---|
| T1, T6 | `DEBUG_PAUSE` bit | when it is set was in neither task; T1 Step 3 now states it |
| T1, T8 | activity RANDOM generator | T1 removed it; now kept as `Activity::random_source`, T8 routes its seed |
| T2-T5, T7 | `refusal-sites.tsv`, `owners.rs` | each constructor-changing task re-derives the tsv (Global Constraints); OPTIONS count moved to T3 |
| T2 | DO refusal pins | `loops.rs:575,600`, `scheduler/tests.rs:1286-1316`, `ir_recorded_cases/loop-refusals` added |
| T3, T4 | `dispatch.rs`, `run.rs`, `lib.rs` | disjoint sites; consistent |
| T4, T7 | b13 crash probe -> oracle-crashes entry | T4 saves the probe, T7 writes the entry |
| T5, T7 | exclusions rows `:530`, `:1895-1925` | consistent; path corrected to docs/superpowers/plans/ |
| T5, T6 | `run/interpret.rs` `run_fragment` | order holds |
| T7, T8 | dispositions LIMIT rows | T8 writes them (was "Task 9") |
| T8, T9 | `Policy`, `SimReport.trace_hash` | hash now T9's |
| T8, T10 | two `SwitchMode` enums | consistent |
| T9, T11 | invariant "ready holds no park reason" | consistent |
| T10, T11 | M11 | gate red via a `sim-gate.tsv` row in T10; crate test stays T11 |
| T10 | policy mix, oracle sets, scheduler inline programs | mix per seed stated; sets populated in Step 1; inline programs dropped from the gate with reason |
| T1..T12 self-consistency | | T1 condition field (I1) fixed to name `ActiveCondition`; remaining tasks agree with themselves per plan-review |

Ruling: per-task check is fmt, clippy, plain debug `cargo test --workspace`, plus `REXX_CORPUS_GATE=1` on `--test corpus` and `--test ir_recorded_oracle`; `whole_groups` in release at Tasks 3, 5, 10, 12 — follows Moritz 2026-10-04 (gates at stage closes); witnesses need the strict mode — cost if wrong: a whole_groups regression found up to two tasks late.
Ruling: scheduler tests' inline programs are not in the seeded gate (they pick their mode in code) — adding a hook is new mechanism the spec does not ask for — cost if wrong: those programs explored only in their fixed modes.
Ruling: plan-review minors M1-M19 applied in the plan except none deferred.
Task 1: dispatched (base e6af1198b), implementer opus
Task 1: Ruling: size_of::<Activation>() pinned at 480 (the brief's packing shrinks it from 512; no variant hits 512 exactly) — R4's purpose is no growth on the per-call path, a smaller struct serves it — cost if wrong: an assert value to change. Also accepted: phase-6-1.txt sorted before phase-6.txt in harness lists; probe (d) gains a SIGNAL ON variant (oracle: REPLY in a CALL ON handler is 91.999).
Task 1: implementer DONE_WITH_CONCERNS (66b0f6854, 16b67c6cc, 12239c974 perf round 1, e77576cd0 gate record); perf +0.10%..+0.27% after round 1. Concerns for T12 queue: SIGNAL ON propagation traceback line missing (with cond3); SELECT CASE value lingers on the activation (no reader found).
Task 1: review dispatched (opus), package review-e6af1198b..e77576cd0.diff
Task 1: review (task-1-review.md): spec ❌ 1 Important (typed debug line's SELECT CASE overwrites the running construct's value; not a regression), 5 Minor.
Task 1: Ruling: Minor 1 (false doc sentence on Activation::cold) joins the fix round — a false sentence violates the Global Constraints' prose rule, so it is not optional polish — cost if wrong: one extra line in a fix round.
Task 1: minor (deferred): no witnesses for the termination restore on SYNTAX unwind and at a REPLY continuation's end (probes unwind/, replyend2/ in /tmp/claude-1000/p61/t1r/)
Task 1: minor (deferred): next_seed walks top_level_activation_mut twice per RANDOM
Task 1: minor (deferred): end_cold swap_remove(0)+clear idiom
Task 1: minor (deferred): INTERPRET inside a typed debug line inherits DEBUG_PAUSE (oracle: false); not a regression; for Task 6
Task 1: fix round 1/5 (2 addressed, 0 open — case text across a typed debug line; Activation::cold doc; commits e77576cd0..b4d847074). Spec section 2's case-text cell corrected by the controller.
Task 1: minor (deferred): current_case_text is never cleared at construct end ("for the construct's duration" unenforced; no reader found)
Task 1: complete (commits e6af1198b..b4d847074, review clean)
Task 2: dispatched (base 3cee3e622), implementer opus
Task 2: implementer DONE_WITH_CONCERNS (5b7acef35, 9777db504 perf r1, fe765db7b, 4d46b76b0, 81a8c1bae); emptyloop wall +4.16..+4.42% (cycles up, instructions -0.32%), three rounds spent -> Moritz. run_repeating unreached by any test (for review). Queued do-with-over-refusal to close at T12.
Task 2: Moritz ruling 2026-10-07: emptyloop wall-clock +4.2..4.4% accepted; Task 12's cumulative measurement re-checks it with a layout control.
Task 2: review (task-2-review.md): 1 Important (COUNTER/DO WITH push a rooted temp per pass: memory grows, INDEX/ITEM values kept alive), 5 Minor, 1 ⚠️ (criterion 5 row reasons for newly visible whole-group gaps not in gate record).
Task 2: Ruling: the ⚠️ is a real gap; the reasons go in the gate record in this fix round — criterion 5 needs them, and they are freshest now — cost if wrong: none.
Task 2: Ruling: Minor 3 (HeaderRole::OverFor doc now false for DO WITH's FOR) joins the round — prose rule forbids false sentences — cost if wrong: none.
Task 2: Ruling: the REPLY trace gap and the TraceObject collector gap are not 6.1 items (reproduced with no loop); queued at Task 12 — cost if wrong: Phase 9 meets them unattributed.
Task 2: minor (deferred): run_repeating and run_loop_with_header's non-Simple arms are dead (only Fallback is a labelled simple block) and Task 2 threaded untested COUNTER/WITH code through them; delete or unreachable! — for the final review
Task 2: minor (deferred): with_advance recomputes control_slot/shape_of per pass; cache in WithState
Task 2: minor (deferred): overlong rewrapped comment lines (run/loops.rs FlatLoop comment; phase-4-exclusions.txt)
Task 2: fix round 1/5 (3 addressed, 0 open — per-pass temp leak; gate-record row reasons; OverFor doc; commits 81a8c1bae..7b84c819d)
Task 2: complete (commits 3cee3e622..7b84c819d, review clean)
Task 2: minor (deferred): unverified that a trapped SYNTAX inside a DO WITH header truncates temps (same shape as controlled loop :2270)
Task 3: dispatched (base 1d308cb9d), implementer opus
Task 3: implementer stalled overnight (idle "waiting for gate events", gate finished 23:21, woken 08:57); then DONE_WITH_CONCERNS (f763836ce..75c41b836). Concerns: object_position kept (DO OVER .context~package~local refuses, oracle rc 0: b11 GUARD is reachable); the_s2_rows TRACE_TraceObject TEST_TRACEOBJECT_COLLECTOR fails in gated concurrency_tests, also at base 1d308cb9d (so from Task 1 or 2); c_condition_d_raise7 now silently wrong (CALL ON ANY traps a reflected NOVALUE; scout A section 4, queued at T12).
Task 3: review (task-3-review.md): 1 Important (b10: a user `+` answering a non-canonical string diverges silently: 'abc' gives 41.1 where the oracle compares as string; ' 2 ' prints [2] vs [ 2 ]; exclusions text now false), 5 Minor.
Task 3: Ruling: object control values and DO OVER .context~package~local (object_position still reachable) join the fix round — b10 and b11 were IMPLEMENT/GUARD in spec section 3; a reachable GUARD becomes IMPLEMENT — cost if wrong: a larger fix round.
Task 3: Ruling: the_s2_rows TEST_TRACEOBJECT_COLLECTOR failure joins this fix round (bisect to Task 1 or 2; expectation line with its reason, or fix if a regression) — a red gated file must not be normalised before Task 5's close — cost if wrong: misattributed to Task 3.
Task 3: Ruling: reviewer Minor 1 (raise novalue/lostdigits return into CALL ON ANY) gets an exclusions known-gap row in this round; Minor 5 (FORWARD ARGUMENTS over MAKEARRAY answering a List, oracle SIGSEGV) and Minor 4 (oracle GUARD WHEN on a compound tail never wakes) go to oracle-crashes.txt after 5 runs each — record where the project looks — cost if wrong: three text rows.
Task 3: minor (deferred): c3 commit 8f080d2fa carries a hand-edited refusal-sites.tsv row never built alone (HEAD regenerated)
Task 3: fix round 1/5 (4 addressed, 2 new open — Critical: control switching to object leaves heap TO/BY unrooted, release panic 'a live value' rc 101; Important: is_true_object accepts any '1' where the oracle needs TheTrueObject; commits 75c41b836..d9192ef4a). Perf against base being measured.
Task 3: perf after fix round 1 (012bf8ab1): against base emptyloop -0.0019%, decloop -2.82%, rexxcps -0.0008%; wall emptyloop +3.96% (inside ±4%). Fix round 2 sent.
Task 3: Ruling: is_true_object vs TheTrueObject identity — option (B): an exclusions known-gap/deviation row (a computed one-byte '1' and logical true share the inline handle; only a user-defined DO comparison or BY's < answering a computed '1', or .true~copy, diverges); option (A), a distinct handle for true, queued at Task 12 — (A) changes a value representation everywhere for a niche observable; literal 1, '1' and 0+1 already agree — cost if wrong: that niche stays divergent until the queued item.
Task 3: Ruling (Moritz 2026-10-08) supersedes option (B): a computed '1' is true in a DO TO test and BY sign check; the oracle's pointer test against TheTrueObject (DoBlock.cpp:213, DoBlockComponents.cpp:173; WHILE/UNTIL fall back to truthValue at :277-316) is an oracle bug. The exclusions row is a deliberate divergence, not a known gap; the queued distinct-handle item (A) is dropped.
Task 3: fix round 2/5 committed (308386167 rooting, b0b72baa4 minors, 1a46f2eb9, 8539cd4a5, 6a3091cc6 Deviation 25); re-review dispatched
Task 3: fix round 2 re-review (task-3-rereview2.md): rooting ADDRESSED; true-object NOT ADDRESSED (crate does an identity test on the inline '1', not the logical test Deviation 25 describes); new Minor: a numeric TO/BY becomes a fresh object per pass (identityHash differs; oracle passes the same object).
Task 3: Ruling (Moritz 2026-10-08, refining the earlier one): the DO TO test and BY sign check judge a user operator's answer by truth value exactly as WHILE/UNTIL do: '1' (any form) ends / means negative, '0' continues, anything else raises Error 34 (Logical value not 0 or 1, the WHILE/UNTIL subcode pattern). Deviation 25 rewritten to state that, with the wider divergence measured.
Task 3: Ruling: the fresh-object-per-pass Minor joins round 3 — an identityHash an Rexx program can read differs from the oracle, and the fix (cache the bound's object once) is small — cost if wrong: one more small change.

## Controller state at compaction, 2026-10-08 ~13:20
- Tasks 1, 2 complete. Task 3 in fix round 3 (implementer agent `t3-impl`, alive): (1) DO TO test and BY sign check use a WHILE-style truth value: fast path LOGICAL_TRUE/LOGICAL_FALSE, else string value, `eval::logical_value`, else 34.901 with the string; replaces `is_true_object` (run/loops.rs ~2989); Deviation 25 rewritten to that rule with measured table; (2) a numeric TO/BY made into an object once per loop, not per pass (`loop_bound_object` run/loops.rs:519-532); (3) perf vs base (budget +0.5%). Then: scoped re-review of round 3 (FIX_BASE 6a3091cc6), then Task 3 complete, then Task 4.
- Watch rule: an implementer's idle "waiting" notice is not a report; check its status/processes within ~30 min (ScheduleWakeup), never at the next user message.
- Per-task check and whole_groups schedule: see the Pre-flight rulings. Reusable prompts: task-reviewer-common.md, re-review-common.md, global-constraints.md. Gates script p61-gates/bggates.sh (Task 12 only).
- To queue at Task 12 (beyond plan Step 1): SIGNAL ON propagation traceback line (Task 1); REPLY trace gap and TraceObject collector gap (Task 2); upstream ticket candidate: oracle DO TO/BY identity test (memory oorexx-do-to-true-identity-bug).

- Task 3 fix round 3: dcd1db992, 0c75907d3, eb9c3c7b8. Perf vs 6.1 base: emptyloop -0.96%, decloop -2.67%, rexxcps -0.49%. Scoped re-review t3-rereview3 dispatched (FIX_BASE 6a3091cc6, HEAD eb9c3c7b8), report task-3-rereview3.md.
- Task 3 re-review 3 (task-3-rereview3.md, through 43621d78e): Finding 2 and Minor 1 addressed; no Critical/Important; four new Minors. Minor cleanup dispatched to t3-minors (doc names, dead arm, record errors).
- Ruling: a user STRING method is not sent by the crate's truth tests (DO, WHILE, UNTIL) — crate-wide gap predating this task, not Task 3 scope — queue it for Task 12 (implement in the shared truth helper, or record). Cost if wrong: one more divergence row lives until close.
- Task 12 queue add: truth tests ignore a user STRING method; WHILE blames a later-pass 34.3 on the DO line (oracle: END); a trapped error in a user `>` during the header's first comparison holds the TO object until another object loop ends (predates Task 3).
- Task 3 minor cleanup dd76f275a (t3-minors): doc names fixed; dead object arm of run_loop_with_header made unreachable! (controller checked: sole caller drive.rs:1630 via FlatStart::Fallback, returned only for LoopKind::Simple at loops.rs:1734); report and Deviation 25 corrected. lib 1016 pass.
- Task 3: complete (dcd1db992..dd76f275a after fix rounds 1-3; re-reviews task-3-rereview.md, -rereview2.md, -rereview3.md). Debug workspace last run at 1a46f2eb9; stage-close gates re-run it.
- Task 4 dispatched: implementer t4-impl (opus), BASE a3c2c3c0a, brief task-4-brief.md, report task-4-report.md.
- Task 3 REOPENED: re-review 3 update (through 43621d78e) found Important New Breakage 1: 945e31f33 read the DO answer with string_value_text (an Array -> "an Array", 34.901) where the crate WHILE uses to_text (.array~of(1) is 1). Ruling: DO test reads as the crate WHILE does (to_text); records claiming requestString behaviour deleted. Fix round 4 sent to t3-minors (sonnet, small one-line fix plus records) while t4-impl runs; both told to stage by path. Cost if wrong: a commit collision in shared files, recoverable from git.
- Task 3 fix round 4 ec97e4190 (t3-minors): loop_truth reads via to_text as condition_value does; witness do_object_compare_array.rex; licensed rows do-compare-array-one/-many; Deviation 25 re-measured. Controller read the diff. lib 1016, gated corpus 29+1, ir_recorded_oracle 21, licensed_divergences 22 pass.
- Ruling: a non-logical DO TO/BY answer raises 34.901, not WHILE 34.3 / UNTIL 34.4 — a DO header has no 34.x subcode and 34.901 is the oracle subcode for a method logical answer — cost if wrong: one subcode in error text.
- Task 3: complete (again; through ec97e4190). Minors from re-review 3 fixed in dd76f275a.
