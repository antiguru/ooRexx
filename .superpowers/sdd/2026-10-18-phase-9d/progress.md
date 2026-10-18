# SDD ledger — plan: docs/superpowers/plans/2026-10-18-phase-9d.md

Spec: docs/superpowers/specs/2026-10-17-phase-9d-divergences-and-close-design.md (reviewed, .superpowers/sdd/2026-10-17-phase-9d-spec-review.md). Plan reviewed twice by fable (plan-review.md). global-constraints.md = plan '## Ruled by Moritz' to '---'.
Moritz 2026-10-18: 'Tasks 1, then pause'.
Task 1: dispatched (implementer opus, BASE 328fc0e28).
Task 1: implementer DONE (8120c43fb, 98e9f4d60): 178 tests, 46 clusters; scclient/sfclient cause = ADDRESS WITH keeps baton (command.rs:728). Deviation candidates: rexxinteger-min-representation, bag-hasitem-nil, stream-binary-records part; owner Q: SIGNAL HALT test needs hostemu. Review dispatched (sonnet).
Task 1 review (sonnet): spec PASS, quality PASS; 14 clusters reproduced both sides incl. the 3 Deviation candidates; minors: yaml note only in report (controller moved it into plan Task 4), call-on-trap-argument not independently reproduced, stream-command-position/collection-set-operations unprobed (may split).
Task 1: complete (8120c43fb, 98e9f4d60). Paused per Moritz. Open for Moritz: Deviation rulings rexxinteger-min-representation (19), bag-hasitem-nil (1), stream-binary-records @@wrong part; owner of hostemu-halt (Phase 10?).
Moritz 2026-10-18: Deviations 44 (MIN representation), 45 (Bag hasItem), 46 (binary LINEOUT positioning) recorded; hostemu-halt -> Phase 10 row via loud Phase 10 Extension refusal. Plan Ruled 7, 8; global-constraints regenerated.
