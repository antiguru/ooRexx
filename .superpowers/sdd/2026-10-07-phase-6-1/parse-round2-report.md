# Phase 6.1 parse round 2 report

Status: DONE.

## Commits

* `23d78ec1b` Phase 6.1 parse round 2: the collector sums survivor bytes (`rust/crates/rexx-core/src/heap.rs` only).
* `c28a63f37` Phase 6.1 parse round 2: gate record, criterion 7 holds (`docs/superpowers/plans/phase-6-1-gate.md` `## Parse round 2` and criterion 7's line, roadmap row 6.1 in `2026-07-27-rust-rewrite.md`).

## The change

Design (A), the survivor sum, with one addition:
* The mark loop sums `held_bytes` for every survivor in release, as the debug build already did. After the sweep, `held_bytes` is set to that sum, so the sweep no longer reads a freed body.
* A string survivor (`Body::Text`) reaches nothing, so the mark loop reads `bytes.heap_len()` and skips the call to `Body::trace`. Every other body traces as before and then reads `held_bytes()`.
* Debug only: the sweep still sums the freed bytes, releases them from the running figure, and asserts the result equals the survivors' sum. This is the original assertion, so a missed charge site still fails a debug run.
* The resurrect loop sums every resurrected UNINIT body in release as well.

Design (B) was not built, because (A) met the target.

## Designs measured

Each variant was built from `git archive HEAD` plus the working-tree patch, with its own target directory (one shared dir for `cur`/`a`, see Concerns) and a `Compiling rexx-exec` line. Binaries are in `/tmp/claude-1000/p61/pr2/bin/`. d% is against base61 `/tmp/claude-1000/p61/t1/bin/base/rexx-run` (sha256 `2ea19b3e...`). `cur` is HEAD `3d43b65cb` (code tree `cf167fba1`), and it reproduces parse +1.1983.

### (A) plain: `held_bytes()` per survivor in the mark loop

`callgrind.sh -r 2`, all programs (`parse-round2-evidence/cg-a.log`, binaries in `cg-a-binaries.txt`):

| program | cur d% | A d% |
|---|---:|---:|
| alloc | +0.4253 | +0.2519 |
| alloc4c | -0.0651 | -1.0022 |
| arith | +0.2725 | -0.0891 |
| assign | -3.1650 | -3.1650 |
| compound | -1.1171 | -1.1171 |
| decloop | -2.2377 | -2.8093 |
| decrender | -1.0607 | -1.3944 |
| dirread | +0.1582 | +0.0692 |
| dispatch | -0.3596 | -0.3596 |
| dispatchclass | -0.6321 | -0.6321 |
| emptyloop | -0.6401 | -0.6401 |
| extcall | +0.0022 | +0.0022 |
| fibcall | +0.4413 | +0.4413 |
| fibfunc | +0.4304 | +0.4304 |
| heapshape | +0.1853 | +1.2886 |
| nop | -1.0862 | -1.0862 |
| parse | +1.1984 | +0.4840 |
| sayloop | -0.4003 | -0.4001 |
| sendloop | -0.4983 | -0.4983 |
| startup | +0.2817 | +0.2821 |
| strings | +0.3735 | -0.1011 |
| textnum | -0.5399 | -0.5399 |
| varlookup | -2.2611 | -2.2611 |
| rexxcps | +0.2694 | -0.1918 |
| pingguard | -0.7107 | -0.7107 |
| pingmsg | +0.0537 | -0.2963 |
| pingsem | -0.4856 | -0.4856 |

parse +0.4840 meets the target, but heapshape rises from +0.1853 to +1.2886 and newly exceeds +0.5%. cgdiff cur to A: heapshape `collect_now` +25.75M, parse `collect_now` -10.97M. heapshape makes 5 collections with about 1.98M survivor visits in all (`Body::trace` called 1,983,903 times), so (A) costs about 13 Ir per survivor. Per line, the cost is the second dispatch on the body that `held_bytes` adds (`match self` 11.9M) plus `heap_len` (3.96M).

This also explains Task 12's "smaller of survivors and dead" (+1.61%), as far as the shape goes: a per-survivor `held_bytes` read is about twice the per-freed-object read the sweep makes. That variant's binary was not rebuilt here, so the explanation is inferred, not measured.

### (A') `Body::trace_held`: trace and held bytes in one match

Quick `-r 1 -p "parse heapshape"`: parse +0.4829, heapshape +0.6938. cgdiff cur to A': `trace_held` +81.54M against `trace` -73.61M, and `collect_now` +3.93M. The fused function's prologue and epilogue (17.9M and 15.9M per line) are paid by about 2M string survivors that trace nothing. Not taken.

### (A3) A' plus the string fast path, and (A4) A plus the string fast path

`callgrind.sh -r 2`, all programs (`parse-round2-evidence/cg-a34.log`, binaries in `cg-a34-binaries.txt`):

| program | cur d% | A3 d% | A4 d% |
|---|---:|---:|---:|
| alloc | +0.4253 | +0.2513 | +0.2515 |
| alloc4c | -0.0651 | -1.2535 | -1.2531 |
| arith | +0.2725 | -0.0903 | -0.0899 |
| assign | -3.1650 | -3.1650 | -3.1650 |
| compound | -1.1171 | -1.1171 | -1.1171 |
| decloop | -2.2377 | -2.8112 | -2.8106 |
| decrender | -1.0607 | -1.3955 | -1.3952 |
| dirread | +0.1582 | +0.0689 | +0.0690 |
| dispatch | -0.3596 | -0.3596 | -0.3596 |
| dispatchclass | -0.6321 | -0.6321 | -0.6321 |
| emptyloop | -0.6401 | -0.6401 | -0.6401 |
| extcall | +0.0022 | +0.0022 | +0.0022 |
| fibcall | +0.4413 | +0.4413 | +0.4413 |
| fibfunc | +0.4304 | +0.4304 | +0.4304 |
| heapshape | +0.1853 | -1.9373 | -1.8518 |
| nop | -1.0862 | -1.0862 | -1.0862 |
| parse | +1.1983 | +0.4817 | +0.4824 |
| sayloop | -0.4005 | -0.4005 | -0.4005 |
| sendloop | -0.4983 | -0.4983 | -0.4983 |
| startup | +0.2809 | +0.2809 | +0.2807 |
| strings | +0.3735 | -0.1025 | -0.1020 |
| textnum | -0.5399 | -0.5399 | -0.5399 |
| varlookup | -2.2611 | -2.2611 | -2.2611 |
| rexxcps | +0.2694 | -0.1936 | -0.1931 |
| pingguard | -0.7106 | -0.7107 | -0.7106 |
| pingmsg | +0.0537 | -0.2974 | -0.2971 |
| pingsem | -0.4857 | -0.4857 | -0.4857 |

Both meet the target. They differ only on heapshape (-1.9373 against -1.8518, about 2.0M Ir) and by under 0.001 point on parse. A4 was taken because it keeps one definition of held bytes (`Body::held_bytes`). A3 needs a second copy in `trace_held`, and a copy that drifts out of step is this phase's defect class.

## Final: `23d78ec1b`

The committed tree differs from A4 by one comment's wording. It was rebuilt from `git archive 23d78ec1b`, with its own target directory and one `Compiling rexx-exec` line (sha256 `2d56a4e252de765c168a9adde9b18a8c581a774abf6dc947363043b091f61729`). It was measured with `callgrind.sh -r 2` over all programs together with base61 and cur: exit 0, spreads 0.0001% at most (`parse-round2-evidence/cg-fin.log`, `cg-fin-binaries.txt`, `cg-fin-summary.tsv`).

| program | base61 Ir | cf167fba1 d% | 23d78ec1b d% |
|---|---:|---:|---:|
| alloc | 20345537302 | +0.4253 | +0.2515 |
| alloc4c | 3140684449 | -0.0651 | -1.2531 |
| arith | 11488858262 | +0.2725 | -0.0899 |
| assign | 18983599250 | -3.1650 | -3.1650 |
| compound | 8937498228 | -1.1171 | -1.1171 |
| decloop | 2469010841 | -2.2377 | -2.8106 |
| decrender | 4228326880 | -1.0607 | -1.3952 |
| dirread | 2644857003 | +0.1582 | +0.0690 |
| dispatch | 20809503043 | -0.3596 | -0.3596 |
| dispatchclass | 15794935302 | -0.6321 | -0.6321 |
| emptyloop | 7786100004 | -0.6401 | -0.6401 |
| extcall | 7552641533 | +0.0022 | +0.0022 |
| fibcall | 8413280478 | +0.4413 | +0.4413 |
| fibfunc | 8228290864 | +0.4304 | +0.4304 |
| heapshape | 2334068375 | +0.1852 | -1.8518 |
| nop | 9283340619 | -1.0862 | -1.0862 |
| parse | 1536336771 | +1.1983 | +0.4824 |
| sayloop | 109048299 | -0.4004 | -0.4004 |
| sendloop | 14013810686 | -0.4983 | -0.4983 |
| startup | 58097187 | +0.2809 | +0.2809 |
| strings | 17537481400 | +0.3735 | -0.1020 |
| textnum | 1155145953 | -0.5399 | -0.5399 |
| varlookup | 13437530430 | -2.2611 | -2.2611 |
| rexxcps | 17788420667 | +0.2694 | -0.1931 |
| pingguard | 1159017374 | -0.7107 | -0.7106 |
| pingmsg | 1676421124 | +0.0537 | -0.2971 |
| pingsem | 1181250540 | -0.4857 | -0.4857 |

* parse +0.4824%, inside +0.5%.
* Nothing newly exceeds +0.5%. fibcall +0.4413 and fibfunc +0.4304 are unchanged.
* No program rises. Every program either falls or moves by fewer than 1,000 Ir (the largest is pingguard, +701).
* cgdiff cur to A4 (`parse-round2-evidence/cgdiff-cur-to-a4.txt`), in self Ir:
  * `collect_now`: alloc -35.31M, alloc4c -37.29M, arith -41.58M, decloop -14.12M, decrender -14.12M, dirread -2.35M, parse -10.98M, strings -83.29M, rexxcps -82.14M, pingmsg -5.87M.
  * heapshape: `Body::trace` -41.61M and `collect_now` -5.94M.
  * Nothing else moves by more than 100K on any of those programs.

## Invariant

* The running figure is exact whenever the debug assertion holds, because release sets it to the very sum the assertion compares against. The debug path is unchanged: freed bytes summed and released, then `assert_eq!(survivor_bytes, held_bytes)`. The debug tests (`cargo test -p rexx-exec -p rexx-core -p rexx-api`, and G6) ran that assertion on every collection and passed.
* `body_bytes_tests::a_collection_sums_the_survivors_body_bytes` covers a kept string, a dead string, a resurrected UNINIT string and an inline string, and now checks the release path. Two mutations, run with `cargo test --release -p rexx-core --lib heap` and then restored (`cmp` against the kept copy):
  * Dropping the resurrect loop's sum fails with left 100, right 400.
  * Making the string fast path answer 0 fails with left 300, right 400.
* R10's `bytes_due` and `bytes_since` are untouched. `live_bytes` and `peak_bytes` read `held_bytes` after it is set, as before.

## Per-step check (working tree at A4, identical to `23d78ec1b` but for the comment)

* `cargo fmt --all`: 0.
* `cargo clippy -p rexx-exec --all-targets --features pinning,sharing -- -D warnings`: 0.
* `cargo clippy --workspace --all-targets -- -D warnings`: 0.
* `cargo test -p rexx-exec -p rexx-core -p rexx-api --no-fail-fast`: exit 0, 2332 passed, 0 failed, 1 ignored. `refusal_sites` is in that run. heap.rs holds no refusal constructor, and body.rs ends unchanged.
* Corpus pair (`REXX_CORPUS_GATE=1 --test corpus --test ir_recorded_oracle`): exit 0, 29 passed with 1 ignored, and 21 passed.
* The same check passed on A (first variant) and on A3.

## Gates

`bggates.sh 23d78ec1b`, run alone after all perf runs, finished 2026-10-10T21:03:14+02:00 (`bg/23d78ec1b/status.txt`):
* G1, G2, G3, G5, G7, G8, G9: exit 0.
* G4 release: exit 0, 3149 passed, 0 failed, 4 ignored.
* G6 debug: exit 0, 3153 passed, 0 failed, 4 ignored.
* P48 reruns 0.

## Concerns

1. The `cur` and A binaries were built one after the other in one target directory (`tgt-cur`). The first `cur` binary was overwritten by A's build and rebuilt (`build-cur2.log` shows `Compiling rexx-core` and `Compiling rexx-exec`). Its parse figure (+1.1983) matches the final fix's `cf167fba1` figure, and every later variant had its own target directory.
2. Release now trusts the survivor sum outright. If a charge site is missed, release no longer drifts: it silently corrects at the next collection, and only a debug run sees the defect. This was the debug-only check before as well, so detection is unchanged. A release build now hides a drift that it previously carried.
3. Task 12's "smaller of" variant was not rebuilt. Its explanation above comes from (A)'s per-survivor cost.
4. `bggates.sh` is not executable (mode 664). The first attempt failed with "Permission denied" before creating anything, and it was run with `bash bggates.sh`.
5. The report is not committed. It is under `.superpowers`, for the controller.

## Fix round 1

The review is `parse-round2-review.md`. Changes:
* The callgrind logs, tables, binaries lists and the cur-to-A4 cgdiff are copied to `parse-round2-evidence/`. The gate record's `## Parse round 2` and this report cite them there.
* The doc comment on `Heap::held_bytes` now states its contract: the survivors' sum at the last collection, plus holds and less releases since. Only the debug build's check in `Heap::collect` uses its value, and a release collection overwrites it unread. The dead release writes stay. Gating the running figure on `debug_assertions` would change every charge site's code, so it is not simple, and nothing shows it is perf-neutral.
* The per-object `debug_assert_eq!` that compared `held_bytes()` with itself on the non-string arm is gone. The string fast path now asserts in debug that `bytes.heap_len()` equals `Body::held_bytes()`, which are two independent computations.
* `Body::trace`'s `Text` arm names the mark-loop fast path that relies on it.
* The gate record's sentence on the committed test now says that the test checks the string and resurrect paths in release, and that only the debug assertion checks the non-string arm.

Accepted gap, per the controller: no release test witnesses the non-string mark arm. If that arm answers 0, every release test stays green, and only the debug collection assertion catches it. G6 runs the debug tests, so the gap is covered at the gate.

Only comments and a debug-only assertion changed, so callgrind was not re-run (`debug_assert_eq!` compiles to nothing in release).

Per-step check on the fix tree: fmt 0, clippy with pinning,sharing 0, workspace clippy 0, `cargo test -p rexx-exec -p rexx-core -p rexx-api --no-fail-fast` exit 0 (2332 passed, 0 failed, 1 ignored), and the corpus pair exit 0 (29 passed with 1 ignored, and 21 passed).
