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
