### Task 2b report: re-establishing the Phase 4d-1 baseline

**Status: DONE**

**Commits:** `4092fb88c3de17438f997f03e07d491e3de562c7`, "Re-measure the Phase 4d-1 baseline after
five post-baseline speedups", followed by `c9a90906bea09bc19ce631b0256650822a2b3684`, "Fix the
Phase 4d-1 re-measurement's spread claim and record cps provenance" -- both on `plan/rust-rewrite`.

**File modified:** `docs/superpowers/plans/perf-baseline.md`.

---

#### What was done

1. Built `rexx-bench-suite` and `rexx-run` in release mode from `rust/` (already up to date; 0.02s
   no-op build).
2. Checked the machine before running: `ps aux` showed only the sandbox process, `rxapi`, and idle
   MCP-server processes, no build or other benchmark in flight.
3. Ran `./target/release/rexx-bench-suite` (not `--self-check`) from `rust/`, stdout, stderr and
   exit status captured to three separate files. Exit status 0. Took roughly the expected ~12
   minutes. Ran once, and used that completed run rather than re-running; that specific choice --
   use the run in hand instead of re-running -- was the coordinator's direction. Deciding whether the
   run's spread was good enough to accept, and writing the justification for it, was mine to do, not
   something the coordinator instructed the outcome of.
4. Read the numbers out of the captured stdout rather than retyping them, and wrote them into
   `perf-baseline.md` verbatim for every heading between "Provenance" and "Axes this crate cannot
   run", matching the convention the existing `107febcd` section already uses.
5. Marked the existing `107febcd` section's heading and lead paragraph as superseded, naming the
   five speedup commits (`3799692d`, `b6b1d8a9`, `e1d50dda`, `c428ec8a`, `04ab4af6`) as what moved
   its numbers, and pointing at the new section. The old table and its "What the baseline says"
   analysis are untouched.
6. Added a new section, "Phase 4d-1 -- re-measured baseline after five speedups, measured
   2026-08-08", containing the fresh harness output plus:
   - a "Spread, and why this run is accepted despite exceeding the old band on two rows" analysis
     that states plainly which rows exceeded the committed band and why the run is accepted anyway
     (see below),
   - a section on the internal `rexxcps` ratio's instability across three separate measurements,
     with the `--self-check` figure's provenance recorded,
   - a comparison table of both baselines' four ratios plus the internal `rexxcps` ratio, with the
     five commits attributed as the cause of the drop and `varlookup`'s much larger drop connected
     to `e1d50dda` landing in its loop-stepping path.
7. On review, corrected a factual error in step 6's spread analysis (below) and a documentation gap
   in its `rexxcps` provenance, and corrected this report's own account of who made the acceptance
   decision.
8. Verified `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
   from `rust/`, each exit status read unpiped (clippy's output was redirected to a file rather than
   piped through `tail`, per the "never read a cargo exit status from a pipeline" constraint), both
   before and after the correction in step 7. Both exit 0 each time. No Rust files were touched by
   this task, so this is confirmation, not new work.
9. Committed only `docs/superpowers/plans/perf-baseline.md`, in two commits (the original measurement
   and the correction); read each hash back with `git log` rather than quoting it from the commit
   command's own output.

#### The spread decision, corrected

**The first version of this report and of the document's spread section had a factual error,
caught on review.** It compared this run's spreads against the wrong reference and, in one case,
against the wrong side, and it computed one ratio wrong. The corrected comparison, taken over the
four axis rows (not `rexxcps`) as the committed baseline's band was:

The committed baseline's band, as measured on `107febcd`: this-crate spreads 1.02%-2.40%, oracle
spreads 2.29%-7.11%.

This run's spreads: this-crate `arith` 1.03%, `compound` 2.79%, `strings` 2.86%, `varlookup` 0.94%;
oracle `arith` 2.04%, `compound` 2.02%, `strings` 4.08%, `varlookup` 5.42%.

**Two of the eight rows exceed the old band, both on the this-crate side: `compound` (2.79%, against
a 2.40% upper edge) and `strings` (2.86%).** Every oracle row lands inside the old band -- two of
them (`arith`, `compound`) tighter than its lower edge, the other two (`strings`, `varlookup`) inside
its upper edge, with `varlookup` (5.42%) well clear of the 7.11% ceiling rather than above it. The
first version of this report and document said four of eight rows exceeded, split across both sides,
and separately described that same 5.42% `varlookup` oracle figure as both "above the old band" and,
two paragraphs later, as "this run's worst row" without noting it wasn't actually above anything --
a self-contradiction the reviewer caught. It is corrected in the document; this report states the
correction rather than repeating the error.

**My acceptance decision survives the correction, and is stronger for it: fewer rows exceed the band
than I originally reported, and all of them are on one side.** The decision was, and remains, made on
the ratio intervals rather than the raw spreads: the four per-axis ratio intervals -- 2.67x-2.72x,
6.32x-6.39x, 10.53x-11.06x, 4.28x-4.38x -- do not come close to overlapping each other, which is the
property this task's per-axis attribution depends on. The excess over the old band on the two rows
that exceed it (0.39 and 0.46 points) is one to two orders of magnitude smaller than the
coordinator's own rejected contended run (82.77% on `strings`, 32.84% on `arith`), which is the
failure mode the band exists to catch. As one sentence of cross-check, three of this run's four
ratios land close to that contended run's own indicative figures (`arith` 2.70x vs 2.60x, `compound`
6.35x vs 6.32x, `strings` 10.77x vs 10.68x), treated in the document as coarse agreement and
explicitly not as a second baseline.

**Whose decision this was, corrected.** The first version of this report said the acceptance was
made "at the coordinator's direction." That is not accurate and is corrected here. The coordinator
sent both the completed run's numbers and the committed baseline's, with the instruction to decide
whether to accept and to justify the decision -- including, explicitly, not to silently widen the
bar. Deciding to accept, and the reasoning for it, is mine, and the document's argument stands on its
own merits rather than on an instruction. What the coordinator did direct, and what I should have
kept separate from the decision itself, was using the already-completed run rather than spending
another ~12 minutes re-running it -- that part of the account was correct in the first draft and
remains so.

#### The internal `rexxcps` ratio, with provenance now recorded

Three measurements of the same tree gave three different internal cps ratios: 7.41x (`--self-check`,
one pair, explicitly not a baseline), 6.21x (the coordinator's own contended run), and 7.31x (this
accepted run, nine pairs). The 7.41x figure had no recorded provenance in the first version of the
document, which the reviewer correctly flagged since nothing in the task's own inputs could confirm
it. Its provenance: the coordinator ran `--self-check` on this tree before dispatching this task and
passed the number in a message, not a committed output file, so it is recorded in the document as
sourced that way rather than as something this task reproduced. That 6.21x-7.41x range is wider than
any of the four wall-clock ratios moved between repeat measurements in this task. The document states
the accepted figure (7.31x, interval 7.14x-7.50x) separately from the observed cross-run range
(6.21x-7.41x), and says explicitly that the sign-test interval does not capture between-run movement,
rather than presenting one number as if it carried the other's precision.

#### Concerns

- Two rows, both on the this-crate side (`compound` 2.79%, `strings` 2.86%), exceed the committed
  baseline's exact numeric band; the decision to accept the run despite that was mine, made on the
  ratio intervals' non-overlap, and is stated as mine in the document.
- The internal `rexxcps` ratio's between-run instability (6.21x-7.41x) is real and unexplained; the
  document records it as an open observation rather than attributing a cause, since nothing in this
  task looked for one.
- This task's own attribution of "`varlookup`'s drop is consistent with `e1d50dda`" is a plausibility
  argument from the commit's own description, not a controlled re-measurement isolating that one
  commit; the document says the finer per-commit attribution is unattempted and left to Task 3 or 4.
- This report's own numbers were wrong once already in this task (the spread comparison) before
  being caught on review; I re-checked every derived percentage and ratio in the corrected sections
  by recomputing from the tables above them rather than trusting the earlier arithmetic, but I did
  not have a second reviewer for this revision.
