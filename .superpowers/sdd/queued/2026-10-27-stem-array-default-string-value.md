**CLOSED 2026-10-29 by Phase 9d Task 8.7c (e620097f0).**
**OPEN, queued 2026-10-27 by the controller; proposed 9d (Task 8.7).**

# A stem's string value with an array default joins the items

Found by the Phase 9d Task 8.5 fix round 4 re-review (task-8.5-rereview-r4.md, queue candidate 1) and
round 4's report ("Found beside the fix"); the stem side predates the task. `t. = .array~of('one','two');
say t.~string`, a Stem subclass instance `u[] = zz; say u~string`, and `trace r; x = t.` answer the items
joined here and `an Array` on the oracle, which rexxref's TRACE rule (instrc.xml:3697-3699) agrees with. A
reference to the same stem answers `an Array` since Task 8.5 round 4, so the two now differ in this crate.
Probe acy/a4.rex of the re-review (copied to docs/superpowers/records/2026-10-18-phase-9d/t8.5/r5/probes/ by
Task 8.5's fix round 5, if kept). Derive the stem's STRING and TRACE rendering from rexxref first.
