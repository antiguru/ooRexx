# Driver-cost spikes, round 1

Four structurally different attacks on the IR driver's per-op cost, run in
parallel from `0ba0f3876`, each on its own `spike/*` branch in its own
worktree. Moritz's framing: try new things, add noise to the experiments, see
what sticks. Every base build had the same `.text` hash,
`748b2f0679dcddb9cf65d2e30d5c50a18e4f90095ecb634f8d018c4bfcce4c32`.

The spike code stays on its branch; the reports, briefs and the measurement
variants that lived on a tmpfs are copied here, because the branches are not
where this project looks.

| spike | branch head | `rexxcps` | verdict |
|---|---|---:|---|
| `frame-arena` | `spike/frame-arena` `bec1d020d` | -1.715% | inconclusive; `varlookup` -3.88%, `compound` -2.52% |
| `pgo` | `spike/pgo` `e57e953a7` | -2.46% held-out, -12.78% in-sample | sticks, but `arith` +9.74% held-out |
| `cold-arms` | `spike/cold-arms` `ee68875d7` | +0.94% | does not stick |
| `handler-table` | `spike/handler-table` `8b79f9269` | +10.04% | does not stick |

Figures are callgrind `summary:` minus `libc.so.6` and `ld-linux`, each spike
against its own base build. **Do not mix absolute figures across reports**:
`pgo.md`'s base reads 31.8 million instructions higher on `rexxcps` than the
other three from the same `.text`, so its subtraction or run environment
differs; its deltas are internally consistent.

## What the round says

**The code generation of the hot path is worth about 13% of `rexxcps`.** The
in-sample PGO build is a memorisation ceiling, not a result, but it prices from
above the cost that every source edit this week has moved unpredictably: once
the compiler knows the hot path, its inlining, layout and allocation choices
take 12.78% off `rexxcps` and 23.04% off `emptyloop` with no source change.
The held-out build recovers a fifth of that and regresses `arith`; a
degenerate one-program profile moves `rexxcps` +1.75%, so the held-out gain is
the profile's content and not the rebuild.

**Splitting the driver into functions loses, in both forms tried.**
Per-arm outlining of cold arms costs +0.94% with the driver kept separate, and
the steady loss is hot helpers leaving the driver, not the outlined code.
One-function-per-op call threading costs +10.04%, with a flat control (the
table present and unused is within 0.01% of base on every axis), at 19.5
instructions per region op on `varlookup` plus 22 per clause to build and drop
the region context. The oracle's small-method shape does not transfer to a
trampoline without guaranteed tail calls.

**Unchecked stable frames are the only source change that gained.** On
`varlookup` the checked-to-unchecked difference is 455,996,180 instructions
against 456,000,686 on the check's own lines, so the removed check is
attributed there. On `rexxcps` the -1.715% is under the floor, and the same
checked/unchecked pair moves `emptyloop`, which reads no registers, by -1.03%
(4 instructions per iteration in `run_ops_from::<true>` self cost). The arena
half alone (base to checked, -0.643%) has no control.

**The driver's noise comes in whole instructions per iteration.** `emptyloop`
moved by exactly 4 per iteration here and by +1 and -4 in the store-fusion
experiment, and `cold-arms` found edits flipping the inlining of
`arith_small_int` for 342,000,000 instructions on `varlookup` -- within 5,773
of store fusion's unexplained 342,005,773. That match is a hypothesis: the
store-fusion builds' per-function rows were not checked for it.

## Files

* `common-brief.md` and `<spike>-brief.md`: the dispatch briefs, unedited.
* `<spike>.md`: each spike's report as committed on its branch.
* `cold-arms-files/`: arm classification, variant patches, measurement scripts,
  results.
* `handler-table-files/`: the table-unused control and `#[inline(never)]`
  variant patches, and the per-round tables.
* `pgo-train-heldout.sh`: the held-out training run.
