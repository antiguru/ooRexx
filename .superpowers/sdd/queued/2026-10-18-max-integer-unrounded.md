**OPEN: found by Phase 9d Task 6.3, not yet ruled.** `2~max(1111111111)` at DIGITS 9 answers the unrounded integer on the oracle and a rounded number here.

# `MAX` with a whole-number argument wider than DIGITS

Found 2026-10-18 by Phase 9d Task 6.3. At `NUMERIC DIGITS 9`, `say 2~max(1111111111)` prints
`1111111111` on the oracle and `1.11111111E+9` here. This looks like the `MAX` side of the integer
representation that Deviation 44 rules on for `MIN`: the oracle's `RexxInteger` compares and
answers an integer without rounding to DIGITS. It has not been investigated. Read
`RexxInteger::Max` (`interpreter/classes/IntegerClass.cpp`) and decide whether it is the same leak.
If it is, Deviation 44's text is extended by Moritz's ruling. If it is not, it is a divergence to
fix. No ooTest test in the 9d set reaches it.

The same leak decides LOSTDIGITS for MAX and MIN (Phase 9d Task 6.3 fix round 1, probe
`docs/superpowers/records/2026-10-18-phase-9d/t6.3/probes/maxstr.rex`). At DIGITS 9 the oracle
raises LOSTDIGITS for `max(1, '1111111111')`, a quoted string, and stays silent for the literal
`max(1, 1111111111)`, because it decides "integer" by the object's type. The crate decides by the
value and stays silent for both. Rule this together with the value question, using what the ANSI
standard says (the 2026-10-18 spike, `.superpowers/sdd/2026-10-18-phase-9d/spike-ansi-model.md`).
