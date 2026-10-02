# 2026-10-01-reply-continuation-state-visibility

Found by Phase 6 S0/S1 final review slice A (2026-10-01): a later guarded send could not see the
writes of a REPLY's continuation (oracle `after argument-value-long`, this crate `after none`).
Probes: scratch p6-final-a/probes/p04_reply_in_arg.rex, p04b_reply_where.rex; final-review-a.md B2.

S2 Task 6 runs the continuation as its own activity. The sender still reads `none`, because the
continuation does not hold the replier's guard: carried by
`2026-10-02-reply-guard-transfer-witness.md` to Task 11.
