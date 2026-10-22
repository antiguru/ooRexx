# A security manager set mid-activation is seen by an internal call

Found by the Phase 9d Task 8.2 re-review (task-8.2-rereview-r1.md, Minor 3); predates the task.

Probe: the review's `u4.rex` with `call r` / `r:` (an internal routine) in place of the `::routine`:
the stem's UNKNOWN method set protected, `.context~package~setSecurityManager(.sm~new)`, then
`call r`, which does `s.~length`, `s.~unknown('LENGTH', .array~new)` and a protected method call.

Oracle: `len 3` / `direct 3` / `prot pm-ran`, no manager call at all: the internal call inherits
the caller's activation settings, captured before setSecurityManager. Crate (base, 9ce1d8384 and
ece5b7dc9): the manager is asked for UNKNOWN and PM.

Open: whether rexxref says when a package's security manager takes effect for running activations
(derive per Ruled 13 before treating the oracle's answer as the rule). Not in the closed
2026-10-19-security-manager-protected-primitive.md.
