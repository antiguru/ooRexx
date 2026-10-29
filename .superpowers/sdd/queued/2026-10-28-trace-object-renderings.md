**CLOSED 2026-10-29 by Phase 9d Task 8.7c (e620097f0).**
**OPEN, queued 2026-10-28 by the controller; proposed 9d (Task 8.7c).**

# TRACE I renderings of an Array PARSE VALUE and of a returned reference

Found by Phase 9d Task 8.7a (review Queue candidates; fix round 1 report, Concerns 1); both predate the task.
* `parse value .array~new(2) with a` under `trace i`: the `>K>` line is `"an Array"` on the oracle and `""` here (docs/superpowers/records/2026-10-18-phase-9d/t8.7a/review-probes/trace-parse-value-a14.rex).
* `h: use arg >x; return >x` called with `>v`: the oracle traces `>O> ">" => "V"`, this crate `"X"` (docs/superpowers/records/2026-10-18-phase-9d/t8.7a/r1/probes/s25.rex).
Derive from rexxref's trace output rules before following the oracle.
