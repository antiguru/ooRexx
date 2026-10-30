# Release baseline pins

These pins preserve the committed pre-8.8 and a+b runtimes for the controller-owned all-program performance comparison. Both source worktrees were clean before and after their builds, and their HEAD revisions matched the requested commits. Both accepted builds exited zero under an 8 GiB memory cap with a 530-second Cargo deadline, using `CARGO_INCREMENTAL=0` and the dedicated release-only target `/tmp/codex-p9d-t88-perf-target`. No benchmark, interpreter, oracle, or test execution was performed by this baseline task.

| Pin | Revision | Binary | SHA-256 | Build time |
| --- | --- | --- | --- | --- |
| Pre-8.8 | `0c2ff21ae1844e86c1937b58fc82f080fc43b5ab` | `/tmp/codex-p9d-t88-pins/8.8-base-rexx` | `4913ad067898fd6a786c12a696b2a21b0d066ad6b019c73273f78e30c16f5c58` | 124.829 s |
| 8.8 a+b | `b300b9c2541cc70c40c62b861438efe415e3c526` | `/tmp/codex-p9d-t88-pins/8.8ab-rexx` | `361392c36b1207a9174dfab7b32b2acb212d2c69e53f3e76f106e3a1cdad42dd` | 99.390 s |

The build command was `cargo build -j4 --release -p rexx-exec --bin rexx` from each worktree's `rust` directory. [8.8-base.json](8.8-base.json) and [8.8ab.json](8.8ab.json) preserve the exact driver commands, environment, Git trees, versions, status, timestamps, elapsed time, source cleanliness, and binary hashes. The toolchain was rustc 1.99.0 (`b940084d7`, LLVM 23.1.1) on `x86_64-unknown-linux-gnu`, with Cargo 1.99.0 (`5f94df478`). Separate `.stdout` and `.stderr` files preserve each build's output, and [build-pin.py](build-pin.py) preserves the bounded driver.

## Rejected cache reuse and recovery

The first a+b command completed in 0.115 seconds and copied the base binary without recompiling the changed runtime input. Cargo's retained dependency file used relative source paths, and both worktrees' `pow.rs` mtimes preceded the base compilation's fingerprint timestamp. This is evidence of invalid cross-worktree freshness reuse, so its JSON and logs were retained as `8.8ab-rejected-cache-reuse.*` and its binary as `/tmp/codex-p9d-t88-pins/8.8ab-rejected-cache-reuse-rexx`. The rejected record contains the original output paths from before relocation, and [cache-recovery.json](cache-recovery.json) names the retained artifacts.

The checked Rust, build, profile, lockfile, Cargo configuration, and toolchain path diff is preserved in [checked-input-diff.txt](checked-input-diff.txt). Its only changed production source linked into `rexx` is `rust/crates/rexx-num/src/pow.rs`, whose patch is preserved in [linked-production-input.patch](linked-production-input.patch). `memcap 8G env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/codex-p9d-t88-perf-target cargo clean --release -p rexx-num` exited zero and removed seven files totaling 1.6 MiB from this task's cache. The accepted retry's stderr confirms rebuilds of `rexx-num`, `rexx-core`, `rexx-parse`, `rexx-classes`, `rexx-api`, and `rexx-exec`, and its binary hash differs from the rejected copy.

The initial free-space check reported 12,220,981,248 bytes available on `/tmp` after worktree creation. The final check reported 11,821,051,904 bytes available after both accepted pins and the rejected artifact were preserved. The source worktrees and release cache remain available for controller validation, alongside the existing c/e1/e2 pins. Shared metadata was neither staged nor committed by this task.
