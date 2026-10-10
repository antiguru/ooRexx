# Concatenation with a variable reference drops its string value

Found by Phase 6.1 scout A (2026-10-07, section 4, `out-op.txt`). Queued by Phase 6.1 Task 12 (2026-10-10). Silent: rc 0 on both.

Probe `b1_varref_concat.rex`, run from a fresh empty directory:

    zz = .array~new(2)
    say (>zz) || 'x'
    say (>zz) 'x'

Oracle, rc 0:

    [stdout]
    an Arrayx
    an Array x
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 0:

    [stdout]
    x
     x
    [stderr]
    (empty)

The reference itself answers: `r = >zz; say r~string` prints `an Array` on both, and `(r) || 'x'` prints `an Arrayx` on the oracle and `x` here (`vr2.rex`, same run).

Suspected site: the concatenation operands' string conversion of a `VariableReference` (`dispatch/reqstr.rs`), which answers the empty string where `r~string` answers the referenced value's.
