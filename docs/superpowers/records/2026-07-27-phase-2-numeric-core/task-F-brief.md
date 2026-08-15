# Task F — The missing NUMERIC DIGITS upper bound

Two defects with one root. Both escaped Tasks 2.1 and 2.2, which are already
marked complete and reviewed, so treat their existing tests as insufficient
rather than as evidence.

## Defect 1 — `Settings::set_digits_str` has no upper bound

`crates/rexx-num/src/settings.rs`. The only limits are `value >= 1` and
`u32::try_from`, so `NUMERIC DIGITS 1000000000` is accepted.

The interpreter rejects it. Measured against `build/bin/rexx`:

    numeric digits  999999999   accepted; length(1/3) is 1000000001
    numeric digits 1000000000   error 26
    numeric digits 2147483647   error 26
    numeric digits 4294967296   error 26

So the maximum is exactly **999999999**, which is `MAX_EXPONENT`, and
exceeding it raises **26** — the same error the existing non-whole-number
path already raises, so no new `SettingsError` variant is needed.

`FUZZ` needs no separate bound: `set_fuzz_str` already rejects
`value >= self.digits`, so it is transitively capped. Confirm that rather
than assuming it — `numeric fuzz 2147483648` after `numeric digits 999999999`
gives error 33, which is the existing behaviour and is correct.

## Defect 2 — `Number::format` overflows `i32` at large `digits`

`crates/rexx-num/src/lib.rs:301`:

    if adjusted >= digits as i32 || n.exponent <= -(2 * digits as i32 + 1)

`2 * digits as i32` overflows once `digits` exceeds 1073741823. Reproduce:

    printf '2147483647|1|+|1e-30\n' > /tmp/d.txt
    cargo build --offline -p rexx-num
    ./target/debug/muldiv /tmp/d.txt
    # panicked at crates/rexx-num/src/lib.rs:301:57:
    # attempt to multiply with overflow

In release this wraps silently and picks the wrong display form, which is
worse than the panic.

Fixing defect 1 makes this unreachable *through `Settings`* — `2 * 999999999
+ 1` fits in `i32`. But `format` is `pub` and takes a bare `u32`, so an
external caller can still reach it. Make the expression not overflow for any
`u32` input (compute in `i64`, or compare without multiplying). Do not
"fix" it by clamping `digits` to `MAX_EXPONENT` inside `format` — that would
silently change the answer for a caller who passed something larger, rather
than being correct for it.

## Defect 3 — the same cast in `pow`

`crates/rexx-num/src/pow.rs:41`:

    if self_.digits.len() as i32 + self_.exponent > digits as i32 {

`digits as i32` wraps to a negative number above `i32::MAX`, which inverts
the comparison rather than panicking — a wrong answer, silently, in release
*and* debug. Same remedy: don't narrow `digits` to `i32`.

I swept the rest of the crate for this shape. `muldiv.rs` uses `checked_add`
throughout and is fine; the remaining `as usize` casts are widening on a
64-bit target and are fine. `addsub.rs:183` (`self.exponent + dropped as
i32`) needs a digit vector of over a billion entries to overflow and is not
reachable in practice — leave it, do not churn it.

## Verification

- A unit test per boundary: 999999999 accepted, 1000000000 rejected with
  code 26.
- A unit test that `Number::format` is well-defined at `u32::MAX` digits
  (any input, just no panic and no wrapped comparison).
- Run in **debug**, not just release — release wraps silently and both
  defects hide. `cargo test --offline --workspace` runs debug by default.
- All four differential sets must stay at 0 divergences:
  fmt_check 1116, fmt_check2 7080, fmt3 12136, edge5 640. Regenerate with
  `python3 crates/rexx-num/tests/gen-curated-sets.py <fmt|fmt2|fmt3|fmtedge>`
  and the oracle drivers in `crates/rexx-num/tests/`.

## Constraints

- Permitted files: `crates/rexx-num/src/settings.rs`,
  `crates/rexx-num/src/lib.rs`, `crates/rexx-num/src/pow.rs`, and their
  tests. Nothing else — in particular `format.rs` is being changed
  concurrently by another agent.
- Do not run git.
- Every cargo command takes `--offline`.
- Zero `unsafe`.
