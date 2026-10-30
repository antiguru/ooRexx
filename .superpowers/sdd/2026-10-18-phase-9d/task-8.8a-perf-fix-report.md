# Task 8.8a: Performance fix round 1

`pow.rs::as_whole` now reuses `plain_integer(ARGUMENT_DIGITS as u64)` when the rounded exponent is nonnegative. The existing rounding, zero, DIGITS width and integrality guards precede this choice. Fractional-zero spellings retain `whole_value(ARGUMENT_DIGITS)`. The change targets the outlined conversion attributed in the brief to the original arithmetic regression of +0.34824337%, whose measurements remain preserved.

## Equivalence and coverage

For a nonzero canonical coefficient of length `L` and nonnegative decimal exponent `E`, the integer width is `L + E`. With `ARGUMENT_DIGITS = 18`, `plain_integer(18)` rejects exactly widths above 18 and otherwise accumulates a magnitude at most `999999999999999999`, then applies the sign. `whole_value(18)` accepts the same widths, since its direct path uses `unsigned_value` with that width and magnitude bound, while its longer-coefficient path truncates to 18 digits and increases `E` by the omitted length, preserving the rejected width. No carry can reduce that width, and accepted magnitudes fit in `i64`, so the signed results agree.

The test commit adds scaled coefficients at the accepted width, their negative counterparts, rejected scaled and machine-width integers, and a rounding carry across the width boundary. The zero base distinguishes an accepted positive exponent from an accepted negative exponent through the existing underflow error. Existing tests retain fractional trailing zeros, rejected nonzero fractions, DIGITS rounding, negative powers, zero, huge precision and overflow coverage. The original semantic regressions were already committed and historically red, as recorded in `task-8.8a-report.md`, and this repair introduces no Deviation.

## Commits and verification

The worktree was `plan/rust-rewrite` at `15ee94dd3d65de95466ae76a43a00276500cca2a`, with clean Rust source. Test commit `b16b854c03132fb51f31a462eb1a633121914566` preceded implementation commit `f56dfd25e4394ca32c7cfb18733d640a4f584ed9`. All successful checks measured the committed implementation with source frozen. Parent-owned performance records and the untracked archive were excluded from staging.

Durable evidence is in `docs/superpowers/records/2026-10-18-phase-9d/t8.8/pow/perf-fix-r1/`. Each command JSON records the exact child command, working directory and environment, and the corresponding `.stdout`, `.stderr` and `.status` files retain separate descriptors. The Python drivers are archived there under their original basenames. Cargo checks used `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, an outer `memcap 8G`, and `timeout -k 5 540`.

| Command from `rust/` | Status | Observed output |
| --- | --- | --- |
| `memcap 8G python3 /tmp/t88a-perf-driver.py rexx-num-tests` runs `cargo test -j 4 --locked -p rexx-num` | 0 | 196 passed, 0 failed, 3 ignored. Power target: 12 passed. |
| `memcap 8G python3 /tmp/t88a-perf-driver.py fmt` runs `cargo fmt --all --check` | 0 | No output. |
| `memcap 8G python3 /tmp/t88a-perf-driver.py clippy` runs `cargo clippy -j 4 --locked -p rexx-num --all-targets -- -D warnings` | 0 | Finished `dev` profile. |
| `memcap 8G python3 /tmp/t88a-perf-driver.py release-build` runs `cargo build -j 4 --locked --release -p rexx-exec --bin rexx` | 0 | Finished release profile in 1m 33s. |
| `python3 /tmp/t88a-pin-witness.py oracle` | 0 | Existing power corpus witness, output below. |
| `memcap 2G python3 /tmp/t88a-pin-witness.py crate` | 0 | Same witness output, empty stderr. |
| Separate `cmp` commands for oracle/crate stdout, stderr and status | 0 each | No output. Exact absolute commands are in `verification-summary.json`. |

The initial sandboxed test launch exited 1 before Cargo because `memcap` could not create its cgroup. Its output remains in `rexx-num-tests-sandbox-failure.*`, while the redirected escalated retry stayed pending until interrupted and produced no successful check. The successful retries used plain approved `memcap` invocations with Python capturing descriptors internally. No check or approval remains live.

The existing `power_integral_exponent.rex` witness ran from separate fresh directories documented in the command JSON files. The oracle child applied the documented 1 GiB address-space bound through `setrlimit(RLIMIT_AS)` before `timeout -k 5 20`, with the reference library path, after reading `rust/corpus/oracle-crashes.txt`. The crate used the preserved release pin under `memcap 2G` and the same timeout and library path. Both printed `2`, `2`, `8`, `0.5`, `8`, `1`, `1`, `8`, `8`, `8`, `2`, `2`, `2.306488E+301029995`, one value per line, with empty stderr and status 0.

## Preserved pins and handoff

The pre-repair pin `/tmp/codex-p9d-t88-pins/8.8e2-perf-r1-rexx` was rehashed and preserved, with SHA-256 `470dcb41db61ab7a4d64f3276ce28e539d5686d130be245a4ec7b30e6d6725ac` and recorded source `1233b81da730142057fcf810e324d10d0e55c109`. The new pin `/tmp/codex-p9d-t88-pins/8.8a-perf-r1-rexx` has SHA-256 `ecb06e984ed736233ce0e2ef779b74941488ae64fdec10333dfb2215c309198a`, matching the release build output, at source `f56dfd25e4394ca32c7cfb18733d640a4f584ed9` and Rust tree `68a291ee5089d9357b51b238056aaefcfac1500f`. `pin-provenance.json` records source-file hashes and compiler SHA-256 `f3834d26669b03f6855fa54bd2381443123e860167bc0c7a8dd137e5cf6e5e4f`, with `rustc 1.99.0 (b940084d7 2026-09-28)` and the exact build command in `release-build.command.json`. Scoped review, the matched all-program performance comparison and broad final gates remain controller-owned obligations, so this report makes no performance acceptance claim.
