# Final spec review, standing in for Moritz

Moritz cannot read the spec from where he is and asked for an independent review before the plan is
written. You review on his behalf: is this the right Phase 6.1, and is the spec sound enough to plan from?

Spec: `docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md` at `fc04e3857`. Its rulings R1-R4
(section 6) are Moritz's and are not up for review unless the spec misapplies them. Context: roadmap row
6.1 (`docs/superpowers/plans/2026-07-27-rust-rewrite.md:658`); the scout reports and three earlier
reviews (`spec-review-{facts,dst,plan}.md`) beside this file; their fixes were applied by hand, so check
that each BLOCKER and IMPORTANT finding of those reviews is either reflected in the spec or rightly
dropped (R1 retired most oracle-judging DST findings).

Read `scout-common.md` beside this file for the rules (read-only; runs only if a finding needs one;
no subagents).

Look for:
- internal contradictions (decisions vs staging vs exit criteria vs rulings), stale text left by the
  hand edits, claims that are false at `fc04e3857`;
- scope: anything the roadmap row requires that the spec drops, and anything the spec adds that the row
  and rulings do not justify;
- exit criteria a person could not decide with a command or a run;
- risks a planner would trip on (ordering, the 512-byte `Activation`, the per-clause branch budget,
  the simulation gate's ability to fail).

Write `.superpowers/sdd/2026-10-07-phase-6-1/spec-review-final.md`: verdict (plan from it / plan after
fixes / rework), findings as BLOCKER / IMPORTANT / MINOR each with the spec's text, evidence, fix; then
questions only Moritz can answer, each with a recommendation. Prose minimal, no em-dashes. Return only
the verdict, counts, and path.
