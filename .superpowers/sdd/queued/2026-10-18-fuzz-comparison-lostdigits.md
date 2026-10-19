**RULED: 9d, ruled by Moritz 2026-10-19 ("proposal ok").** Task 6.14 owns it.

**OPEN: found by the Phase 9d ANSI spike, not yet ruled.** Under `NUMERIC FUZZ 1`, a comparison of operands wider than DIGITS minus FUZZ raises LOSTDIGITS on the oracle and not here.

# LOSTDIGITS from a comparison under NUMERIC FUZZ

Found 2026-10-18 by the Phase 9d ANSI spike (`.superpowers/sdd/2026-10-18-phase-9d/spike-ansi-model.md`).
At the default DIGITS with `NUMERIC FUZZ 1` and a `SIGNAL ON LOSTDIGITS` trap,
`1.00000001 = 1.00000002` raises LOSTDIGITS on the oracle and nothing here. The comparison runs at
DIGITS minus FUZZ, so the oracle checks the operands against that smaller precision. Witness it in
the numeric area, next to Task 6.3's `lostdigits_*` programs.
