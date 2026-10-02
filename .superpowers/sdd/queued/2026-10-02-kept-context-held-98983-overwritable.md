# A held 98.983 from a kept-context member can be lost later in the same native call

A thread-context member reached through a context another activity kept does nothing and holds
98.983 for the running call (Task 7 fix round 2, ruling P37). An extension that keeps calling the
API after that can lose the held condition: `kept->ClearCondition(); context->RaiseException0(93900)`
traps 93.900, and `kept->String(); context->ClearCondition()` answers normally. On the oracle the
thread check throws out of the native at once (then hangs).

Fix when needed: make the held 98.983 immune to a later ClearCondition or raise in the same native
call. Probes J and K: Task 7 review, "Re-review 2", `.superpowers/sdd/2026-10-01-phase-6-s2-s5/
task-7-review.md`; scratch `rev-t7/probes`.
