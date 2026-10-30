# Task 8.8a: Performance repair review

## Spec compliance

**Verdict: Compliant for the scoped repair.** The only runtime change selects `plain_integer(ARGUMENT_DIGITS as u64)` for nonnegative decimal exponents at `rust/crates/rexx-num/src/pow.rs:58`. The preceding rounding, zero, DIGITS-width and integrality checks remain intact at `pow.rs:29`, and negative decimal exponents retain `whole_value` at `pow.rs:61`. The source and test diff adds no dependency, unsafe site, copied accumulation loop, removed comment, or Deviation. Matched all-program performance acceptance and broad final gates remain controller-owned and unproven, as stated in `task-8.8a-perf-fix-report.md:33`.

## Checked invariants

The existing helper contract supports the equivalence argument in `task-8.8a-perf-fix-report.md:7`. For a nonzero canonical coefficient of length L and nonnegative decimal exponent E, `plain_integer(18)` rejects L + E above 18 before accumulation, then applies the sign, as checked at `rust/crates/rexx-num/src/lib.rs:427`. `whole_value(18)` either directly uses `unsigned_value` or first reduces the coefficient length while increasing E by the same amount, so its width rejection agrees, as checked at `lib.rs:712` and `lib.rs:760`. The 18-digit maximum fits in positive `i64`, making both the accumulation and final negation safe, while zero returns before either converter at `pow.rs:32`.

The tests distinguish accepted scaled 18-digit exponents, negative powers and rejected 19-digit or machine-width values at `rust/crates/rexx-num/tests/pow.rs:127`. The final assertion at `tests/pow.rs:137` preserves rejection after an all-nines rounding carry crosses the DIGITS width. Existing coverage checks ordinary negative powers and zero at `tests/pow.rs:13` and `tests/pow.rs:23`, huge precision at `tests/pow.rs:70`, and fractional-zero normalization with an otherwise overflowing coefficient at `tests/pow.rs:104`. The fractional-zero branch stays unchanged, so this repair preserves the earlier correction instead of attempting conversion before normalization.

## Evidence checked

Stored successful command metadata identifies implementation `f56dfd25e4394ca32c7cfb18733d640a4f584ed9`, `CARGO_INCREMENTAL=0`, the assigned target directory, Cargo `-j 4`, and a 540-second timeout in `docs/superpowers/records/2026-10-18-phase-9d/t8.8/pow/perf-fix-r1/rexx-num-tests.command.json:1`. The raw test results total 196 passed, zero failed and three existing allocation-heavy ignores, with the power target's 12 passes at `rexx-num-tests.stdout:227` and ignore reasons at `rexx-num-tests.stdout:146`. Formatting, all-target clippy with denied warnings, and the release build each have status zero, with no warning diagnostics, in their corresponding `.status` and `.stderr` files. The first failed attempt contains only the cgroup error at `rexx-num-tests-sandbox-failure.stderr:1`, so it supplies no Cargo result and is superseded by the successful recorded run.

The existing power witness has byte-identical oracle/crate stdout, empty stderr and status zero in `oracle-power.stdout:1`, `crate-power.stdout:1`, and their separate descriptor/status records. The witness command records and archived driver show separate fresh directories, the documented oracle address-space bound and timeout, and the preserved crate pin at `t88a-pin-witness.py:42`. A read-only SHA-256 check independently confirmed both runtime pins and both changed Rust files against `pin-provenance.json:1`, including the candidate digest `ecb06e984ed736233ce0e2ef779b74941488ae64fdec10333dfb2215c309198a`. These checks establish the recorded source and witness evidence, without establishing the pending performance result.

## Code quality

**Verdict: Approved. No high-confidence correctness or maintainability findings.** The repair reuses the existing conversion contract with a narrow condition and preserves the general normalization path. Focused inspection outside the diff addressed converter equivalence and retained regression coverage, while the beginning of `as_whole` was read because the diff starts mid-function. No tests were rerun, and no source, index, HEAD or branch state was changed during review.
