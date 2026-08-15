# Task A — port `dividePower`, close the two negative-power divergences

Status: DONE. Both target cases now match the oracle; all six regression sets
remain at zero divergences.

## What `dividePower` does differently from the general division

`NumberString::dividePower` (`interpreter/classes/NumberStringMath2.cpp:1059`)
long-divides 1 by the power accumulator, and differs observably from
`NumberString::Division` in two ways that matter here:

1. **No rounding and no range check of its own.** It emits at most
   `workDigits + 1` truncated quotient digits and stops; `power`'s tail does
   the single rounding back to the caller's DIGITS. The Rust path went through
   `div_unchecked(one, acc, work + 2)`, which (a) *rounded* the quotient at
   `work + 2` before `pow` rounded again — a double rounding — and (b) still
   ran an unconditional `check_range()` on its assembled result even in
   "unchecked" mode (`muldiv.rs`, the final `assemble(...).check_range()`).

2. **The dividend is a synthetic 1** padded with zeros to the divisor's
   length, with the result exponent tracked as
   `-accum.exponent - accum.len + 1` and decremented per brought-down zero,
   rather than derived from two operand exponents.

Point 1(b) is exactly the DIGITS 2 failure: for `730361.1e999999992 ** -1`
the quotient at working precision is `1.36986...e-999999998`, whose *raw*
(last-digit) exponent is ≈ −1000000004, outside ±999999999 — so the old path
threw E42 even though the rounded 2-digit result `1.4E-999999998` is fine.
The C++ never checks the intermediate.

## The second bug was upstream of the reciprocal

With `divide_power` ported, `129720.468 ** -23` at DIGITS 7 still came out
`2.516547E-118` vs oracle `2.516546E-118`. Since the ported reciprocal is a
pure truncation of `1/acc`, the accumulator itself had to differ at working
precision. It did: the C++ multiply loop (`NumberStringMath2.cpp:911-949`) is
**high-bit-first** — the accumulator starts as the base (consuming the
power's leading 1-bit), then each remaining bit, high to low, squares the
accumulator and multiplies the base back in when set. The Rust port went
low-bit-first. The two orders produce intermediates whose last working digits
differ; final rounding hides that for positive powers (which is why all
positive corpora passed either way), but the reciprocal's truncated quotient
exposes the accumulator's full working-precision digit string.

I verified against the C++ that `multiplyPower` + `adjustNumber` per step is
equivalent to the existing Rust `mul(…, work)` (full product, fold overflow
into the exponent, half-up round at `work`), so only the sequencing needed to
change, not the per-multiplication arithmetic. Note the C++ loop body reads
"multiply then square", but because the first tested bit is the cleared
leading 1, the linearized operation sequence is square-then-multiply per
remaining bit — confirmed by trace for 23 = 10111b:
x² → x⁴ → x⁵ → x¹⁰ → x¹¹ → x²² → x²³.

## Changes

`rust/crates/rexx-num/src/pow.rs`
- Replaced the low-bit-first square-and-multiply with the interpreter's
  high-bit-first sequence (`acc = left; for bits below the top, high→low:
  square, multiply if set`). A side effect matching the C++: `x ** -1` and
  `x ** 1` now perform no multiplication at all, so no intermediate range
  check runs there either.
- Replaced `div_unchecked(...)` with a faithful port of `dividePower`:
  `divide_power(accum, digits)` plus `subtract_multiple` (port of
  `subtractDivisor`, including its two-position borrow handling). The port
  keeps the C++ structure: the guess divisor `divChar` from the first two
  divisor digits plus one, guesses wrapped up to 1, the equal-compare
  early-out that bumps the digit and ends the division, zeros only appended
  after a first significant digit, and termination at `digits + 1` result
  digits or a zero remainder. `calc_exp` is carried in i64 (C++ uses 64-bit
  `wholenumber_t`) and saturates into the i32 exponent field, where the
  caller's range check rejects anything that big anyway.
- `pow`'s tail is unchanged: single `round_to(digits).check_range()`, then
  trailing-zero strip — which matches the C++ tail (strip leading zeros,
  round once if over DIGITS, strip trailing zeros).

`rust/crates/rexx-num/src/muldiv.rs`
- Removed `div_unchecked`/`div_inner` and the `range_check` flag; `div` is
  back to a single always-checked body (behaviour of `/`, `%`, `//`
  unchanged — muldiv sets confirm).
- `strip_leading` is now `pub(crate)` for reuse by `divide_power`.

`rust/crates/rexx-num/tests/pow.rs`
- Two new tests pinning the fixed cases:
  `a_reciprocal_out_of_range_at_working_precision_is_not_an_overflow`
  (`730361.1e999999992 ** -1` at DIGITS 2 = `1.4E-999999998`) and
  `the_reciprocal_is_rounded_exactly_once`
  (`129720.468 ** -23` at DIGITS 7 = `2.516546E-118`).

## Verification

Target sets:

| set | cases | divergences |
|---|---|---|
| pow.txt | 2,112 | **0** (was 1) |
| p22.txt | 8,000 | **0** (was 1) |

Regression sets (all must be and are zero):

| set | cases | divergences |
|---|---|---|
| addsub | 9,248 | 0 |
| addsub2 | 8,112 | 0 |
| muldiv | 17,424 | 0 |
| md2 | 17,496 | 0 |
| rand1 | 20,000 | 0 |
| p21 | 8,000 | 0 |

All corpora were additionally run through the **debug** build so the new
`debug_assert`s in `divide_power`/`subtract_multiple` (non-zero divisor,
no negative remainder) were exercised across all 18,112 pow cases; none
fired.

- `cargo test --offline`: entire workspace green; `tests/pow.rs` runs
  7 tests, 7 passed (the 5 existing plus the 2 new).
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: clean
  (`Finished`, no diagnostics).

## Notes / residual risk

- The task briefing recorded that exponentiation "had to go low-bit-first
  rather than high-bit-first". The C++ is unambiguously high-bit-first, and
  with the exact C++ sequencing (accumulator seeded with the base, leading
  bit consumed, square-then-multiply) every corpus passes. The earlier
  low-bit finding most likely came from a *different* high-bit variant
  (accumulator seeded with 1, or the loop body's literal multiply-then-square
  order applied without the one-bit shift), which computes a different — even
  wrong — multiplication chain.
- `pow`'s final `check_range()` has no C++ counterpart (`power` never range
  checks its final result; only the up-front magnitude prechecks exist).
  A case like `2e999999999 ** -1` would print `5E-1000000000` under the
  interpreter but E42 here. No corpus case distinguishes the two today; left
  as-is deliberately, since regressions define correctness and the corpora
  pass. Worth an oracle probe if extreme-exponent reciprocals ever join the
  corpus.
