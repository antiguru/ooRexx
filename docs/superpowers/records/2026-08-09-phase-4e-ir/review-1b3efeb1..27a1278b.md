# Review: Phase 4e task 1 -- re-establish the benchmark anchor

Commit reviewed: `27a1278ba6bb068d505a1689a91465fd8265028b`.
Base: `1b3efeb1affe32e30b4d21b03d95d42cce142e5e`.

## Verdict: spec compliance

Met.
All four required artifacts exist and do what the brief asked: `rust/bench-programs/emptyloop.rex` is deterministic and sized inside the corpus's 0.5-2s window, `PROGRAMS` and `AXES` both carry the new entry, all three named policing tests genuinely run and pass (re-run independently below, not zero-match), and `docs/superpowers/plans/phase-4e-anchor.md` holds a build identity, an oracle fingerprint, and per-axis figures with the required "tree-walker only, at the start of the phase" statement.
The ns/iteration-instead-of-ns/clause substitution is a disclosed, reasoned deviation from the brief's literal wording (manufacturing a per-clause count would be a new, unverified claim), not a missed requirement, and it still reports the number of interleaved passes the brief asked for.
The compliance defects below are in the document's supporting prose, not in whether the required deliverables exist.

## Verdict: task quality

Not clean.
The arithmetic is exactly right everywhere I recomputed it (every ratio, every ratio interval, every ns/iteration figure, every spread, the Bonferroni bound, ns/iteration vs raw-median cross-check) and the sha256/policing-test/cargo-test/fmt/clippy claims all reproduced independently, which is real, checked rigor.
Against that, the document carries two checkably false statements in its own contextual prose (one about the oracle's git history not existing, the other citing a "resolved ambiguity" section that does not exist in either the brief or the plan), and its own comparison against `perf-baseline.md` undersells how much two axes actually moved.
None of the three affects the anchor's primary deliverable -- the per-axis oracle-vs-tree-walker figures at commit `1b3efeb1` -- but they are exactly the "reads plausible, is false" shape this project keeps getting bitten by, inside the one document later tasks are told to treat as ground truth.

## Findings

### Important

* **"No `rexx-exec` or `rexx-core` source changed" is false, and it is the reasoning for calling two axes' movement noise.**
  The anchor states the five pre-existing axes "differ slightly from `perf-baseline.md`'s own `d233d1e9` figures -- run-to-run variation on this machine, not a code change; no `rexx-exec` or `rexx-core` source changed between that measurement and this one."
  `git diff --stat d233d1e9..1b3efeb1 -- rust/crates/rexx-exec rust/crates/rexx-core` shows both changed: commit `f9d9f46c` (Task 0 of this same phase) rewrote 183 lines of `rexx-exec/src/run.rs`, extracting `grant_procedure_permission` and `apply_flow` out of `run_activation`'s hot per-clause loop, and added 31 lines to `rexx-core/src/roots.rs` (`reserve_temps`/`temp_at`/`set_temp`, the new register region).
  That commit's own message calls it "a pure extraction," but it is a real change to the exact loop every loop axis executes, landed between the two measurements being compared.
  The two axes with the largest movement are `alloc4c` (ratio 2.08x to 2.00x, -3.85%) and `compound` (6.08x to 5.81x, -4.44%); the other three moved about 1% or less.
  This does not mean the extraction caused the movement -- `compound` has an independently documented history of cross-run instability at this same magnitude (`perf-baseline.md`'s "unstable across runs" section: 6.35x to 6.08x, disjoint intervals, no code change either time) -- but the anchor's own ruling-out of a code change is not available as the explanation, because a code change touching the relevant loop did happen.
* **The "Ambiguity resolved" instruction the anchor and the SDD report both cite does not exist.**
  The anchor states "the criterion harness was not run for this document (this task's own 'Ambiguity resolved' instructions size against the interleaved suite, not against criterion)," and the report states the same choice was made "per the brief's own resolved ambiguity."
  `grep -in ambigu` against both `task-1-brief.md` and the phase plan `docs/superpowers/plans/2026-08-09-phase-4e-ir.md` returns nothing; neither document contains a section, heading, or sentence resolving this or any other ambiguity, and the plan's own "Decisions this plan makes that the spec left open" section does not mention criterion vs. the interleaved suite.
  The underlying decision is defensible from the brief's actual text -- step 4 asks the anchor to record "the number of interleaved passes," which only the interleaved suite reports -- but the citation attached to it points at an instruction that was never written down anywhere reviewable.
* **The anchor does not say what its own comparison shows: two axes carry roughly a 4% cross-run floor, which bears directly on whether "axis X moved by N%" is falsifiable against this document.**
  Per the first finding, `alloc4c` and `compound` both moved close to 4% between the immediately preceding measurement and this one with no attributable code change ruled in or out.
  Every later task in this phase that predicts a movement on `arith` or `compound` predicts well above that floor (Task 9: -51.6% on `compound`), so this does not block those gates, but a future task claiming a small win on `alloc4c` or `compound` (under roughly 5%) would not be distinguishable from this document's own demonstrated noise, and the anchor does not flag that for the reader the way it flags `compound`'s cps-ratio instability by cross-reference.

### Minor

None.
Everything else checked -- every ratio and ratio interval, every ns/iteration and iters/s-net figure, the Bonferroni joint-coverage arithmetic (verified against `median_interval_indices`'s actual n=9 output of 96.09%), the clause-count claims for `varlookup` (two assignments) and `arith` (two `NUMERIC DIGITS` plus five assignments) against the actual `.rex` files, the alphabetical placement of `emptyloop` in `AXES`, the `PROVENANCE`-block/authored-prose boundary of the "byte for byte" claim, and the "throughput ratio equals wall ratio" identity -- was correct.

## What I independently verified, not just recomputed

* Rebuilt `rexx-run` and `rexx-bench-suite` from the current tree (already at `27a1278b`) and got sha256 `98ee5da45a2db83241489c52820e78ad3ea214061aa5ef29fc0ec0c8c0a4b935` and `025f67242f3831e57867e801992271b812ed6edf3c0779cd3d6843b39f0281e0` respectively -- both match the anchor exactly, which is real support for the "measured commit is the parent, not this commit" reasoning: the binaries the anchor measured are what this commit's tree produces.
* Ran `the_benchmark_list_accounts_for_every_program`, `the_axis_list_covers_every_bench_program`, and `every_loop_axis_has_a_loop_bound` individually and read the run counts: 1 passed each, not a zero-match false green.
* Ran `cargo test --workspace` and `cargo test --workspace --release` from `rust/`: 1337 passed, 0 failed, 0 `FAILED` occurrences, both profiles -- matches the report's claim exactly.
* Ran `cargo fmt --all --check` (exit 0, no output) and `cargo clippy --workspace --all-targets -- -D warnings` (exit 0) -- matches the report, with the same caveat the report already discloses (warm target, not the phase-gate clean-target run).
* Ran `emptyloop.rex` once under the oracle from a fresh directory: exit 0, stdout `done`, empty stderr -- consistent with the determinism claim.

## Cannot verify from diff

* The actual wall-time medians, min/max, and sign-test intervals throughout "Provenance," "Axes," and `samples/rexxcps.rex` are the harness's live measurement output.
  They are internally consistent (every ratio, interval, and derived figure recomputes correctly from them) and the binaries that produced them are confirmed to match the commit, but the raw numbers themselves cannot be checked without re-running the multi-minute interleaved suite, which is out of scope for this review.
* Whether `n = 25000000` was chosen as "the smallest round number" that lands inside the 0.5-2s window, as the SDD report states -- no evidence of intermediate values tried between 3,000,000 and 25,000,000 is in the diff or report, so this specific characterization is unverifiable (though the chosen value does land inside the window, which is the part that matters).
