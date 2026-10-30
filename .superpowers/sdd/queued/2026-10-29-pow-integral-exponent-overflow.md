**CLOSED 2026-10-30 by Phase 9d Task 8.8 (3148553fe).**


# A whole-number power exponent with trailing fraction zeros overflows

`numeric digits 30; say 2 ** '1.00000000000000000000'`: this crate raises 26.8 (rc 230), the oracle prints `2` (measured 2026-10-29 at 97eee8a9a). `rexx-num/src/pow.rs:57` accumulates every coefficient digit into an i64 before dividing away the fraction, so a 21-digit coefficient overflows. Derive the whole-number rule (and its digit ceiling) from rexxref (arithmetic, power) and reuse the whole-number converter (`rexx-num/src/lib.rs:712`) if it fits.
Probe: docs/superpowers/records/2026-10-18-phase-9d/codex-review/pow-integral-exponent.rex.
