**OPEN: found by the Phase 9d Task 6.3 review, not yet ruled.** A `DO` repetitor or `FOR` count wider than DIGITS raises 26.2 or 26.3 here, while the oracle runs the loop.

# A loop count wider than NUMERIC DIGITS

Found 2026-10-18 by the Phase 9d Task 6.3 review (M2, probes `rep.rex`, `rep2.rex` in its record).
At `NUMERIC DIGITS 9`, the crate rejects `do 1111111111` with 26.2 and `do i = 1 for 1111111111`
with 26.3, rc 230. The oracle runs both loops and raises no LOSTDIGITS, even with a trap set. The
oracle's repetitor and `FOR` handling is in `interpreter/instructions/DoBlock*.cpp` and
`BaseDoInstruction.cpp`. Read how they convert the count, without the DIGITS check.
