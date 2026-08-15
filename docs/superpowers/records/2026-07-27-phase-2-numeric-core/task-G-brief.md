# Task G — Make division stop being the bottleneck

Phase 2's parity gate failed: the Rust arithmetic runs the `arith.rex`
workload in 1.98 s against the C++ interpreter's 1.15 s for the whole
program, so 1.72x slower while doing strictly less work (no lexer, parser,
dispatch or variable lookup). Task E has the numbers; do not re-measure the
baseline, it was taken immediately before this task was written.

## Where the time actually goes

`perf record` over the benchmark binary, 30k samples:

    48.20%  rexx_num::muldiv::long_divide
    10.22%  rexx_num::muldiv::mul_magnitudes
     4.61%  rexx_num::addsub::add_signed
     3.06%  rexx_num::muldiv::div
     2.55%  rexx_num::Number::round_to
     ~10.5% malloc/free/memmove in libc, summed

So this is one function, not a diffuse representation problem. Task E's
report speculated that `Vec<u8>`-per-digit and per-iteration allocation were
the cause; the profile says otherwise — allocation is about a tenth of the
time and division is nearly half. Do not act on that speculation.

## The cause

`long_divide` (`crates/rexx-num/src/muldiv.rs:200`) finds each quotient digit
by **repeated subtraction**: up to nine full-width `sub_in_place` +
`cmp_digits` + `strip_leading` passes per quotient digit. At DIGITS 20 that
is roughly 95 full-width subtractions per division. `strip_leading` also does
`Vec::drain(..lead)`, a memmove, on every one of those passes.

## The fix, which is already in this repo

`divide_power` (`crates/rexx-num/src/pow.rs:143`) is the C++ algorithm,
ported for `dividePower` in an earlier task. It estimates each quotient digit
from the divisor's leading two digits:

    div_char = divisor[0] * 10 + divisor[1] + 1     // +1 makes the guess err low, never high
    m        = max(multiplier * 10 / div_char, 1)
    subtract_multiple(&mut left, divisor, m)        // subtract m*divisor in ONE pass

That converges in one or two inner passes instead of up to nine. Porting it
into `long_divide` moves the Rust *toward* the C++ implementation, not away
from it — the interpreter's own division works this way.

Share the helper rather than copying it: `subtract_multiple` and
`strip_leading` should be used by both call sites. You may modify `pow.rs`
for that purpose. Consider also whether `strip_leading`'s `drain` can become
an index offset, but only after the main change is measured — do not bundle
speculative micro-optimisations.

## What must not change

- `long_divide` returns `(quotient, remainder, shift)` and **stops early
  when the division comes out even**, which is observable: `1 / 1` is `1`,
  not `1.00000000`. Preserve this exactly.
- `%` and `//` depend on the exact remainder, not just the quotient.
- This project's governing rule: a reformulation is safe exactly when it
  preserves *where rounding happens*. Six ports in this phase failed by
  moving a rounding point. `long_divide` itself does no rounding — its
  caller does — so keep it that way and do not let the estimation change how
  many digits are produced.

## Verification — this is the gate, not the tests

All differential sets must be at **0 divergences** afterwards. Division and
remainder are heavily covered, which is what makes this change safe to make
at all:

    python3 crates/rexx-num/tests/gen-curated-sets.py <muldiv|md2|addsub|addsub2|pow|fmt3|fmtedge>
    build/bin/rexx crates/rexx-num/tests/data-addsub-oracle.rex cases.txt   # arithmetic
    build/bin/rexx crates/rexx-num/tests/data-format-oracle.rex cases.txt   # fmt sets
    ./target/release/muldiv cases.txt      # arithmetic
    ./target/release/fmt-check cases.txt   # fmt sets

Also generate fresh randomised cases and check those too:
`cargo run --offline -p rexx-num --bin gen-cases -- <seed> <n>`. Use seeds
nobody has used yet; the point is cases this change has not been tuned on.

Then re-run `cargo bench --offline -p rexx-num --bench arith` and report the
new mean with its range.

## Constraints

- Permitted files: `crates/rexx-num/src/muldiv.rs`, `crates/rexx-num/src/pow.rs`,
  and their tests. Nothing else.
- Do not run git.
- Every cargo command takes `--offline`.
- Zero `unsafe`. The workspace denies it at the root.
- Report the speedup honestly. If it does not reach parity, say so and say
  what the profile looks like afterwards — a partial win with a clear next
  bottleneck is a good outcome, and inventing a better number is not.
