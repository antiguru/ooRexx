# Task G — Division estimation in `long_divide`

Status: **DONE** (partial win — 28.5% faster, not yet C++ parity; next bottleneck identified below).

## What changed

Files touched (only the permitted ones):

- `rust/crates/rexx-num/src/muldiv.rs`
- `rust/crates/rexx-num/src/pow.rs`

Two changes, made and measured separately:

1. **Quotient-digit estimation.** `long_divide`'s inner loop no longer finds each
   quotient digit by up to nine full-width subtract/compare/strip passes. It now
   uses the interpreter's own estimate, exactly as `divide_power` already did:
   `div_char = divisor[0]*10 + divisor[1] + 1` (computed once per call), guess
   `m = max(multiplier*10 / div_char, 1)`, then `subtract_multiple` removes
   `m * divisor` in one pass. Guesses err low, never high, so the digit
   accumulates from one or two passes instead of up to nine. The equal-remainder
   case still zeroes the remainder and completes the digit, exactly as the old
   `sub_in_place` path did. `subtract_multiple` moved from `pow.rs` to
   `muldiv.rs` as `pub(crate)` and is now shared by both call sites (no copy);
   `pow.rs` imports it. The now-unreferenced `cmp_digits` and `sub_in_place`
   were deleted. Bench effect: 1.9666 s → 1.4815 s.

2. **`strip_leading`'s drain became an index offset** — the brief's sanctioned
   follow-up, done only after change 1 was measured. Inside `long_divide` the
   live remainder is now `rem[start..]`; leading zeros are skipped by advancing
   `start` instead of `Vec::drain`'s memmove on every pass. One `split_off` at
   return preserves the exact returned remainder. `strip_leading` itself is
   unchanged and still used by `divide_power`. Bench effect: 1.4815 s → 1.4067 s.

What did NOT change: the outer loop structure, the number of quotient digits
produced, the early-stop on an even division (`1 / 1` is `1`, verified by the
curated sets), the returned `(quotient, remainder, shift)` contract, and the
absence of any rounding inside `long_divide`. `divide_power`'s behaviour is
untouched. Zero `unsafe`.

## Benchmark (`cargo bench --offline -p rexx-num --bench arith`)

| | Lower | Mean | Upper |
|---|---:|---:|---:|
| Before (Task E, taken immediately before this task) | 1.9634 s | **1.9666 s** | 1.9695 s |
| After estimation only | 1.4789 s | **1.4815 s** | 1.4843 s |
| After estimation + offset | 1.4047 s | **1.4067 s** | 1.4089 s |

C++ `interpreter/arith` baseline: **1.1570 s**. The gap closed from 1.70x to
**1.22x** — not parity. The benchmark's built-in answer assertion
(`4629643519330627.7808`) passed on every sample.

## Differential verification — all at 0 divergences

Curated sets (oracle `build/bin/rexx` + `data-addsub-oracle.rex`, harness
`rust/target/release/muldiv`), re-run after each of the two changes:

| Set | Cases | Divergences |
|---|---:|---:|
| muldiv | 17,424 | 0 |
| md2 | 20,184 | 0 |
| addsub | 8,712 | 0 |
| addsub2 | 8,112 | 0 |
| pow | 2,112 | 0 |
| cmp | 32,368 | 0 |

Fresh randomised sets, seeds not used by any earlier task:

| Set | Cases | Divergences |
|---|---:|---:|
| seed 86753091, full operator mix | 25,000 | 0 |
| seed 271828182, full operator mix | 25,000 | 0 |
| seed 314159265, 200k generated, filtered to `/` `%` `//` only | 85,772 | 0 |

The 85,772-case division-only set was additionally run under the **debug**
build, where `subtract_multiple`'s `debug_assert` (an over-guess would drive
the remainder negative) is armed on every subtraction: 0 divergences, no
assert fired.

## Tests and lints

- `cargo test --offline --workspace`: **134 passed, 0 failed** (up from 127 at
  Task E; the increase is the other agent's concurrent work, nothing failed).
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: clean.

## Profile afterwards (perf, 9.5k samples over the bench binary)

    25.78%  muldiv::long_divide        (was 48.20%)
    18.72%  muldiv::mul_magnitudes     (was 10.22%)
     6.50%  addsub::add_signed
    ~15.0%  malloc/free/memmove/memcmp in libc, summed
     3.62%  muldiv::div
     3.17%  Number::round_to

`long_divide` is still the top symbol but its absolute cost fell ~62% (48.2% of
1.97 s → 25.8% of 1.41 s). What remains in it is the per-digit scalar work
itself: ~21 outer steps per DIGITS-20 divide, each doing a slice compare, a
~20-byte `subtract_multiple`, and the even-stop zero scan. The clearest next
targets, in order: `mul_magnitudes` (18.7%, O(n²) per-digit u16 grid — now
nearly as expensive as division), allocator traffic (~15% across four libc
symbols, one `Vec` per intermediate result), and vectorising/word-packing the
per-digit inner loops. All are representation-level work beyond this task's
sanctioned scope.

## Concerns

- Parity (D9) is still not met on this dimension: 1.4067 s vs 1.1570 s, with
  the Rust side still paying no interpreter overhead. The remaining gap looks
  like representation cost (digit-per-byte + Vec-per-result), not algorithm.
- The correctness of the estimate rests on the same never-overshoot invariant
  `divide_power` already relies on (divisor's first two digits + 1). It is
  enforced by a `debug_assert` exercised across ~86k division cases in debug;
  release builds would wrap a `u8` if it were ever violated.
- Both oracle-facing behaviours called out in the brief (early stop on even
  division; exact remainder for `%` and `//`) are covered by the curated
  muldiv/md2 sets and the 85k random division set, all at 0.

No git commands were run; the work is left uncommitted for central staging.
