# Final validation blocked at release artifact identity

Validation froze `8b7767ed3fda908fffe75181b2c8bd12dd7509f8`, whose recorded Rust tree matches runtime source `9d919b7c3832c55edc3a0a07974309f092102201`. Freeze, real preflight, toolchain/source provenance, formatting, and workspace all-target clippy passed with status zero. The release Cargo no-run build also exited zero after 115.9558 seconds, with complete fresh CLI, corpus, and suite artifacts in its retained JSON output. The approved driver then exited one because that CLI's bytes differ from the supplied immutable pin, so no release or workspace test ran.

| Completed action | Status |
| --- | --- |
| `freeze` | 0 |
| `preflight` | 0 |
| `provenance` | 0 for every child |
| `fmt` | 0 |
| `clippy` | 0, 9.3845 seconds |
| `release-build` | Cargo child 0, driver 1 at artifact assertion |

The Cargo-built `/tmp/codex-p9d-t88-target/release/rexx` hashes to `9c3d09f9f9e39810985b1200007035488a3eba9d00ab6ee7382b6d7aea1b4d06` and is 67,515,656 bytes. The immutable `/tmp/codex-p9d-t88-pins/8.8-final-fix-r2-rexx` remains `d9210f4b9f17498ff59da85b6226a532aebaf0765bd8e3fb482025d54ff02145` and 67,517,088 bytes. Both are release artifacts, while the fresh Cargo CLI reports empty package features, optimization level 3, debug information 2, disabled debug assertions, disabled overflow checks, and a non-test profile. Retained rustix release fingerprints differ in the `pty` feature, which the crate declares in a dev-dependency, but this observation is only a diagnosis clue pending explicit build-variant reconciliation.

Raw `release-build.stdout`, `release-build.stderr`, and `release-build.json` retain the successful Cargo command before the failing guard. `release-build-artifact-mismatch.json` retains every fresh relevant executable, profile, feature list, size, and hash alongside the unchanged immutable pin. No `release-artifacts-verified.json` marker was written, and no guard was weakened or command retried. `progress.json` records the blocker, with no live tool session and the tracked/index/source freeze still active.

Corpus and suite denominators, per-test suite deltas, fresh workspace target/test counts, doctests, M6 pinning, and completeness audit remain unexecuted. The final config SHA256 is `008d05b7b4986a58fa11560c8373fcd335a4357b84102efd0a4893e89f463475`, and `input-freeze.json` retains the source/profiling/pin/index identities. The user ZIP, tracked source, HEAD, index and tracked reports were not modified. Root must reconcile the build-artifact identity before further execution, while performance-budget acceptance remains a separate pending user decision.
