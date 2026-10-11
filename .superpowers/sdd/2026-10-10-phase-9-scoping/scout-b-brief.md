# Scout B: where performance parity stands

Read `rules.md` in this directory first. Scratch directory: `/tmp/claude-1000/p9scope/b/`.
Report: `.superpowers/sdd/2026-10-10-phase-9-scoping/scout-b-report.md`.

Row 9 said "every benchmark at parity". Moritz struck that from Phase 9 on 2026-10-10 ("2x is
achievable, 1x will be difficult as-is"); parity becomes its own work item. Measure the gap at HEAD,
so that item and Phase 9's non-regression bar start from a current figure.

1. **Which benchmarks.** Enumerate what "every benchmark" can mean in this tree: the axes
   `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs` runs, the programs in `rust/bench-programs/`,
   and any whole-program benchmarks from `samples/` named in the plan's D9. Read
   `docs/superpowers/plans/perf-baseline.md` (newest section), `paths-to-parity.md`, and the parity
   clauses in the roadmap (Global Constraints `:39`, Phase 4's "debt route" paragraph) to state what
   "parity" is defined as and which axes may carry recorded debt.
2. **The oracle build.** `rust/CLAUDE.md` says a timing claim needs an `-O3`/`Release` oracle, and
   `build/` is `RelWithDebInfo`. Find out whether a Release oracle exists on this machine (look at
   how the newest perf-baseline section and `paths-to-parity.md` got theirs, with its sha256). If
   none exists, do not build one: report what would be needed and measure against `build/` labelled
   as such.
3. **Measure.** Build release at HEAD. Run the suite the way perf-baseline's newest section did
   (interleaved, its rep count, cycles and instructions per side), under memcap/ulimit as the rules
   say. Produce one table: axis, crate, oracle, ratio in cycles (or wall clock), ratio in
   instructions, and the figure at `paths-to-parity.md`'s date for comparison.
4. **Shape.** For each axis over parity, one line on what the existing records attribute the gap
   to, citing the record. Do not optimise or profile anew.
5. Machine load matters: check `uptime` before and after, and say if other jobs were running.
