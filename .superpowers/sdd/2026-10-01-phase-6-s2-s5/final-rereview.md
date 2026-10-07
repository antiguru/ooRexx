# Re-review of f65766456 (P90)

Verdict: one Minor finding; I1, M1, M2, M4 and the queued files hold.

## Findings

**N1 (Minor). The corrected u3/u4 sentence pairs a this-crate fact with an oracle fact.**
`docs/superpowers/plans/phase-6-gate.md` ~line 147-149: "`u3` (this crate runs it at the forced
collection, on activity 2) and `u4` (the oracle runs it at an activation return,
`RexxActivation.cpp:705`) differ in collection timing". Each clause is true (u3 table row at ~1253:
ours `uninit dropped 2|activity end|main end`; u4 oracle row ~1254), but "u3 and u4 differ" reads as
u3 differing from u4. The differences are oracle-vs-here within each probe. The u3 oracle half
(termination, thread 1) is in the bullet above; u4's here half (at activity end, table ~1254) is in
the table only. Suggested: "`u3` and `u4` each differ from the oracle in collection timing: here
`u3` runs at the forced collection, on activity 2; the oracle runs `u4` at an activation return
(`RexxActivation.cpp:705`)." No falsehood, so it does not block.

## Checked, none

- I1: `a_store_wakes_every_guard_when_waiter` reads both `~result`s, so a waiter never woken ends in
  the deadlock refusal (rc 120), not a pass. The store can precede a park only if a waiter takes
  over 0.2 s of wall clock to reach `guard off when`; that makes the test weaker under extreme load
  (a late waiter sees v = 1 and passes without a wake), never a wrong answer. `SysSleep` blocks main
  in all three modes (`in_modes`: unswitched, collect, every), the shape the neighbouring tests
  rely on. No degenerate implementation passes it except one that wakes every waiter.
- M2: the narrowed doc matches the test: the method reads `time('E')` and seeds `random(...,7)`
  before the REPLY, the second program shows the REPLY clause's `time('R')` reset.
- M3: no false statement remains (u3 forced collection on activity 2 matches u3/u4 bullets and the
  criterion 10 table; the omission of the forced collection is fixed). See N1 for the wording.
- M1, M4, unjudged mutants: queued files match `final-review.md` (M1 evidence, M4 probe, M6, M7,
  M9-M12 as in "Declined to judge"); the elapsed-clock item gained the RANDOM and 0.70 measurements;
  the roadmap 6.1 row names `setlocal-scope`, `interpret-syntax-traceback-line` and
  `unjudged-scheduler-mutants`, and `seeded-random-switch-mode` exists.
- No em-dashes or en-dashes in the added lines (the roadmap `—` in the 6.1 row is the existing
  unchanged cell), no set counts in the added prose, nothing forward-looking beyond queue
  references.
