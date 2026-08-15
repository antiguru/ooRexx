# Task C — Formatting (Phase 2, Task 2.7)

Implement `ENGINEERING` form and the `FORMAT()` and `TRUNC()` builtins in
`rust/crates/rexx-num/src/format.rs`.

## What already exists

`Number::format(digits)` in `lib.rs` renders SCIENTIFIC form and is
correct — do not change its behaviour. Its rules, measured, are:

- exponential once the **adjusted** exponent (that of the most significant
  digit) is `>= digits`
- exponential once the **raw** exponent (that of the least significant
  digit) is `<= -(2 * digits + 1)`

The two ends use different exponents. They coincide only for a single-digit
mantissa, which is why probing with `1eN` values alone cannot distinguish
them: `1e-18` prints plain but `10e-19`, the same value, prints `1.0E-18`.

## What to add

1. **ENGINEERING form.** The exponent is forced to a multiple of 3, so
   `12345678901 * 1` is `12.3456789E+9` where SCIENTIFIC gives
   `1.23456789E+10`. Add a form-aware entry point rather than changing
   `format`; `Form` and `Settings` are in `settings.rs`.

2. **`FORMAT(number, before, after, expp, expt)`** — all arguments after the
   first optional. Reproduce the interpreter exactly, including which
   argument combinations raise errors and which error numbers.

3. **`TRUNC(number, n)`** — truncate to `n` decimal places, `n` defaulting
   to 0. Truncation, not rounding.

## Verification

Write Rexx probe programs and run them under `build/bin/rexx` to establish
what the interpreter actually does, then reproduce it. Do not rely on
documentation or on the ANSI standard — where they disagree with the
interpreter, the interpreter wins.

Cover at minimum: `FORMAT` with 0, 1, 2, 3, 4 and 5 arguments; negative
numbers; values needing rounding; values too wide for the requested `before`
(an error — find its number); `expp`/`expt` controlling exponent width;
`TRUNC` with and without a second argument, and on negatives.

Existing corpus programs `rust/corpus/num/form_notation.rex` and
`rust/corpus/num/format_trunc.rex` show some of the expected output.

## Interfaces

`Number` is in `rust/crates/rexx-num/src/lib.rs`, with private fields
`negative: bool`, `digits: Vec<u8>` (most significant first), `exponent: i32`
and `pub(crate)` helpers `assemble`, `truncated_to`, `adjusted_exponent`,
`round_to`. `Form`, `Settings` and `SettingsError` are in `settings.rs`.
Errors carry the interpreter's numbers via a `code()` method.
