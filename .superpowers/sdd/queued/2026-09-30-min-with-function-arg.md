**CLOSED: Phase 6 Task 9 fix round 1.** its witness corpus/lang/builtin_argument_call.rex agrees.

# `say min(3, f())` prints 5, oracle 3

Found by Phase 6 Task 8's implementer (2026-09-30); present on the base binary too, not caused by
Task 8. Details and the oracle command in .superpowers/sdd/2026-09-29-phase-6-s0-s1/task-8-report.md.

2026-09-30, Phase 6 Task 9 (`f00c8921e`): `say min(3, f())` and `interpret "say min(3, f())"` print 3,
as the oracle does, because a call in a builtin's argument now compiles to its own op and lends the
argument stack. The tree form still lost it: `say min(3, f()) (o~~m == o)` printed `5 1`, oracle
`3 1` (Task 9 review, I1).

Closed 2026-09-30 by Task 9 fix round 1: a call entered from `begin_invoke_call` now lends the
argument stack to its callee's tail, as the compiled call ops do. Witness
`rust/corpus/lang/builtin_argument_call.rex`.
