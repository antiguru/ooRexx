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
Task 2: dispatched (base b4d847074), implementer opus
