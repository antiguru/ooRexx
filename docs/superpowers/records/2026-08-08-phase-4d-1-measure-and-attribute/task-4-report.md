# Task 4 report: make the allocation axis measurable

Status: DONE.
Commits: `d233d1e90313b1d5f7f283ea09de749958861f61` (adds `alloc4c.rex` and registers it in both harnesses), `22b4609c5632bb3e69794c513a24012abba82a4a` (folds the fresh measurement into `perf-baseline.md`'s current section and adds the credibility/attribution analysis), `ad7c36f04b9165efc6857ac065757efbc895b3f0` (review fix: corrects a false cross-run stability claim, records the `compound` disjoint-interval finding as an open question for Task 8, discloses and checks the string-length asymmetry between `alloc4c.rex` and `alloc.rex`).

## Step 1: what blocks `alloc.rex`

Ran `alloc.rex` on both sides from a fresh directory.
Oracle: exit 0, stdout `21` (n=3 probe).
This crate: exit 120, stderr `rexx-exec: a message send is not implemented (Phase 5)`.

Isolated which construct is responsible by testing `.array~of(1,2,3)` alone and `.string~new("item")` alone.
Both fail with the identical exit status and message.
`a~size` and `s~length` were not reachable to test in isolation (the objects that would receive them never construct), but the message itself ("a message send is not implemented") already names the mechanism, not a specific class.
Conclusion: no single construct in `alloc.rex` blocks it; the whole object-model / message-send surface is absent on this crate, confirmed by `rexx-exec/tests/spike.rs`'s own committed expectation of that exact error text.
Allocation throughput does not require that surface, which is what makes the axis coverable now.

## Step 2: `alloc4c.rex`

Created `rust/bench-programs/alloc4c.rex`.
Per iteration it does a compound-variable creation (`tab.i = i`, a tail never seen before) and a `||` concatenation (`s = "item" || i`), then accumulates two small-integer terms (a constant 3 standing in for the array's fixed size, and `length(s)`) so the accumulator itself never allocates -- the same property `alloc.rex`'s `a~size`/`s~length` additions have.

The file's header states what carries over from `alloc.rex` and what does not:

* Carries over: two heap-allocating operations per iteration, one container-like allocation and one short string, with the arithmetic staying in tagged-small-integer range.
* Does not carry over: `alloc.rex`'s array and string are rebound to the same loop-local variable every pass and would be garbage under a real collector; `alloc4c`'s compound-variable tail is a **new** name every pass (`tab.1` .. `tab.n`), so it is a genuinely live, growing table on any interpreter, oracle included -- not collectible churn. The axis therefore measures allocation throughput, not the collection pressure `alloc.rex` was sized to force.
* Also disclosed after review: `alloc.rex`'s string is a fixed four characters; `alloc4c`'s grows with `i`, five to eleven characters. Checked rather than asserted immaterial -- two 500,000-iteration probes at 5 and 11 characters peaked at 69,108 KB and 68,812 KB resident, no measurable difference.

Sized by timing under the oracle the same way the other axes were: `n = 1000000` lands the oracle at 1.13 s (inside the corpus's 0.5-2s window) and this crate at 2.34 s.

## Step 3: same work on both sides

From a fresh directory, both interpreters produce exit 0 and stdout `12888896`, byte-identical, stable across repeated runs on each side.

## Step 4: harness registration and measurement

Added `alloc4c` to `AXES` in `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs` (`Role::Loop`, sorted between `alloc` and `arith`, matching the directory listing `verify_axis_list` checks against) and to `PROGRAMS` in `rust/crates/rexx-bench/src/lib.rs` (the separate list the criterion harness and its own consistency test read). Updated `rust/bench-programs/README.md`'s table and removed a stale "all seven" count (now a floating count that would rot on the next addition).

Ran the full `rexx-bench-suite` (all five `Role::Loop` axes plus `rexxcps`, ~9 minutes) at commit `d233d1e9`, machine quiet (load average 0.86-4.25 on 32 cores). Per-side spreads: `alloc4c` 2.95% oracle / 2.21% this crate, both inside the previously accepted 0.94%-5.42% band; the four pre-existing axes ran 0.75%-5.47%, one row (`strings`, this crate, 5.47%) exceeding the historical `107febcd` band by 3.07 points -- accepted on the same reasoning the doc's "Spread" section already uses (ratio intervals stay disjoint; nowhere near the rejected contended run's 82.77%).

**`alloc4c` measured 2.08x, interval 2.03x-2.11x, SLOWER -- the tightest of the five axes**, ahead of `arith` at 2.70x.

Folded this run's full output into `perf-baseline.md`'s current section (not the superseded `107febcd` one), replacing the previous run's tables and updating every downstream paragraph that quoted its numbers (spreads, ratio intervals, the cps-stability section, the five-commit delta table), so the section stays internally consistent rather than citing stale figures next to fresh tables.

Added two new pieces of analysis to the doc, both requested mid-task:

1. A subsection connecting 2.08x to the header's preserve/not-preserve split: a compound-variable creation skips whatever per-operation overhead separates this crate's (entirely absent) message-send path from the oracle's, and `alloc4c`'s tails are a genuinely live table on both sides rather than the ephemeral churn `alloc.rex` was sized to force a collector against -- both plausible reasons the ratio could be narrower here than `alloc.rex` itself would show, neither confirmed by any measurement this task took.
2. A flag for Task 6: allocation throughput came out closest to the oracle, not furthest, which argues against (or at least complicates) any attribution story where allocation cost dominates the other four axes' gap.

## Verification

* `cargo fmt --all --check` from `rust/`: exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` from `rust/`: exit 0.
* `cargo test --workspace --release` from `rust/`: exit 0, zero `FAILED` lines across the whole run (this also exercises the `rexx-bench` crate's own consistency tests, which caught a missed `PROGRAMS` update on the first pass).

## Files touched

* `rust/bench-programs/alloc4c.rex` (new)
* `rust/bench-programs/README.md`
* `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`
* `rust/crates/rexx-bench/src/lib.rs`
* `docs/superpowers/plans/perf-baseline.md`

## Concerns

* The 2.08x figure's narrowness relative to the other four axes is not fully explained -- the doc names two plausible structural reasons (message-send overhead `alloc4c` cannot exercise; `alloc.rex`'s ephemeral churn vs. `alloc4c`'s genuine growth) but neither is measured directly, only argued. Task 6 should not treat 2.08x as a clean allocation-only figure without weighing that.
* `strings`' this-crate spread (5.47%) is the widest single cell across all five axes in this run, exceeding the historical band by more than double the previous run's largest excess. Review caught that my first pass had partly explained this away with a false claim (that the wall-clock ratios reproduce as tightly across runs as the cps ratio does); corrected, and the honest support for accepting the run is narrower: ratio intervals stay disjoint within this run, and this run's spreads are nowhere near the rejected contended run's 82.77%, but nothing here shows the run reproduces its predecessor closely.
* **New finding, surfaced by the same correction, and recorded in the doc as an open question for Task 8 rather than resolved here:** `compound`'s ratio interval in the run this section replaces (6.32x-6.39x) and in this run (6.02x-6.14x) are disjoint, on two nine-pair sign-test runs both judged quiet by every check available. That is evidence a single run's reported interval can understate real between-run variance enough to flip a gate call decided on interval overlap (Global Constraints `:39`). Two runs cannot build a variance model, so this is a flag, not an answer.
