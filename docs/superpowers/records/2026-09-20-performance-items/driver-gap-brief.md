# The interpretation machinery, which is 63.7% of the gap to the oracle

`rexxcps`, 20,000,000 clauses: we run **1,014.6 instructions per clause**, the
C++ oracle **534.4**. A category rollup of both profiles
(`docs/superpowers/records/2026-09-20-performance-items/2026-09-22-category-rollup.md`)
puts **305.67 of that 480.15 gap in interpretation machinery**, and nothing else
is a quarter of it. Our three `run_ops_from` instantiations are **252.18 per
clause**, against **98.29** for the oracle's entire machinery category.

This task attacks that. Read the rollup first, and
`2026-09-21-reprofile-report.md` beside it.

## The decomposition, measured in the shipping profile

Self cost inside the three `run_ops_from` instantiations, by the file each
instruction was attributed to after inlining. Derived from
`valgrind --tool=callgrind --dump-instr=yes` with both position columns decoded;
the parser's whole-program self total reconciles to the run's `summary:` line
with **difference 0**, which is the check that caught an earlier version of it
undercounting by 40%.

| inside the driver | Ir/clause | share of the driver |
|---|---:|---:|
| `ir/drive.rs` itself | 116.35 | 46.1% |
| `core/src/slice/index.rs` | 19.86 | 7.9% |
| `run.rs`, inlined | 18.75 | 7.4% |
| `core/src/slice/iter/macros.rs` | 18.09 | 7.2% |
| `rexx-core/src/roots.rs` | 17.05 | 6.8% |
| `trace.rs`, inlined | 11.78 | 4.7% |
| `activation.rs`, inlined | 7.85 | 3.1% |

The hottest attributed lines in `drive.rs`:

| line | Ir/clause | what it is |
|---|---:|---|
| `:664` | **25.22** | `for region_op in ops {`, the inner region walk's loop header |
| `:544` | 6.17 | `let op = &stream[pc as usize];`, the outer fetch |
| `:519` | 4.22 | `if pc >= stop {`, the outer bound test |
| `:987` | 3.04 | a register-write bound assertion |
| `:565` | 2.45 | `if granting {` |
| `:0` | **27.77** | no line attribution, and worth identifying |

## The three leads, in the order the evidence supports

**1. The region walk. `:664` at 25.22 plus `iter/macros.rs` at 18.09 is about 43
instructions per clause spent iterating, before any op does anything**, and
`index.rs` adds 19.86 more. The driver is two-level: an outer loop over ops and
an inner walk over a clause's region. That inner walk is the single most
expensive line in the interpreter.

**Do not assume the fix.** A closely related change measured **+2.015% on
`emptyloop`** yesterday: re-slicing to turn two bounds checks into one cost more
than the check it removed. The same day, cutting the outer stream to its bound
was worth **-0.86%** on `varlookup`. One technique, three sites, three signs.

**2. `drive.rs:0`, 27.77 per clause with no line.** That is more than any
attributed line except the region walk, and nobody knows what it is. Identify it
before designing anything around it -- it is plausibly the `match op` jump table
itself, in which case it is the dispatch and not a candidate.

**3. `trace.rs` at 11.78 per clause, inlined into the driver, on a run with
tracing off.** Two previous items attacked trace emission; this is what is left,
and it is a gate that answers "no" 38 million times.

## The constraint that has already reverted one commit

**Any conditional in the driver's per-clause path costs about 0.5% on `rexxcps`
and 1.25% on `emptyloop`, whatever it does.** Measured in five shapes. A
speculation was reverted at a measured +0.501% for its guard against a -0.543%
saving. **If your design adds a test to the hot path to remove work from the hot
path, stop and report that** rather than tuning it.

The corollary is the one that matters here: **the win has to come from doing
fewer, wider things per clause, not from a cheaper test.** The `Condition` +
`JumpUnless` fusion measured 54.8 instructions per removed op, of which only 8
was dispatch and **34.8 was the value handoff between the two ops**. That is the
shape that pays.

## Rules for every figure

* **Measure in the shipping profile.** Settled 2026-09-22: at `-O2` without LTO,
  three of five landed (commit, axis) cells **flip sign**, and an `-O2` profile
  reports 5.319% of cost in `RootSet` calls that fat LTO inlines to exactly
  zero. Do not build a cheaper profile to explore in.
* `rexxcps` carries about 0.01% intrinsic spread because it renders `TIME()`
  into its own output; `emptyloop` and `varlookup` reproduce to 0.0001%, so a
  sub-percent question is decided on the narrow axes and `rexxcps` is still
  always reported.
* Interleave arms, own `CARGO_TARGET_DIR` per build, and confirm each measured
  binary against a rebuild by `.text` hash:
  `objcopy -O binary --only-section=.text <bin> <out> && sha256sum <out>`.
  A whole-file hash cannot do it: `debug = true` puts the target directory's
  path in the debug info.
* **A file-level share inside one function ranks nothing.** Measured: a change
  that cut the program 0.864% made its own file's attributed share *rise*. The
  table above is a decomposition, not a ranking; treat each row as a place to
  look.

## How it is judged

**The oracle differential is the arbiter.** `corpus_differential` has been 604 of
604 STRICT for four commits; keep it there. Gates as `rust/CLAUDE.md` defines
them, both pairs, builds outside the cap and tests under `memcap 8G`. Expected at
BASE: 133 binaries, 2651 passed / 0 failed / 4 ignored release, 2652 / 0 / 4
debug. A sub-second clippy against a warm target is provisional; re-run it cold.
Sum test tallies from each log's own `test result:` lines, never a summary.

Commit before any long run; leave the tree frozen until the status file says
finished. I will not commit to this tree while you hold it.

## Scope, and when to stop

**One commit per idea, measured, kept or reverted on its own number.** A reverted
candidate with its measurement is a result, not a failure: three were reverted
yesterday and two of them were more useful than the commits beside them.

**Do not attempt a rewrite of the driver.** If the decomposition points at one,
say so with the evidence and stop -- that is a decision for Moritz, not a task.

## Report

`.superpowers/sdd/2026-09-22-driver-gap-report.md`, or `.txt` if your harness
refuses `.md`, and say which. **Replies truncate at about 8 KB: gate statuses
first**, then commits, then the measurement per idea, then concerns. Quote the
command beside every figure and give the exit status you observed.
