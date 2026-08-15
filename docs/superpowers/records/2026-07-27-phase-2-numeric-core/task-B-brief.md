# Task B — Comparison operators (Phase 2, Task 2.6)

Implement Rexx comparison in `rust/crates/rexx-num/src/compare.rs`.

## Operators

Numeric (ignore leading zeros and format; compare by value):
`=`  `<`  `>`  `<=`  `>=`  `\=`  `<>`  `><`

Strict (bytewise on the string forms, no numeric conversion):
`==`  `\==`  `<<`  `>>`  `<<=`  `>>=`

## Behaviours to reproduce

- `1 = 1.0` is true; `1 == 1.0` is false. Numeric comparison ignores form.
- `" 1" == "1"` is false — strict comparison does not strip blanks.
- `NUMERIC FUZZ` relaxes `=` (and the other numeric comparisons) by that
  many digits, and does NOT affect `==`. The comparison is made at
  `DIGITS - FUZZ` significant digits.
- `0 = -0` is true.
- `1e2 = 100` is true; `"1e2" == "100"` is false.
- `<<` and `>>` are strict string ordering: `"a" << "b"` is true.

## Verification

The differential harness compares against the real interpreter. Case format
is `digits|a|op|b`, one per line. Oracle:

    build/bin/rexx rust/crates/rexx-num/tests/data-addsub-oracle.rex cases.txt

It prints `digits|a|op|b=result`, where a comparison yields `1` or `0`, and a
failure yields `<E42>` style markers.

Generate your own case list covering every operator against a wide range of
value pairs and DIGITS/FUZZ settings, get the oracle's answers, and compare.
Aim for at least several thousand cases. Report the divergence count.

Note FUZZ cannot be expressed in the `digits|a|op|b` format; write a separate
small Rexx probe for the FUZZ behaviours and reproduce what it shows.

## Interfaces

`Number` is in `rust/crates/rexx-num/src/lib.rs`, with private fields
`negative: bool`, `digits: Vec<u8>` (most significant first, no leading zero
unless the value is zero), `exponent: i32`. It exposes `parse`, `format`,
`round_to`, `is_zero`, and `pub(crate)` helpers `assemble`, `truncated_to`,
`in_range`, `check_range`, `adjusted_exponent`.

Arithmetic lives in `addsub.rs`, `muldiv.rs`, `pow.rs` and returns
`Result<Number, ArithError>`.
