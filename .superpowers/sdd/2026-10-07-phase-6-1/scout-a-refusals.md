# Scout A: disposition of every class (b) and (c) loud refusal

Read `scout-common.md` beside this file first. Your name for paths: `sa`.

Roadmap 6.1 exit: every loud refusal naming Phase 5 or no phase (census classes (b) and (c)) is
implemented, or re-homed to a remaining phase (9, 10, 11) with a true reason, or recorded as a deviation
with owner none; then `tests/closed_phases.rs` polices Phase 5 too.

1. Re-derive the census at HEAD with the census appendix scripts (copy them out of the census file into
   `/tmp/claude-1000/p61/sa/`). Report drift against the census.
2. For each (b) and (c) site (group by constructor + feature where the census table already groups), decide
   reachability by running: write the smallest Rexx program that reaches it, run ours and the oracle. A site
   you cannot reach after trying: record the attempts.
3. Propose a disposition per group: IMPLEMENT (with the oracle's observable and a size estimate: S <50 lines,
   M <300, L more, and the files), REHOME (which remaining phase, and the roadmap row text that makes it that
   phase's: quote it), DEVIATION (why matching the oracle is wrong or impossible), or GUARD (unreachable;
   reclassify to d-guard with the attempts as evidence).
4. Note which ooTest rows each implementable group unlocks (census C6, `docs/superpowers/records/2026-10-01-phase-6-s2-s5/whole-groups/table.txt`).
5. What `closed_phases.rs` needs to police Phase 5, and every test or derived table pinning the text
   (queued `2026-09-28-phase-5-refusal-labels`).

Include the queued items `2026-10-02-do-with-over-refusal` and `2026-09-28-use-arg-message-term` in
your groups (they are class (c)); for DO, cover COUNTER on every loop kind and `DO x OVER stem.` too.

Write `.superpowers/sdd/2026-10-07-phase-6-1/scout-a-report.md`: the census delta, a table
(group, sites, reachable y/n + probe, oracle observable, disposition, size, ooTest rows), then the
closed_phases notes. Probes in an appendix, as run.
