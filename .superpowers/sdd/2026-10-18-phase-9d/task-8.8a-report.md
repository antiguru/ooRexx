# Task 8.8a: Integral power exponents

The power conversion now removes insignificant fractional zeros before converting an integral exponent. It reuses the existing whole-number converter after the original DIGITS rounding and integrality checks. The converter also enforces the documented platform whole-number limit. Existing rounding and sign behavior remain outside this change.

## Changes and evidence

* Regression commit: `bfdb4520f`; implementation commit: `3148553fe`.
* The local reference power section defines a whole-number exponent bounded by the platform limit. `rexx-num/src/pow.rs` now calls `whole_value(ARGUMENT_DIGITS)` after the existing guards.
* The controller ran the committed regression with `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target memcap 8G timeout 540 cargo test -j 4 --locked -p rexx-num --test pow`. Exit 101: the trailing-zero exponent and platform-limit regressions failed, while existing tests passed.
* At `3148553fe`, the implementer reported exit 0 for `cargo test -j4 --locked -p rexx-num`, including the power target, `cargo fmt --all --check`, `cargo clippy -j4 --locked -p rexx-num --all-targets -- -D warnings`, and `cargo test -j4 --locked -p rexx-parse --test sourceline_oracle`. Cargo commands used `memcap 8G env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target timeout -k 5 540`.
* The implementer built `rexx` with the same capped prefix and `cargo build -j4 --locked -p rexx-exec --bin rexx`, exit 0. The controller copied the final corpus program into the fresh directory `/tmp/codex-t88-pow-final.QAyFoW` and ran the oracle with its documented address-space limit, library path, and timeout. The crate ran under `memcap 2G`, the same library path, and a timeout.
* Both final processes exited 0. Separate `cmp` checks of stdout and stderr exited 0. Durable descriptors and statuses are `docs/superpowers/records/2026-10-18-phase-9d/t8.8/pow/{oracle,crate}-power.{stdout,stderr,status}`.
* The sourceline fixture was captured through the sanctioned oracle driver and filtered to its source records. The final witness excludes the unrelated negative carry-rounding shape, where the oracle has a documented sign defect. No new Deviation was introduced.
* Read-only scoped review: `task-8.8a-review.md`, implementation and quality approved with no correctness findings. Its stale-report and old-witness evidence gaps are resolved by this report and the final descriptor capture.

## Remaining batch checks

Task 8.8 remains open. Full corpus, coverage, collection stress, owner/refusal checks, whole-suite per-test comparison, workspace tests and all-program performance validation are controller-owned batch checks. They have not been claimed as passing for this slice. Preserve the target cache for later slices.
