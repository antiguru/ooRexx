# Task 1 report: re-establish the benchmark anchor

BASE `1b3efeb1affe32e30b4d21b03d95d42cce142e5e` (Task 0's commit plus the pre-flight-scan fix,
already landed).
Commit `27a1278ba6bb068d505a1689a91465fd8265028b`.

## What was run

* Read the harness: `rust/crates/rexx-bench/src/lib.rs` (`PROGRAMS`, `NOT_BENCHMARKED`,
  `programs_dir`), `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs` (`AXES`, `Role`, `loop_count`,
  `measure_interleaved`), `rust/bench-programs/README.md`, `rust/bench-programs/varlookup.rex`,
  `arith.rex`, `compound.rex`, `strings.rex`, `alloc4c.rex` (to write the anchor's clause-count
  disclosure).
* Sizing probe for `emptyloop.rex`: `n = 3000000` (the plan's own example) ran the oracle in
  0.119 s from a fresh `mkdir`'d directory under `ulimit -v 8388608` -- far below the 0.5-2 s
  target every other axis in this corpus uses. `n = 25000000` ran in 0.904 s, inside the window; a
  second, independent fresh-directory run confirmed exit 0, empty stderr, and byte-identical stdout
  (`done`) between the two runs.
* Registered `emptyloop` in `rexx_bench::PROGRAMS` (criterion's list) and in `rexx-bench-suite.rs`'s
  `AXES` (`Role::Loop`, alphabetically between `dispatch` and `heapshape`, matching the sorted
  directory listing both policing tests check against).
* Ran the three named policing tests individually, from `rust/`, reading the run count each time
  (never the bare exit status, per the brief's own warning that a zero-match filter exits 0):
  `the_benchmark_list_accounts_for_every_program` -- 1 passed; `the_axis_list_covers_every_bench_program`
  -- 1 passed; `every_loop_axis_has_a_loop_bound` -- 1 passed.
* `cargo test --workspace` and `cargo test --workspace --release`, both from `rust/`, stdout and
  stderr captured to separate files (never `2>&1`), exit status read unpiped: **1337 passed, 0
  failed** in both profiles -- unchanged from the recorded baseline, as expected for a
  benchmark-corpus-only change.
* `cargo fmt --all --check` -- exit 0, no output.
* `cargo clippy --workspace --all-targets -- -D warnings` -- exit 0 (not from a clean target
  directory; this task's own gate does not require the phase-boundary clean-target run, only the
  phase gate at Task 11 does).
* Built `rexx-run` and `rexx-bench-suite` in `--release`, ran the interleaved suite with no
  arguments (unpinned) once, capturing stdout (the report) and stderr (progress) to separate files.
  The run completed in the background; a monitor set to poll the launched PID (not a `pgrep -f`
  self-match) reported completion. The coordinator separately confirmed the run had already
  finished cleanly at 09:56:48 and that nothing else had touched the machine since.
* After committing, rebuilt `rexx-bench-suite` from the committed tree and confirmed the sha256
  matched the one recorded in the anchor document (`025f67242f...`), so the binary measured is
  exactly what the commit produces.

## Raw figures

The full harness output is quoted byte-for-byte in
`docs/superpowers/plans/phase-4e-anchor.md` under "Provenance" through "Axes this crate cannot
run". Summary:

| axis | oracle median | rust median | ratio | verdict |
|---|---:|---:|---:|---|
| `alloc4c` | 1.1438 s | 2.2839 s | 2.00x | SLOWER |
| `arith` | 1.1616 s | 3.1044 s | 2.67x | SLOWER |
| `compound` | 1.1490 s | 6.6810 s | 5.81x | SLOWER |
| `emptyloop` (new) | 0.8955 s | 2.9825 s | 3.33x | SLOWER |
| `strings` | 0.8670 s | 9.0791 s | 10.47x | SLOWER |
| `varlookup` | 1.2075 s | 5.3131 s | 4.40x | SLOWER |

`rexxcps` internal cps ratio: 7.33x. `alloc`, `dispatch`, `heapshape` remain blocked (exit 120,
"a message send is not implemented (Phase 5)"). All nine sampled pairs per axis (one warm-up pair
discarded), oracle and this crate alternating, per `PAIRS`/`WARMUP_PAIRS`.

Oracle fingerprint (unchanged from every fingerprint `perf-baseline.md` has recorded since
2026-08-08): `bin/rexx` sha256 `bb5bb8cc...`, `lib/librexx.so.4` sha256 `42136c40...`,
`lib/librexxapi.so.4` sha256 `3536b763...`.
`rexx-run` sha256 `98ee5da4...` -- unaffected by this task, which touches no `rexx-exec`/`rexx-core`
source.

## What was decided, and why

* **Sized `emptyloop` at `n = 25000000`, not the plan's illustrative `n = 3000000`.** The brief's own
  text asks for the oracle's own timing to be checked before fixing `n`, and the illustrative value
  measured at 0.119 s -- an order of magnitude short of the 0.5-2 s window this corpus's other six
  loop axes are sized to. Chose the smallest round number whose direct timing landed inside that
  window.
* **Sized against the interleaved suite, not the criterion harness**, per the brief's own resolved
  ambiguity. Did not run `benches/interpreter.rs` for this document. Estimated its cost instead: at
  `sample_size(10)` and this axis's ~0.9 s (oracle) / ~3.0 s (rust) per-sample cost, ten samples of
  the slower side is about 30 s -- at, not past, the existing 30 s measurement-time ceiling for this
  benchmark group, so this axis does not force a change to that harness's existing configuration.
* **Re-measured all six loop axes in one sitting rather than copying the five pre-existing ones from
  `perf-baseline.md`.** Global Constraints requires that benchmark comparisons interleave within one
  sitting, and `emptyloop`'s sample has to sit in the same sitting as whatever it is compared
  against for spread and ratio purposes. The five pre-existing axes' numbers here therefore differ
  slightly (a percent or so) from `perf-baseline.md`'s most recent (`d233d1e9`) figures --
  run-to-run variation on an otherwise-unchanged tree, not a regression. Did not edit
  `perf-baseline.md`'s own sections; they stand as their own record.
* **Reported ns/iteration, not ns/clause**, against the brief's literal wording. The harness
  computes throughput from each program's own `n = ...` loop bound (a fact about the program text),
  not from a per-clause count the interpreter reports. `samples/rexxcps.rex` is the one figure in
  this document that is a genuine per-clause count, because that program is the oracle's own
  instrument and prints it directly. Manufacturing a clause count for the seven axis programs by
  guessing would have been a new, unverified claim rather than a restatement of what was measured,
  so the anchor states the axis programs' rough clause-per-iteration shape in prose (read from each
  `.rex` file) and reports the well-defined ns/iteration figure instead, flagging the substitution
  explicitly rather than silently renaming it.
* **Recorded that the measured commit (`1b3efeb1`, from the harness's own `git rev-parse HEAD`) is
  this task's parent, not this task's own commit.** The harness was run before committing, against a
  working tree that held this task's changes uncommitted; the commit that followed contains exactly
  that diff and nothing else, so the binaries measured are what the named commit produces. Confirmed
  by rebuilding `rexx-bench-suite` after committing and matching its sha256 against the one recorded
  in the anchor.
* **Added a new heading ("Why the blocked axes carry no ratio") rather than appending prose directly
  under the harness's own "Axes this crate cannot run" table.** The document claims that section's
  content is the harness's stdout byte for byte; appending unheaded prose there would have blurred
  that boundary the same way `perf-baseline.md`'s own precedent avoids it (its `alloc4c` commentary
  gets its own heading immediately after the quoted block ends).

## What could not be verified

* **Whether `n = 25000000` remains inside the 0.5-2 s window on any machine other than this one.**
  The suite is Linux-only by construction (Global Constraints, and the plan's own open platform
  question), and this task ran on one machine only.
* **Run-to-run stability of the `emptyloop` ratio specifically**, beyond the one sign-test interval
  this run reports (3.20x-3.38x). `perf-baseline.md`'s own "internal cps ratio is unstable across
  runs" section already established for this machine that a single run's interval understates
  between-run movement on other axes (`compound`'s two historical intervals are disjoint); nothing
  in this task's single run either confirms or contradicts that this axis is any more or less
  stable, since it has no prior run to compare against.
* **Criterion's actual wall-clock cost for `emptyloop`.** Estimated from the interleaved suite's own
  per-sample timings rather than measured directly, per the resolved ambiguity's instruction not to
  run the criterion harness for this task.
