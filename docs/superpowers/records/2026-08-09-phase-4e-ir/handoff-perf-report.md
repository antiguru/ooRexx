# Handoff items 1 and 3: working notes

Items 1 and 3 of `docs/superpowers/plans/phase-4e-handoff.md`, chased before Phase 4f.
The conclusions are appended to that file; this one holds the method, the raw figures and the dead ends.

## Method

Two instruments, and the reason for using both.

* **`rexx-arms`** at the committed lengths, one sitting, both arms, both sizes, five rounds, medians, on `instructions:u` and `cycles:u`.
  This is the phase's own instrument and the one the items' facts are stated on.
  Rows appended to `rust/bench-baselines/phase-4e-arms.tsv` under tasks `item1` and `item3`.
* **`callgrind`** at two short lengths, differenced.
  Exact and deterministic between processes, which is what makes a per-commit bisect affordable: a bisect over five builds costs minutes rather than an hour, and no median is needed.
  Per-pass is `(Ir(2n) - Ir(n)) / n`, so the fixed cost cancels exactly as it does in `rexx-arms`.

**The two agree.**
On `emptyloop`'s arm gap, callgrind reads 119.004 / 140.001 / 125.005 at the three builds where `rexx-arms` reads 119.000 / 140.000 / 125.000.
That agreement is what licenses the callgrind bisects and decompositions below to carry the same weight as the `rexx-arms` figures.

Builds were made in a detached worktree under the scratchpad, one binary per commit, release profile unchanged.
Spike sources were restored with `cp` from a backup and verified with `sha256sum -c`, and every spike binary was built from a tree that `git status` reported clean afterwards.

## Item 1: `arith`'s tree-walker arm

### The bisect

Only three commits between `e63e8a00` and `16077ea1` carry Rust source that reaches the tree-walker: `b3345d91`, `ea17a699`, `f0d12ebe`.
`16077ea1` itself changes only `tests/ir_dual_cases/arithmetic`.

callgrind, tree-walker arm, per pass:

| build | per pass | delta on previous |
| --- | --- | --- |
| `e63e8a00` | 57723.481 | |
| `b3345d91` | 57884.108 | +160.627 |
| `ea17a699` | 57883.349 | -0.759 |
| `f0d12ebe` | 57883.263 | -0.086 |
| `16077ea1` | 57883.362 | +0.099 |

`rexx-arms`, tree-walker arm, `instructions:u`, per pass:

| build | per pass | delta on `e63e8a00` |
| --- | --- | --- |
| `e63e8a00` | 61843.919 | |
| `b3345d91` | 62014.251 | +170.332 |
| `16077ea1` | 62030.499 | +186.580 |

The two instruments disagree about the tail: at the committed length `rexx-arms` puts +16.2 after `b3345d91`, where callgrind at the shorter length puts -0.7 there.
Recorded as measured rather than explained.
They agree that `b3345d91` is the dominant term and that no other commit in the range is comparable to it.

### The spikes

* **S1, the redundant settings read.** `arith_small_int` reads `activation().settings.digits()` before deciding, so the general path reads it twice where the pre-split body read it once.
  Moving the read inside the `SmallInt` arm removes exactly that second read and is semantically identical.
  Built at `b3345d91`: **57962.160**, which is 78.05 per pass *worse*.
  Refuted.
* **S2, the pre-split body at the range head.** `e63e8a00`'s body restored inline in `eval_arithmetic` at `16077ea1`, with `arith_small_int` and `arith_general` left in place for `Op::Arith`.
  callgrind 57749.761; `rexx-arms` 61868.915, which is 161.584 below `16077ea1` and 24.996 above `e63e8a00`.
* **S3, the same restore one commit earlier.** The same body restored at `b3345d91`: callgrind 57746.526, which is 23.045 above `e63e8a00`.

S2 and S3 land within 3.2 of each other, which says the residual belongs to `b3345d91` rather than to the commits after it, and that it survives restoring the tree-walker's own body.

### What the profile showed that the item did not expect

`nm -C` finds no `arith_small_int` and no `arith_general` symbol in any binary of the range, and callgrind records no call edge to either.
Both are inlined into `eval_arithmetic` outright.
`eval_arithmetic` and its recursive instance carry +137.67 of the +160.63 per pass, and `eval` and its recursive instance carry +23.00.

## Item 3: an all-`Generic` body

### The bisect

callgrind, `emptyloop`, per pass, both arms, and the gap that is the IR arm's own cost:

| build | tw | ir | gap |
| --- | --- | --- | --- |
| `132c3395` | 1494.003 | 1592.993 | 98.991 |
| `e7b8eef8` | 1496.010 | 1596.010 | 100.000 |
| `7d9cfb9a` | 1495.002 | 1614.000 | 118.998 |
| `e63e8a00` | 1495.000 | 1614.004 | 119.004 |
| `b3345d91` | 1494.998 | 1614.008 | 119.009 |
| `ea17a699` | 1494.998 | 1623.009 | 128.011 |
| `f0d12ebe` | 1494.998 | 1627.002 | 132.004 |
| `16077ea1` | 1494.999 | 1627.000 | 132.002 |
| `6b5fac3b` | 1497.997 | 1637.999 | 140.002 |
| `5ab3028f` | 1498.009 | 1637.998 | 139.989 |
| `8368d357` | 1498.000 | 1638.001 | 140.001 |

Every move lands in `run_ops::<false>`'s self cost, per `compare` of the profiles either side of it: +9.000 at `ea17a699`, +4.000 at `f0d12ebe`, +8.000 at `6b5fac3b`.
`6b5fac3b` adds a further +3.000 in `step_in_temps_frame`, which both arms pay and which is the whole of the tree-walker arm's own +3.

Two of the five moves add no `Op` variant at all: `7d9cfb9a` adds +19 with the variant count unchanged, `f0d12ebe` adds +4 with it unchanged.

### The decomposition

A five-`nop` loop body against a one-`nop` loop body separates the part that scales with the body's clause count from the part that does not, and subtracting the tree-walker arm leaves the IR arm's own cost.
`callgrind` confirms exactly one `run_ops::<false>` entry per pass for both programs, so the fixed part is per entry.

| build | IR-only per `Op::Generic` | IR-only per `run_ops` entry |
| --- | --- | --- |
| `7d9cfb9a` | 22.000 | 97.001 |
| `e63e8a00` | 22.005 | 96.991 |
| `b3345d91` | 21.999 | 97.004 |
| `ea17a699` | 29.006 | 98.986 |
| `f0d12ebe` | 29.001 | 102.996 |
| `16077ea1` | 29.001 | 103.003 |
| `6b5fac3b` | 31.001 | 108.990 |
| `8368d357` | 31.002 | 108.988 |
| `widen-12-heavy` | 23.000 | 102.005 |
| `chunkwidth` | 31.004 | 108.988 |

`emptyloop` pays one of each per pass: 22.0 + 97.0 = 119.0 at `e63e8a00`, 31.0 + 109.0 = 140.0 at head.

### The spikes

* **Dispatch width.** `Op` given twelve or twenty-four further variants, each with a driver arm in both of `run_ops`' exhaustive matches, emitted by `compile` behind a condition that is never true at run time and not foldable at compile time.
  `size_of::<Op>()` unchanged, which the crate's own `const` assertion enforces.
  The arms are demonstrably in the binary: `run_ops`' symbols go from 12228 bytes to 13387 with twelve trivial arms, 14716 with twenty-four, and 28354 with twelve arms carrying `Op::Generic`'s body, and the string `REXX_SPIKE_WIDEN` is present in each spike binary and absent from the control.
  Gap per pass: control 140.001, twelve trivial 136.005, twenty-four trivial 137.002, twelve heavy 125.004.
  Confirmed on `rexx-arms` at the committed length: the twelve-heavy build's gap is 125.000 against the control's 140.000, and its arm ratio is 1.08234 against 1.09223.
  Refuted, and refuted in the wrong direction: doubling the dispatch makes the axis cheaper.
* **`Chunk` width.** Two dead fields of exactly the shapes `hints` and `calls` have, constructed and never read.
  Per `Op::Generic` 31.004 against 31.002, per entry 108.988 against 108.988, gap 139.993 against 139.990.
  Refuted, at exactly zero.
* **Op-stream stride.** Not a build: `ir/mod.rs` carries `const _: () = assert!(size_of::<Op>() == 12);` at every commit in the range, so the stride the driver indexes the stream at never moved.

## Dead ends, so that nobody repeats them

* Moving the `digits` read inside `arith_small_int`'s matching arm to remove the general path's second read.
  It is a true redundancy and removing it costs 78 instructions per pass on `arith`'s tree-walker arm.
* Widening the driver's `match` as an explanation for an all-`Generic` body's cost.
  Two spikes and three variant counts say the relationship does not exist, and the sign is against it.
* `Chunk` gaining fields as an explanation for the per-entry half.
  Zero, on both halves and on the gap.
* Reading a per-function delta as a mechanism.
  `eval_arithmetic` carrying the whole of item 1's delta is compatible with several mechanisms, and the one the item named turned out to be the wrong one of them; only S1 and S2 separated them.

## Files

* Binaries: `bins/rexx-run-<commit>` and `bins/rexx-run-{S1-digitsonce,S2-presplit-tw,S3-presplit-at-b3345d91,widen-12-heavy,widen-12-trivial,widen-24-trivial,chunkwidth}` under the session scratchpad.
* callgrind profiles: `cgrun/` for `arith`, `cgloop/` and `cgloop5/` for `emptyloop`.
* `rexx-arms` output: `arms-item1.out` and `arms-item3.out`, appended verbatim to `rust/bench-baselines/phase-4e-arms.tsv`.
