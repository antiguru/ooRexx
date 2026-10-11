# Task 1 report

Commit bee47022f on plan/rust-rewrite (base 180e5b41e).

## What changed
* `group_runner.rs`: new `read_bytes`; `read_lossy` wraps it (read-only use). `skip`, `without_rxfuncquery` (now `pub`) and the moved `mark_starts` match and rewrite `&[u8]` (`trim_ascii_start`, byte windows). `mark_starts`, `started`, `START_LINE`, `START_MARKED` moved from `concurrency_tests.rs::group_runs::whole_groups`; the call sites import them. `whole_groups` still uses `read_lossy` only for reading (`units`, `source_test_names`).
* `concurrency_tests.rs`: new top-level (ungated) `mod byte_exact_copies` with three tests: skip over a copy of C2X.testGroup changes only the renamed method's line (byte compare per line, file asserted non-UTF-8); `without_rxfuncquery` over ooTest.frm with 0xFF in a comment keeps it; `mark_starts` keeps a leading 0xFF.
* `rust/corpus/phase-9.txt` (comment header only), registered after `phase-8.txt` in corpus.rs, coverage.rs, collect_stress.rs, trace_oracle.rs, ir_recorded.rs.
* `docs/superpowers/plans/phase-9a-gate.md` with `## Perf pin`.

## Red before, green after
Before (HEAD's group_runner.rs plus `pub` and lossy shims for the moved fns; restored after):
`cargo test --release -j 4 -p rexx-exec --test concurrency_tests byte_exact` -> 3 FAILED
* mark_starts: left 239 right 255 (leading 0xFF became U+FFFD)
* without_rxfuncquery: panic "the copied ooTest.frm: InvalidData, stream did not contain valid UTF-8"
* skip: "line 64 changed", 0xAA bytes became 239,191,189
After: same command, exit 0, 3 passed.
Note: a first draft of the skip test panicked "a test method" because C2X's methods are quoted (`::method "test_1"`); fixed in the test, not the code.

## Checks (exit status unpiped)
* `cargo fmt` -> 0
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` (CARGO_INCREMENTAL=0) -> 0
* `memcap 8G cargo test --release -j 4 -p rexx-exec --test concurrency_tests byte_exact` -> 0 (this also compiles concurrency_tests)
* `memcap 8G cargo test --release -j 4 -p rexx-exec --test corpus --test coverage --test collect_stress --test trace_oracle --test ir_recorded subset_file` -> 0, five `reads_every_phase_subset_file` tests ok (they accept the empty list; no placeholder program was needed)
* Not run: full suite, REXX_CORPUS_GATE, the gated whole_groups parts.

## Perf pin
* `git archive a98238be5` whole commit, extracted to /tmp/claude-1000/p9/pin/src, touched, `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/p9/pin/target memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`, exit 0, 1m21s, `Compiling rexx-exec` present (/tmp/claude-1000/p9/pin-build.log).
* Binary: /tmp/claude-1000/p9/pin/rexx-run, 58,283,928 bytes, sha256 `c97bf40a561cc46a4fa41ac64e7c8a16d42483f3a1869774a1c5e03610e49667`. Smoke: `say 1+1` printed 2, rc 0.
* /tmp/claude-1000/p9/pin/{src,target} deleted. df before: /tmp 46G free, inodes 830k free.

## Inferred, not run
* `without_rxfuncquery` byte semantics for whitespace: `trim_ascii_start` trims ASCII whitespace only, where `str::trim_start` trimmed Unicode whitespace; ooTest.frm has no non-ASCII leading whitespace (the existing assertions would fire otherwise, and the test passes on the real file).
* Unquoted method names with non-ASCII bytes cannot match (as before, names were ASCII-only when unquoted); quoted names compare via lossy conversion.
* Whole_groups gated code compiles but was not run.
* Note: the repo's `grep` is ugrep with `-I`, so plain grep on C2X.testGroup shows nothing; use `-a`.
