# Task F report — the missing NUMERIC DIGITS upper bound

Status: **DONE**

## Defect 1 — `Settings::set_digits_str` had no upper bound

`crates/rexx-num/src/settings.rs`, `set_digits_str`. Added the cap alongside
the existing `value < 1` check:

```rust
if value < 1 || value > crate::MAX_EXPONENT as i64 {
    return Err(SettingsError::NotWholeNumber);
}
```

Used the existing `MAX_EXPONENT` constant (999,999,999) rather than a new
literal, and the existing `SettingsError::NotWholeNumber` (code 26) rather
than a new variant, per the brief. No changes needed to `set_fuzz_str` --
confirmed it is transitively capped by the existing `value >= self.digits`
check, since `digits` can now never exceed `MAX_EXPONENT`.

## Defect 2 — `Number::format` overflowed `i32` at large `digits`

`crates/rexx-num/src/lib.rs:301` (original line). The expression
`2 * digits as i32` narrowed the `u32` parameter to `i32` before doubling,
overflowing past 1,073,741,823. Fixed by shadowing `digits` with an `i64`
version immediately before the comparison and doing the whole comparison
(both the adjusted-exponent side and the raw-exponent side) in `i64`:

```rust
let digits = digits as i64;
if adjusted as i64 >= digits || n.exponent as i64 <= -(2 * digits + 1) {
```

No input is clamped -- `format` remains correct (not just non-panicking) for
every `u32` value, including `u32::MAX`. `digits` isn't referenced again
after that point in the function, so the shadow is safe.

## Defect 3 — the same cast shape in `pow.rs`

`crates/rexx-num/src/pow.rs:41`, in the private `as_whole` helper.
`digits as i32` wrapped negative above `i32::MAX`, inverting the
"does the exponent fit in `digits` digits" comparison -- a silent wrong
answer in both debug and release, not a panic. Fixed the same way, comparing
in `i64` instead of narrowing:

```rust
if self_.digits.len() as i64 + self_.exponent as i64 > digits as i64 {
    return None;
}
```

Left `muldiv.rs` (already `checked_add` throughout) and `addsub.rs:183`
untouched, per the brief -- the latter needs a digit vector over a billion
entries long to overflow and isn't reachable in practice.

## Boundary values confirmed against `build/bin/rexx`

Re-ran every claim in the brief rather than trusting it:

```
999999999   => accepted; length(1/3) = 1000000001
1000000000  => <E26>
2147483647  => <E26>
4294967296  => <E26>
numeric digits 999999999; numeric fuzz 2147483648 => <E33>
```

All matched the brief exactly. Also reproduced the pre-fix panic by hand
before touching code:

```
printf '2147483647|1|+|1e-30\n' > /tmp/d.txt
./target/debug/muldiv /tmp/d.txt
# thread 'main' panicked at crates/rexx-num/src/lib.rs:301:57:
# attempt to multiply with overflow
```

Post-fix, the same input produces
`2147483647|1|+|1e-30=1.000000000000000000000000000001` with no panic.

## Tests added

- `crates/rexx-num/tests/settings.rs::digits_is_capped_at_max_exponent` --
  999999999 accepted; 1000000000, 2147483647, 4294967296 all rejected with
  `SettingsError::NotWholeNumber` (code 26).
- `crates/rexx-num/tests/parse.rs::format_does_not_overflow_at_extreme_digits`
  -- `Number::format` at `digits = 2147483647` and `digits = u32::MAX`,
  checked against the actual (correct) expected string, not just "doesn't
  panic". Added here rather than `tests/format.rs` because that file's
  `format_form`/`format_with`/`trunc` live in `format.rs`, which is off
  limits (concurrent edit); `parse.rs` is the existing home for the plain
  `Number::format` defined in `lib.rs`.
- `crates/rexx-num/tests/pow.rs::an_exponent_fits_within_digits_beyond_i32_max`
  -- `pow("0", "5", 3_000_000_000)` and `pow("0", "7", u32::MAX)` both `"0"`.
  Chose a zero base deliberately: it hits `pow`'s early zero-base return
  before the square-and-multiply loop, so the test stays fast even though
  `digits` is enormous (the squaring loop's working precision scales with
  `digits`, which would otherwise try to allocate a multi-gigabyte digit
  vector). Verified by hand that the pre-fix comparison would have wrongly
  rejected these as non-whole (`digits as i32` wraps to a large negative
  number for both, e.g. `3_000_000_000i32` wraps to `-1_294_967_296`, making
  the "exceeds digits" check spuriously true for every exponent).

## Verification

- `cargo test --offline --workspace`: all passing, 0 failed (134 tests total
  in this run, up from the 127 baseline -- the rise is the 3 tests above plus
  concurrent additions from the other agent working on `format.rs`).
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: clean,
  exit 0.
- The four differential FORMAT sets, regenerated fresh and diffed against
  `build/bin/rexx` via `data-format-oracle.rex` / `fmt-check`:
  - `fmt`: 1800 cases, 0 divergences
  - `fmt2`: 6720 cases, 0 divergences
  - `fmt3`: 12136 cases, 0 divergences (matches the brief's count exactly)
  - `fmtedge`: 640 cases, 0 divergences (matches the brief's count exactly)

  Note: my `fmt`/`fmt2` case counts (1800/6720) don't match the brief's
  quoted 1116/7080. `fmt3` and `fmtedge` match exactly, so this is most
  likely a difference in how the brief author counted (e.g. after some
  dedup/filter step not in the current `gen-curated-sets.py`) rather than a
  wrong generator on my end -- I ran the exact command given
  (`python3 crates/rexx-num/tests/gen-curated-sets.py <name>`) with no
  modification. Since the requirement was 0 divergences, and that held for
  the actual case count each run produced, I'm not treating this as a
  blocker, but flagging it in case it's a sign the generator drifted since
  the brief was written.

## Constraints honoured

- Touched only `crates/rexx-num/src/{settings.rs,lib.rs,pow.rs}` and their
  tests (`tests/{settings.rs,parse.rs,pow.rs}`). Did not open `format.rs`
  for writing.
- No `unsafe` introduced (the workspace already forbids it at the lint
  level).
- No git commands run.
- Did not clamp any input to `MAX_EXPONENT` inside `format` or `pow` --
  both remain correct (not just panic-free) for the full `u32` domain.

## Concerns

None that block. The one item worth double-checking independently is the
fmt/fmt2 case-count mismatch noted above -- it doesn't affect correctness
(0 divergences either way) but is worth someone confirming the generator
hasn't silently changed shape since the brief's numbers were captured.
