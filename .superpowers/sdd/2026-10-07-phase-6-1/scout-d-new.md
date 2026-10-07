# Scout D: size subclass `NEW` on String, Stem, Method, Routine, Message, VariableReference

Read `scout-common.md` beside this file for the rules; use HEAD `fc04e3857` instead of `be19fd06a`.
Your name for paths: `sd`.

Spec ruling R2 (`docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md` section 6): b14 lands in
6.1 with subclass `NEW` if `NEW` is M or less (S <50 lines, M <300, L more); else Phase 9's row is amended.
Background: scout A's b14 and b8 probes (`scout-a-report.md`, rows b14 and section 4; probe texts in its
Appendix B, `b8_*`). Today `.k~new` for `::class k subclass string` (and Stem, Method, Routine, Message)
refuses `method "NEW" of class "K" is not implemented (Phase 9)`; VariableReference's answers where the
oracle raises 93.967.

1. For each class: what the oracle's `NEW` does for a subclass (C++ `newRexx` or equivalent, file:line):
   arguments, how the instance is built, `INIT` dispatch, what instance state the subclass instance has.
2. Why ours refuses: the code path (file:line), and what a subclass instance of a primitive-backed class
   needs in our object model (primitive payload plus an instance variable pool? existing precedent: which
   primitive-backed classes already support subclass `NEW`, e.g. Directory, StringTable, Array; how they do it).
3. A design per class and its size with files. Then b14 itself once NEW exists (scout A: S).
4. Probes: run scout A's `b8_*` and b14 probes at HEAD on both engines to confirm the starting point.

Write `.superpowers/sdd/2026-10-07-phase-6-1/scout-d-report.md`: per class table (oracle rule, our gap,
design, size), the total, and a recommendation under R2.
