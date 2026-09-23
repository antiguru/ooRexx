# Spike: pgo -- profile-guided optimisation of the shipping build

Spike branch: `spike/pgo`. Base: `0ba0f3876` of `plan/rust-rewrite` (worktree
was initially cut at `c2edef977`, a master-lineage commit; corrected to
`0ba0f3876` before any build or measurement, per the coordinator's
authorisation).

## Prediction (written before any measurement)

No source changes; only build flags change (`-Cprofile-generate` /
`-Cprofile-use`, `lto = "fat"`, `codegen-units = 1` kept from the release
profile). PGO's mechanism is block layout, hot/cold splitting, branch-weight
informed inlining, and register allocation informed by block frequency, not
a change to which instructions exist on the path actually taken. Callgrind
counts retired instructions on the executed path; most of what PGO changes
(code placement, prediction hints) is invisible to that counter. So:

* **rexxcps callgrind instructions (ex-libc):** predict small movement, -1%
  to +1% for `held-out`. `in-sample` may show a larger apparent win purely
  from memorising the loop shape (predict up to -3%), which is why it is
  reported separately and is not a result.
* **varlookup, arith, emptyloop, dispatch, compound (instructions):** predict
  flat, within +-1%, held-out and in-sample alike -- these are short, and PGO
  has less to lay out per clause than the interpretation driver has.
* **Cycles (`perf stat -e cycles`):** predict this is where PGO shows up, if
  anywhere: -2% to -6% on `rexxcps` held-out, from `run_ops_from::<true>`'s
  57%-dead arms getting pushed out of the icache-hot region. Predict
  in-sample cycles improve more than held-out (memorised layout matches the
  exact benchmark's branch pattern).
* **Profile toolchain:** `rustc 1.98.1`, LLVM 22.1.8. Only `llvm-profdata-21`
  (LLVM 21.1.8) and the nightly toolchain's (LLVM 23.1.1-rust) are available
  offline; neither matches. Predict the merge or the profile-use build
  reports a version mismatch (warning or hard error) with at least one, and
  that this measurably weakens or nullifies any win -- flagged as the
  headline risk before running anything.
* **Correctness floor:** predict green on the held-out-trained build --
  PGO does not change program semantics, only code shape. A toolchain crash
  from a version-mismatched profile is the realistic way this is wrong.
* **Frame size / `run_ops_from::<true>` size:** predict the instruction
  count changes (PGO commonly duplicates or splits blocks), described only,
  not treated as evidence, per the brief.

## Toolchain check

* `rustc 1.98.1 (48a229cea 2026-09-01)`, LLVM version 22.1.8.
* `/usr/bin/llvm-profdata-21`: LLVM 21.1.8.
* `~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/.../llvm-profdata`:
  LLVM 23.1.1-rust-1.100.0-nightly.
* `rustup component add llvm-tools --toolchain stable` fails offline
  (`error opening file for download: ... Read-only file system`), so the
  exact LLVM-22 `llvm-profdata` cannot be obtained in this sandbox. Neither
  available candidate is version-matched. Both are tried below and the
  outcome reported verbatim, per the brief ("say so; a profile that did not
  apply measures nothing").

## Base build

* `cargo build --release -p rexx-exec --bin rexx-run` from `rust/`,
  `CARGO_TARGET_DIR=<scratch>/target-base`.
* Copied to `<scratch>/bin/base-rexx-run`.
* `.text` sha256: `748b2f0679dcddb9cf65d2e30d5c50a18e4f90095ecb634f8d018c4bfcce4c32`

## Training sets

* `held-out`: every program the corpus differential reads --
  `cat corpus/phase-*.txt | grep -v '^#' | grep -v '^\s*$' | sort -u`, which is
  604 unique paths (the same 604 the correctness floor's `corpus_differential`
  test reports). This is derived, not assumed: every path was checked to
  exist under `corpus/`. `corpus/oracle-crashes.txt`'s 15 entries are inline
  code snippets (`## N. \`snippet\``), not corpus file paths, so none of the
  604 overlap it and nothing needed excluding on that account. None of the
  604 are the six measured axis programs (those live in `bench-rexxcps/` and
  `bench-programs/`, a different directory tree).
* `in-sample`: the six axis programs themselves, one run each -- all six
  already iterate 4.6-25 million times internally, so a single run gives the
  profiler ample branch-count data without needing repeats.
* Both trained via a `-Cprofile-generate` binary built with the exact same
  base source, run with stdin from `/dev/null` and each invocation capped by
  `timeout 5` (`held-out`) or `timeout 60/120` (`in-sample`, since `rexxcps`
  itself takes ~2.8s natively). Run from a fresh empty directory
  (`<scratch>/run-heldout/`, `<scratch>/run-insample/`), never the
  scratchpad root. All 604 `held-out` programs ran to completion inside their
  timeout; none hung. Training script: `<scratch>/train-heldout.sh`.

## Merge and profile-use build

* `llvm-profdata-21` (LLVM 21.1.8, one major below the compiler's LLVM 22.1.8):
  merges both `held-out` and `in-sample` raw profiles cleanly, exit 0, no
  warning printed.
* The nightly toolchain's `llvm-profdata` (LLVM 23.1.1-rust, one major above):
  **hard failure** --
  `warning: ...profraw: raw profile version mismatch: Profile uses raw
  profile format version = 10; expected version = 11` /
  `error: no profile can be merged`, exit 2. This is the mismatch the brief
  asked to watch for -- it just failed with the other candidate, not the one
  used for the reported numbers.
* Built `-Cprofile-use=<merged>.profdata` for both `held-out` and `in-sample`
  with `llvm-profdata-21`'s output. Neither build printed a profile
  cfg-mismatch warning (the default-on warning that `-Cllvm-args=-pgo-warn-mismatch`
  turned out not to be the right spelling for; `--no-pgo-warn-mismatch` is,
  and warnings are on by default, so no flag was needed to see them -- none
  fired).
* Three distinct `.text` hashes confirm three distinct builds:
  `base` `748b2f06...`, `held-out` `26f86704...`, `in-sample` `cb9fbb98...`.

## Callgrind instructions (common table, round 1; round 2 is a determinism check, see below)

`valgrind --tool=callgrind`, ex-libc (everything attributed to `libc.so.6` or
`ld-linux*.so*` subtracted from `PROGRAM TOTALS` via `callgrind_annotate
--auto=no --threshold=100`), from a fresh empty run directory per invocation.

| axis | base (ex-libc Ir) | held-out (ex-libc Ir) | held-out delta | in-sample (ex-libc Ir) | in-sample delta |
|---|---:|---:|---:|---:|---:|
| rexxcps | 18,670,562,219 | 18,211,162,534 | **-2.46%** | 16,285,355,267 | -12.78% |
| varlookup | 17,124,834,983 | 16,656,919,384 | **-2.73%** | 14,819,196,589 | -13.46% |
| arith | 11,790,608,810 | 12,939,559,174 | **+9.74%** | 11,701,139,205 | -0.76% |
| emptyloop | 9,687,822,265 | 9,475,900,401 | **-2.19%** | 7,456,184,144 | -23.04% |
| dispatch | 20,713,025,806 | 17,501,064,578 | **-15.51%** | 14,161,361,080 | -31.63% |
| compound | 9,916,377,165 | 9,639,233,865 | **-2.80%** | 7,838,912,451 | -20.95% |

**Round 2 (determinism check).** All 18 round-2 combinations reproduced
round 1 to within 0.03% (the largest gap: `rexxcps` in-sample, +0.027%; most
axis/variant pairs matched to 5-6 significant figures, e.g. `arith` base
11,790,608,810 vs 11,790,608,785). Callgrind's Ir counting is deterministic
here as expected; the table above is round 1, round 2 only confirms it and
is not shown separately.

This is not the flat, sub-1%-on-non-rexxcps-axes picture the prediction
guessed. Four axes (`rexxcps`, `varlookup`, `emptyloop`, `compound`) cluster
around -2.2% to -2.8% held-out, consistent with each other and with the
degenerate-profile control being a real, if modest, held-out win rather than
noise. `dispatch` is a much bigger held-out win, -15.5%. `arith` is the one
axis that got *worse* under held-out training, +9.7% -- and per the message
to the coordinator above, `arith_small_int` is the one function whose
inlining status flips specifically between held-out (inlined away) and
base/in-sample (kept standalone); that is a plausible mechanism for the
regression, though this spike did not isolate it further.

## Cycles (`perf stat -e cycles`)

Five interleaved runs per binary on `rexxcps` and `varlookup`, `perf stat -e
cycles -x,`, stdin `/dev/null`, from a fresh empty directory. All 60 runs
(30 forward-order, 30 reversed-order control below) reported `100.00%`
multiplexing -- no `<not counted>`, no retry needed. Raw data:
`<scratch>/perf/results-forward.tsv`, `<scratch>/perf/results-reversed.tsv`.

**This machine is shared with several other spike agents' own callgrind and
build jobs running the whole time** (`ps aux` during this spike showed
concurrent `valgrind --tool=callgrind` processes from `frame-arena` and
`handler-table` spikes, unrelated to this one). Ambient load swung between a
"quiet" and a "loud" epoch mid-measurement -- `varlookup`'s reversed-order
run shows cycle counts roughly halve for three rounds, then return to the
loud-epoch magnitude for the last two. This makes an absolute percentage
unreliable; the *direction* is what is checked below.

**Order-reversal control.** The first run measured variants in the fixed
order base, held-out, in-sample, every round -- so if ambient load merely
drifted downward over the run, whichever binary ran last would look
artificially faster for reasons having nothing to do with PGO. A second run
reversed the order (in-sample, held-out, base, every round) to check for
this. Result: in `rexxcps`, the ranking base (slowest) > held-out >
in-sample (fastest) holds in 4 of 5 reversed rounds despite `base` now
running *last* in each round (round 1 of the reversal coincides with the
load transition and is the one exception -- excluded below). In `varlookup`,
`base` is the slowest of the three in every one of the 5 reversed rounds,
again despite running last. **The ranking does not depend on run position,
in either axis** -- that rules out the specific artifact the reversal was
built to catch.

| axis | variant | mean cycles (stable rounds) | vs base |
|---|---|---:|---:|
| rexxcps | base | 6,155,273,558 (n=9) | -- |
| rexxcps | held-out | 5,848,188,639 (n=9) | -4.99% |
| rexxcps | in-sample | 5,051,262,058 (n=8, excludes 1 outlier round) | -17.94% |
| varlookup ("quiet" epoch) | base | 2,694,019,373 (n=3) | -- |
| varlookup ("quiet" epoch) | held-out | 2,509,253,242 (n=3) | -6.86% |
| varlookup ("quiet" epoch) | in-sample | 2,396,689,925 (n=3) | -11.03% |
| varlookup ("loud" epoch) | base | 5,556,883,518 (n=7) | -- |
| varlookup ("loud" epoch) | held-out | 4,512,249,773 (n=7) | -18.80% |
| varlookup ("loud" epoch) | in-sample | 3,813,028,141 (n=7) | -31.38% |

`rexxcps`'s excluded outlier: forward round 5, `in-sample`,
7,379,780,268 cycles against that round's other four in-sample values of
4.96-5.01B -- a single-run spike, not reproduced in any of the other 8
in-sample rounds; kept out of the mean, not out of the raw file.

Reading across axes: **cycles move much further than instructions did.**
`rexxcps` Ir was -2.46% held-out; cycles are -4.99%. `varlookup` Ir was
-2.73% held-out; cycles are -6.86% to -18.80% depending on the epoch. This is
what the brief predicted PGO would do -- move branch layout and prediction
rather than instruction count -- but the size of the move cannot be pinned
down more precisely than "clearly negative, roughly 2-4x the Ir delta in
proportional terms" given how much the ambient load on this box varies run
to run. Treat the direction as solid and the magnitude as a range, not a
point estimate.

## Frame size / `run_ops_from::<true>`, description only

Measured via `nm -C -S` (byte size) and `objdump -d` instruction-line counts
over the symbol's address range, not treated as evidence per the brief.

| build | `run_ops_from::<true>` bytes | instructions | `loop_advance` bytes | `flat_loop_step_top` bytes |
|---|---|---|---|---|
| base | 15,029 | 3,467 | 6,083 | 3,710 |
| held-out | 16,669 (+10.9%) | 3,726 (+7.5%) | 6,462 (+6.2%) | 3,356 (-9.5%) |
| in-sample | **symbol absent** | -- | 6,125 (~base) | 9,511 (+156%) |

## Control: a degenerate profile

`held-out`'s `rexxcps` callgrind result (below) moves by more than the
common.md driver-edit noise floor discusses, but this is not a driver edit --
it is a build-flag-only change, so that specific noise floor does not
transfer as-is. To separate "any `-Cprofile-use` build moves the number a
couple of percent regardless of what it learned" from "this profile
specifically helps," a third variant was built and measured the same way:
trained on one program (`say 1`, one run, one clause) instead of the 604-file
`held-out` set or the six-axis `in-sample` set -- a profile that is real (not
empty/malformed) but carries almost no information about the interpreter's
hot paths.

`rexxcps`, round 1, ex-libc Ir: `base` 18,670,562,219; `degenerate`
18,997,662,510 -- **+1.75%**, the opposite direction from `held-out`'s -2.46%
(below). An arbitrary profile-use build does not reproduce `held-out`'s
result; it moves the other way. That separates the `held-out` number from
generic "any profile-guided rebuild moves the needle" noise -- the direction
depends on what was learned, not just on taking the `-Cprofile-use` path.

Under `in-sample` training, `run_ops_from::<true>` (the only instantiation the
six axes call, per `common.md`) is gone as a standalone symbol -- fully
inlined into its caller. `flat_loop_step_top` absorbs roughly 5,800 of those
bytes, well under the donor's 15,029, consistent with cold-arm elimination
alongside the inlining (the "57% never executed" arms of `common.md` getting
dropped rather than carried in). Under `held-out` training the function stays
a standalone symbol and *grows*, in both bytes and instructions -- the
opposite of the hot/cold-splitting shrink the prediction guessed at.

## Additional inlining checks (asked mid-spike)

Whole-binary `nm -C` and per-axis `callgrind_annotate` self-Ir rows:

* `Interp::arith_small_int`: standalone in `base` and `in-sample`; **absent
  (fully inlined) in `held-out`**. On the `varlookup` axis specifically, its
  self-Ir row is absent in *all three* builds -- that callsite is inlined
  everywhere regardless of PGO; the whole-binary difference comes from a
  different callsite (plausibly `arith.rex`'s own).
* `Option<LoopHeaderValues>>::drop_glue` vs `LoopHeaderValues::drop_glue`:
  `base` instantiates the `Option`-wrapped drop glue; both `held-out` and
  `in-sample` instantiate the unwrapped one instead. On `emptyloop`, no
  self-Ir row for either symbol in any of the three builds -- inlined into
  the caller everywhere.
* `run_ops_from::<true>`: present in `base` and `held-out`, absent
  (inlined) in `in-sample` -- see above.

Net: PGO's inlining choices move in both directions depending on what the
profile saw; there is no uniform "more profile coverage -> more inlining"
rule visible here.

## Correctness floor

* `cargo test -p rexx-exec --release` on the `held-out` build:
  **833 passed, 14 failed**. All 14 failures are
  `the worktree's build/lib is three directories above this crate: ... No
  such file or directory` (`crates/rexx-exec/src/dispatch/library.rs:795`) --
  a worktree whose `CARGO_MANIFEST_DIR/../../../build/lib` doesn't exist,
  because this spike's worktree (nested under `.claude/worktrees/`) never had
  the native-extensions build step run in it. **Control:** the identical
  plain `base` build (no PGO flags, separate `CARGO_TARGET_DIR`) was run
  through the same command and failed the **same 14 tests, same names,
  833 passed** -- this is a pre-existing environmental gap in the worktree,
  not a PGO regression.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test
  corpus corpus_differential` on the `held-out` build:
  `604 of 604 matching` (STRICT mode, quoted verbatim from the test's own
  output).

## Verdict

**Sticks, with a large, axis-dependent, and direction-inconsistent effect --
not the small uniform win the prediction expected.** Held-out PGO training
(604 programs, none of them the measured axes) moves callgrind instructions
by -2.2% to -2.8% on four of six axes (`rexxcps`, `varlookup`, `emptyloop`,
`compound`), by -15.5% on `dispatch`, and by **+9.7% (a regression)** on
`arith`. The degenerate-profile control (trained on one trivial program)
moves `rexxcps` the *other* direction (+1.75%), which is evidence the
held-out result is not just "any profile-use build moves the number a
couple of percent" -- but it does not explain why `arith` alone regresses.
Cycles move further than instructions (confirmed by an order-reversal
control that rules out a run-position artifact) but the magnitude is not
pinnable down to a point estimate on this shared, noisy machine. Correctness
holds on the held-out build (604/604 differential, no new unit-test
failures). No profile mismatch was hit with the toolchain actually used
(`llvm-profdata-21`); the version-matched candidate for this rustc's LLVM 22
could not be obtained offline, and the one-major-above candidate (nightly's,
LLVM 23) hard-failed to merge at all -- reported as asked, not worked around.

## Concerns

* **`arith`'s regression is unexplained, not just unflattering.** +9.7% on
  held-out is bigger in magnitude than most of the wins, and this spike did
  not isolate why (the `arith_small_int` inlining flip is a plausible
  mechanism, not a demonstrated one -- no further isolation was done, e.g.
  building a profile-use variant with that one function's inlining pinned).
  A PGO rollout would need to know whether other cold-in-training-but-hot-in-
  production axes regress the same way before shipping this.
* **`dispatch`'s -15.5% is the biggest number in this report and was not
  independently controlled the way `rexxcps` was.** The degenerate-profile
  control only measured `rexxcps`; it is plausible but unverified that
  `dispatch`'s win is similarly real (not degenerate-profile noise) rather
  than, say, a training-set idiosyncrasy specific to how many `held-out`
  programs happen to exercise method dispatch.
* **This machine was shared with several other agents' CPU-bound jobs for
  the entire spike**, confirmed by `ps aux` and by the bimodal cycle counts
  in the perf data. Callgrind Ir counts are unaffected by this (confirmed:
  round 2 reproduced round 1 to <0.03%), but the cycle numbers' *magnitude*
  is not trustworthy beyond "clearly negative, roughly 2-4x the Ir delta."
  Re-running the cycle measurements on a quiet machine would tighten this
  considerably.
* **No version-matched `llvm-profdata` was available.** `llvm-profdata-21`
  (one LLVM major below the compiler) merged and applied silently, with no
  `pgo-warn-mismatch`-style warning at any stage -- but "no warning printed"
  is not the same guarantee as "used the exact tool the compiler team
  intended," and this was not independently cross-checked against a version-
  matched build (none could be obtained in this sandbox; would need network
  access to `rustup component add llvm-tools`, which failed with a read-only
  filesystem error here).
* **In-sample numbers are not a ceiling to plan around** -- they are shown
  to illustrate memorisation (e.g. `dispatch` -31.6%, `run_ops_from::<true>`
  vanishing entirely), not as a target. Restated because this is the kind of
  number that gets miscited later.
* Six axes, two rounds each, on a machine under variable third-party load,
  from a single build of `rustc 1.98.1` / LLVM 22.1.8 -- this is one spike's
  worth of evidence, not a robustness study.

## Artifacts

* `<scratch>/report.md` (this file).
* `<scratch>/bin/` -- `base-rexx-run`, `heldout-rexx-run`,
  `insample-rexx-run`, `degenerate-rexx-run`, and their `.text` extracts.
* `<scratch>/heldout-list.txt` -- the 604-path held-out training list.
* `<scratch>/train-heldout.sh`, `<scratch>/run-callgrind.sh`,
  `<scratch>/run-perf.sh`, `<scratch>/run-perf-reversed.sh`,
  `<scratch>/exlibc.sh` -- the scripts this spike ran.
* `<scratch>/callgrind/*.out`, `*.log` -- all 36 raw callgrind runs.
* `<scratch>/perf/results-forward.tsv`, `results-reversed.tsv` -- raw perf
  data.
* `*.profdata`, `profraw-*/` -- the merged profiles and raw counters (large;
  not committed to git, kept in scratch only).
